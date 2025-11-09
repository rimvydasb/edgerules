pub mod decision_service;
pub mod edge_rules;
pub mod execution_context;

// WASM-only portable decision service API (JsValue based)
#[cfg(all(target_arch = "wasm32", feature = "wasm", feature = "mutable_decision_service"))]
#[path = "portable_js.rs"]
pub mod portable;

pub use decision_service::DecisionService;

pub use crate::typesystem::types::ToSchema;
