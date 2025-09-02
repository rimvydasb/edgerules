# EdgeRules

JSON-native business rules for the edge.

## Preface

**EdgeRules** is a structure and programming language specification for defining algorithms and business rules.
The project was started early in 2022 to create a simple, safe, and expressive language for business users and
developers
to oppose poor DMN FEEL decisions and bizarre syntax choices. Unfortunately, the Jsonnet project wasn't on my radar at that time, and it
appeared to be the closest to what I wanted to achieve. Nonetheless, EdgeRules had its unique features and goals:
hard to fail strategies such that the absence of reference loops, no nulls, fully traceable, referentially transparent,
and the most crucial target was a small WASM binary size for inexpensive use in client browsers... until it exploded to 600Kb,
and I barely implemented one-third of my ideas... Due to the shift in my focus, I dropped the project in late 2023.
In late 2025, I moved the project to GitHub and kept it for my experimentation and research. For this reason, the
project might be volatile.

### Similar projects:

- **Jsonnet**: Data-templating language (superset of JSON) for generating JSON/YAML; pure expressions, no side effects.
- **FEEL**: Friendly Enough Expression Language, part of DMN standard; designed for business users to define decision
  logic.

## Features / Roadmap

- [x] Referentially transparent (pure functions, no side effects)
- [x] No null, nil or NaN
- [x] Immutable by default
- [x] Shallow learning curve: easy to read for non-technical users
- [x] Statically typed
- [x] ~ Traceable
- [x] Hard to fail: no exceptions, no nulls, no NaNs, no undefined variables
- [x] Hard to fail: no reference loops (Cycle-reference prevention)
- [x] Boolean literals (`true`/`false`) and logical operators (`and`, `or`, `xor`, `not`)
- [ ] Hard to fail: no infinite loops
- [ ] Full DMN FEEL coverage
- [ ] Strongly typed and statically typed with type inference
- [ ] Fractional mathematics for infinite precision
- [ ] Infinite lists

### Supported Types

- [ ] `number`, &#9744; `string`, &#9744; `date`
- [x] array type `[]`

### Rule Features

- [ ] First-class conditions
- [ ] Pattern matching using `match`
- [ ] None coalescing for optionals (`foo ?? bar` yields `foo` if `foo` has a value, otherwise `bar`)
- [ ] `if`,`then`,`else`
- [x] `and`,`or`,`xor`,`not`
- [ ] `@Context`

## Special Values

### Missing

- Value is expected, but not found:
    - Filter is applied on a list, but list item that matches filter is not found
    - Decision Table is executed, but does not hit any row
- Treatment:
    - All calculations that involves `NotFound` will result to `NotFound`
- Info:
    - User cannot assign this value from the code

| Name                   | Description                         | Treatment              | Can be assigned by user |
|------------------------|-------------------------------------|------------------------|-------------------------|
| &#9744;`Missing`       | value is expected, but not found    | override by `Missing`  | Yes                     |
| &#9744;`NotApplicable` | value is not expected and not found | treat as 0             | Yes                     |
| &#9744;`NotFound`      | value entry is not found            | override by `NotFound` | No - system only        |

## Examples / Basic syntax

```edgerules
// TBC
```

## Resources

- JavaScript FEEL: https://github.com/EdgeVerve/feel
- OpenRules FEEL: https://openrules.com/ReleaseNotes_6.4.0.htm#Implementing_FEEL_Expressions
- Comunda FEEL: https://docs.camunda.io/docs/components/modeler/feel/language-guide/feel-data-types/
- Oracle
  FEEL: https://docs.oracle.com/en/cloud/paas/integration-cloud/user-processes/define-expressions-friendly-enough-expression-language-feel.html

# Development

## Prerequisites

- Node.js 18+
- just: `cargo install just` (or `brew install just`)
- wasm-pack: `cargo install wasm-pack` (or `brew install wasm-pack`)

## WASM (Web/Node.js)

- For Web: `just web` (artifacts in `target/pkg-web/`)
- For Node.js: `just node` (artifacts in `target/pkg-node/`)
- Optional (if Binaryen is installed): `wasm-opt -Oz -o target/pkg-web/edge_rules_bg.min.wasm target/pkg-web/edge_rules_bg.wasm`

## WASI CLI (Wasmer/Wasmtime) (@Todo not working now)

- Build: `just wasi` (prints size and runs demo)
- Run (wasmtime/wasmer): `wasmtime target/wasm32-wasip1/release/edgerules-wasi.wasm -- "{ value: 2 + 2 }"`

## Validation and Testing (@Todo not working now)

- Size check: `du -h target/pkg-web/*.wasm target/pkg-node/*.wasm`
- Run (wasmtime/wasmer): `wasmtime target/wasm32-wasip1/release/edgerules-wasi.wasm -- "{ value: 2 + 2 }"`

## Readings

- [ ] https://rust-unofficial.github.io/patterns/idioms/index.html

# Quality Assurance

```bash
cargo run --bin generate-examples
```
