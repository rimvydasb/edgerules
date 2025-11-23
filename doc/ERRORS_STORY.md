# Ideal Errors Story

There are various problems related to errors in EdgeRules.
Most of them already marked with `@todo` in the code.
This story gradually unfolds the ideal error handling strategy for EdgeRules.

## Introduction

1. `ParseErrors` that must fail source loading - no linking or even execution must be started. Prefixed with `[parse]`
2. `LinkingErrors` that must fail linking - no execution must be started. Prefixed with `[link]`
3. `RuntimeError` that must fail execution. Prefixed with `[run]`

## Tasks

### Reduce Debug usage and eliminate Debug usage in WASM to reduce WASM size

It is important to reduce the size of the WASM binary and eliminate internal
structures (Enums) exposure in the error messages and WASM itself.

[ ] Find out where {:?} is used in trace and comment out those trace functions
[ ] Remove Debug derivation from those structures that are not used in tests
[ ] Include Debug derivation only in non WASM builds
[ ] Run all tests at the end and fix compilation errors
[ ] Run `just demo-node` and check the error messages

The goal is very important to reduce the size of the WASM binary.