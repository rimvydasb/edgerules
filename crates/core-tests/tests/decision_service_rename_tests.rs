use edge_rules::runtime::decision_service::DecisionService;
use edge_rules::typesystem::types::ValueType;
use edge_rules::typesystem::values::ValueEnum;

#[test]
fn test_rename_simple_field() {
    let source = r#"
        input: 10
    "#;
    let mut service = DecisionService::from_source(source).unwrap();
    
    // Rename 'input' to 'output'
    service.rename_entry("input", "output").unwrap();
    
    // Check that 'input' is gone and 'output' exists with correct type/value
    let result = service.get_linked_type("input");
    assert!(result.is_err());
    
    let type_info = service.get_linked_type("output").unwrap();
    assert!(matches!(type_info, ValueType::NumberType));

    // Verify value preservation
    let value = service.evaluate_field("output").unwrap();
    assert_eq!(value, ValueEnum::from(10));
}

#[test]
fn test_rename_nested_field() {
    let source = r#"
        config: {
            retries: 3
            timeout: 500
        }
    "#;
    let mut service = DecisionService::from_source(source).unwrap();

    // Rename 'config.retries' to 'config.attempts'
    service.rename_entry("config.retries", "config.attempts").unwrap();

    let result = service.get_linked_type("config.retries");
    assert!(result.is_err());

    let type_info = service.get_linked_type("config.attempts").unwrap();
    assert!(matches!(type_info, ValueType::NumberType));
}

#[test]
fn test_rename_invalid_cross_context() {
    let source = r#"
        {
            ctx1: { a: 1 }
            ctx2: { b: 2 }
        }
    "#;
    let mut service = DecisionService::from_source(source).unwrap();

    // Try to rename 'ctx1.a' to 'ctx2.a'
    let result = service.rename_entry("ctx1.a", "ctx2.a");
    assert!(result.is_err());
}

#[test]
fn test_rename_not_found() {
    let source = "input: 1";
    let mut service = DecisionService::from_source(source).unwrap();

    let result = service.rename_entry("non_existent", "new_name");
    assert!(result.is_err());
}

#[test]
fn test_rename_duplicate() {
    let source = r#"
        {
            a: 1
            b: 2
        }
    "#;
    let mut service = DecisionService::from_source(source).unwrap();

    // Try to rename 'a' to 'b' which already exists
    let result = service.rename_entry("a", "b");
    assert!(result.is_err());
}

#[test]
fn test_rename_user_function() {
    let source = r#"
        func add(a, b): { res: a + b }
    "#;
    let mut service = DecisionService::from_source(source).unwrap();

    service.rename_entry("add", "sum").unwrap();

    let result = service.get_linked_type("add");
    assert!(result.is_err());

    let type_info = service.get_linked_type("sum").unwrap();
    if let ValueType::ObjectType(_) = type_info {
        // Success
    } else {
        panic!("Expected ObjectType for renamed user function");
    }
}
