# EdgeRules Project: Gemini Manual

This document provides a comprehensive guide for AI agents to understand, build, and contribute to the EdgeRules
project.

## Main Instructions

- Always use Markdown format for documents and documentation.
- Line length is at least 120 characters.
- Tab size is 4 spaces.

## Project Overview

EdgeRules is a lightweight, embeddable, JSON-native business rules engine written in Rust. It is designed for
performance and safety, with a strong focus on small WASM output sizes for edge computing environments. The engine
features a custom DSL for defining rules, static typing, and aims for full DMN FEEL coverage.

The project is structured as a Rust workspace with the following key crates:

*   `crates/core`: Contains the core logic of the rules engine, including the AST, tokenizer, runtime, and type system.
*   `crates/cli`: A command-line interface for interacting with the rules engine.
*   `crates/wasm`: Provides the necessary bindings to compile the core engine to WebAssembly, allowing it to be used in
    JavaScript environments (web and Node.js).
*   `crates/core-tests`: Contains the test suite for the core crate.

## Building and Running

The project uses `just` as a command runner to simplify the build process. The main commands are defined in the
`Justfile`.

### Key Commands

* **Build all artifacts:**
    * Native CLI: `just cli`
    * WASM for Web: `just web`
    * WASM for Node.js: `just node`
    * WASM for Wasmtime: `just wasi`

* **Run tests:**
    * `cargo test --all` or `just test`
    * `just wasm-test`

* **Run linters:**
    * `just clippy`

* **Format code:**
    * `just fmt`

### Development Conventions

*   **Code Style:** The project follows standard Rust formatting, enforced by `rustfmt`.
*   **Testing:** The project has a comprehensive test suite in `crates/core-tests`. New features should be accompanied by
    tests.
*   **WASM:** The WASM build is a key deliverable. When making changes to the core engine, consider the impact on the
    WASM binary size and performance. The `wasm-bindgen` crate is used to generate the JavaScript interface. The
    `crates/wasm/src/lib.rs` file defines the public API for the WASM module.
*   **Error Handling:** The library uses a custom `EvalError` enum to represent parsing and runtime errors. In the WASM
    interface, errors are thrown as JavaScript exceptions.
*   **State Management (WASM):** The WASM module uses a `thread_local` static variable to hold the state of the
    `DecisionServiceController`. This is an important architectural detail to be aware of when working with the WASM
    bindings.

# Codebase Analysis

This analysis covers inconsistencies, design issues, and abstraction levels, focusing on their impact on AI-driven
development, maintainability, and performance.

## Inconsistencies

Several inconsistencies were identified between the project's documentation, stated goals, and implementation.

*   **Contradiction: Global State vs. High Performance**
    *   **Finding:** The EDGE_RULES_API_STORY.md and the implementation in crates/wasm/src/lib.rs confirm that the
        WASM-based DecisionService is a global, static, thread_local instance.
    *   **Impact:** This architectural choice directly contradicts the stated goal of a high-performance engine. It
        introduces global state, which prevents concurrent execution of rules in environments like web workers,
        severely limiting parallelism and scalability. For an AI agent, this global state is a major source of
        confusion, as it's not clear from a function's signature that it will be modifying a hidden, shared object.

### Inconsistency: Default API vs. Stated Intent

**Finding:** The documentation mentions a plan for a lightweight, immutable DecisionService for edge
execution. However, the `mutable_decision_service` feature, which provides an extensive API for an editor GUI,
is enabled by default.

**Impact:** This makes the default API surface larger and more complex than necessary for many use cases. An AI
agent trying to use the library for a simple evaluation task might be exposed to a confusing array of mutation
methods, increasing the risk of incorrect usage.

### Inconsistency: Data Portability vs. Metadata Loss

**Finding:** The "Known Limitations" in EDGE_RULES_API_STORY.md explicitly state that the EdgeRulesModel API
loses metadata like `@version` and `@model_name`.

**Impact:** This undermines the purpose of the "EdgeRules Portable" format, which is designed for persistence
and lossless transfer of models. When an agent saves and reloads a model, it will unexpectedly lose data,
leading to subtle bugs.

### Design Issues

The codebase exhibits several design patterns that reduce clarity and increase cognitive load for both human and AI
developers.

*   **Design Flaw: Global Static Instance in WASM**
    *   **Finding:** As mentioned above, the `thread_local!` static variable for the DecisionService is a significant
        design choice.
    *   **Impact:** The WASM module uses a `thread_local!` static variable to hold the state of the
        `DecisionServiceController`. This is an architectural choice that simplifies state management and avoids
        concurrency issues in typical web scenarios. It is designed for single-threaded usage in web or Node.js
        contexts where each WASM module instance is isolated per thread or request.

*   **Weak Abstraction: Confusing Separation of Concerns**
    *   **Finding:** There's an unclear separation of concerns between the core and wasm crates. The core crate has a
        `wasm` module inside it (crates/core/src/wasm.rs), while the wasm crate (crates/wasm/) also contains
        WASM-specific logic.
    *   **Impact:** This makes it difficult to understand the project's structure. An AI agent looking for WASM-related
        code has to search in multiple places. A cleaner design would be for the core crate to be completely
        agnostic of WASM, with the `wasm` crate handling all FFI and data conversion.

*   **Tight Coupling: Low-Level AST Manipulation in Public API**
    *   **Finding:** The programmatic API, as described in EDGE_RULES_API_STORY.md, requires the user to manually
        construct AST nodes (e.g., `ExpressionEnum::from(true)`).
    *   **Impact:** This is a low-level, tightly coupled design that exposes implementation details to the user. It's
        verbose, error-prone, and requires the user to have a deep understanding of the internal AST structure. For
        an AI, generating code against such an API is difficult and brittle. A small change in the AST structure
        would break all client code.

### Boilerplate and Abstraction (DRY vs. KISS)

*   **Excessive Boilerplate: Programmatic API**
    *   **Finding:** The current API for building models programmatically is extremely boilerplate-heavy due to the
        direct AST manipulation.
    *   **Recommendation:** Introduce a higher-level "builder" API that uses the Builder pattern to construct models.
        This would provide a more ergonomic, fluent interface and hide the underlying AST structure.

*   **Potential Under-abstraction: WASM Data Conversion**
    *   **Finding:** The `wasm_convert.rs` file likely contains a significant amount of repetitive code for converting
        between Rust's `ValueEnum` and JavaScript's `JsValue`.
    *   **Recommendation:** Investigate using a more generic approach, potentially leveraging macros or a serialization
        framework like `serde` (with `serde-wasm-bindgen`), to reduce this boilerplate. This would make the
        conversion logic more maintainable and less error-prone. **Caution:** `serde` can increase WASM binary size,
        so this needs to be benchmarked.

## Code Improvement Plan

Here are 10 actionable recommendations to address these issues, prioritized by their impact on maintainability,
performance, and AI-driven development.

### Eliminate Global State in WASM

**Explanation:** Refactor the WASM API to be stateless. The `create_decision_service` function should return a
handle (e.g., a `JsValue` wrapping a pointer or an index) to the DecisionService instance. Subsequent calls
(`execute_decision_service`, etc.) should take this handle as an argument.

**Targets:** Maintainability, Performance, Readability.

### Decouple Core and WASM Crates

**Explanation:** Move all WASM-specific code from `crates/core/src/wasm.rs` into the `crates/wasm` crate. The core
crate should be completely agnostic of its consumers (WASM, CLI, etc.).

**Targets:** Maintainability, Readability.

### Introduce a High-Level Builder API

**Explanation:** Create a fluent "builder" API for programmatically constructing EdgeRulesModel instances. This
will abstract away the low-level AST manipulation and provide a more ergonomic and stable API.

**Targets:** Boilerplate Reduction, Maintainability, Readability.

### Preserve Metadata in Portable Format

**Explanation:** Fix the EdgeRulesModel API to correctly handle metadata (`@version`, `@model_name`) when
serializing and deserializing the "EdgeRules Portable" format.

**Targets:** Maintainability, Correctness.

### Make mutable_decision_service Opt-In

**Explanation:** Change the `mutable_decision_service` feature to be opt-in rather than opt-out. The default API
should be the minimal, immutable one intended for edge execution.

**Targets:** Readability, Maintainability.

### Abstract WASM Data Conversion

**Explanation:** Refactor the `wasm_convert.rs` module to reduce boilerplate. Consider using more
generic approach to handle the conversion between `ValueEnum` and `JsValue`. Try not using macros.
Check if all tests pass after the refactor. Stop refactoring if there's a risk to increase WASM size.

**Targets:** Boilerplate Reduction, Maintainability, DRY/KISS Balance.

Next Steps: review `wasm_convert.rs` and related tests. Propose a refactored design.

### Improve Error Handling in WASM

**Explanation:** The current error handling in WASM throws JavaScript errors with simple string messages. Use
custom JavaScript Error subclasses to provide more structured error information (e.g., error codes, context).

**Targets:** Maintainability, Testability.

### Add Comprehensive Benchmarks

**Explanation:** Create a comprehensive benchmarking suite using a tool like `criterion`. This should cover both the
native and WASM builds, and track performance and binary size over time.

**Targets:** Performance, WASM Size.

### Deprecate and Remove evaluate_method

**Explanation:** Follow through on the `@Todo` in the documentation and remove the `evaluate_method` function to
clean up the API.

**Targets:** Maintainability, Readability.