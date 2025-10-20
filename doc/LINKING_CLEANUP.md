# Linking CLean Up Story

Linking process prepares code for execution. It is responsible for:
1. Resolving function calls
2. Resolving type references
3. Type checking and wiring
4. Optimisations

From the code perspective linking does not end execution in execution service failure and errors are still propagated into runtime.
To fully solve this problem (and additional ones), following changes must be introduced:

## If any linking error, execution must not start

Find out `fn into_runtime(error: LinkingError) -> Self` - this could be a good starting point solving this puzzle, 
because in general, Linking errors must be passed to the user and runtime must not even start.

Interesting that `pub fn to_runtime(self) -> Result<EdgeRulesRuntime, LinkingError> {...` and
`pub fn to_runtime_snapshot(&mut self) -> Result<EdgeRulesRuntime, LinkingError> {...`
already propagates LinkingError so it is absolutely unclear why `into_runtime` exists at all.

## Errors are of three types that must be mentioned in the error itself for a better debugging

1. `ParseErrors` that must fail source loading - no linking or even execution must be started. Prefix error with `[parse]`
2. `LinkingErrors` that must fail linking - no execution must be started. Prefix error with `[link]`
3. `RuntimeError` that must fail execution. Prefix error with `[run]`

Create test cases for all three scenarios and cover all `ParseErrorEnum`, `LinkingErrorEnum` and `RuntimeErrorEnum` with tests.

## Clean up

1. Remove `RuntimeErrorEnum::LinkingError` - it is not needed anymore
2. Find and fix test `let end = runtime.evaluate_field("calendar.config.end")?` - todo is in the code

