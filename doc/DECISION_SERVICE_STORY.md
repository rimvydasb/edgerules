# Decision Service Story

EdgeRules must expose decision service capabilities.

## Terminology:

- **stand-alone model** - evaluatable model that does not have any external context or requires any input
- **decision service** - an API with a model that that exposes decision service method to evaluate with a given context and input
- **decision request** - a request object passed to the decision service model to the decision service method
- **decision service method** - a method defined in a service model. There can be multiple decision methods in a service model
- **decision response** - the result of a decision service method call
- **decision service model** - an EdgeRules model that contains at least one decision service method
- **extended context** - a context that contains input data, provided context, decision service model and evaluated expressions
- **decision trace** - a trace of the decision service execution that contains final extended content
and all intermediate steps (TBC)

## Requirements:

- decision service model must be linked and reused for the next **decision service method** call
to avoid re-linking and unnecessary overhead
- after each **decision service method** call, the decision service model must be reusable 
for the next call without any side effects from the previous execution
- If decision service has none or more than one argument, return an error with a proper message

## Limitations and Notes:

- WASM has a poor multi-threading support, so the decision service model can be locked for the next call
until the previous call is finished. True multithreading functionality is postponed until WASM supports it.
- Decision trace is postponed until the basic functionality is implemented and tested.
- As it appears, decision service method can have only one input parameter that is a **decision request** object.

## Decision Service API

- `DecisionService::new(service_name: &str, model: &str) -> Result<DecisionService, Error>`
1. Parses and links the given model code string
2. Creates a new decision service with the given name
3. Returns an error if the model is not linked

- `DecisionService::evaluate(&self, service_method: &str, service_method: &context, decision_request: &str) -> Result<ValueEnum, EvalError>`
1. Takes the linked decision service model and ensures that it will not be changed during the evaluation
2. Finds function that will be used as a service method:
- function must be defined in the model on the top level
- function must have the same name as the given service method
- function must have exactly one input parameter
3. Applies the given context on top of the decision service model
4. Calls the service method with the given decision request as a parameter

## WASM API

- creates a new decision service from the given model string. The given model must be linked and ready for evaluation.
```edgerules
create_decision_service(
    service_name: *const c_char, 
    model: *const c_char
) -> String 
```

- evaluates the decision  service model with the given context and decision request. 
The result is a string representation of the evaluated model.

```edgerules
evaluate_decision_service(
    service_name: *const c_char, 
    service_method: *const c_char, 
    context: *const c_char,
    decision_request: *const c_char
) -> String
```

## WASM Intermediate API

Upgrade wasm API to work with JavaScript values instead of strings:

- `evaluate_all(code: &str) -> JsValue` – loads model code and returns the fully evaluated model as JSON output.
- `evaluate_expression(code: &str) -> JsValue` – evaluates a standalone expression and returns the result as JavaScript value.
- `evaluate_field(code: &str, field: &str) -> JsValue` – loads `code`, then evaluates a field/path.
- `evaluate_method(code: &str, method: &str, args: &JsValue) -> JsValue` – loads `code`, then calls a top-level method
  with given `args`.

Use `js-sys` and do not use `serde_json`, because serde is too big.
As of now, only primitive types are supported as arguments: numbers, strings, booleans, arrays, date.
As an output, primitives, objects, arrays of objects must be supported.

Example data conversion:

```rust
use wasm_bindgen::prelude::*;
use js_sys::Date;
use time::{Date, OffsetDateTime, UtcOffset};

#[wasm_bindgen]
pub fn convert_js_date(js_date: JsValue) -> Result<String, JsValue> {
    if !js_date.is_instance_of::<Date>() {
        return Err(JsValue::from_str("Expected JS Date"));
    }

    let d: Date = js_date.into();
    // get_time() → milliseconds since UNIX epoch as f64
    let millis = d.get_time();
    let seconds = (millis / 1000.0).trunc() as i64;
    let nanos = ((millis % 1000.0) * 1_000_000.0).round() as i128;

    // Construct OffsetDateTime in UTC
    let odt = OffsetDateTime::from_unix_timestamp(seconds)
        .map_err(|e| JsValue::from_str(&e.to_string()))?
        .replace_nanosecond(nanos as u32)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Extract the Date part
    let date: Date = odt.date();

    Ok(format!("Rust time::Date = {}", date))
}
```

Example JSON creation:

```rust
use wasm_bindgen::prelude::*;
use js_sys::{Object, Reflect};
use wasm_bindgen::JsValue;
use std::collections::HashMap;

#[wasm_bindgen]
pub fn make_json_manual() -> JsValue {
    let mut outer = HashMap::new();
    outer.insert("first", vec![("x", 10), ("y", 20)]);
    outer.insert("second", vec![("a", 1), ("b", 2)]);

    let outer_obj = Object::new();
    for (outer_key, inner_pairs) in outer {
        let inner_obj = Object::new();
        for (k, v) in inner_pairs {
            Reflect::set(&inner_obj, &JsValue::from_str(k), &JsValue::from_f64(v as f64)).unwrap();
        }
        Reflect::set(&outer_obj, &JsValue::from_str(outer_key), &inner_obj).unwrap();
    }
    JsValue::from(outer_obj)
}
```