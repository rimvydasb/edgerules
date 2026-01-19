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

> EdgeRules Portable Format is designed to exchange the code rather than the data. For this reason, user must know
> applied exceptions:
> 1. Variables are represented as JSON stings: { "var": "path.to.variable" }.
> 2. Strings are escaped with quotes: { "const": "\"string value\"" } or { "const": "'string value'" }.

### TypeScript Interface

```typescript
export type PortableScalar = string | number | boolean;

export type PortableExpressionString = string;

export type PortableValue =
    | PortableScalar
    | PortableObject
    | PortableValue[];

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

export interface PortableError {
    message?: string; // formatted message (might be deprecated)
    error: {
        type: string; // `FieldNotFound`, `CyclicReference`, `TypesNotCompatible`, etc.
        fields?: string[]; // for now only `FieldNotFound` and `CyclicReference` uses it (TBC, must be ordered map of key and string)
        subject?: string; // Used by `DifferentTypesDetected` and `TypesNotCompatible`, and `FieldNotFound` (deprecated, must be in fields)
        unexpected?: string; // used only for `TypesNotCompatible` (deprecated)
        expected?: string[]; // used only for `TypesNotCompatible` (deprecated)
        message?: string; // raw error message without formatting, now only `EvalError` uses it, @TBC
    };
    location: string; // Fully qualified path of the problem, e.g. "calculations.takeDate.year"
    expression: string; // Problematic expression snippet, e.g.  "d.nonexistent"
    stage: 'linking' | 'runtime';
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

Stateless utility for quick evaluation. These methods do not persist any state between calls.

#### `evaluate(input: string | object, field?: string): any`

Evaluates the provided EdgeRules code or portable model.

* **Parameters:**
    * `input`: The EdgeRules DSL source code (string) or a Portable Context object.
        * If `input` is a string:
            * If it is wrapped in `{}` (e.g., `{ a: 1 }`), it is treated as a full model.
            * Otherwise (e.g., `1 + 2`), it is treated as a single expression.
    * `field`: (Optional) The dot-separated path to the field to evaluate.
        * If provided, only this field is evaluated.
        * **Note:** Field path is not applicable if `input` is a single expression string.
* **Returns:**
    * If `field` is provided: The result of that field.
    * If `input` is a single expression (and no `field`): The result of the expression.
    * If `input` is a model (and no `field`): The fully evaluated context object.
* **Throws:**
    * `PortableError`: For syntax errors, linking errors, runtime errors, or invalid usage (e.g., field path with
      expression).

#### `printExpressionJs(code: string): string`

(Requires `to_js` feature) Transpiles an EdgeRules expression into a JavaScript expression.

* **Parameters:**
    * `code`: The EdgeRules expression to transpile.
* **Returns:** A string containing the equivalent JavaScript code.
* **Throws:**
    * `PortableError`: If the expression cannot be parsed or transpiled.

#### `printModelJs(code: string): string`

(Requires `to_js` feature) Transpiles an entire EdgeRules model into a JavaScript module/object.

* **Parameters:**
    * `code`: The EdgeRules DSL model code.
* **Returns:** A string containing the equivalent JavaScript code.
* **Throws:**
    * `PortableError`: If the model cannot be parsed or transpiled.

> **Deprecated:** `evaluateAll`, `evaluateExpression`, and `evaluateField` are deprecated in favor of `evaluate`.

### `DecisionService` (Stateful)

The `DecisionService` maintains a compiled model, allowing for incremental updates and repeated execution against
requests.

#### Initialization

* `new DecisionService(model: string | object)`
    * Creates a new decision service.
    * **Parameters:**
        * `model`: Can be an EdgeRules DSL string or a `PortableContext` (JSON) object.
    * **Note:** The WASM binding currently uses a thread-local singleton; initializing a new `DecisionService` replaces
      the previous one for the WASM module instance.

#### Execution

* `execute(method: string, request: any): any`
    * Executes a function defined in the model.
    * **Parameters:**
        * `method`: The name or path of the function to execute.
        * `request`: The input data to pass to the function.
    * **Returns:** The result of the function execution.

#### CRUD Operations

The `DecisionService` provides methods to modify the decision model at runtime.

#### `get(path: string): object`

Retrieves the value or definition at the specified path.

- **Parameters:**
    - `path`: Dot-separated path to the field (e.g., `"rules.eligibility"`) or array element (e.g., `"rules[0]"`).
- **Returns:** The value at the path. If the path points to a context, it returns a JSON object.
- **Throws:**
    - `EntryNotFoundError`: If the path does not exist.
    - `WrongFieldPathError`: If the path is invalid, empty, out of bounds for arrays, or index is negative.

#### `set(path: string, value: PortableValue): object`

Sets a value or definition at the specified path.

- **Parameters:**
    - `path`: Dot-separated path to the field (e.g., `"rules.eligibility"`) or array element (e.g., `"rules[0]"`).
    - `value`: The value to set. Can be a primitive, object, or a function definition.
- **Returns:** The set value.
- **Throws:**
    - `WrongFieldPathError`: If the path is invalid or attempts to add an array element with gaps (e.g., setting index 5
      when length is 3).
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
    - `LinkingError`: If the rename breaks existing references (e.g., referencing a function that was renamed without
      updating the call site). Note: updating references is not automatic.

#### `getType(path: string): string | object`

Retrieves the type definition of the entry at the specified path.

- **Parameters:**
    - `path`: Dot-separated path to the field (e.g., `"rules.eligibility"`).
- **Returns:** The type definition.
    - For primitives: returns a string (e.g., `"number"`, `"string"`, `"boolean"`).
    - For complex types: returns a JSON object describing the structure (e.g., `{ "name": "string", "age": "number" }`).
    - For wildcard `"*"`: returns the entire model schema.
- **Throws:**
    - `EntryNotFoundError`: If the path does not exist.
    - `WrongFieldPathError`: If the path is invalid or empty.

**Array Access Exceptions:**

* **Set:**
    * **No Gaps:** You cannot add an element at an index that skips existing positions (e.g., `arr[5]` if length is 2).
    * **Overwrite:** Overwriting an existing index replaces the value without shifting.
    * **Type Safety:** Setting an element must respect the array's type (e.g., cannot put a string in a number array).
* **Remove:**
    * **Shift:** Removing an element (e.g., `arr[1]`) shifts subsequent elements left (index 2 becomes 1).
    * **Empty:** Removing the last element leaves an empty array.
* **General:**
    * **Bounds:** accessing `arr[10]` when length is 5 throws `WrongFieldPathError`.
    * **Negative Index:** `arr[-1]` throws `WrongFieldPathError`.

**Rename Exceptions:**

* **Same Context:** Renaming `user.firstName` to `customer.firstName` throws `WrongFieldPathError` because the parent
  context changes from `user` to `customer`.
* **Collision:** Renaming `a` to `b` when `b` exists throws `DuplicateNameError`.
* **Root vs Nested:** Renaming a root element to a nested path (or vice versa) throws `WrongFieldPathError`.

### JavaScript Example

```javascript
import {DecisionEngine, DecisionService} from 'edge-rules';

// 1. Stateless Evaluation
const code = `
    {
        input: 10
        factor: 2
        result: input * factor
    }
`;
// Evaluate a specific field
const result = DecisionEngine.evaluate(code, 'result');
console.log(result); // 20

// 2. Stateful Decision Service
const model = {
    '@version': '1.0',
    'taxRate': 0.2,
    'calculateTax': {
        '@type': 'function',
        '@parameters': {'amount': 'number'},
        'result': 'amount * taxRate'
    }
};

// Initialize service with a portable model
const service = new DecisionService(model);

// Execute a function
const tax = service.execute('calculateTax', 100);
console.log(tax.result); // 20

// Modify the model at runtime
service.set('taxRate', 0.25);
const newTax = service.execute('calculateTax', 100);
console.log(newTax.result); // 25

// Inspect types
const taxRateType = service.getType('taxRate');
console.log(taxRateType); // "number"

const funcType = service.getType('calculateTax');
console.log(funcType);
// Output:
// {
//   "@type": "function",
//   "@parameters": { "amount": "number" },
//   "result": "number"
// }

try {
    service.execute('calculateTax', 'invalid argument');
} catch (e) {
    console.error('Error Type:', e.error.type);
    console.error('Location:', e.location);
    console.error('Expression:', e.expression);
}
```

## Rust API Specification

### `EdgeRulesModel` (`crates/core`)

The primary struct for building and manipulating the AST before compilation.

- `new() -> Self`: Creates an empty model.
- `append_source(code: &str) -> Result<(), ParseErrors>`: Parses and appends source code to the model.
- `set_expression(path: &str, expr: ExpressionEnum) -> Result<(), ContextQueryErrorEnum>`: Inserts/updates an
  expression.
- `set_user_function(def: FunctionDefinition, context_path: Option<Vec<&str>>) -> Result<(), ContextQueryErrorEnum>`:
  Inserts/updates a function.
- `set_user_type(path: &str, body: UserTypeBody) -> Result<(), ContextQueryErrorEnum>`: Inserts/updates a type
  definition.
- `set_invocation(path: &str, spec: InvocationSpec) -> Result<(), ContextQueryErrorEnum>`: Inserts a function
  invocation.
- `remove_expression(path: &str) -> Result<(), ContextQueryErrorEnum>`: Removes an expression.
- `remove_user_type(path: &str) -> Result<(), ContextQueryErrorEnum>`: Removes a type definition.
- `remove_user_function(path: &str) -> Result<(), ContextQueryErrorEnum>`: Removes a user function.
- `rename_entry(old_path: &str, new_path: &str) -> Result<(), ContextQueryErrorEnum>`: Renames an entity within its
  context.
- `get_expression(path: &str) -> Result<Rc<RefCell<ExpressionEntry>>, ContextQueryErrorEnum>`: Retrieves an expression
  entry.
- `get_expression_type(path: &str) -> Result<ValueType, ContextQueryErrorEnum>`: Retrieves the type of an expression.
- `get_user_type(path: &str) -> Result<UserTypeBody, ContextQueryErrorEnum>`: Retrieves a user type definition.
- `get_user_function(path: &str) -> Result<Rc<RefCell<MethodEntry>>, ContextQueryErrorEnum>`: Retrieves a user function
  entry.
- `to_runtime() -> Result<EdgeRulesRuntime, LinkingError>`: Consumes the model and compiles it into a runtime.
- `to_runtime_snapshot() -> Result<EdgeRulesRuntime, LinkingError>`: Compiles the model into a runtime without consuming
  it.

### `DecisionService` (`crates/core`)

Wrapper around `EdgeRulesModel` and `EdgeRulesRuntime` to facilitate service-oriented execution.

- `from_source(source: &str) -> Result<Self, EvalError>`: Creates a service from source code.
- `from_context(context: Rc<RefCell<ContextObject>>) -> Result<Self, EvalError>`: Creates a service from an existing
  context object.
- `from_model(model: EdgeRulesModel) -> Result<Self, EvalError>`: Creates a service from a model.
- `execute(&mut self, method: &str, request: ValueEnum) -> Result<ValueEnum, EvalError>`: Executes a service method.
- `evaluate_field(&mut self, path: &str) -> Result<ValueEnum, EvalError>`: Evaluates a specific field path.
- `get_linked_type(&mut self, path: &str) -> Result<ValueType, ContextQueryErrorEnum>`: Retrieves the linked type of a
  field.
- `rename_entry(&mut self, old_path: &str, new_path: &str) -> Result<(), EvalError>`: Renames an entry within the
  service.
- `ensure_linked(&mut self) -> Result<(), EvalError>`: Ensures the underlying runtime is linked and up-to-date.
- `get_model(&mut self) -> Rc<RefCell<EdgeRulesModel>>`: Returns `Rc<RefCell<EdgeRulesModel>>` for mutation (requires
  `mutable_decision_service` feature).

## Limitations

1. **Single Active Service (WASM)**: The WASM binding currently uses a thread-local singleton for the active
   `DecisionService` controller. Only one service instance can be active at a time per WASM module instance.
2. **Invocation Arguments**: Arguments in `@arguments` must be resolvable expressions.
3. **Metadata**: Only specific metadata keys (`@version`, `@model_name`) are preserved in the root context.

# Next Steps:

- [x] Instead of three methods `evaluateAll`, `evaluateExpression`, and `evaluateField`, provide a single method
  `evaluate`:
    - [x] New `evaluate` method accepts code in string or portable format, and an optional field path.
    - [x] If field path is provided, evaluates that field; otherwise, evaluates the entire model.
    - [x] It could be that in `wasm_convert.rs`, old methods `evaluate_all_inner` and `evaluate_field_inner` are merged
      into a single `evaluate_inner` method that accepts both code and portable with optional field path.
    - [x] If provided code is properly wrapped with `{}` treat it as a full model; otherwise, treat it as a single
      expression.
        - [x] For a single expression evaluation, use existing `evaluate_expression_str`
    - [x] If single expression is provided and user accidentally provides optional field path, throw an error indicating
      field path is not applicable.
    - [x] Deprecate `evaluateAll`, `evaluateExpression`, and `evaluateField` methods, replace all tests to use a single
      `evaluate` method.
- [x] Update documentation and examples to reflect the new `evaluate` method.
- [x] Update JavaScript and Rust (if needed) tests.
- [x] Add additional tests where portable is accepted as an input for a new `evaluate` method.
- [x] Write unhappy path tests for the new `evaluate` method when user provides single expression with field path.
- [x] Check tasks if completed. Check if unhappy tests are also written for `evaluate` method in JavaScript.
- [x] `DecisionEngine` markdown documentation is a bit messy with gaps - fix the formatting.
- [x] Fix variable does not need to be mutable crates/wasm/src/wasm_convert.rs:40:13 let mut service =
  model_from_portable(input)?; - maybe other linting errors still exist?
- [x] Run just test-node again - check if any problems.
- [x] Add a possibility to pass method name to `evaluate` method to execute a function directly.
- [x] Throw exception if provided method does not exist, or it has arguments. If method has arguments, then
  `DecisionService` should be used instead - mention this in exception and documentation.
- [x] Add JavaScript test for this case: happy and unhappy paths.
- [x] The new `evaluate` method should perfectly fine execute any method if it does not have arguments:

```edgerules
{
    value: 42
    func main(): {
        result: value * 10
    }
}
```

Then calling `evaluate(code, 'main')` should return `{result: 420}`, because method return is an object. Add a test for
this case if not exists.

- [x] Check tasks if completed.
