# EdgeRules API Specification

## Overview

EdgeRules provides a lightweight, embeddable rules engine. The system consists of a core Rust library
(`edge-rules`) and a WASM wrapper (`edge-rules-wasi`) for usage in web and edge environments.

The API supports two main modes of operation:

1. **Stateless Evaluation** (`DecisionEngine`): One-off evaluation of expressions or fields.
2. **Stateful Decision Service** (`DecisionService`): Maintains a compiled model, allowing for incremental updates and
   repeated execution against requests.

## Portable Format Specification

The **EdgeRules Portable Format** is a JSON-based schema for exchanging models, types, functions, and invocations. It
serves as the canonical serialization format.

> EdgeRules Portable Format is designed to exchange the code rather than the data. For this reason, user must know
> applied exceptions:
>
> 1. Variables are represented as JSON stings: { "var": "path.to.variable" }.
> 2. Strings are escaped with quotes: { "const": "\"string value\"" } or { "const": "'string value'" }.

### TypeScript Interface

```typescript
export type PortableScalar = string | number | boolean;

export type PortableExpressionString = string;

export type PortableValue = PortableScalar | PortableObject | PortableValue[];

export interface PortableTypeDefinition {
    "@type": "type";
    "@ref"?: string;

    [key: string]: PortableValue | PortableExpressionString | undefined;
}

export interface PortableFunctionDefinition {
    "@type": "function";
    "@parameters": Record<string, string | null | undefined>;

    [key: string]: PortableValue | PortableExpressionString;
}

export interface PortableInvocationDefinition {
    "@type": "invocation";
    "@method": string;
    "@arguments"?: (PortableValue | PortableExpressionString)[];
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
    "@version"?: string | number;
    "@model_name"?: string;
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
    stage: "linking" | "runtime";
}
```

### Common Metadata

- `@version`: Model version string.
- `@model_name`: Human-readable model name.
- `@type`: Discriminator for entry type (`function`, `type`, `invocation`). If omitted, implies a context object or
  static value.

### Entities

#### 1. Function

Defines a reusable user function.

- `@type`: `"function"`
- `@parameters`: Object mapping parameter names to types (or `null` for any).
- `result`: (Optional) Main body expression.
- _Additional keys_: Treated as local context variables or sub-functions.

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

- `@type`: `"type"`
- `@ref`: (Optional) Reference to an existing type (e.g., `<string>`).
- _Body_: If `@ref` is absent, keys define fields and their types (using `<Type>` syntax or nested objects).

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

- `@type`: `"invocation"`
- `@method`: Fully qualified path to the function (e.g., `lib.utils.calc`).
- `@arguments`: Array of expressions (strings, numbers, or portable objects) passed to the function.

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

- **Parameters:**
    - `input`: The EdgeRules DSL source code (string) or a Portable Context object.
        - If `input` is a string:
            - If it is wrapped in `{}` (e.g., `{ a: 1 }`), it is treated as a full model.
            - Otherwise (e.g., `1 + 2`), it is treated as a single expression.
    - `field`: (Optional) The dot-separated path to the field to evaluate.
        - If provided, only this field is evaluated.
        - **Note:** Field path is not applicable if `input` is a single expression string.
- **Returns:**
    - If `field` is provided: The result of that field.
    - If `input` is a single expression (and no `field`): The result of the expression.
    - If `input` is a model (and no `field`): The fully evaluated context object.
- **Throws:**
    - `PortableError`: For syntax errors, linking errors, runtime errors, or invalid usage (e.g., field path with
      expression).

#### `printExpressionJs(code: string): string`

(Requires `to_js` feature) Transpiles an EdgeRules expression into a JavaScript expression.

- **Parameters:**
    - `code`: The EdgeRules expression to transpile.
- **Returns:** A string containing the equivalent JavaScript code.
- **Throws:**
    - `PortableError`: If the expression cannot be parsed or transpiled.

#### `printModelJs(code: string): string`

(Requires `to_js` feature) Transpiles an entire EdgeRules model into a JavaScript module/object.

- **Parameters:**
    - `code`: The EdgeRules DSL model code.
- **Returns:** A string containing the equivalent JavaScript code.
- **Throws:**
    - `PortableError`: If the model cannot be parsed or transpiled.

> **Deprecated:** `evaluateAll`, `evaluateExpression`, and `evaluateField` are deprecated in favor of `evaluate`.

### `DecisionService` (Stateful)

The `DecisionService` maintains a compiled model, allowing for incremental updates and repeated execution against
requests.

#### Initialization

- `new DecisionService(model: string | object)`
    - Creates a new decision service.
    - **Parameters:**
        - `model`: Can be an EdgeRules DSL string or a `PortableContext` (JSON) object.
    - **Note:** The WASM binding currently uses a thread-local singleton; initializing a new `DecisionService` replaces
      the previous one for the WASM module instance.

#### Execution

- `execute(method: string, args?: any | any[]): any`
    - Executes a function defined in the model or evaluates a field by path.
    - **Parameters:**
        - `method`: The name or path of the function or field to execute/evaluate.
        - `args`: (Optional) The input argument or an array of arguments to pass to the function.
            - If omitted (`null` or `undefined`), the path is evaluated as a field.
            - If an array is provided, it is treated as a list of arguments for function execution.
            - Providing an empty array `[]` indicates a function execution with no arguments.
    - **Returns:** The result of the execution.

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
    - `path`: Dot-separated path to the field (e.g., `"rules.eligibility"`) or `*` for the entire model schema.
- **Returns:** The type definition.
    - For primitives: returns a string (e.g., `"number"`, `"string"`, `"boolean"`).
    - For functions: returns the return type of the function (e.g., `"number"` or a complex object type).
    - For types: returns the structure of the type (e.g., `{ "name": "string", "age": "number" }`).
    - For wildcard (`*`): returns a JSON object describing the schema of all fields and sub-contexts, bypassing type and
      function definitions.
- **Throws:**
    - `EntryNotFoundError`: If the path does not exist.
    - `WrongFieldPathError`: If the path is invalid or empty.

**Array Access Exceptions:**

- **Set:**
    - **No Gaps:** You cannot add an element at an index that skips existing positions (e.g., `arr[5]` if length is 2).
    - **Overwrite:** Overwriting an existing index replaces the value without shifting.
    - **Type Safety:** Setting an element must respect the array's type (e.g., cannot put a string in a number array).
- **Remove:**
    - **Shift:** Removing an element (e.g., `arr[1]`) shifts subsequent elements left (index 2 becomes 1).
    - **Empty:** Removing the last element leaves an empty array.
- **General:**
    - **Bounds:** accessing `arr[10]` when length is 5 throws `WrongFieldPathError`.
    - **Negative Index:** `arr[-1]` throws `WrongFieldPathError`.

**Rename Exceptions:**

- **Same Context:** Renaming `user.firstName` to `customer.firstName` throws `WrongFieldPathError` because the parent
  context changes from `user` to `customer`.
- **Collision:** Renaming `a` to `b` when `b` exists throws `DuplicateNameError`.
- **Root vs Nested:** Renaming a root element to a nested path (or vice versa) throws `WrongFieldPathError`.

### JavaScript Example

```javascript
import {DecisionEngine, DecisionService} from "edge-rules";

// 1. Stateless Evaluation
const code = `
    {
        input: 10
        factor: 2
        result: input * factor
    }
`;
// Evaluate a specific field
const result = DecisionEngine.evaluate(code, "result");
console.log(result); // 20

// 2. Stateful Decision Service
const model = {
    "@version": "1.0",
    taxRate: 0.2,
    calculateTax: {
        "@type": "function",
        "@parameters": {amount: "number"},
        result: "amount * taxRate",
    },
};

// Initialize service with a portable model
const service = new DecisionService(model);

// Execute a function
const tax = service.execute("calculateTax", 100);
console.log(tax.result); // 20

// Modify the model at runtime
service.set("taxRate", 0.25);
const newTax = service.execute("calculateTax", 100);
console.log(newTax.result); // 25

// Inspect types
const taxRateType = service.getType("taxRate");
console.log(taxRateType); // "number"

const funcType = service.getType("calculateTax");
console.log(funcType); // "number"

try {
    service.execute("calculateTax", "invalid argument");
} catch (e) {
    console.error("Error Type:", e.error.type);
    console.error("Location:", e.location);
    console.error("Expression:", e.expression);
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
- `get_expression_type(path: &str) -> Result<ValueType, ContextQueryErrorEnum>`: Retrieves the type of expression.
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
- `execute(&mut self, path: &str, args: Option<Vec<ValueEnum>>) -> Result<ValueEnum, EvalError>`: Executes a service
  method or evaluates a field.
- `execute_method(&mut self, method: &str, args: Vec<ValueEnum>) -> Result<ValueEnum, EvalError>`: Executes a service
  method with multiple arguments.
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

# Next Steps

Currently `getType`, when called on complex type, collects types of function and type definitions. That is incorrect. It
should return the type of all fields in the complex type and bypass all definitions, so, for the code:

```edgerules
{
    func add(a, b): a + b
    type User: {
        name: <string>
        age: <number>
    }
    existing: "existing value"
}
```

```javascript
const type = service.getType("*");
assert.deepEqual(type, {
    existing: "string",
});
```

**Complex and Simple Expression types:**

- [x] Find `pub fn get_type(&self, field_path: &str) -> Result<ValueType, ContextQueryErrorEnum>` and understand how it
  works. Pay attention that `ValueType` is returned. Both function and type definitions "types" could easily fit to
  `ValueType::ObjectType` or other `ValueType` variants.
- [x] Fix `get_type` to bypass all definitions and collect types of fields and sub-contexts only.
- [x] Fix `JavaScript` tests, for example `it('renames an invocation', () => {` is known to be not properly working,
  because it captures function definition.

**Function Return Types:**

- [x] Fix `to_schema` for `ContextObject` - it must completely bypass and ignore definitions: type definitions and
  function definitions.
- [x] Ensure that WASM API `getType` works correctly when type is requested for the function, e.g.

```javascript
const type = service.getType("add");
assert.deepEqual(type, "number")
```

- [x] For all user defined functions in `evaluation_user_functions_tests.rs` call `get_type` and ensure that it returns
  correct return type definition. Pay attention to the fact that some functions have hidden fields. Complex functions
  correctly report their return type hiding all internal variables if `RETURN_EXPRESSION` is used.
- [x] Ensure that `getType` works correctly when type is requested for the inline and complex function. However, inner
  functions and type definitions will not be returned - `getType` for function basically returns function return type.

> Correct type definitions are asserted such as
> `assert_eq!(runtime.get_type("*").unwrap().to_string(), "{field: number; nested: {val: number}}");`

> Note that for functions `get_type` basically returns already linked function type.

**Types:**

- [x] When `getType` is called on the type, e.g. `service.getType("User")`, it should return the type definition of the
  `User` type, such as example below (the whole behaviours is very similar to `service.get("User")`), but only fields
  and types are returned:

```json
{
  "name": "string",
  "age": "number"
}
```

**Completing:**

- [x] Whenever possible use `rt.get_type("*").unwrap().to_string()` instead of `to_schema` in all the tests! Review the
  tests to ensure that `get_type("*").unwrap().to_string()` is used instead of `to_schema` for type assertions.
- [x] Update Rust tests
- [x] Update JavaScript tests
- [x] Update documentation in EDGE_RULES_API_SPEC.md
- [x] If task is completed, mark it as done.

# ContextObject Type

The problem is that `ValueType::ObjectType(Rc<RefCell<ContextObject>>)` represents `ContextObject` type - that is
incorrect, because `ContextObject` stores a lot of things including definitions that are absolutely not tyype related.

- [ ] Change object type tp be represented as string to type map, such as
  `ValueType::ObjectType(Vec<(&'static str, ValueType)>)` - strings are field names and `ValueType` are field types.
  This will allow to bypass all definitions and focus only on fields and sub-contexts.
- [ ] Comparing object types field order does not matter. Add a couple of tests to prove that.
- [ ] Correctly implement `impl TypedValue for ContextObject` to return correct type information based on the fields and
  sub-contexts, bypassing all definitions. Based on the best practices and performance considerations, it might be
  needed to cache the type and carefully rebuild it, because `ContextObject` appears to be mutable.
- [ ] Remove `to_schema` method from `ContextObject` and replace all its usages with `get_type().to_string()` (from
  TypedValue) that should print the type.