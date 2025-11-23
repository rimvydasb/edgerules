# Ideal Errors Story

There are various problems related to errors in EdgeRules.
Most of them already marked with `@todo` in the code.
This story gradually unfolds the ideal error handling strategy for EdgeRules.

## Introduction

1. `ParseErrors` that must fail source loading - no linking or even execution must be started. Prefixed with `[parse]`
2. `LinkingErrors` that must fail linking - no execution must be started. Prefixed with `[link]`
3. `RuntimeError` that must fail execution. Prefixed with `[run]`

## Tasks

### unclear or unneeded parts

All existing tests, while run with coverage, do not touch this part of the code.
Try to find out why it is needed and either cover with test case or remove it.

```rust
other => {
    // @Todo: why this part is even needed?
    let literal = other.into_string_or_literal()?;
    ComparatorEnum::try_from(literal.as_str())?
}
```