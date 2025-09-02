## Todo / Optimization Checklist

### Build & Profiles

- [ ] Release profile: enable lto = "fat", codegen-units = 1, opt-level = "z" (or
  "s" if speed wins), panic = "abort", and strip = "debuginfo".
- [ ] wasm-pack: add wasm-opt -Oz step (either via wasm-pack -- flags or a post
  step).
- [ ] Targets: build with --no-default-features --features wasm for WASM to avoid
  pulling native-only deps.

### Features & Dependencies

- [ ] Default features: remove wasm-bindgen and console_error_panic_hook from
  default; introduce feature sets:
    - [ ] native: logging, env_logger, dev ergonomics.
    - [ ] wasm: wasm-bindgen, console_error_panic_hook (release off by default),
      light allocator.
- [ ] Dev-only deps: move regex and env_logger to [dev-dependencies] (they are
  only used in tests/utilities).
- [ ] Allocator: consider dlmalloc for WASM (smaller) and gate behind wasm
  feature.

### Logging & Panics

- [ ] Gate all log::{trace,debug} behind cfg(feature = "native") or
  cfg(debug_assertions) to avoid string formatting in release/wasm.
- [ ] In WASM release builds, disable console_error_panic_hook (keep only for
  debug); use panic = "abort".

### Linking & Evaluation Hot Paths

- [ ] Variable linking:
    - [ ] Pre-resolve variable paths to a compact handle (e.g., an interned
      FieldId with parent pointer) during linking. Avoid repeated browse/hash
      lookups at runtime.
    - [ ] Replace Vec<String> in VariableLink with a small-path representation
      (e.g., SmallVec<[u32; 3]> of interned ids) to cut allocs and size.
- [ ] Name interning:
    - [ ] Introduce a global or per-tree string interner (e.g., string_cache or a
      simple FxHashMap<String, u32>) and store ids, not String, for all_field_names,
      locks, and lookups.
- [ ] Field locks:
    - [ ] NodeData::lock_field currently allocates/clones String per lock; switch
      to id-based locking (bitset or small set of u32).
- [ ] Object printing:
    - [ ] to_code/Display produce many temporary Strings; gate behind feature
      debug_print and avoid calling it in hot paths.

### Data Structures & Algorithms

- [ ] Hash maps:
    - [ ] Consider hashbrown (or ahash for speed) for HashMap/HashSet. For size,
      hashbrown is typically neutral/slightly smaller than std; ahash can add a bit
      of size.
- [ ] VecDeque churn:
    - [ ] browse converts back and forth to VecDeque frequently. Accept a slice
      &[u32] (interned path) and index into it, eliminating conversions.
- [ ] Trait objects:
    - [ ] Box<dyn Metaphor> increases binary size. If the set is small/known,
      consider an enum for built-ins or per-metaphor feature flags; keep dyn behind
      features where needed.

### WASM Bindings & JS

- [ ] wasm-bindgen:
    - [ ] Keep bindings minimal and avoid exposing internal types; prefer opaque
      handles or serialized results to reduce glue code.
- [ ] Demos:
    - [ ] Run web demo with --no-default-features --features wasm and wasm-opt -Oz
      to baseline size. Consider precompressing in CI (gzip/brotli for delivery).

### Maintainability

- [ ] Error types:
    - [ ] Replace frequent format! in hot paths with lightweight error enums +
      Display built only on error. Defer string building until needed.
- [ ] Tests:
    - [x] Keep regex usage behind #[cfg(test)]; migrate to [dev-dependencies].
    - [ ] Add perf smoke-tests (Criterion on native) to catch regressions in
      browse/eval and array filtering.
- [ ] Module cohesion:
    - [ ] Extract a small “symbol table” module responsible for interning, field
      ids, and fast lookup (shared by linker/eval).
    - [ ] Clarify node kinds with an enum (no stringing node type names on hot
      path).

###  Cargo.toml suggestions (illustrative)

- [ ] Profiles:
    - [ ] [profile.release] lto = "fat", codegen-units = 1, opt-level = "z", panic
      = "abort", strip = "debuginfo"
- [ ] Features:
    - [ ] [features] default = ["native"]; native = ["log"]; wasm =
      ["wasm-bindgen", "console_error_panic_hook?"]
- [ ] Dev deps:
    - [ ] Move regex, env_logger under [dev-dependencies].

### Technical Debt / Non-Functional Requirements

- [ ] Refactor test cases for a better navigation
- [ ] Use FromStr https://doc.rust-lang.org/std/str/trait.FromStr.html
- [ ] Use TryFrom https://doc.rust-lang.org/std/convert/trait.TryFrom.html & From
- [ ] Investigate if Error should be used as https://doc.rust-lang.org/std/error/trait.Error.html and read more about
  error propagation
- [ ] Use OnceCell and Cell instead of Rc and RefCell in Context

###  Next steps:

- [ ] Move regex and env_logger to dev-dependencies and adjust features/defaults
  to keep WASM minimal.
- [ ] Add release profile tweaks and a wasm feature that disables logging/panics
  for size.
- [ ] Introduce a simple field-name interner and refactor VariableLink to store
  interned ids (incrementally: start with storing interned u32 paths while
  keeping compatibility).
- [ ] Micro-opt browse to take slices and avoid VecDeque conversions.

### Technical Backlog

- [ ] Friendly object reflections: `_fields`, `_methods`, `_objectName`, `_uid`
- [ ] Custom domain types
- [ ] Special values: `Missing`, `NotApplicable`, `NotFound`
- [ ] Return statement for objects using `return` field
- [ ] Inheritance via `merge(...)`
- [ ] Auto types: `applicant = {...}` will get `ApplicantType`
- [ ] Manual types: `applicant: Customer = {...}` will get `Customer` as a type
- [ ] Higher abstractions layer (Domain Semantics) using @Annotations
- [ ] Returns custom function call request same as `callGPT` in `getAgentAnswer`. Work on external user functions:
  `external.callGPT`
- [ ] Complete type inference
- [x] Function calls
- [ ] Optional argument labels in call expressions (`object.function(input: customer)`)
- [ ] Function overloading: `calfunction(a: number)` and `calfunction(a: string)` can be in the same scope


### Rejected Features

- (X) First-class functions

### Function Examples

- [ ] All functions that can be applied for a certain type, could also be called from a variable from that type

```edgerules
{
    myarray = [1,2,3,4]
    sum1 = sum(myarray)
    sum2 = myarray.sum 
}
```

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

## Decision tables (TBC)

```edgerules
{
    decisionTable(applicant): [
        rule1: [applicant.age < 18, "minor"],
        rule2: [applicant.age >= 18, "adult"],            
    ][.. = true].length > 0
    result: decisionTable(applicant)
}
```