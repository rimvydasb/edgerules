use edge_rules::runtime::decision_service::DecisionService;
use edge_rules::typesystem::types::ValueType;

#[test]
fn test_get_linked_type_simple_field() {
    let source = r#"
        input: 10
    "#;
    let service = DecisionService::from_source(source).unwrap();
    let type_info = service.get_linked_type("input").unwrap();

    assert!(matches!(type_info, ValueType::NumberType));
}

#[test]
fn test_get_linked_type_nested_field() {
    let source = r#"
        config: {
            retries: 3
            timeout: 500
        }
    "#;
    let service = DecisionService::from_source(source).unwrap();

    let retries_type = service.get_linked_type("config.retries").unwrap();
    assert!(matches!(retries_type, ValueType::NumberType));

    let timeout_type = service.get_linked_type("config.timeout").unwrap();
    assert!(matches!(timeout_type, ValueType::NumberType));
}

#[test]
fn test_get_linked_type_user_function() {
    let source = r#"
        func add(a, b): {
            res: a + b
        }
    "#;
    let service = DecisionService::from_source(source).unwrap();

    let func_type = service.get_linked_type("add").unwrap();
    // User functions are currently represented as ObjectType pointing to their body context
    if let ValueType::ObjectType(_) = func_type {
        // Success
    } else {
        panic!("Expected ObjectType for user function, got {:?}", func_type);
    }
}

#[test]
fn test_get_linked_type_user_type() {
    let source = r#"
        type Person: {
            name: <string>
            age: <number>
        }
    "#;
    let service = DecisionService::from_source(source).unwrap();

    let type_info = service.get_linked_type("Person").unwrap();
    if let ValueType::ObjectType(_) = type_info {
        // Success
    } else {
        panic!("Expected ObjectType for user type, got {:?}", type_info);
    }
}

#[test]
fn test_get_linked_type_wildcard() {
    let source = r#"
    {
        input: 10
        func calc(): { res: 20 }
    }
    "#;
    let service = DecisionService::from_source(source).unwrap();

    let root_type = service.get_linked_type("*").unwrap();
    if let ValueType::ObjectType(_) = root_type {
        // Success
    } else {
        panic!("Expected ObjectType for wildcard, got {:?}", root_type);
    }
}

#[test]
fn test_get_linked_type_not_found() {
    let source = "input: 1";
    let service = DecisionService::from_source(source).unwrap();

    let result = service.get_linked_type("non_existent");
    assert!(result.is_err());
}
