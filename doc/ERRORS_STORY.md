# Ideal Errors Story

There are various problems related to errors in EdgeRules.
Most of them already marked with `@todo` in the code.
This story gradually unfolds the ideal error handling strategy for EdgeRules.

## Introduction

1. `ParseErrors` that must fail source loading - no linking or even execution must be started. Prefixed with `[parse]`
2. `LinkingError` that must fail linking - no execution is started. Prefixed with `[link]`
3. `RuntimeError` that must fail execution. Prefixed with `[run]`

## Tasks
