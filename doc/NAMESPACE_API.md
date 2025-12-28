To provide a clean API for JavaScript developers, you should move from a flat procedural style to a structured,
object-oriented design. This approach groups stateless logic into a static utility class and stateful logic into a
managed service class, improving discoverability and reducing name verbosity.

---

### API Organization Comparison

| Feature               | Current (Flat)                  | Proposed (Structured)           |
|-----------------------|---------------------------------|---------------------------------|
| **Logic Type**        | Global functions                | Namespaced Classes              |
| **Stateless Methods** | `evaluate_all(code)`            | `DecisionEngine.evaluate(code)` |
| **State Management**  | Global singleton (hidden)       | Service Instance (explicit)     |
| **Method Names**      | `set_to_decision_service_model` | `service.set(path, value)`      |
| **DX/Autocomplete**   | Lists 30+ unrelated items       | Scoped to `Engine` or `Service` |

---

### 1. The Stateless "Engine" (Static Namespace)

Since the evaluation methods do not depend on the `thread_local` state, group them into a struct with no constructor. In
JavaScript, this will act as a namespace with static methods.

```rust
#[wasm_bindgen]
pub struct DecisionEngine;

#[wasm_bindgen]
impl DecisionEngine {
    #[wasm_bindgen(js_name = "evaluateAll")]
    pub fn evaluate_all(code: &str) -> JsValue {
        match wasm_convert::evaluate_all_inner(code) {
            Ok(value) => value,
            Err(err) => throw_portable_error(err),
        }
    }

    #[wasm_bindgen(js_name = "evaluateExpression")]
    pub fn evaluate_expression(code: &str) -> JsValue {
        match wasm_convert::evaluate_expression_inner(code) {
            Ok(value) => value,
            Err(err) => throw_portable_error(err),
        }
    }

    #[wasm_bindgen(js_name = "evaluateField")]
    pub fn evaluate_field(code: &str, field: &str) -> JsValue {
        match wasm_convert::evaluate_field_inner(code, field) {
            Ok(value) => value,
            Err(err) => throw_portable_error(err),
        }
    }
}

```

### 2. The Stateful "Service" (Instance-based)

Wrap the `thread_local` management inside a `DecisionService` struct. This makes the lifecycle of the service clear to
the JS user.

```rust
#[wasm_bindgen]
pub struct DecisionService;

#[wasm_bindgen]
impl DecisionService {
    #[wasm_bindgen(constructor)]
    pub fn new(model: &JsValue) -> DecisionService {
        let controller = match DecisionServiceController::from_portable(model) {
            Ok(ctrl) => ctrl,
            Err(err) => throw_portable_error(err),
        };
        set_decision_service(controller);
        DecisionService
    }

    #[wasm_bindgen(getter)]
    pub fn model(&self) -> JsValue {
        match with_decision_service(|svc| svc.model_snapshot()) {
            Ok(value) => value,
            Err(err) => throw_portable_error(err),
        }
    }

    pub fn execute(&self, method: &str, request: &JsValue) -> JsValue {
        let response = match with_decision_service(|svc| {
            let req_val = js_request_to_value(request)?;
            svc.execute_value(method, req_val)
        }) {
            Ok(value) => value,
            Err(err) => throw_portable_error(err),
        };
        response.to_js().unwrap_or_else(|e| utils::throw_js_error(e.to_string()))
    }

    pub fn get(&self, path: &str) -> JsValue {
        match with_decision_service(|svc| svc.get_entry(path)) {
            Ok(value) => value,
            Err(err) => throw_portable_error(err),
        }
    }

    pub fn set(&self, path: &str, object: &JsValue) -> JsValue {
        match with_decision_service(|svc| svc.set_entry(path, object)) {
            Ok(value) => value,
            Err(err) => throw_portable_error(err),
        }
    }

    pub fn remove(&self, path: &str) -> bool {
        match with_decision_service(|svc| svc.remove_entry(path)) {
            Ok(_) => true,
            Err(err) => throw_portable_error(err),
        }
    }
}

```

---

### How to use this in JavaScript

The final result is a modern API that feels like a native JavaScript library.

```javascript
import init, {DecisionEngine, DecisionService} from './pkg/your_wasm.js';

await init();

// Stateless Usage
const result = DecisionEngine.evaluateExpression("10 + 20");

// Stateful Usage
const service = new DecisionService(initialModelJson);

// Get current model via getter
console.log(service.model);

// Mutations with clean names
service.set("rules.discount", {value: 0.15});
const response = service.execute("calculateTotal", {price: 100});

// Removal
const success = service.remove("rules.obsolete");

```

### Benefits of this Approach

* **Logical Separation:** It is immediately obvious which methods require an initialized model and which do not.
* **Concise Naming:** You no longer need to include "decision_service_model" in every function name because the context
  is provided by the class instance.
* **Standard Tooling:** IntelliJ and VS Code will group these methods correctly in the autocomplete dropdown, making the
  API much easier to learn.

### Next Steps

- [ ] Implement proposed changes in the codebase.
- [ ] Update `wasm`, `wasm-js`, `wasm-performance` tests to use the new API structure.
- [ ] Make sure all Just tests pass.
- [ ] Update README.md and other documentation to reflect the new API design.
- [ ] Review implementation and check tasks that are done in `NAMESPACE_API.md`
- [ ] Fix `to_schema` - this method can only be invoked on `ContextObject` as it is right now, but it skips the type
  information for functions and types itself. `to_schema` will return linked types only of expression entries.
- [ ] Add the new method `get_expression_type` near `get_expression`. `get_expression_type` will return the type of
  the expression at a given path. Method will work the same as `get_expression` - it will accept path as string.
  `get_expression_type` can will return the return types of functions as well and linked expression types.
  This method will not be able to return types of type definitions: it will return None option in that case.
- [ ] Add the new method `getType` that will return the type of the entry at a given path in the `DecisionService`
  class:

```typescript

// assume provieded model is in portable format:
const service = new DecisionService(`func eligibilityDecision(applicant): {
        rules: [
            {name: "INC_CHECK"; rule: applicant.income > applicant.expense * 2}
            {name: "MIN_INCOM"; rule: applicant.income > 1000}
            {name: "AGE_CHECK"; rule: applicant.age >= 18}
        ]
        result: {
            firedRules: for invalid in rules[rule = false] return invalid.name
            status: if count(rules) = 0 then "ELIGIBLE" else "INELIGIBLE"
        }
    }
    applicantEligibility: eligibilityDecision({
        income: 1100
        expense: 600
        age: 22
    }).result`);

const entryType = service.getType("applicantEligibility");
assert.strictEqual(entryType, {firedRules: "string[]", status: "string"});

// can get return type of function entry
const entryType2 = service.getType("eligibilityDecision");
assert.strictEqual(entryType2, {firedRules: "string[]", status: "string"});

// can get type of specific nested entry
const entryType3 = service.getType("applicantEligibility.status");
assert.strictEqual(entryType3, "string");

// can also get type of nested entries from function
const entryType4 = service.getType("eligibilityDecision.firedRules");
assert.strictEqual(entryType4, "string[]");
```

`getType` should work the same as now we have `get` method, but will use `get_expression_type`. See
`example_ruleset_collecting`
for reference implementation in Rust.