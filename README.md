# EdgeRules

JSON-native business rules for the edge

# Preface

**EdgeRules** is a structure and programming language specification for defining algorithms and business rules.
The project was started early in 2022 to create a simple, safe, and expressive language for business users and
developers
to oppose poor FEEL decisions and bizarre syntax. Unfortunately, the Jsonnet project wasn't on my radar at that time; it
appeared to be the closest to what I wanted to achieve. Nonetheless, EdgeRules had its unique features and goals:
hard to fail strategies such that the absence of reference loops, nulls, fully traceable, referentially transparent,
and the most crucial target was a small WASM binary size to be used in client browsers... until it exploded to 600Kb,
and I barely implemented one-third of my ideas... Due to the shift in my focus, I dropped the project in late 2023.

In late 2025, I moved the project to GitHub and kept it for my experimentation and research. For this reason, the
project might be volatile.

Similar projects:

- **Jsonnet**: Data-templating language (superset of JSON) for generating JSON/YAML; pure expressions, no side effects.
- **FEEL**: Friendly Enough Expression Language, part of DMN standard; designed for business users to define decision
  logic.

## Project Goals / Roadmap / Features

- [x] Referentially transparent
- [x] No null, nil or NaN
- [x] Immutable by default
- [x] Shallow learning curve: easy to read for non-technical users
- [x] Statically typed
- [x] ~ Traceable
- [x] Hard to fail: no exceptions, no nulls, no NaNs, no undefined variables
- [x] Hard to fail: no reference loops
- [ ] Hard to fail: no infinite loops
- [ ] FEEL coverage

## Technical Debt / Non-Functional

- [ ] Refactor test cases for a better navigation
- [ ] Use FromStr https://doc.rust-lang.org/std/str/trait.FromStr.html
- [ ] Use TryFrom https://doc.rust-lang.org/std/convert/trait.TryFrom.html & From
- [ ] Investigate if Error should be used as https://doc.rust-lang.org/std/error/trait.Error.html and read more about
  error propagation
- [ ] Use OnceCell and Cell instead of Rc and RefCell in Context

## Features

### Roadmap

- [ ] Strongly typed
- [ ] Statically typed with type inference
- [x] Cycle-reference prevention
- [ ] Fractional mathematics for infinite precision
- [ ] Halting resistance
- [ ] Friendly object reflections: `_fields`, `_methods`, `_objectName`, `_uid`
- [ ] Custom domain types
- [ ] Special values: `Missing`, `NotApplicable`, `NotFound`
- [x] Optional chaining by default: `object.parent.field`
- [ ] Return statement for objects using `return` field
- [ ] Inheritance via `merge(...)`
- [ ] Auto types: `applicant = {...}` will get `ApplicantType`
- [ ] Manual types: `applicant: Customer = {...}` will get `Customer` as a type
- [ ] Higher abstractions layer (Domain Semantics) using @Annotations
- [ ] Infinite lists
- [ ] Returns custom function call request same as `callGPT` in `getAgentAnswer`. Work on external user functions:
  `external.callGPT`
- [ ] ??? Tuples
- [ ] ??? Type casting using `as`
- [ ] Complete type inference

### Rejected Features

- [ ] First-class functions

### Function Features

- [x] Function calls
- [ ] Optional argument labels in call expressions (`object.function(input: customer)`)
- [ ] Function overloading: `calfunction(a: number)` and `calfunction(a: string)` can be in the same scope

### Supported Types

- [ ] `number`, &#9744; `string`, &#9744; `date`
- [x] array type `[]`

### Rule Features

- [ ] First-class conditions
- [ ] Pattern matching using `match`
- [ ] None coalescing for optionals (`foo ?? bar` yields `foo` if `foo` has a value, otherwise `bar`)
- [ ] `if`,`then`,`else`
- [ ] `and`,`or`,`xor`
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

### Structure Examples

```edgerules
// TBC
```

- Field names are always unique in the structure
- Copying a structure instance always makes a deep copy
- Struct members are *public* by default

### Function Examples

- [ ] All functions that can be applied for a certain type, could also be called from a variable from that type

```edgerules
{
    myarray = [1,2,3,4]
    sum1 = sum(myarray)
    sum2 = myarray.sum 
}
```

## Considerations

### Code Style

| LISP Style          | Chaining Style      | Verbal Style       |
|---------------------|---------------------|--------------------|
| `sum([1,2,3,4])`    | `[1,2,3,4].sum()`   | `sum [1,2,3,4]`    |
| `left([1,2,3,4],2)` | `[1,2,3,4].left(2)` | `[1,2,3,4] left 2` |

> LISP style was selected

## FEEL Coverage

| List                    | String:                  | Numeric                 | Range                   | Conversion                | Context     | Boolean    |
|-------------------------|--------------------------|-------------------------|-------------------------|---------------------------|-------------|------------|
| &#9744; list contains   | &#9744; substring        | &#9744; decimal         | &#9744; before          | date                      | get entries | not        |
| &#9744; count           | &#9744; string length    | &#9744; floor           | &#9744; after           | date and time             | get value   | is defined |
| &#9744; min             | &#9744; upper case       | &#9744; ceiling         | &#9744; meets           | time                      | put         |            |
| &#9744; max             | &#9744; lower case       | &#9744; abs             | &#9744; met by          | number                    | put all     |            |
| &#9745; sum             | &#9744; substring before | &#9744; modulo          | &#9744; overlaps        | string                    | context     |            |
| &#9744; product         | &#9744; substring after  | &#9744; sqrt            | &#9744; overlaps before | duration                  |             |            |
| &#9744; mean            | &#9744; replace          | &#9744; log             | &#9744; overlaps after  | years and months duration |             |            |
| &#9744; median          | &#9744; contains         | &#9744; exp             | &#9744; finishes        |                           |             |            |
| &#9744; stddev          | &#9744; starts with      | &#9744; odd             | &#9744; finished by     |                           |             |            |
| &#9744; mode            | &#9744; ends with        | &#9744; even            | &#9744; includes        |                           |             |            |
| &#9744; and             | &#9744; matches          | &#9744; round up        | &#9744; during          |                           |             |            |
| &#9744; all             | &#9744; split            | &#9744; round down      | &#9744; starts          |                           |             |            |
| &#9744; or              | &#9744; extract          | &#9744; round half up   | &#9744; started by      |                           |             |            |
| &#9744; any             |                          | &#9744; round half down | &#9744; coincides       |                           |             |            |
| &#9744; sublist         |                          |                         |                         |                           |             |            |
| &#9744; append          |                          |                         |                         |                           |             |            |
| &#9744; concatenate     |                          |                         |                         |                           |             |            |
| &#9744; insert before   |                          |                         |                         |                           |             |            |
| &#9744; remove          |                          |                         |                         |                           |             |            |
| &#9744; reverse         |                          |                         |                         |                           |             |            |
| &#9744; index of        |                          |                         |                         |                           |             |            |
| &#9744; union           |                          |                         |                         |                           |             |            |
| &#9744; distinct values |                          |                         |                         |                           |             |            |
| &#9744; flatten         |                          |                         |                         |                           |             |            |
| &#9744; sort            |                          |                         |                         |                           |             |            |
| &#9744; string join     |                          |                         |                         |                           |             |            |

## Decision tables in Edge Rules

```edgerules
{
    decisionTableResult = decisionTable([applicant],
        rules: [
            rule1: [applicant.age < 18, "minor"],
            rule2: [applicant.age >= 18, "adult"],            
        ]
    )
}
```

## Resources

- JavaScript FEEL: https://github.com/EdgeVerve/feel
- OpenRules FEEL: https://openrules.com/ReleaseNotes_6.4.0.htm#Implementing_FEEL_Expressions
- Comunda FEEL: https://docs.camunda.io/docs/components/modeler/feel/language-guide/feel-data-types/
- Oracle
  FEEL: https://docs.oracle.com/en/cloud/paas/integration-cloud/user-processes/define-expressions-friendly-enough-expression-language-feel.html

## JavaScript/WASM Usage

Two ways to use EdgeRules outside Rust:

### Building WASM

- Targets: build with `--features wasm` to keep the binary small. For debug console panics, use `--features wasm_debug`.
- Release profile is tuned for size: LTO fat, codegen-units=1, opt-level=z, panic=abort, strip=debuginfo.

Commands

- Web (wasm-bindgen): `wasm-pack build --release --target web -- --features wasm`
- Node.js (wasm-bindgen): `wasm-pack build --release --target nodejs -- --features wasm`
- Optional (if Binaryen is installed): `wasm-opt -Oz -o pkg/optimized.wasm pkg/*.wasm`
- Size check: `du -h pkg/*.wasm`

WASI CLI (optional)

- Build: `cargo build --release --target wasm32-wasip1 -p edge-rules --bin edgerules-wasi` or `npm run build:wasi`
- Run (wasmtime/wasmer): `wasmtime target/wasm32-wasip1/release/edgerules-wasi.wasm -- "{ value: 2 + 2 }"`

Caveat: wasm-pack output is not WASI

- Artifacts in `pkg/` produced by `wasm-pack` depend on wasm-bindgen JS glue (imports like `__wbindgen_*`). These are not runnable directly under `wasmtime`.
- Use the generated JS wrappers (Node/Web) to load and call the module, or build the WASI CLI target (`wasm32-wasip1`) for running with `wasmtime`.
- Run (wasmtime/wasmer): `wasmtime pkg/optimized.wasm -- "{ value: 2 + 2 }"`

Notes

- Logging is disabled in `wasm` builds to reduce size. Use `wasm_debug` to enable the panic hook during development.
- If `wasm-opt` is not installed, `npm run build:wasm:*` still produces a working build; size is larger.

- Node.js/Web (wasm-bindgen): call functions directly from JS
- WASI CLI (Wasmer/Wasmtime): run the `.wasm` as a command-line utility

### Node.js

Prerequisites:

- Node.js 18+
- wasm-pack: `cargo install wasm-pack` (or `brew install wasm-pack`)

Build for Node:

```bash
wasm-pack build --release --target nodejs
```

Example (`node-demo.mjs`):

```js
import init, {evaluate_value, evaluate_field, to_trace, init_panic_hook} from './pkg/edge_rules.js';

await init();
init_panic_hook();

console.log(await evaluate_value("{ value : 2 + 3 }")); // "5"
console.log(await evaluate_field("{ x : 1; y : x + 2 }", "y")); // "3"
console.log(to_trace("{ a : 1; b : a + 2 }"));
```

Run:

```bash
node node-demo.mjs
```

### Browser

Build for the web:

```bash
wasm-pack build --release --target web
```

Minimal HTML example (place next to `pkg/`):

```html

<script type="module">
    import init, {evaluate_value, evaluate_field, to_trace, init_panic_hook} from './pkg/edge_rules.js';

    (async () => {
        await init();
        init_panic_hook();
        console.log(await evaluate_value('{ value : 10 + 20 }'));
        console.log(await evaluate_field('{ total : sum([1,2,3]) }', 'total'));
        console.log(to_trace('{ a : 1; b : a + 2 }'));
    })();
</script>
```

### WASI CLI (Wasmer/Wasmtime)

Build the WASI binary:

```bash
rustup target add wasm32-wasip1
cargo build --target wasm32-wasip1 --release
```

Run with Wasmer:

```bash
wasmer run target/wasm32-wasip1/release/edgerules-wasi.wasm -- "{ value : 2 + 3 }"
wasmer run --dir . target/wasm32-wasip1/release/edgerules-wasi.wasm -- @tests/record_1.txt
```

Or with Wasmtime:

```bash
wasmtime run target/wasm32-wasip1/release/edgerules-wasi.wasm -- "{ value : 2 + 3 }"
```

Notes:

- `evaluate_value` returns the `value` field if present; use `evaluate_field(code, name)` for other fields.
- `to_trace` prints the evaluated structure (useful for debugging/explainability).
- Call `init_panic_hook()` once in JS to get human-friendly error messages in the console.

## Readings

- [ ] https://rust-unofficial.github.io/patterns/idioms/index.html

# QA

```bash
cargo run --bin generate-examples
```
