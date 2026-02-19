mod utilities;

use std::rc::Rc;
use edge_rules::runtime::edge_rules::EdgeRulesModel;
use edge_rules::runtime::TypedValue;
use edge_rules::test_support::{ContextQueryErrorEnum, UserTypeBody};

#[test]
fn test_get_type_on_various_user_functions() {
    let code = r#"
    {
        func add(a: <number>, b: <number>): a + b
        func greet(name: <string>): 'Hello ' + name
        func complex(x: <number>): {
            y: x * 2
            return: y + 1
        }
        func noArgs(): 42
    }
    "#;
    let mut model = EdgeRulesModel::new();
    model.append_source(code).unwrap();
    let runtime = model.to_runtime().unwrap();

    // Simple inline function
    assert_eq!(runtime.get_type("add").unwrap().to_string(), "number");
    
    // Simple inline function with string
    assert_eq!(runtime.get_type("greet").unwrap().to_string(), "string");

    // Complex function with return field
    assert_eq!(runtime.get_type("complex").unwrap().to_string(), "number");

    // Function with no args
    assert_eq!(runtime.get_type("noArgs").unwrap().to_string(), "number");

    // All model
    assert_eq!(runtime.get_type("*").unwrap().to_string(), "{}");
}

#[test]
fn test_get_type_on_nested_functions() {
    let code = r#"
    {
        nested: {
            func inner(a: <number>): a * a
        }
    }
    "#;
    let mut model = EdgeRulesModel::new();
    model.append_source(code).unwrap();
    let runtime = model.to_runtime().unwrap();

    assert_eq!(runtime.get_type("nested.inner").unwrap().to_string(), "number");
    assert_eq!(runtime.get_type("nested").unwrap().to_string(), "{}");
    assert_eq!(runtime.get_type("*").unwrap().to_string(), "{nested: {}}");
}

#[test]
fn test_get_type_wildcard_bypasses_functions() {
    let code = r#"
    {
        func add(a, b): a + b
        field: 10
        nested: {
            func sub(a, b): a - b
            val: 20
        }
    }
    "#;
    let mut model = EdgeRulesModel::new();
    model.append_source(code).unwrap();
    let runtime = model.to_runtime().unwrap();

    // Wildcard should only show "field" and "nested" (as object)
    // and "nested" schema should only show "val"
    assert_eq!(runtime.get_type("*").unwrap().to_string(), "{field: number; nested: {val: number}}");
}

#[test]
fn test_get_type_on_definition() {
    let code = r#"
    {
        type Person: { name: <string>; age: <number> }
        p: { name: 'Alice'; age: 30 } as Person
    }
    "#;
    let mut model = EdgeRulesModel::new();
    model.append_source(code).unwrap();
    let runtime = model.to_runtime().unwrap();

    // Test get_type on a field - currently returns structure even if casted to named type
    // @Todo: Should it return "Person" instead of structure?
    assert_eq!(runtime.get_type("p").unwrap().to_string(), "{name: string; age: number}");

    // Test get_type on a type definition - returns the structure
    // Since it's now a UserType(TypeObject), calling to_string() on get_type() calls ContextObject::to_schema()
    assert_eq!(runtime.get_type("Person").unwrap().to_string(), "{name: string; age: number}");

    // Test primitive alias
    let mut model2 = EdgeRulesModel::new();
    model2.append_source("{ type MyNumber: <number> }").unwrap();
    let runtime2 = model2.to_runtime().unwrap();
    assert_eq!(runtime2.get_type("MyNumber").unwrap().to_string(), "number");

    // Test alias of another named type
    let mut model3 = EdgeRulesModel::new();
    model3.append_source(r#"{
        type Person: { name: <string> }
        type Alias: <Person>
    }"#).unwrap();
    let runtime3 = model3.to_runtime().unwrap();
    // Alias returns the structure of the aliased type
    assert_eq!(runtime3.get_type("Alias").unwrap().to_string(), "{name: string}");

    // Test get_type("*") - should now bypass functions and type definitions
    let mut model4 = EdgeRulesModel::new();
    model4.append_source(r#"{
        func add(a, b): a + b
        type User: { name: <string> }
        existing: "existing value"
    }"#).unwrap();
    let runtime4 = model4.to_runtime().unwrap();
    assert_eq!(
        runtime4.get_type("*").unwrap().to_string(),
        "{existing: string}"
    );
}

#[test]
fn test_get_type_on_nested_definition() {
    let code = r#"
    {
        nested: {
            type Address: { street: <string> }
            addr: { street: 'Main St' } as Address
        }
    }
    "#;
    let mut model = EdgeRulesModel::new();
    model.append_source(code).unwrap();
    let runtime = model.to_runtime().unwrap();

    // Field casted to Address
    assert_eq!(runtime.get_type("nested.addr").unwrap().to_string(), "{street: string}");
    // Definition itself
    assert_eq!(runtime.get_type("nested.Address").unwrap().to_string(), "{street: string}");
}

#[test]
fn test_get_type_on_function() {
    let code = r#"
    {
        func add(a: <number>, b: <number>): a + b
    }
    "#;
    let mut model = EdgeRulesModel::new();
    model.append_source(code).unwrap();
    let runtime = model.to_runtime().unwrap();

    // get_type on function should return its return type
    assert_eq!(runtime.get_type("add").unwrap().to_string(), "number");
}

#[test]
fn test_get_user_type_assertions() {
    let mut model = EdgeRulesModel::new();

    // 1. Test Primitive Alias
    model.append_source("{ type MyNumber: <number> }").unwrap();
    let user_type = model.get_user_type("MyNumber").expect("Should find MyNumber");

    match &user_type {
        UserTypeBody::TypeRef(tref) => {
            assert_eq!(tref.to_string(), "number");
        }
        _ => panic!("Expected TypeRef for MyNumber, got {:?}", user_type),
    }

    // 2. Test Object Type
    model.append_source("{ type Person: { name: <string>; age: <number> } }").unwrap();
    let user_type = model.get_user_type("Person").expect("Should find Person");

    match &user_type {
        UserTypeBody::TypeObject(obj) => {
            // Explicitly link the type object to resolve its field types
            edge_rules::link::linker::link_parts(Rc::clone(obj)).expect("link type object");

            let borrowed = obj.borrow();
            assert!(borrowed.field_name_set.contains("name"));
            assert!(borrowed.field_name_set.contains("age"));
            // Verify structure via schema string.
            assert_eq!(borrowed.get_type().to_string(), "{name: string; age: number}");
        }
        _ => panic!("Expected TypeObject for Person, got {:?}", user_type),
    }

    // 3. Test Nested Type
    let mut model = EdgeRulesModel::new();
    model.append_source("{ nested: { type Address: { city: <string> } } }").unwrap();
    let user_type = model.get_user_type("nested.Address").expect("Should find nested.Address");

    match &user_type {
        UserTypeBody::TypeObject(obj) => {
            edge_rules::link::linker::link_parts(Rc::clone(obj)).expect("link type object");
            assert_eq!(obj.borrow().get_type().to_string(), "{city: string}");
        }
        _ => panic!("Expected TypeObject for nested.Address, got {:?}", user_type),
    }
}

#[test]
fn test_get_user_type_not_found() {
    let model = EdgeRulesModel::new();
    let result = model.get_user_type("NonExistent");
    assert!(result.is_err());
}

#[test]
fn test_get_user_type_on_non_type_field() {
    let mut model = EdgeRulesModel::new();
    model.append_source("{ field: 123 }").unwrap();

    // get_user_type should only return type definitions, not regular fields
    let result = model.get_user_type("field");
    assert!(matches!(result, Err(ContextQueryErrorEnum::EntryNotFoundError(_))));
}

#[test]
fn test_get_user_type_on_function() {
    let mut model = EdgeRulesModel::new();
    model.append_source("func add(a, b): a + b").unwrap();

    // get_user_type should only return type definitions, not functions
    let result = model.get_user_type("add");
    assert!(matches!(result, Err(ContextQueryErrorEnum::EntryNotFoundError(_))));
}
