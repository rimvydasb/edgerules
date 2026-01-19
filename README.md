# EdgeRules

JSON-native business rules for the edge.

## Preface

**EdgeRules** is JSON-native, Domain-Specific Language with statically-typed semantics and
missing data handling that is positioned as a safer alternative of FEEL and other bulkier DMN runtimes.

Main features: simple expressive language for business users and developers
with safety features such as no nulls, no runtime exceptions, no side effects,
cycle-prevention that comes with a small runtime and hardly matched interpretation performance
of Pratt’s top‑down operator precedence.

- Language Reference / Demo: [edgerules-page](https://rimvydasb.github.io/edgerules-page/)
- [Development](AGENTS.md)
- [License](LICENSE)

### Comparison to the similar projects:

- **Jsonnet**: Data-templating language (superset of JSON) for generating JSON/YAML; pure expressions, no side effects.
- **FEEL**: Friendly Enough Expression Language, part of DMN standard; designed for business users to define decision
  logic.

|                          | Jsonnet          | DMN FEEL          | EdgeRules             |
|--------------------------|------------------|-------------------|-----------------------|
| Null value treatment     | N/A              | nn (By Trisotech) | Native Special Values |
| Objects, lists, types    | Yes              | Yes               | Yes                   |
| Strict Types             | No               | No                | Yes                   |
| Time and date operations | No               | Yes               | Yes                   |
| GUI Needed               | No               | Yes, DMN Modeler  | No                    |
| Runtime                  | Rust, WASM, etc. | Java              | Rust, WASM            |
| Purpose                  | Data templating  | Business rules    | Business rules        |

## Deployment Options

EdgeRules is ready for four options based on your requirements:

| Option            | Description | Size (approx) |
|-------------------|-------------|---------------|
| Native CLI        | `just cli`  | ~1.8MB        |
| WASM for Web      | `just web`  | ~400KB        |
| WASM for Node.js  | `just node` | ~400KB        |
| WASM for Wasmtime | `just wasi` | ~1.5MB        |
| Rust Crate        | TBA         |               |

## Features / Roadmap

- [x] Referentially transparent (pure functions, no side effects)
- [x] No null, nil or NaN
- [x] Immutable by default
- [x] Statically typed
- [x] ~ Traceable
- [x] Hard to fail: no exceptions, no nulls, no NaNs, no undefined variables; CLI I/O errors surface as readable
  messages
- [x] Hard to fail: no reference loops (Cycle-reference prevention)
- [ ] Hard to fail: no infinite loops (TBA: optimistic limits strategy)
- [x] Boolean literals (`true`/`false`) and logical operators (`and`, `or`, `xor`, `not`)
- [x] Full DMN FEEL coverage
- [x] Strongly typed and statically typed with type inference
- [ ] Fractional mathematics for infinite precision
- [ ] Infinite lists
- [ ] Structures composition
- [ ] First-class conditions
- [ ] Pattern matching using `match`
- [ ] None coalescing for optionals (`foo ?? bar` yields `foo` if `foo` has a value, otherwise `bar`)

## Basic API Usage (Rust)

EdgeRules exposes a small, stateful API for loading source and evaluating expressions/fields. The typical flow is:

```rust
use edge_rules::runtime::edge_rules::{EdgeRulesModel, EvalError};

fn main() -> Result<(), EvalError> {
    // Create a model builder
    let mut model = EdgeRulesModel::new();

    // Append some code (can be called multiple times to extend/override)
    model.append_source("{ value: 3 }")?;

    // Evaluate a pure expression using the loaded context without consuming the builder
    let runtime = model.to_runtime_snapshot()?;
    let val = runtime.evaluate_expression_str("2 + value")?;
    assert_eq!(val.to_string(), "5");

    // Append more source and reuse the same builder
    model.append_source("{ calendar: { config: { start: 7; end: start + 5 } } }")?;

    // Build a fresh runtime snapshot to evaluate fields
    let runtime = model.to_runtime_snapshot()?;
    let start = runtime.evaluate_field("calendar.config.start")?;
    assert_eq!(start.to_string(), "7");

    let end = runtime.evaluate_field("calendar.config.end")?;
    assert_eq!(end.to_string(), "12");

    Ok(())
}
```

## WASM

EdgeRules provides a structured, object-oriented API for Web and Node.js environments via `wasm_bindgen`.

### Stateless API (DecisionEngine)

Stateless evaluation methods are grouped under the `DecisionEngine` namespace:

- `DecisionEngine.evaluate(input: string | object, field?: string) -> any` – evaluates the provided EdgeRules code or portable model.
    - If `input` is a string wrapped in `{}`, it treats it as a model.
    - If `input` is a string (not wrapped in `{}`), it treats it as an expression.
    - If `input` is an object, it treats it as a portable model.
    - If `field` is provided, it evaluates that specific field (not applicable for expressions).
- `DecisionEngine.printExpressionJs(code: string) -> string` – transpiles an expression to JavaScript code.
- `DecisionEngine.printModelJs(code: string) -> string` – transpiles a model to JavaScript code.

> **Deprecated:** `evaluateAll`, `evaluateExpression`, and `evaluateField` are supported but deprecated.

### Stateful API (DecisionService)

Stateful decision services are managed via the `DecisionService` class. This wrapper manages the underlying thread-local
controller.

- `new DecisionService(model: object | string)` – consumes a portable model (object or string), initializes the service,
  and stores it in a thread-local slot.
- `service.model -> object` – (getter) returns the current normalized model snapshot.
- `service.execute(service_method: string, decision_request: any) -> object` – executes `service_method` against the
  stored service using a portable request value.
- `service.get(path: string) -> any` – fetches the portable model entry stored at `path` (use `"*"` for full model).
- `service.getType(path: string) -> string | undefined` – returns the schema/type definition of the entry at `path`.
- `service.set(path: string, object: any) -> any` – sets or replaces a model entry at `path` and returns the updated
  snapshot.
- `service.remove(path: string) -> boolean` – removes an entry at `path` and returns `true` on success.

> All functions throw JavaScript exceptions when the model or invocation is invalid.
> All exports return native JavaScript primitives, arrays, or plain objects.

> **Note:** The thread-local controller enforces single-instance behavior per WASM module instance.
> Instantiating a new `DecisionService` replaces the previous one and the same thread/context.

### Optional Function Groups for WASM

To keep Web/Node WASM builds small, WASM uses Node.js and web native base64 and regexp functionality.
However, if needed, regexp and base64 can be embedded into WASM via features.
For CLI/WASI builds, all features are always enabled and base64 with regexp libraries are linked in (that increases the
package size)

- `regex_functions`: Enables built-in regex-powered string ops used by the DSL `regexSplit` and `regexReplace`, and
  disables native regex functions on Node/Web.
- `base64_functions`: Enables built-in `toBase64` and `fromBase64`, disables native base64 functions on Node/Web.

## CLI

Build and try the native CLI:

- `just cli` – builds `edgerules` natively, prints binary size, and runs a quick arithmetic check.

Usage examples:

- `edgerules "{ value : 1 + 2 }"` → prints `3`
- `edgerules @path/to/file.txt` → loads code from file
- `echo "{ value : 2 * 3 }" | edgerules` → reads from stdin

## Known Design Exceptions

- WASM contains `DECISION_SERVICE` instance that is not designed to work in multithreaded environments.
  It is intended for single-threaded usage in web or Node.js contexts where each WASM module instance
  is isolated per thread or request. This design choice simplifies state management and avoids
  concurrency issues in typical web scenarios.

- Due to requirement to have small WASM size, serde is not used for JSON (de)serialization.
  Custom lightweight JSON parser and serializer is implemented instead to keep small WASM size.
  Do not introduce serde dependencies in the core runtime!

## Resources

- For developer: https://rust-unofficial.github.io/patterns/idioms/index.html
- JavaScript FEEL: https://github.com/EdgeVerve/feel
- OpenRules FEEL: https://openrules.com/ReleaseNotes_6.4.0.htm#Implementing_FEEL_Expressions
- Comunda FEEL: https://docs.camunda.io/docs/components/modeler/feel/language-guide/feel-data-types/
- Oracle
  FEEL: https://docs.oracle.com/en/cloud/paas/integration-cloud/user-processes/define-expressions-friendly-enough-expression-language-feel.html