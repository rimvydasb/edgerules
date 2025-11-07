# Edge Rules API Story

## Old EdgeRulesModel API

- load_source(&str) -> Result<(), ParseErrors>
- to_runtime() -> Result<EdgeRulesRuntime, LinkingError>
- to_runtime_snapshot() -> Result<EdgeRulesRuntime, LinkingError>

## New EdgeRules API

The goal is to bring a flexible full CRUD API for Edge Rules management.

Clarifications:
1. set operations override existing entries if the name already exists.
2. remove operations do nothing if the name does not exist.
3. get operations return None if the name does not exist.

### Expressions API

Possibility to assign an expression to the field of an Edge Rule. Fully qualified 
field names are supported (e.g., "model.enabled", "model.settings.threshold").

- set_expression(expression_name: &str, expression: ExpressionEnum) -> Result<(), ParseErrorEnum>
- remove_expression(expression_name: &str) -> Result<(), ParseErrorEnum>
- get_expression(expression_name: &str) -> Option<Rc<RefCell<ExpressionEntry>>>

Example: `...set_expression("model.enabled", ...)?`

### User Types API

- set_user_type(type_name: &str, type_definition: UserTypeBody) -> Result<(), ParseErrorEnum>
- remove_user_type(type_name: &str) -> Result<(), ParseErrorEnum>
- get_user_type(type_name: &str) -> Option<Rc<RefCell<UserTypeBody>>>

### User Functions API

- set_user_function(function_name: &str, arguments: Vec<FormalParameter>, body: Rc<RefCell<ContextObject>>) -> Result<(), ParseErrorEnum>
- remove_user_function(function_name: &str) -> Result<(), ParseErrorEnum>
- get_user_function(function_name: &str) -> Option<Rc<RefCell<MethodEntry>>>

### Additional Context Objects API

- merge_context_object(Rc<RefCell<ContextObject>>) -> Result<(), EdgeRulesError>
- append_source(&str) -> Result<(), ParseErrors>

> _append_source_ is renamed older _load_source_ method.