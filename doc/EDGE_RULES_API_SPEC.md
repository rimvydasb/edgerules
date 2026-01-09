# EdgeRules API Specification

## Overview

EdgeRules provides a lightweight, embeddable rules engine. The system consists of a core Rust library (
`edge-rules-core`) and a WASM wrapper (`edge-rules-wasm`) for usage in web and edge environments.

The API supports two main modes of operation:

1. **Stateless Evaluation** (`DecisionEngine`): One-off evaluation of expressions or fields.
2. **Stateful Decision Service** (`DecisionService`): Maintains a compiled model, allowing for incremental updates and
   repeated execution against requests.

## Portable Format Specification

The **EdgeRules Portable Format** is a JSON-based schema for exchanging models, types, functions, and invocations. It
serves as the canonical serialization format.

### TypeScript Interface

```typescript
export type PortableScalar = string | number | boolean;

export type PortableExpressionString = string;

export type PortableValue =
    | PortableScalar
    | PortableScalar[]
    | PortableObject
    | PortableObject[];

export interface PortableTypeDefinition {
    '@type': 'type';
    '@ref'?: string;

    [key: string]: PortableValue | PortableExpressionString | undefined;
}

export interface PortableFunctionDefinition {
    '@type': 'function';
    '@parameters': Record<string, string | null | undefined>;

    [key: string]: PortableValue | PortableExpressionString;
}

export interface PortableInvocationDefinition {
    '@type': 'invocation';
    '@method': string;
    '@arguments'?: (PortableValue | PortableExpressionString)[];
}

export interface PortableObject {
    [key: string]:
        | PortableValue
        | PortableExpressionString
        | PortableTypeDefinition
        | PortableFunctionDefinition
        | PortableInvocationDefinition;
}

export interface PortableContext extends PortableObject {
    '@version'?: string | number;
    '@model_name'?: string;
}
```

### Common Metadata

* `@version`: Model version string.
* `@model_name`: Human-readable model name.
* `@type`: Discriminator for entry type (`function`, `type`, `invocation`). If omitted, implies a context object or
  static value.

### Entities

#### 1. Function

Defines a reusable user function.

* `@type`: `"function"`
* `@parameters`: Object mapping parameter names to types (or `null` for any).
* `result`: (Optional) Main body expression.
* *Additional keys*: Treated as local context variables or sub-functions.

```json
{
  "isEligible": {
    "@type": "function",
    "@parameters": {
      "age": "number"
    },
    "result": "age >= 18"
  }
}
```

#### 2. Type

Defines a user-defined type schema.

* `@type`: `"type"`
* `@ref`: (Optional) Reference to an existing type (e.g., `<string>`).
* *Body*: If `@ref` is absent, keys define fields and their types (using `<Type>` syntax or nested objects).

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

* `@type`: `"invocation"`
* `@method`: Fully qualified path to the function (e.g., `lib.utils.calc`).
* `@arguments`: Array of expressions (strings, numbers, or portable objects) passed to the function.

```json
{
  "score": {
    "@type": "invocation",
    "@method": "calcScore",
    "@arguments": [
      "input.data"
    ]
  }
}
```

## WASM API Specification

### `DecisionEngine` (Stateless)

Stateless utility for quick evaluation.

* `evaluateAll(code: string): JsValue`
    * Evaluates all fields in the provided EdgeRules DSL string. Returns the resulting context object.
* `evaluateExpression(code: string): JsValue`
    * Evaluates a single expression string.
* `evaluateField(code: string, field: string): JsValue`
    * Parses the code and evaluates a specific field path within it.

### CRUD Operations

The `DecisionService` provides methods to modify the decision model at runtime.

#### `get(path: string): object | any`

Retrieves the value or definition at the specified path.

- **Parameters:**
    - `path`: Dot-separated path to the field (e.g., `"rules.eligibility"`) or array element (e.g., `"rules[0]"`).
- **Returns:** The value at the path. If the path points to a context, it returns a JSON object.
- **Throws:**
    - `EntryNotFoundError`: If the path does not exist.
    - `WrongFieldPathError`: If the path is invalid, empty, out of bounds for arrays, or index is negative.

#### `set(path: string, value: any): object | any`

Sets a value or definition at the specified path.

- **Parameters:**
    - `path`: Dot-separated path to the field (e.g., `"rules.eligibility"`) or array element (e.g., `"rules[0]"`).
    - `value`: The value to set. Can be a primitive, object, or a function definition.
- **Returns:** The set value.
- **Throws:**
    - `WrongFieldPathError`: If the path is invalid or attempts to add an array element with gaps (e.g., setting index 5 when length is 3).
    - `LinkingError`: If the new value's type is incompatible with the existing structure or array type.
    - `PortableError`: For other parsing or structural errors.

#### `remove(path: string): void`

Removes the entry at the specified path.

- **Parameters:**
    - `path`: Dot-separated path to the field (e.g., `"rules.eligibility"`) or array element (e.g., `"rules[0]"`).
- **Returns:** `void`.
- **Throws:**
    - `EntryNotFoundError`: If the path does not exist.
    - `WrongFieldPathError`: If the path is invalid, out of bounds for arrays, or index is negative.

#### `rename(oldPath: string, newPath: string): void`

Renames an entry (field, function, type, or invocation) from `oldPath` to `newPath`.

- **Parameters:**
    - `oldPath`: The current full path of the entry (e.g., `"applicant.age"`).
    - `newPath`: The new full path of the entry (e.g., `"applicant.years"`).
- **Returns:** `void` (or boolean true in some bindings).
- **Throws:**
    - `EntryNotFoundError`: If `oldPath` does not exist.
    - `WrongFieldPathError`:
        - If `newPath` is invalid or empty.
        - If `oldPath` and `newPath` do not share the same parent context (you cannot move entries between contexts).
    - `DuplicateNameError`: If an entry with the name of `newPath` already exists in the target context.
    - `LinkingError`: If the rename breaks existing references (e.g., referencing a function that was renamed without updating the call site). Note: updating references is not automatic.

**Array Access Exceptions:**

*   **Set:**
    *   **No Gaps:** You cannot add an element at an index that skips existing positions (e.g., `arr[5]` if length is 2).
    *   **Overwrite:** Overwriting an existing index replaces the value without shifting.
    *   **Type Safety:** Setting an element must respect the array's type (e.g., cannot put a string in a number array).
*   **Remove:**
    *   **Shift:** Removing an element (e.g., `arr[1]`) shifts subsequent elements left (index 2 becomes 1).
    *   **Empty:** Removing the last element leaves an empty array.
*   **General:**
    *   **Bounds:** accessing `arr[10]` when length is 5 throws `WrongFieldPathError`.
    *   **Negative Index:** `arr[-1]` throws `WrongFieldPathError`.

**Rename Exceptions:**

*   **Same Context:** Renaming `user.firstName` to `customer.firstName` throws `WrongFieldPathError` because the parent context changes from `user` to `customer`.
*   **Collision:** Renaming `a` to `b` when `b` exists throws `DuplicateNameError`.
*   **Root vs Nested:** Renaming a root element to a nested path (or vice versa) throws `WrongFieldPathError`.

### Error Handling

## Rust API Specification

### `EdgeRulesModel` (`crates/core`)

The primary struct for building and manipulating the AST before compilation.

- `new() -> Self`: Creates an empty model.
- `append_source(code: &str) -> Result<(), ParseErrors>`: Parses and appends source code to the model.
- `set_expression(path: &str, expr: ExpressionEnum) -> Result<(), ContextQueryErrorEnum>`: Inserts/updates an expression.
- `set_user_function(def: FunctionDefinition, context_path: Option<Vec<&str>>) -> Result<(), ContextQueryErrorEnum>`: Inserts/updates a function.
- `set_user_type(path: &str, body: UserTypeBody) -> Result<(), ContextQueryErrorEnum>`: Inserts/updates a type definition.
- `set_invocation(path: &str, spec: InvocationSpec) -> Result<(), ContextQueryErrorEnum>`: Inserts a function invocation.
- `remove_expression(path: &str) -> Result<(), ContextQueryErrorEnum>`: Removes an expression.
- `remove_user_type(path: &str) -> Result<(), ContextQueryErrorEnum>`: Removes a type definition.
- `remove_user_function(path: &str) -> Result<(), ContextQueryErrorEnum>`: Removes a user function.
- `rename_entry(old_path: &str, new_path: &str) -> Result<(), ContextQueryErrorEnum>`: Renames an entity within its context.
- `get_expression(path: &str) -> Result<Rc<RefCell<ExpressionEntry>>, ContextQueryErrorEnum>`: Retrieves an expression entry.
- `get_expression_type(path: &str) -> Result<ValueType, ContextQueryErrorEnum>`: Retrieves the type of an expression.
- `get_user_type(path: &str) -> Result<UserTypeBody, ContextQueryErrorEnum>`: Retrieves a user type definition.
- `get_user_function(path: &str) -> Result<Rc<RefCell<MethodEntry>>, ContextQueryErrorEnum>`: Retrieves a user function entry.
- `to_runtime() -> Result<EdgeRulesRuntime, LinkingError>`: Consumes the model and compiles it into a runtime.
- `to_runtime_snapshot() -> Result<EdgeRulesRuntime, LinkingError>`: Compiles the model into a runtime without consuming it.

### `DecisionService` (`crates/core`)

Wrapper around `EdgeRulesModel` and `EdgeRulesRuntime` to facilitate service-oriented execution.

- `from_source(source: &str) -> Result<Self, EvalError>`: Creates a service from source code.
- `from_context(context: Rc<RefCell<ContextObject>>) -> Result<Self, EvalError>`: Creates a service from an existing context object.
- `from_model(model: EdgeRulesModel) -> Result<Self, EvalError>`: Creates a service from a model.
- `execute(&mut self, method: &str, request: ValueEnum) -> Result<ValueEnum, EvalError>`: Executes a service method.
- `evaluate_field(&mut self, path: &str) -> Result<ValueEnum, EvalError>`: Evaluates a specific field path.
- `get_linked_type(&mut self, path: &str) -> Result<ValueType, ContextQueryErrorEnum>`: Retrieves the linked type of a field.
- `rename_entry(&mut self, old_path: &str, new_path: &str) -> Result<(), EvalError>`: Renames an entry within the service.
- `ensure_linked(&mut self) -> Result<(), EvalError>`: Ensures the underlying runtime is linked and up-to-date.
- `get_model(&mut self) -> Rc<RefCell<EdgeRulesModel>>`: Returns `Rc<RefCell<EdgeRulesModel>>` for mutation (requires `mutable_decision_service` feature).

## Limitations

1. **Single Active Service (WASM)**: The WASM binding currently uses a thread-local singleton for the active
   `DecisionService` controller. Only one service instance can be active at a time per WASM module instance.
2. **Invocation Arguments**: Arguments in `@arguments` must be resolvable expressions.
3. **Metadata**: Only specific metadata keys (`@version`, `@model_name`) are preserved in the root context.
