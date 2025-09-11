# Enable first‑class user functions

Enable first‑class user functions to be passed as arguments to built‑in
higher‑order functions (e.g., sort(list, mySortFunction), future map,
filter, reduce), without caching returned results across invocations, while
reusing the function definition itself for efficiency.

## Core Concepts

- Function Value: A new first‑class runtime value that references a user
  function definition plus the lexical parent context it is defined in.
- Function Type: A new static type to describe parameter and return types
  of user functions (arity and types are important for linking/validation of
  higher‑order calls).
- Stateless Invocation: Invocation creates a fresh evaluation context each
  time, populated with the call arguments; no result memoization is retained
  between invocations.

## Syntax & Authoring

- Named function values (v1, supported now):
- Define a function in any context:

```edgerules
mySortFunction(x,y) : { result: x.s > y.s or (x.s = y.s and x.n < y.n) }
```

- Pass by name without parentheses: sortedList : sort(list,
  mySortFunction)
- Anonymous/inline lambdas (v2, optional later):
- Target shape (defer implementation): sort(list, function(x,y) x < y)
  or sort(list, (x,y) => x < y)
- Return location:
- Prefer _return (engine’s RETURN_EXPRESSION) if present; otherwise,
  read result field.
- Return must evaluate to boolean for comparator use.

## Parsing & Tokenization

- No grammar change for v1: an identifier used as an argument is parsed as a
  variable expression.
- When browsing variable paths, if the resolved content is a function
  definition (metaphor), surface it as a function value at runtime (not as
  an error).
- Keep existing call syntax for invocation (myFn(a,b)). Only treat bare
  identifiers (no ()) as function values.

## Linking & Types

- Add ValueType::FunctionType { params: Vec<ValueType>, returns:
  ValueType }.
    - For user functions with unspecified argument types, their ValueType
      may be UndefinedType at link time; the linker can still validate arity.
- EObjectContent linking:
- For MetaphorRef(FunctionDefinition), return FunctionType with
  parameter types derived from FormalParameter.value_type, and a return
  type of BooleanType if _return/result can be linked to boolean; else
  UndefinedType.
- Validation rules per higher‑order function:
- sort(list, comparator):
- Accept right arg as:
- `String` → field-name comparator (shortcut; already supported).
- `FunctionType([T, T] -> Boolean)` → named comparator.
- Validate arity = 2; item type T compatible with `list`’s element type
  (same type or supertype/object that can accept that item in practice). If
  only `UndefinedType` is known, allow and defer exact checks to runtime.
- Validate (if known) returns boolean; otherwise runtime‑check.
- General HOF (future): accept FunctionType with arity and types appropriate
  to the operator (e.g., filter expects [T] -> Boolean, map expects [T] -> U,
  reduce expects [Acc, T] -> Acc).

## Runtime Value Model

- Extend ValueEnum with a FunctionValue variant:
- Holds an Rc to the defining MethodEntry (FunctionDefinition/metaphor)
  and an Rc to the lexical parent ContextObject (or a compact adapter to
  produce a FunctionContext).
- Does not hold any result cache.
- Evaluating a variable that resolves to MetaphorRef(FunctionDefinition)
  must yield ValueEnum::FunctionValue.
- Evaluating a user function call (e.g., myFn(a,b)) remains as is (builds an
  ExecutionContext for results), and is distinct from a function value.

## Invocation Semantics (Stateless)

- Calling a FunctionValue:
- Build a FunctionContext by binding the provided arguments by position
  to the function’s formal parameters.
- Create a fresh ExecutionContext per call (create_eval_context), with
  only input values stacked. No reuse of prior call state; no cross‑call
  memoization.
- Determine the comparator result:
- If `_return` exists: evaluate it.
- Else if `result` exists: evaluate it.
- Else: runtime error “Function has no return/result”.
- For sort, interpret boolean as “x precedes y” when true. Ensure comparator
  is strict and consistent; otherwise, sort remains well‑defined but stability
  may be used to resolve ties.

## Error Handling

- Linking errors:
- “Function ‘name’ expected 2 arguments, but got N” when the
  comparator’s arity mismatches.
- “Unexpected type ‘type’, expected ‘function(…, …) -> boolean’” if the
  second argument is neither a field name string nor a function.
- If list element type conflicts with comparator param types (when
  known): “types X and Y must match”.
- Runtime errors:
- “Comparator did not produce boolean” if evaluated return is not
  boolean.
- “Cannot access field ‘f’ on value ‘…’” for field-name sort when items
  lack the field.
- “Function has no return/result” when neither _return nor result is
  present.

### Sort Behavior

- Argument forms:
- sort(list) → natural order (numbers, strings, fallback to stringified
  compare).
- sort(list, "field") → sort by named field on object items (already
  implemented).
- sort(list, mySortFunction) → call comparator with (x, y) items.
- Stability:
- Prefer stable sorting to ensure deterministic ordering when comparator
  returns equal conditions frequently (ties).

### No Caching Constraint

- FunctionValue is a thin descriptor (definition + lexical context) and must
  not own any computed result cache.
- Each call constructs a new ephemeral ExecutionContext; results are not
  stored in a global map and are discarded after the call completes.
- The only allowed caching is intra‑call evaluation in the ephemeral context
  (existing stack) to avoid re‑computing subexpressions within a single
  comparator invocation.

## Extensibility

- This mechanism generalizes to:
- filter(list, predicateFn) where predicateFn(T) -> boolean.
- map(list, mapperFn) where mapperFn(T) -> U.
- reduce(list, reducerFn, initial) where reducerFn(Acc, T) -> Acc.
- Named functions stay the primary mechanism. Anonymous functions can be
  added later by synthesizing a FunctionDefinition/FunctionContext at parse
  time for the lambda expression.

## Implementation Plan (for the coding agent)

- Types
- Add ValueType::FunctionType { params: Vec<ValueType>, returns:
  ValueType }.
- Add ValueEnum::FunctionValue(FunctionRef); FunctionRef holds:
- `method: Rc<RefCell<MethodEntry>>`
- `defining_context: Rc<RefCell<ContextObject>>`
- Linking
- In EObjectContent<ContextObject>::link, implement
  MetaphorRef(FunctionDefinition):
- Produce `FunctionType` from `FormalParameter`s and link `_return` or
  `result` for return type if possible; else `UndefinedType`.
- Browsing/Evaluation
- In BrowseResultFound::eval, implement MetaphorRef to produce
  ValueEnum::FunctionValue.
- Higher‑order built‑ins
- Update validators to accept FunctionType in addition to current
  accepted forms.
- For sort, detect:
- `String` (field) → existing path.
- `FunctionValue` → comparator path: for each comparison, create fresh
  eval context with `(x, y)`, evaluate boolean `_return` or `result`, and
  order accordingly (stable sort).
- Tests
- Positive: sort(list, mySortFunction) with the provided example;
  cross‑type lists.
- Negative: wrong arity, wrong return type, param type conflicts,
  missing result/_return, missing item fields for field sort.
- Non‑caching: repeated sort on same data with comparator having side
  effects simulated via a monotonic counter in the function’s body should
  not leak state across invocations (counter should reset per call or be
  explicitly disallowed).

## Performance & Safety Notes

- Sorting invokes the comparator O(n log n) times; building a fresh
  ExecutionContext per invocation is expected. If needed, we can pool
  contexts, but must ensure no cross‑call cached results remain (clear stack).
- Comparators should be side‑effect‑free and must not mutate the list
  elements or global context; we can document this restriction and optionally
  detect writes to the parent context during comparator execution.

## Backwards Compatibility

- Existing named function calls and field‑name sort remain unchanged.
- Passing a function by name without () only changes behavior where a
  function definition is in scope; otherwise, unchanged variable resolution
  applies.

This spec gives us a clear path to add first‑class function values and
use them in sort (and other HOFs), reusing function definitions, per‑call
stateless evaluation, and robust linking/runtime validation.