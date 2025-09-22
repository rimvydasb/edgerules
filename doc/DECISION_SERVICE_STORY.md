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