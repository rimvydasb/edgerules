# EdgeRules

JSON-native business rules for the edge.

## Preface

**EdgeRules** is a structure and programming language specification for defining algorithms and business rules.
The project was started early in 2022 to create a simple, safe, and expressive language for business users and
developers
to oppose poor DMN FEEL language syntax decisions such as bizarre syntax choices and no proper missing value handling.
Unfortunately, the Jsonnet project wasn't on my radar at that time, and it
appeared to be the closest to what I wanted to achieve. Nonetheless, EdgeRules had its unique features and goals:
hard to fail strategies such that the absence of reference loops, no nulls, fully traceable, referentially transparent,
and the most crucial target was a small WASM binary size for inexpensive use in client browsers... until it exploded to
600Kb,
and I barely implemented one-third of my ideas... Due to the shift in my focus, I dropped the project in late 2023.
In late 2025, I moved the project to GitHub and kept it for my experimentation and research. For this reason, the
project might be volatile.

- [Language Reference](REFERENCE.md)
- For Stories and Epics check: [doc](doc)
- [General ToDo](TODO.md)
- [Development](AGENTS.md)
- [Complex examples and problems with results](tests/EXAMPLES-output.md)
- [License](LICENSE)

### Comparison to the similar projects:

- **Jsonnet**: Data-templating language (superset of JSON) for generating JSON/YAML; pure expressions, no side effects.
- **FEEL**: Friendly Enough Expression Language, part of DMN standard; designed for business users to define decision
  logic.

|                       | Jsonnet | DMN FEEL                   | EdgeRules             |
|-----------------------|---------|----------------------------|-----------------------|
| Null value treatment  | N/A     | nn (Proposed By Trisotech) | Native Special Values |
| Objects, lists, types | Yes     | Yes                        | Yes                   |
| TBC.                  |         |                            |                       |

## Features / Roadmap

- [x] Referentially transparent (pure functions, no side effects)
- [x] No null, nil or NaN
- [x] Immutable by default
- [x] Statically typed
- [x] ~ Traceable
- [x] Hard to fail: no exceptions, no nulls, no NaNs, no undefined variables
- [x] Hard to fail: no reference loops (Cycle-reference prevention)
- [ ] Hard to fail: no infinite loops (TBA: optimistic limits strategy)
- [x] Boolean literals (`true`/`false`) and logical operators (`and`, `or`, `xor`, `not`)
- [ ] Full DMN FEEL coverage
- [ ] Strongly typed and statically typed with type inference
- [ ] Fractional mathematics for infinite precision
- [ ] Infinite lists
- [ ] Structures composition
- [ ] First-class conditions
- [ ] Pattern matching using `match`
- [ ] None coalescing for optionals (`foo ?? bar` yields `foo` if `foo` has a value, otherwise `bar`)

## Basic API Usage (Rust)

EdgeRules exposes a small, stateful API for loading source and evaluating expressions/fields. The typical flow is:

- Create `EdgeRules`
- Incrementally `load_source("...")` (can be called multiple times)
- Evaluate using `evaluate_field("path")` or `evaluate_expression("expr")`

All evaluations use the code loaded into the same `EdgeRules` instance.

```rust
use edge_rules::runtime::edge_rules::EdgeRules;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a service instance (stateful)
    let mut service = EdgeRules::new();

    // Load some code (can be called multiple times to extend/override)
    service.load_source("{ value: 3 }")?;

    // Evaluate a pure expression using the loaded context
    // e.g., "2 + value" where value comes from the loaded source above
    let val = service.evaluate_expression("2 + value")?;
    assert_eq!(val.to_string(), "5");

    // Load more source and reuse the same instance
    service.load_source("{ calendar: { config: { start: 7; end: start + 5 } } }")?;

    // Evaluate a field/path from the loaded model
    let start = service.evaluate_field("calendar.config.start");
    assert_eq!(start, "7");

    let end = service.evaluate_field("calendar.config.end");
    assert_eq!(end, "12");

    Ok(())
}
```

## WASM

WASM exported methods via `wasm_bindgen`:

- `evaluate_all(code: &str) -> String` – loads model code and returns the fully evaluated model as code.
- `evaluate_expression(code: &str) -> String` – evaluates a standalone expression against an empty context.
- `evaluate_field(code: &str, field: &str) -> String` – loads `code`, then evaluates a field/path.

## Resources

- For developer: https://rust-unofficial.github.io/patterns/idioms/index.html
- JavaScript FEEL: https://github.com/EdgeVerve/feel
- OpenRules FEEL: https://openrules.com/ReleaseNotes_6.4.0.htm#Implementing_FEEL_Expressions
- Comunda FEEL: https://docs.camunda.io/docs/components/modeler/feel/language-guide/feel-data-types/
- Oracle
  FEEL: https://docs.oracle.com/en/cloud/paas/integration-cloud/user-processes/define-expressions-friendly-enough-expression-language-feel.html
