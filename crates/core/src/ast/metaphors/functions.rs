use std::cell::RefCell;

use crate::ast::context::context_object::{ContextObject, ExpressionEntry};
use crate::ast::context::context_object_builder::ContextObjectBuilder;
use crate::ast::context::context_object_type::FormalParameter;
use crate::ast::context::function_context::{FunctionContext, RETURN_EXPRESSION};
use crate::ast::metaphors::metaphor::UserFunction;
use crate::ast::token::ExpressionEnum;
use crate::ast::utils::{array_to_code_sep, trim};
use crate::ast::Link;
use crate::link::linker;
use crate::tokenizer::C_ASSIGN;
use crate::typesystem::errors::{LinkingError, ParseErrorEnum};
use crate::utils::intern_field_name;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use std::rc::Weak;

use crate::typesystem::types::{TypedValue, ValueType};
use std::collections::HashSet;

/// Non executable function definition holder. For an executable function definition see FunctionContext.
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub struct FunctionDefinition {
    pub name: String,
    pub arguments: Vec<FormalParameter>,
    /// Function body later is used as a context object for execution context so it is Rc.
    /// RefCell is added only for linking purposes. Better to remove later.
    pub body: Rc<RefCell<ContextObject>>,
}

impl FunctionDefinition {
    pub fn build(
        name: String,
        arguments: Vec<FormalParameter>,
        body: Rc<RefCell<ContextObject>>,
    ) -> Result<Self, ParseErrorEnum> {
        let mut seen: HashSet<&str> = HashSet::new();
        for argument in &arguments {
            if !seen.insert(argument.name.as_str()) {
                return Err(ParseErrorEnum::OtherError(format!(
                    "Duplicate function argument name '{}'",
                    argument.name
                )));
            }
        }

        Ok(FunctionDefinition {
            name,
            arguments,
            body,
        })
    }
}

impl Display for FunctionDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({}) {} {}",
            self.name,
            array_to_code_sep(self.arguments.iter(), ", "),
            C_ASSIGN,
            self.body.borrow()
        )
    }
}

impl TypedValue for FunctionDefinition {
    fn get_type(&self) -> ValueType {
        ValueType::ObjectType(Rc::clone(&self.body))
    }
}

impl UserFunction for FunctionDefinition {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_parameters(&self) -> &Vec<FormalParameter> {
        &self.arguments
    }

    fn create_context(
        &self,
        parameters: Vec<FormalParameter>,
        parent: Option<Rc<RefCell<ContextObject>>>,
    ) -> Link<FunctionContext> {
        {
            let mut body = match self.body.try_borrow_mut() {
                Ok(b) => b,
                Err(_) => {
                    return Err(LinkingError::new(
                        crate::typesystem::errors::LinkingErrorEnum::CyclicReference(
                            "function".to_string(),
                            self.name.clone(),
                        ),
                    ));
                }
            };
            body.parameters = parameters.clone();
            if parameters.iter().any(|p| p.name == "it") {
                body.allow_it = true;
            }
        }

        linker::link_parts(Rc::clone(&self.body))?;

        let parent_weak = parent.map(|p| Rc::downgrade(&p)).unwrap_or_else(Weak::new);
        let ctx = FunctionContext::create_for(Rc::clone(&self.body), parameters, parent_weak);

        Ok(ctx)
    }
}

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub struct InlineFunctionDefinition {
    pub name: String,
    pub arguments: Vec<FormalParameter>,
    parent: RefCell<Option<Weak<RefCell<ContextObject>>>>,
    cached_body: Rc<RefCell<ContextObject>>,
}

impl InlineFunctionDefinition {
    pub fn build(
        name: String,
        arguments: Vec<FormalParameter>,
        body: ExpressionEnum,
    ) -> Result<Self, ParseErrorEnum> {
        let mut seen: HashSet<&str> = HashSet::new();
        for argument in &arguments {
            if !seen.insert(argument.name.as_str()) {
                return Err(ParseErrorEnum::OtherError(format!(
                    "Duplicate function argument name '{}'",
                    argument.name
                )));
            }
        }

        let mut builder = ContextObjectBuilder::new();
        builder
            .add_expression(RETURN_EXPRESSION, body)
            .map_err(|err| {
                ParseErrorEnum::OtherError(format!("Failed to build inline function body: {}", err))
            })?;
        if arguments.iter().any(|p| p.name == "it") {
            builder.set_allow_it(true);
        }
        let cached_body = builder.build();
        cached_body.borrow_mut().parameters = arguments.clone();

        Ok(InlineFunctionDefinition {
            name,
            arguments,
            parent: RefCell::new(None),
            cached_body,
        })
    }

    pub fn set_parent(&self, parent: &Rc<RefCell<ContextObject>>) {
        *self.parent.borrow_mut() = Some(Rc::downgrade(parent));
        self.cached_body.borrow_mut().node.node_type =
            crate::link::node_data::NodeDataEnum::Internal(
                Rc::downgrade(parent),
                Some(intern_field_name(self.name.as_str())),
            );
    }

    fn ensure_body(&self) -> Link<Rc<RefCell<ContextObject>>> {
        Ok(Rc::clone(&self.cached_body))
    }

    pub fn get_body_entry(&self) -> Rc<RefCell<ExpressionEntry>> {
        self.cached_body
            .borrow()
            .expressions
            .get(RETURN_EXPRESSION)
            .cloned()
            .unwrap()
    }
}

impl Display for InlineFunctionDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let body = self.cached_body.borrow();
        let expression_str = body
            .expressions
            .get(RETURN_EXPRESSION)
            .map(|e| e.borrow().expression.to_string())
            .unwrap_or_else(|| "???".to_string());

        write!(
            f,
            "{}({}) {} {}",
            self.name,
            array_to_code_sep(self.arguments.iter(), ", "),
            C_ASSIGN,
            trim(&expression_str, '(', ')')
        )
    }
}

impl TypedValue for InlineFunctionDefinition {
    fn get_type(&self) -> ValueType {
        match self.ensure_body() {
            Ok(body) => ValueType::ObjectType(body),
            Err(_) => ValueType::UndefinedType,
        }
    }
}

impl UserFunction for InlineFunctionDefinition {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_parameters(&self) -> &Vec<FormalParameter> {
        &self.arguments
    }

    fn create_context(
        &self,
        parameters: Vec<FormalParameter>,
        parent: Option<Rc<RefCell<ContextObject>>>,
    ) -> Link<FunctionContext> {
        let body = self.ensure_body()?;
        {
            let mut borrowed = match body.try_borrow_mut() {
                Ok(b) => b,
                Err(_) => {
                    return Err(LinkingError::new(
                        crate::typesystem::errors::LinkingErrorEnum::CyclicReference(
                            "function".to_string(),
                            self.name.clone(),
                        ),
                    ));
                }
            };
            borrowed.parameters = parameters.clone();
        }
        linker::link_parts(Rc::clone(&body))?;

        let parent_weak = parent
            .map(|p| Rc::downgrade(&p))
            .or_else(|| self.parent.borrow().clone())
            .unwrap_or_else(Weak::new);

        Ok(FunctionContext::create_for(body, parameters, parent_weak))
    }
}

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub enum UserFunctionDefinition {
    Function(FunctionDefinition),
    Inline(InlineFunctionDefinition),
}

impl UserFunctionDefinition {
    pub fn get_body(&self) -> Link<Rc<RefCell<ContextObject>>> {
        match self {
            UserFunctionDefinition::Function(function_def) => Ok(Rc::clone(&function_def.body)),
            UserFunctionDefinition::Inline(inline_def) => inline_def.ensure_body(),
        }
    }

    pub fn set_parent_with_alias(&self, parent: &Rc<RefCell<ContextObject>>, alias: &'static str) {
        match self {
            UserFunctionDefinition::Function(function_def) => {
                function_def.body.borrow_mut().node.node_type =
                    crate::link::node_data::NodeDataEnum::Internal(
                        Rc::downgrade(parent),
                        Some(alias),
                    );
            }
            UserFunctionDefinition::Inline(inline_def) => inline_def.set_parent(parent),
        }
    }

    pub fn get_name(&self) -> String {
        match self {
            UserFunctionDefinition::Function(function_def) => function_def.name.clone(),
            UserFunctionDefinition::Inline(inline_def) => inline_def.name.clone(),
        }
    }

    pub fn set_name(&mut self, new_name: String) {
        match self {
            UserFunctionDefinition::Function(function_def) => function_def.name = new_name,
            UserFunctionDefinition::Inline(inline_def) => inline_def.name = new_name,
        }
    }
}

impl Display for UserFunctionDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UserFunctionDefinition::Function(function_def) => Display::fmt(function_def, f),
            UserFunctionDefinition::Inline(inline_def) => Display::fmt(inline_def, f),
        }
    }
}

impl TypedValue for UserFunctionDefinition {
    fn get_type(&self) -> ValueType {
        match self {
            UserFunctionDefinition::Function(function_def) => function_def.get_type(),
            UserFunctionDefinition::Inline(inline_def) => inline_def.get_type(),
        }
    }
}

impl UserFunction for UserFunctionDefinition {
    fn get_name(&self) -> String {
        match self {
            UserFunctionDefinition::Function(function_def) => function_def.get_name(),
            UserFunctionDefinition::Inline(inline_def) => inline_def.get_name(),
        }
    }

    fn get_parameters(&self) -> &Vec<FormalParameter> {
        match self {
            UserFunctionDefinition::Function(function_def) => function_def.get_parameters(),
            UserFunctionDefinition::Inline(inline_def) => inline_def.get_parameters(),
        }
    }

    fn create_context(
        &self,
        parameters: Vec<FormalParameter>,
        parent: Option<Rc<RefCell<ContextObject>>>,
    ) -> Link<FunctionContext> {
        match self {
            UserFunctionDefinition::Function(function_def) => {
                function_def.create_context(parameters, parent)
            }
            UserFunctionDefinition::Inline(inline_def) => {
                inline_def.create_context(parameters, parent)
            }
        }
    }
}
