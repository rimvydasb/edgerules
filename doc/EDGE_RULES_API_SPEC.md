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

### Error Handling

## Rust API Specification

### `EdgeRulesModel` (`crates/core`)

The primary struct for building and manipulating the AST before compilation.

- `new() -> Self`: Creates an empty model.
- `set_expression(path, expr)`: Inserts/updates an expression.
- `set_user_function(def, context_path)`: Inserts/updates a function.
- `set_user_type(path, body)`: Inserts/updates a type definition.
- `set_invocation(path, spec)`: Inserts a function invocation.
- `remove_*(path)`: Removes corresponding entities.
- `to_runtime_snapshot()`: Compiles the model into an `EdgeRulesRuntime` for execution.

### `DecisionService` (`crates/core`)

Wrapper around `EdgeRulesModel` and `EdgeRulesRuntime` to facilitate service-oriented execution.

- `from_model(EdgeRulesModel) -> Result<Self>`: Creates service from a model.
- `execute(method, request) -> Result<ValueEnum>`: Executes a service method.
- `get_model()`: Returns `Rc<RefCell<EdgeRulesModel>>` for mutation.

## Limitations

1. **Single Active Service (WASM)**: The WASM binding currently uses a thread-local singleton for the active
   `DecisionService` controller. Only one service instance can be active at a time per WASM module instance.
2. **Invocation Arguments**: Arguments in `@arguments` must be resolvable expressions.
3. **Metadata**: Only specific metadata keys (`@version`, `@model_name`) are preserved in the root context.

## Next Steps: Array CRUD Support

- [x] Check what is already implemented for this story part: implementation is not commited yet.
- [x] Implement `set`, `get`, `remove` support for array elements.
- [x] Update `test-decision-service.mjs` with tests covering basic array CRUD operations.
- [x] Unit test Rust `EdgeRulesModel` methods for array element manipulation.
- [x] Add a complex JavaScript test: find out `example_variable_library`, use that decision service definition in
  `test-decision-service.mjs` tests. Apply CRUD operations on `eligibilityDecision` rules. Ensure decision service is
  still executable and produces expected results.
- [x] Apply other CRUD operations on the new `example_variable_library` test in `test-decision-service.mjs`.
- [x] Perform updated code review to ensure quality, consistency and maintainability.

**`set` array element exceptions:**

- [x] Overwriting existing array element should not shift other elements.
- [x] Adding new array element is possible only if previous elements exist (no gaps allowed). Throw
  `WrongFieldPathError`  if user tries to add element at index 5 while only 3 elements exist.
- [x] When setting element that does not match the array element type, I'm expecting `LinkingError`.
- [x] Update `test-unhappy.mjs` to cover these exceptions.
- [x] Update `EDGE_RULES_API_SPEC.md` to explain these exceptions.

**`get` array element exceptions:**

- [x] Throw `WrongFieldPathError` if index is out of bounds, index is negative, or path is not an array.
- [x] Update `test-unhappy.mjs` to cover these exceptions.
- [x] Update `EDGE_RULES_API_SPEC.md` to explain these exceptions.

**`remove` array element exceptions:**

- [x] Throw `WrongFieldPathError` if index is out of bounds, index is negative, or path is not an array.
- [x] Leave empty array when last element is removed.
- [x] Shift remaining elements to fill the gap when an element is removed from the middle.
- [x] Update `test-unhappy.mjs` to cover these exceptions.
- [x] Update `EDGE_RULES_API_SPEC.md` to explain these exceptions.

**Implement additional support when basic array CRUD is done and tested:**

- [ ] Support of matrix (multidimensional) arrays.

## Next Steps: Rename Support

- [ ] Implement `rename` support for renaming fields, functions, types, and invocations so user will be able to do like
  this:

```javascript
// rename nested field `applicant.age` to `applicant.years`
decisionService.rename("applicant.age", "years");

// rename any nested function `eligibility.checkAge` to `verifyAge`
decisionService.rename("eligibility.checkAge", "eligibility.verifyAge");
```

**`rename` element exceptions:**

- [ ] Throw `WrongFieldPathError` if path does not exist, or new name is invalid (empty or contains invalid characters).
- [ ] Expecting `LinkingError` if new name conflicts with existing sibling entry.
- [ ] It is not possible to move entry or rename root context, only renaming last segment of the path is supported.
- [ ] Update `test-unhappy.mjs` to cover these exceptions.
- [ ] Update `EDGE_RULES_API_SPEC.md` to explain these exceptions.
