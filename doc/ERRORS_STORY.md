# Ideal Errors Story

There are various problems related to errors in EdgeRules.
Most of them already marked with `@todo` in the code.
This story gradually unfolds the ideal error handling strategy for EdgeRules.

## Introduction

1. `ParseErrors` that must fail source loading - no linking or even execution must be started. Prefixed with `[parse]`
2. `LinkingErrors` that must fail linking - no execution must be started. Prefixed with `[link]`
3. `RuntimeError` that must fail execution. Prefixed with `[run]`

## Tasks

### complete normal error stacking

It is not good that previous errors is simply formatted as string.
Find out a better way to stack errors.

```rust
impl ParseErrorEnum {
    // @todo: complete normal error stacking
    pub fn before(self, before_error: ParseErrorEnum) -> ParseErrorEnum {
        if before_error == ParseErrorEnum::Empty {
            return self;
        }

        UnknownError(format!("{} → {}", before_error, self))
    }
}
```

### enums fixes

Review all `@todo` in the error enums and fix them gradually.

```rust
    // @todo: InvalidType is used only with `Expected expression, got definition` - use WrongFormat instead
    // also, "Expected expression, got definition" is not even covered with tests - is it even possible to reach that error?
    InvalidType(String),

    // @todo: UnknownParseError must be split to OtherError and WrongFormat
    UnknownParseError(String),

    // @Todo: use WrongFormat where possible instead of UnknownParseError if issue is format related
    // expected format description
    // WrongFormat {
    //     expected_format: String,
    // },

    // @Todo: UnknownError must be removed, use UnknownParseError instead
    UnknownError(String),

    // @Todo: rename to UnexpectedEnd
    Empty,
```