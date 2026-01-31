use crate::ast::context::context_object::ContextObject;
use crate::ast::expression::cast_value_to_type;
use crate::ast::metaphors::metaphor::UserFunction;
use crate::ast::token::ExpressionEnum;
use crate::link::linker::link_parts;
use crate::runtime::edge_rules::{ContextQueryErrorEnum, EdgeRulesModel, EdgeRulesRuntime, EvalError, MethodEntry};
use crate::typesystem::errors::RuntimeError;
use crate::typesystem::types::ValueType;
use crate::typesystem::values::ValueEnum;
use crate::runtime::execution_context::ExecutionContext;
use std::cell::RefCell;
use std::rc::Rc;

/// Maintains a reusable rules model and linked runtime tree for decision-service style execution.
pub struct DecisionService {
    model: Rc<RefCell<EdgeRulesModel>>,
    static_context: Rc<RefCell<ContextObject>>,
    runtime_dirty: bool,
}

impl DecisionService {
    /// Builds a decision service from an already linked context tree.
    pub fn from_context(context: Rc<RefCell<ContextObject>>) -> Result<Self, EvalError> {
        link_parts(Rc::clone(&context))?;

        let mut model = EdgeRulesModel::new();
        model.merge_context_object(Rc::clone(&context))?;

        Ok(Self { model: Rc::new(RefCell::new(model)), static_context: context, runtime_dirty: false })
    }

    /// Parses EdgeRules DSL source and links it into a reusable decision service.
    pub fn from_source(source: &str) -> Result<Self, EvalError> {
        let mut model = EdgeRulesModel::new();
        model.append_source(source).map_err(EvalError::from)?;
        Self::from_model(model)
    }

    /// Executes a decision-service method with the provided request payload.
    pub fn execute(&mut self, service_method: &str, decision_request: ValueEnum) -> Result<ValueEnum, EvalError> {
        let method_path = Self::clean_method_name(service_method)?;
        let runtime_method_name = Self::runtime_method_name(&method_path);

        let runtime = self.ensure_runtime()?;
        let method_entry = self.resolve_method_entry(&method_path)?;
        let (parameter_count, param_type_opt) = {
            let borrowed = method_entry.borrow();
            let params = borrowed.function_definition.get_parameters();
            let count = params.len();
            // Get the declared type of the single argument if available
            let type_opt = if count == 1 { params[0].declared_type().cloned() } else { None };
            (count, type_opt)
        };
        Self::ensure_single_argument(&method_path, parameter_count)?;

        let final_request = if let Some(tref) = param_type_opt {
            let expected_type =
                runtime.context.borrow().object.borrow().resolve_type_ref(&tref).unwrap_or(ValueType::UndefinedType);
            cast_value_to_type(decision_request, expected_type, Rc::clone(&runtime.context), Some("Request"))
                .map_err(EvalError::from)?
        } else {
            decision_request
        };
        runtime.call_method(runtime_method_name, vec![ExpressionEnum::from(final_request)]).map_err(EvalError::from)
    }

    /// Evaluates a field by path in the decision service.
    /// Mainly used for testing.
    pub fn evaluate_field(&mut self, path: &str) -> Result<ValueEnum, EvalError> {
        let runtime = self.ensure_runtime()?;
        runtime.evaluate_field(path).map_err(EvalError::from)
    }

    #[cfg(feature = "mutable_decision_service")]
    pub fn get_model(&mut self) -> Rc<RefCell<EdgeRulesModel>> {
        self.runtime_dirty = true;
        Rc::clone(&self.model)
    }

    pub fn from_model(mut model: EdgeRulesModel) -> Result<Self, EvalError> {
        let runtime = model.to_runtime_snapshot()?;

        // Link all root functions to ensure they are valid entry points.
        // Any root function can be executed, so we must ensure they are free of static errors.
        {
            let ctx = runtime.static_tree.borrow();
            for entry in ctx.metaphors.values() {
                let borrowed_entry = entry.borrow();
                if let Ok(body) = borrowed_entry.function_definition.get_body() {
                    link_parts(Rc::clone(&body)).map_err(EvalError::from)?;
                }
            }
        }

        Ok(Self {
            model: Rc::new(RefCell::new(model)),
            static_context: Rc::clone(&runtime.static_tree),
            runtime_dirty: false,
        })
    }

    fn ensure_runtime(&mut self) -> Result<EdgeRulesRuntime, EvalError> {
        if self.runtime_dirty {
            let runtime = self.model.borrow_mut().to_runtime_snapshot()?;
            {
                let ctx = runtime.static_tree.borrow();
                for entry in ctx.metaphors.values() {
                    let borrowed_entry = entry.borrow();
                    if let Ok(body) = borrowed_entry.function_definition.get_body() {
                        link_parts(Rc::clone(&body)).map_err(EvalError::from)?;
                    }
                }
            }
            self.static_context = Rc::clone(&runtime.static_tree);
            self.runtime_dirty = false;
            return Ok(runtime);
        }

        Ok(EdgeRulesRuntime::new(Rc::clone(&self.static_context)))
    }

    pub fn get_linked_type(&mut self, path: &str) -> Result<ValueType, ContextQueryErrorEnum> {
        let _ = self.ensure_runtime().map_err(|err| ContextQueryErrorEnum::ContextNotFoundError(err.to_string()))?;
        EdgeRulesRuntime::new(Rc::clone(&self.static_context)).get_type(path)
    }

    pub fn rename_entry(&mut self, old_path: &str, new_path: &str) -> Result<(), EvalError> {
        self.runtime_dirty = true;
        self.model.borrow_mut().rename_entry(old_path, new_path).map_err(EvalError::from)
    }

    #[cfg_attr(not(all(target_arch = "wasm32", feature = "wasm")), allow(dead_code))]
    pub fn ensure_linked(&mut self) -> Result<(), EvalError> {
        self.ensure_runtime().map(|_| ())
    }

    fn resolve_method_entry(&self, method_path: &str) -> Result<Rc<RefCell<MethodEntry>>, ContextQueryErrorEnum> {
        self.model.borrow().get_user_function(method_path)
    }

    fn ensure_single_argument(method_path: &str, params: usize) -> Result<(), EvalError> {
        if params != 1 {
            return Err(Self::config_error(format!(
                "Decision service method '{}' must declare exactly one argument, found {}",
                method_path, params
            )));
        }
        Ok(())
    }

    fn clean_method_name(service_method: &str) -> Result<String, EvalError> {
        let trimmed = service_method.trim();
        if trimmed.is_empty() {
            return Err(Self::config_error("Decision service method name cannot be empty"));
        }
        Ok(trimmed.to_string())
    }

    fn runtime_method_name(method_path: &str) -> &str {
        method_path.rsplit('.').next().unwrap_or(method_path)
    }

    fn config_error(message: impl Into<String>) -> EvalError {
        EvalError::from(RuntimeError::eval_error(message.into()))
    }
}
