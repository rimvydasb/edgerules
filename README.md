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

# Development

## Prerequisites

- Node.js 18+
- wasm-pack: `cargo install wasm-pack` (or `brew install wasm-pack`)

## WASM (Web/Node.js)

- For Web: `npm run build:wasm:web`
- For Node.js: `npm run build:wasm:node`
- Optional (if Binaryen is installed): `wasm-opt -Oz -o pkg/optimized.wasm pkg/*.wasm`

## WASI CLI (Wasmer/Wasmtime) (@Todo not working now)

- For WASI (@Todo: not working now): `npm run build:wasi`
- Build: `cargo build --release --target wasm32-wasip1 -p edge-rules --bin edgerules-wasi` or `npm run build:wasi`
- Run (wasmtime/wasmer): `wasmtime target/wasm32-wasip1/release/edgerules-wasi.wasm -- "{ value: 2 + 2 }"`

## Validation and Testing (@Todo not working now)

- Size check: `du -h pkg/*.wasm`
- Run (wasmtime/wasmer): `wasmtime pkg/edge_rules_bg.opt.wasm -- "{ value: 2 + 2 }"`

## Readings

- [ ] https://rust-unofficial.github.io/patterns/idioms/index.html

# Quality Assurance

```bash
cargo run --bin generate-examples
```