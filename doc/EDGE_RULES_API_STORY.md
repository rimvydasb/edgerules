# Edge Rules API Story

This document captures the upcoming CRUD-oriented API that will allow hosts (Rust, JS/WASM, Node/Web)
to build and mutate Edge Rules models without re-parsing source text each time. It ties the desired
surface area to the structures that already exist in the codebase so the work is immediately
implementable.

## Background

The current public API on `EdgeRulesModel` is limited to:

- `load_source(&str) -> Result<(), ParseErrors>`
- `to_runtime(self) -> Result<EdgeRulesRuntime, LinkingError>`
- `to_runtime_snapshot(&mut self) -> Result<EdgeRulesRuntime, LinkingError>`

`load_source` tokenizes the provided string via `tokenizer::parser::tokenize` (which itself uses
`ASTBuilder` to reduce the token stream) and feeds the resulting expressions or definitions into
`ContextObjectBuilder`. Once the caller is ready to execute rules, the builder is turned into a
`ContextObject` tree and linked (`link_parts`) to produce an `EdgeRulesRuntime`.

This approach is convenient for instant execution services/CLI, but is insufficient for EdgeRules Editor GUI
where user edits and executes code on the fly.

1. Build models programmatically (e.g., from JSON/JsValue) instead of EdgeRules DSL strings.
2. Incrementally compose or update expressions, user-defined types, and user functions.
3. Inspect and remove previously defined items.

## Design Goals

1. Provide explicit CRUD functions on `EdgeRulesModel` for expressions, user types, user functions,
   and context objects.
2. Keep the behavior consistent with the parser-generated AST (`ContextObjectBuilder` remains the
   single source of truth).
3. Preserve the validation semantics that already exist:
   - Duplicate field names remain an error (`ParseErrorEnum::UnknownError` today).
   - User function parameter names must stay unique (`FunctionDefinition::build`).
   - Type alias/name collisions remain errors.
4. Avoid introducing new error types unless absolutely required; reuse `ParseErrorEnum`,
   `ParseErrors`, and `LinkingError`.
5. Do not silently mutate nested structures—the caller must intentionally provide nested
   `ContextObject`s (exactly how `load_source` behaves today).

## Core Building Blocks

## EdgeRules CRUD API

Clarifications:
1. set operations override existing entries if the name already exists.
2. remove operations do nothing if the name does not exist.
3. get operations return None if the name does not exist.

### Expressions API

```
set_expression(field_name: &str, expression: ExpressionEnum) -> Result<(), ParseErrorEnum>
remove_expression(field_name: &str) -> Result<(), ParseErrorEnum>
get_expression(field_name: &str) -> Option<Rc<RefCell<ExpressionEntry>>>
```

#### Example 1

After applying `model.set_expression("enabled", ExpressionEnum::from(true))?;`
appends an internal structure:
```
{
    enabled: true
}
```

After applying `model.set_expression("other.enabled", ExpressionEnum::from(true))?;`
an exception should be thrown because `other` context does not exist yet.

After applying `model.set_expression("other", ExpressionEnum::from(ContextObjectBuilder::new().build()?))?;`
an internal structure is:
```
{
    enabled: true
    other: {
    }
}
```

After applying `model.set_expression("other.enabled", ExpressionEnum::from(true))?;`
an internal structure is:
```
{
    enabled: true
    other: {
        enabled: true
    }
}
```

### User Types API

```
set_user_type(type_name: &str, type_definition: UserTypeBody) -> Result<(), ParseErrorEnum>
remove_user_type(type_name: &str) -> Result<(), ParseErrorEnum>
get_user_type(type_name: &str) -> Option<UserTypeBody>
```

#### Example 2

After applying `model.set_user_type("MyType", UserTypeBody::from("string"))?;`
the internal structure is:
```
{
    type MyType: string;
}
```

After applying `model.set_user_type("other.MyType", UserTypeBody::from("string"))?;`
an exception should be thrown because `other` context does not exist yet.

### User Functions API

```
set_user_function(definition: FunctionDefinition, context_path: Option<Vec<&'static str>>) -> Result<(), ParseErrorEnum>
remove_user_function(function_name: &str) -> Result<(), ParseErrorEnum>
get_user_function(function_name: &str) -> Option<Rc<RefCell<MethodEntry>>>
```

#### Example 3

After applying `model.set_user_function(definition, [other])?;` where already existing model looks like this:
```
{
    other: {
    }
}   
```

the internal structure is:
```
{
    other: {
        func newFunction(a,b,c): {
            ...
        }
    }
}
```

### Context Objects & Source ingestion

```
merge_context_object(object: Rc<RefCell<ContextObject>>) -> Result<(), ParseErrorEnum>
append_source(code: &str) -> Result<(), ParseErrors>
```

> _append_source_ is renamed older _load_source_ method. append_source method is rewritten
> to use new API internally.

# Todo:

- [x] ContextObjectBuilder should not produce ParseErrorEnum, because errors are related to duplicates only 
so duplication related error should be returned and handled appropriately. Introduce `DuplicateNameError`
with `NameKind` and `name: String` that is a duplicated name.
- [x] NameKind must also have `UserType` variant and type duplication prevention.
