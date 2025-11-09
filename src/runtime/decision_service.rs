use crate::ast::context::context_object::ContextObject;
use crate::ast::metaphors::metaphor::UserFunction;
use crate::ast::token::ExpressionEnum;
use crate::link::linker::link_parts;
use crate::runtime::edge_rules::{
    ContextUpdateErrorEnum, EdgeRulesModel, EdgeRulesRuntime, EvalError, MethodEntry,
};
use crate::typesystem::errors::{ParseErrorEnum, RuntimeError};
use crate::typesystem::values::ValueEnum;
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
        model
            .merge_context_object(Rc::clone(&context))
            .map_err(Self::context_update_error)?;

        Ok(Self {
            model: Rc::new(RefCell::new(model)),
            static_context: context,
            runtime_dirty: false,
        })
    }

    /// Parses EdgeRules DSL source and links it into a reusable decision service.
    pub fn from_source(source: &str) -> Result<Self, EvalError> {
        let mut model = EdgeRulesModel::new();
        model.append_source(source).map_err(EvalError::from)?;
        Self::from_model(model)
    }

    /// Executes a decision-service method with the provided request payload.
    pub fn execute(
        &mut self,
        service_method: &str,
        decision_request: ValueEnum,
    ) -> Result<ValueEnum, EvalError> {
        let method_path = Self::clean_method_name(service_method)?;
        let runtime_method_name = Self::runtime_method_name(&method_path);

        let method_entry = self.resolve_method_entry(&method_path)?;
        let parameter_count = {
            let borrowed = method_entry.borrow();
            borrowed.function_definition.get_parameters().len()
        };
        Self::ensure_single_argument(&method_path, parameter_count)?;

        let runtime = self.ensure_runtime()?;
        runtime
            .call_method(
                runtime_method_name,
                vec![ExpressionEnum::from(decision_request)],
            )
            .map_err(EvalError::from)
    }

    #[cfg(feature = "mutable_decision_service")]
    pub fn get_model(&mut self) -> Rc<RefCell<EdgeRulesModel>> {
        self.runtime_dirty = true;
        Rc::clone(&self.model)
    }

    pub(crate) fn from_model(mut model: EdgeRulesModel) -> Result<Self, EvalError> {
        let runtime = model.to_runtime_snapshot()?;
        Ok(Self {
            model: Rc::new(RefCell::new(model)),
            static_context: Rc::clone(&runtime.static_tree),
            runtime_dirty: false,
        })
    }

    fn ensure_runtime(&mut self) -> Result<EdgeRulesRuntime, EvalError> {
        if self.runtime_dirty {
            let runtime = self.model.borrow_mut().to_runtime_snapshot()?;
            self.static_context = Rc::clone(&runtime.static_tree);
            self.runtime_dirty = false;
            return Ok(runtime);
        }

        Ok(EdgeRulesRuntime::new(Rc::clone(&self.static_context)))
    }

    fn resolve_method_entry(
        &self,
        method_path: &str,
    ) -> Result<Rc<RefCell<MethodEntry>>, EvalError> {
        self.model
            .borrow()
            .get_user_function(method_path)
            .ok_or_else(|| Self::config_error(format!("Method '{}' not found", method_path)))
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
            return Err(Self::config_error(
                "Decision service method name cannot be empty",
            ));
        }
        Ok(trimmed.to_string())
    }

    fn runtime_method_name(method_path: &str) -> &str {
        method_path.rsplit('.').next().unwrap_or(method_path)
    }

    fn context_update_error(err: ContextUpdateErrorEnum) -> EvalError {
        EvalError::from(ParseErrorEnum::from(err))
    }

    fn config_error(message: impl Into<String>) -> EvalError {
        EvalError::from(RuntimeError::eval_error(message.into()))
    }
}
