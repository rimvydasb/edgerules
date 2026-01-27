use crate::ast::context::context_object::ContextObject;
use crate::ast::context::context_object_type::EObjectContent;
use crate::ast::context::function_context::FunctionContext;
use crate::ast::context::function_context::RETURN_EXPRESSION;
use crate::ast::expression::{CastCall, EvaluatableExpression, StaticLink};
use crate::ast::metaphors::metaphor::UserFunction;
use crate::ast::token::{ComplexTypeRef, ExpressionEnum};
use crate::ast::utils::array_to_code_sep;
use crate::ast::{is_linked, Link};
use crate::link::linker;
use crate::link::node_data::ContentHolder;
use crate::runtime::execution_context::*;
use crate::typesystem::errors::{LinkingError, RuntimeError};
use crate::typesystem::types::ValueType;
use crate::typesystem::values::ValueEnum;
use crate::typesystem::values::ValueEnum::Reference;
use crate::utils::intern_field_name;
use std::cell::RefCell;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

/// User function is a function that is defined in the code by user with a custom name. This is kind of non-built-in function
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub struct UserFunctionCall {
    pub name: String,
    pub args: Vec<ExpressionEnum>,
    pub definition: Link<FunctionContext>,
    #[allow(dead_code)]
    pub return_type: Link<ValueType>,
}

impl UserFunctionCall {
    pub fn new(name: String, args: Vec<ExpressionEnum>) -> UserFunctionCall {
        UserFunctionCall {
            name,
            args,
            definition: LinkingError::not_linked().into(),
            return_type: LinkingError::not_linked().into(),
        }
    }
}

// eval context is not immediately evaluated for output values, but passed to the caller
impl EvaluatableExpression for UserFunctionCall {
    fn eval(&self, context: Rc<RefCell<ExecutionContext>>) -> Result<ValueEnum, RuntimeError> {
        let values = self
            .args
            .iter()
            .map(|expr| expr.eval(Rc::clone(&context)))
            .collect();

        match &self.definition {
            Ok(definition) => {
                let eval_context = definition.create_eval_context(values, Rc::clone(&context))?;
                ExecutionContext::eval_all_fields(&eval_context)?;

                let return_key = intern_field_name("return");
                if let Ok(content) = eval_context.borrow().get(return_key) {
                    return eval_content_to_value(content);
                }

                let hidden_return = intern_field_name(RETURN_EXPRESSION);
                if let Ok(content) = eval_context.borrow().get(hidden_return) {
                    return eval_content_to_value(content);
                }

                Ok(Reference(eval_context))
            }
            Err(_error) => Err(RuntimeError::internal_integrity_error(403)),
        }
    }
}

fn eval_content_to_value(
    content: EObjectContent<ExecutionContext>,
) -> Result<ValueEnum, RuntimeError> {
    match content {
        EObjectContent::ConstantValue(v) => Ok(v),
        EObjectContent::ObjectRef(ctx) => Ok(ValueEnum::Reference(ctx)),
        _ => Err(RuntimeError::internal_integrity_error(403)),
    }
}

impl StaticLink for UserFunctionCall {
    fn link(&mut self, ctx: Rc<RefCell<ContextObject>>) -> Link<ValueType> {
        // The call site is linked lazily: each UserFunctionCall instance links only once and caches the resolved definition.
        // This lets different call sites type-check against the same function independently.
        if !is_linked(&self.definition) {
            // Step 1: resolve the function definition in the current scope.
            let definition = linker::find_implementation(Rc::clone(&ctx), self.name.clone())?;

            // Step 2: validate that we have the correct number of arguments.
            let param_len = {
                let method = definition.borrow();
                method.function_definition.get_parameters().len()
            };
            if self.args.len() != param_len {
                return LinkingError::other_error(format!(
                    "Function {} expects {} arguments, but {} were provided",
                    self.name,
                    param_len,
                    self.args.len()
                ))
                .into();
            }

            // Step 3: link each argument expression and ensure it matches the declared parameter type.
            let ctx_name = ctx.borrow().node.node_type.to_code();
            let function_name = self.name.clone();

            let (declared_parameters, function_body_ctx) = {
                let borrowed = definition.borrow();
                let params = borrowed.function_definition.get_parameters().clone();
                let body = borrowed.function_definition.get_body()?;
                (params, body)
            };

            let mut parameters = Vec::new();

            for (parameter, input_argument) in declared_parameters.iter().zip(self.args.iter_mut())
            {
                // Link the argument within the current call context. Passing the function's own context is disallowed to
                // prevent accidental self-references before the function body is evaluated.
                let arg_link_result = if let ExpressionEnum::Variable(var) = input_argument {
                    if var.path.len() == 1 && var.path[0] == ctx_name {
                        LinkingError::other_error(format!(
                            "Cannot pass context `{}` as argument to function `{}` defined in the same context",
                            ctx_name, function_name
                        ))
                        .into()
                    } else {
                        input_argument.link(Rc::clone(&ctx))
                    }
                } else {
                    input_argument.link(Rc::clone(&ctx))
                };

                let mut resolved_type = match arg_link_result {
                    Ok(t) => t,
                    Err(err) => {
                        if matches!(err.kind(), crate::typesystem::errors::LinkingErrorEnum::NotLinkedYet) {
                            ValueType::UndefinedType
                        } else {
                            return Err(err);
                        }
                    }
                };

                if let ValueType::ObjectType(obj) = &resolved_type {
                    let _ = linker::link_parts(Rc::clone(obj));
                }

                if let Some(tref) = parameter.declared_type() {
                    // Step 4: resolve the parameter's declared type (including aliases) and coerce when safe.
                    let expected_type =
                        resolve_declared_type(tref, Some(&function_body_ctx), &ctx)?;

                    // Alias parameters may need an explicit cast to resolve the correct runtime type.
                    if resolved_type != expected_type
                        && complex_type_ref_contains_alias(tref)
                        && can_cast_alias(&resolved_type, &expected_type)
                    {
                        let original = std::mem::replace(
                            input_argument,
                            ExpressionEnum::Value(ValueEnum::BooleanValue(false)),
                        );
                        *input_argument =
                            ExpressionEnum::from(CastCall::new(original, tref.clone()));
                        resolved_type = input_argument.link(Rc::clone(&ctx))?;
                    }
                    let validated = LinkingError::expect_single_type(
                        &format!(
                            "Argument `{}` of function `{}`",
                            parameter.name, function_name
                        ),
                        resolved_type.clone(),
                        &expected_type,
                    )?;
                    parameters.push(parameter.with_runtime_type(validated));
                } else {
                    parameters.push(parameter.with_runtime_type(resolved_type));
                }
            }

            // Step 5: build and cache the callable function context with all resolved parameter types.
            self.definition = Ok(definition
                .borrow()
                .function_definition
                .create_context(parameters, Some(Rc::clone(&ctx)))?);

            // Determine return type respecting explicit return field when present
            let return_key = intern_field_name("return");
            let hidden_return = intern_field_name(RETURN_EXPRESSION);
            let mut rt: Option<ValueType> = None;

            // @Todo: investigate if is_ok does not hide important linking errors
            if linker::link_parts(Rc::clone(&function_body_ctx)).is_ok() {
                let borrowed_body = function_body_ctx.borrow();
                if let Some(entry) = borrowed_body
                    .expressions
                    .get(return_key)
                    .or_else(|| borrowed_body.expressions.get(hidden_return))
                {
                    if let Ok(ft) = &entry.borrow().field_type {
                        rt = Some(ft.clone());
                    }
                }
            }
            self.return_type = Ok(rt.unwrap_or(ValueType::ObjectType(function_body_ctx)));
        }

        self.return_type.clone()
    }
}

fn resolve_declared_type(
    tref: &ComplexTypeRef,
    function_ctx: Option<&Rc<RefCell<ContextObject>>>,
    call_ctx: &Rc<RefCell<ContextObject>>,
) -> Link<ValueType> {
    match tref {
        ComplexTypeRef::BuiltinType(vt, _) => Ok(vt.clone()),
        ComplexTypeRef::List(inner, _) => {
            let inner_type = resolve_declared_type(inner, function_ctx, call_ctx)?;
            Ok(ValueType::ListType(Some(Box::new(inner_type))))
        }
        ComplexTypeRef::Alias(_, _) => {
            if let Some(context) = function_ctx {
                if let Ok(vt) = context.borrow().resolve_type_ref(tref) {
                    return Ok(vt);
                }
            }

            call_ctx.borrow().resolve_type_ref(tref)
        }
    }
}

fn complex_type_ref_contains_alias(tref: &ComplexTypeRef) -> bool {
    match tref {
        ComplexTypeRef::Alias(_, _) => true,
        ComplexTypeRef::List(inner, _) => complex_type_ref_contains_alias(inner),
        ComplexTypeRef::BuiltinType(_, _) => false,
    }
}

fn can_cast_alias(actual: &ValueType, expected: &ValueType) -> bool {
    match (actual, expected) {
        (ValueType::ObjectType(_), ValueType::ObjectType(_)) => true,
        (ValueType::ListType(inner_actual), ValueType::ListType(inner_expected)) => {
            match (inner_actual.as_ref(), inner_expected.as_ref()) {
                (Some(actual_inner), Some(expected_inner)) => {
                    can_cast_alias(actual_inner, expected_inner)
                }
                (None, None) => true,
                _ => false,
            }
        }
        _ => false,
    }
}

impl Display for UserFunctionCall {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}({})",
            self.name,
            array_to_code_sep(self.args.iter(), ", ")
        )
    }
}
