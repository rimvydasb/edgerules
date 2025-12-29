# EdgeRules API Specification

## Overview

EdgeRules provides a lightweight, embeddable rules engine. The system consists of a core Rust library (`edge-rules-core`) and a WASM wrapper (`edge-rules-wasm`) for usage in web and edge environments.

The API supports two main modes of operation:
1.  **Stateless Evaluation** (`DecisionEngine`): One-off evaluation of expressions or fields.
2.  **Stateful Decision Service** (`DecisionService`): Maintains a compiled model, allowing for incremental updates and repeated execution against requests.

## Portable Format Specification

The **EdgeRules Portable Format** is a JSON-based schema for exchanging models, types, functions, and invocations. It serves as the canonical serialization format.

### Common Metadata
*   `@version`: Model version string.
*   `@model_name`: Human-readable model name.
*   `@type`: Discriminator for entry type (`function`, `type`, `invocation`). If omitted, implies a context object or static value.

### Entities

#### 1. Function
Defines a reusable user function.
*   `@type`: `"function"`
*   `@parameters`: Object mapping parameter names to types (or `null` for any).
*   `result`: (Optional) Main body expression.
*   *Additional keys*: Treated as local context variables or sub-functions.

```json
{
  "isEligible": {
    "@type": "function",
    "@parameters": { "age": "number" },
    "result": "age >= 18"
  }
}
```

#### 2. Type
Defines a user-defined type schema.
*   `@type`: `"type"`
*   `@ref`: (Optional) Reference to an existing type (e.g., `<string>`).
*   *Body*: If `@ref` is absent, keys define fields and their types (using `<Type>` syntax or nested objects).

```json
{
  "Customer": {
    "@type": "type",
    "name": "<string>",
    "tags": "<string[]>"
  }
}
```

#### 3. Invocation
Represents a call to a user function within the model structure.
*   `@type`: `"invocation"`
*   `@method`: Fully qualified path to the function (e.g., `lib.utils.calc`).
*   `@arguments`: Array of expressions (strings, numbers, or portable objects) passed to the function.

```json
{
  "score": {
    "@type": "invocation",
    "@method": "calcScore",
    "@arguments": ["input.data"]
  }
}
```

## WASM API Specification

### `DecisionEngine` (Stateless)
Stateless utility for quick evaluation.

*   `evaluateAll(code: string): JsValue`
    *   Evaluates all fields in the provided EdgeRules DSL string. Returns the resulting context object.
*   `evaluateExpression(code: string): JsValue`
    *   Evaluates a single expression string.
*   `evaluateField(code: string, field: string): JsValue`
    *   Parses the code and evaluates a specific field path within it.

### `DecisionService` (Stateful)
Maintains a mutable `EdgeRulesModel`.

*   `constructor(model: JsValue)`
    *   Initializes service with a Portable Format object.
*   `execute(method: string, request: JsValue): JsValue`
    *   Executes a named function in the model using the provided request object as the argument.
*   `set(path: string, object: JsValue): JsValue`
    *   Updates or inserts an entry (expression, function, type, invocation) at the specified path. Returns the updated portable representation.
*   `get(path: string): JsValue`
    *   Retrieves the portable representation of an entry at `path`. Use `"*"` for the full model.
*   `remove(path: string): boolean`
    *   Deletes the entry at `path`.
*   `getType(path: string): JsValue`
    *   Returns the inferred or defined type schema for the entry at `path`.

## Rust API Specification

### `EdgeRulesModel` (`crates/core`)
The primary struct for building and manipulating the AST before compilation.

*   `new() -> Self`: Creates an empty model.
*   `set_expression(path, expr)`: Inserts/updates an expression.
*   `set_user_function(def, context_path)`: Inserts/updates a function.
*   `set_user_type(path, body)`: Inserts/updates a type definition.
*   `set_invocation(path, spec)`: Inserts a function invocation.
*   `remove_*(path)`: Removes corresponding entities.
*   `to_runtime_snapshot()`: Compiles the model into an `EdgeRulesRuntime` for execution.

### `DecisionService` (`crates/core`)
Wrapper around `EdgeRulesModel` and `EdgeRulesRuntime` to facilitate service-oriented execution.

*   `from_model(EdgeRulesModel) -> Result<Self>`: Creates service from a model.
*   `execute(method, request) -> Result<ValueEnum>`: Executes a service method.
*   `get_model()`: Returns `Rc<RefCell<EdgeRulesModel>>` for mutation.

## Limitations

1.  **Single Active Service (WASM)**: The WASM binding currently uses a thread-local singleton for the active `DecisionService` controller. Only one service instance can be active at a time per WASM module instance.
2.  **Invocation Arguments**: Arguments in `@arguments` must be resolvable expressions.
3.  **Metadata**: Only specific metadata keys (`@version`, `@model_name`) are preserved in the root context.
