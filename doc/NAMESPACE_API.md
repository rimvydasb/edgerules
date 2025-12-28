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