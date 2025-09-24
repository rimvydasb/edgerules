## Todo / Optimization Checklist

> Critical Gaps

- All three binaries (edgerules, er, edgerules-wasi) point at the same
  native entry; the WASI target can’t get its own feature set or I/O defaults,
  which blocks size-sensitive builds and automation promised in the docs
  (Cargo.toml:14-23).
- Parsing still forces tokenize(&code.to_string()), re-allocating the whole
  source for every call and passing &String through the tokenizer layer, which
  slows the CLI and hampers agents that repeatedly re-run parses (src/runtime/
  edge_rules.rs:130, src/tokenizer/parser.rs:24).

High-Impact Opportunities

- Replace HashMap<String, …>/HashSet<String> hot paths with interned u32 ids
  (and migrate VariableLink) so linking and locking stop cloning strings; this
  also shrinks the binary by removing duplicate literals (src/ast/context/
  context_object.rs:68, src/link/node_data.rs:193).
- Gate trace!/debug! blocks behind a cfg(any(debug_assertions, feature =
  "native")) to avoid shipping format strings into release WASM (src/link/
  linker.rs:17, src/runtime/execution_context.rs:263).
- Cache variable handles during linking so evaluate_field and
  evaluate_expression stop re-linking every call; persist the resolved
  handles inside ExecutionContext to cut repeated graph walks (src/runtime/
  edge_rules.rs:277).
- Collapse the pervasive Rc<RefCell> churn by making ContextObject
  immutable at runtime and storing evaluation results in a separate arena;
  most RefCell usage is only to bypass borrowing rules (src/ast/context/
  context_object.rs:64, src/runtime/execution_context.rs:37).

Agent & Maintainability

- Enforce the documented naming standards (no ctx/cfg) so automated
  agents don’t have to fight inconsistent style (src/runtime/
  execution_context.rs:312).
- Split the WASI entry into src/bin/edgerules-wasi.rs so command examples in
  AGENTS.md match the filesystem, easing scripted use.
- Surface a high-level module map/AST diagram in doc/ to shorten orientation
  time (the Mermaid stub is present but unused: src/ast/context/!uml-
  functions.mermaid).
- Expand the smoke tests to cover CLI/WASM size assertions, giving agents a
  quick regression check they can run before altering hot paths.

Improvement Plan

2. Introduce a lightweight string interner module shared by linker/runtime;
   migrate ContextObject and VariableLink to store ids, remove String cloning,
   and swap global maps to FxHashMap.
3. Wrap all logging macros behind compile-time guards and confirm the
   release WASM shrinks (compare wasm-opt output before/after).
4. Teach the linker to emit field handles (SmallVec of interned ids) and
   let ExecutionContext cache evaluation results keyed by those handles,
   eliminating per-call link() work.
5. Refactor the tokenizer API to accept &str, stop cloning the source
   string, and add a bench/dev test so future grammar work keeps parse costs
   flat.

These steps unlock smaller WASM artifacts (priority 1), cut hot-path
allocations (priority 2), and align the layout/documentation with the
workflow expectations for Codex CLI and GPT-5 coding agents (priority 3).

### Features & Dependencies

- [x] Default features: remove wasm-bindgen and console_error_panic_hook from
  default; introduce feature sets:
    - [ ] native: logging, env_logger, dev ergonomics.
    - [ ] wasm: wasm-bindgen, console_error_panic_hook (release off by default),
      light allocator.
- [x] Allocator: consider dlmalloc for WASM (smaller) and gate behind wasm feature.

### Logging & Panics

- [ ] Gate all log::{trace,debug} behind cfg(feature = "native") or
  cfg(debug_assertions) to avoid string formatting in release/wasm.

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

- [ ] Use std::collections::HashMap with FxHash. It’s fast on WASM, tiny, and avoids the heavier SipHash and aHash
  paths.
- [x] Browse converts back and forth to VecDeque frequently. Accept a slice &[u32] (interned path) and index into it,
  eliminating conversions.
- [x] Trait objects: Box<dyn Metaphor> increases binary size. If the set is known,
  consider an enum for built-ins

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

### Cargo.toml suggestions (illustrative)

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

### Next steps:

- [ ] Add release profile tweaks and a wasm feature that disables logging/panics
  for size.
- [ ] Introduce a simple field-name interner and refactor VariableLink to store
  interned ids (incrementally: start with storing interned u32 paths while
  keeping compatibility).
- [x] Micro-opt browse to take slices and avoid VecDeque conversions.

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
- [ ] Modulo `%` exists internally but is not tokenized

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