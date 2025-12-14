# Actionable TODO List

## Critical

- Refactor `UserFunctionRef`: Implement `UserFunctionRef` handling in `EObjectContent::link` and `BrowseResultFound::eval` (currently panics with `todo!`).
- Remove `UndefinedType`: Ensure `ValueType::UndefinedType` never exists in runtime execution to prevent undefined behavior.
- Fix Loop Linking: Fix `link_parts` in `ForFunction` to handle return expressions referring to list item fields correctly.
- Prevent Infinite Loops: Ensure infinite loops do not occur in `ASTBuilder::finalize`.

## Important

- Split `OtherLinkingError`: Refactor `LinkingErrorEnum::OtherLinkingError` into specific error variants.
- Refactor `ValueEnum`: Replace `NumberEnum::SV` with `ValueOrSv` in `ValueEnum` for better type safety.
- Define Range Semantics: Clarify and implement inclusive/exclusive and infinity/static semantics for `ValueEnum::RangeValue`.
- Support Nested Arrays: Implement support for nested arrays in `ArrayValue`.
- Fix `TypeValue`: Update `ValueEnum::get_type` to return the specific `TypeValue` as a type.
- Implement Fractions: Implement `Fraction(numerator, denominator)` in `NumberEnum`.
- Linking Error in Loops: Change the runtime error when iterating through a non-list type to a linking error.
- Enforce Static Linking: Ensure static linking is fully completed in the link phase for `VariableLink`.
- Improve Variable Error: Return `LinkingErrorEnum::FieldNotFound` instead of `UndefinedType` for unlinked variables.
- Clarify `StaticObject`: Determine if `StaticObject` needs separate linking or if it's the caller's responsibility.
- Enforce Homogeneous Lists: Throw a linking error if a collection contains elements of different types.
- Aggregate Types: Implement type aggregation for collections containing multiple objects with different structures.
- Prevent Duplicates: Add a check to prevent adding duplicate fields in `ContextObjectBuilder`.
- Propagate Errors: Return and propagate errors when adding expressions in `ContextObjectBuilder`.
- Check Equality Types: Update `ContextObject::eq` to evaluate types as well.
- Review Context Mutability: Clarify why object in `ExecutionContext` is mutable or make it immutable.
- Remove Runtime `LinkingError`: Ensure `LinkingError` is not returned in execution context methods.
- Optimize AST Cloning: Find a cheaper way to clone the AST tree in `to_runtime_snapshot`.
- Preserve Links: Preserve already set links to speed up the next linking in `to_runtime_snapshot`.
- Fix `context_unwrap`: Replace the hack in `context_unwrap` with a proper solution.
- Fix Tokenizer Errors: Accumulate errors properly in `ASTBuilder` instead of pushing them back.
- Check Context Level: Check level before adding in `build_context` and `build_sequence`.
- Clarify Operator Logic: Clarify the logic in `build_any_operator` for unparsed left sides.
- Count Brackets: Implement brackets counting and error returning in `tokenize`.
- Implement Range Parsing: Implement range parsing logic in `tokenize`.
- Resolve Deep Nesting: Resolve deep nesting linking errors in tests.
- Unreserve type: Make type not reserved based on context.
- Fix Casting: Throw exception for invalid type casting instead of allowing it.
- Implement Flatten: Implement `flatten` for lists.
- CLI Tests: Add integration test harness for `crates/cli`.
- Decision Service Tests: Add nested path tests for decision service model updates.
- Special Value Literals: Add support for literals for special values.
- Duration Literals: Add support for literal durations.

## Optional

- Refactor Primitives: Move primitive values into a `PrimitiveValue` enum.
- Revisit `RangeValue`: Evaluate if `RangeValue` should be a value or strictly a filter method.
- Refactor `TypeValue`: Rethink `ValueEnum::TypeValue` design.
- Verify `RefCell`: Investigate if `RefCell` is necessary for `ValueType::ObjectType`.
- Remove `RangeType` Display: Remove the `RangeType` display implementation.
- Optimize `iterate_range`: Simplify the `iterate_range` implementation.
- Verify Priorities: Verify if the priority `17` for `FilterArray` is correct.
- Implement `Display`: Implement `Display` trait for `EUnparsedToken`.
- Remove `RefCell`: Attempt to remove `RefCell` from `ExpressionEnum::StaticObject`.
- Move `ObjectField`: Move `ObjectField` to `EUnparsedToken`.
- Lazy Evaluation: Consider lazy evaluation for `CollectionExpression`.
- Optimize Insertion: Optimize `ContextObjectBuilder` field insertion.
- Review `Rc`: Review `Rc` usage reference.
- Prevent Re-execution: Investigate preventing re-execution in `link_parts`.
- Debug Display: Trace calls to "OrphanedChild" path in `NodeDataEnum::fmt`.
- Verify Internal Node: Run tests with coverage for Internal node path.
- Deprecate `push_element`: Deprecate `ASTBuilder::push_element`.
- Move Operators: Move math operator handling to `ASTBuilder`.
- Simplify Comparators: Simplify comparator operator acquisition.
- Improve Error Location: Improve error location reporting in tests.
- Support Origin Override: Implement ability to override Special Value origin.
- More Date/Time Functions: Add support for more date/time functions.

## TODOs Needing Clarification

- `crates/core/src/typesystem/errors.rs`: "this is absolutely unclear how it happens in runtime, because linking solves types." (TypeNotSupported)
- Ambiguity: The role of `TypeValue` as a value vs. a type definition is ambiguous (`crates/core/src/typesystem/values.rs`).
- `crates/core/src/tokenizer/builder.rs`: Clarify logic in `build_any_operator` (handling of unparsed left tokens).
- `crates/core/src/ast/token.rs`: Question correctness of hardcoded priority value `17` for `FilterArray`.

## High-Risk TODOs

- `crates/core/src/typesystem/types.rs`: `ValueType::UndefinedType` must not exist in runtime; if it does, it's a critical failure in the linking phase.
- `crates/core/src/ast/context/context_object_type.rs`: `todo!("UserFunctionRef")` will cause a panic if encountered during linking or evaluation.
- `crates/core/src/tokenizer/builder.rs`: `finalize` must ensure no infinite loops occur to avoid hanging the parser.
- `crates/core/src/ast/foreach.rs`: `link_parts` failing with unknown field for list items is a core bug for loops over objects.
