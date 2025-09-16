# Decision Service Story

EdgeRules must expose decision service capabilities.

## Terminology:

- **decision request** - a request to evaluate a decision service model with a given context
- **decision response** - the result of evaluating a decision service model with a given context
- **decision service model** - an EdgeRules model that is evaluated as a decision service

## Requirements:

- decision service model must be linked and reused for the next decision request to avoid re-linking
  and unnecessary overhead
- decision service model must be evaluated with a given context (decision request)
- after each execution decision service model must be reusable for the next decision request
  without any side effects from the previous execution

## Decision Service API

- `DecisionService::new(service_name: &str, model: &str) -> Result<DecisionService, Error>` - creates a new decision
  service
  from the given model string. The given model must be linked and ready for evaluation.
- `DecisionService::evaluate(&self, context: &str) -> Result<ValueEnum, EvalError>` - evaluates the decision service model
  with the given context string. The result is a string representation of the evaluated model.

## WASM API

- `create_decision_service(service_name: *const c_char, model: *const c_char) -> String` - creates a new decision
  service
  from the given model string. The given model must be linked and ready for evaluation.
- `evaluate_decision_service(service_name: *const c_char, context: *const c_char) -> String` - evaluates the decision
  service model
  with the given context string. The result is a string representation of the evaluated model.