use crate::ast::context::context_object::ContextObject;
use crate::ast::context::context_object_builder::ContextObjectBuilder;
use crate::ast::context::context_object_type::FormalParameter;
use crate::ast::context::function_context::{FunctionContext, RETURN_EXPRESSION};
use crate::ast::expression::missing_for_type;
use crate::ast::expression::{EvaluatableExpression, StaticLink};
use crate::ast::token::ExpressionEnum::Value;
use crate::ast::token::{ComplexTypeRef, ExpressionEnum};
use crate::ast::{is_linked, Link};
use crate::link::linker::link_parts;
use crate::link::node_data::{NodeData, NodeDataEnum};
use crate::runtime::execution_context::*;
use crate::tokenizer::utils::Either;
use crate::typesystem::errors::{LinkingError, ParseErrorEnum, RuntimeError, RuntimeErrorEnum};
use crate::typesystem::types::{Integer, TypedValue, ValueType};
use crate::typesystem::values::ValueEnum::{Array, RangeValue};
use crate::typesystem::values::{ArrayValue, ValueEnum};
use crate::utils::context_unwrap;
use std::cell::RefCell;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Range;
use std::rc::Rc;
fn flatten_list_type_for_for(value_type: ValueType) -> ValueType {
    match value_type {
        ValueType::ListType(Some(inner)) => flatten_list_type_for_for(*inner),
        other => other,
    }
}

/// for in_loop_variable in in_expression return return_expression
/// in_expression.map(in_loop_variable -> return_expression)
/// map(in_expression,(in_loop_variable) return_expression)
#[derive(Debug)]
pub struct ForFunction {
    pub in_loop_variable: String,
    pub in_expression: ExpressionEnum,
    /// In definition return_expression is wrapped in InlineFunctionContext
    pub return_expression: Rc<RefCell<ContextObject>>,
    pub return_type: Link<ValueType>,
}

impl Display for ForFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let return_expression = context_unwrap(self.return_expression.borrow().to_string());
        write!(
            f,
            "for {} in {} return {}",
            self.in_loop_variable, self.in_expression, return_expression
        )
    }
}

impl Display for Either<ExpressionEnum, FunctionContext> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Either::Left(expression) => write!(f, "{}", expression),
            Either::Right(inline_function) => write!(f, "{}", inline_function),
        }
    }
}

impl ForFunction {
    pub fn new(
        in_loop_variable: String,
        in_expression: ExpressionEnum,
        return_expression: ExpressionEnum,
    ) -> Result<Self, ParseErrorEnum> {
        let mut builder = ContextObjectBuilder::new();
        builder.add_expression(RETURN_EXPRESSION, return_expression)?;

        Ok(ForFunction {
            in_loop_variable,
            in_expression,
            return_expression: builder.build(),
            return_type: LinkingError::not_linked().into(),
        })
    }

    fn create_in_loop_context(
        &self,
        parent: &Rc<RefCell<ExecutionContext>>,
        value: ExpressionEnum,
    ) -> Result<Rc<RefCell<ExecutionContext>>, RuntimeError> {
        let mut obj = ContextObjectBuilder::new();
        obj.add_expression(self.in_loop_variable.as_str(), value)
            .map_err(|err| RuntimeError::eval_error(err.to_string()))?;

        Ok(ExecutionContext::create_temp_child_context(
            Rc::clone(parent),
            obj.build(),
        ))
    }

    fn iterate_values(
        &self,
        values: Vec<ValueEnum>,
        parent: Rc<RefCell<ExecutionContext>>,
    ) -> Result<ValueEnum, RuntimeError> {
        let mut result: Vec<ValueEnum> = Vec::new();

        let element_type = match self.return_type.clone()? {
            ValueType::ListType(item_type) => item_type
                .as_ref()
                .map(|inner| (**inner).clone())
                .unwrap_or(ValueType::UndefinedType),
            err => {
                // @Todo: it should be linking error, not a runtime
                return RuntimeError::eval_error(format!(
                    "Cannot iterate through non list type `{}`",
                    err
                ))
                .into();
            }
        };

        for loop_value in values {
            let ctx = self.create_in_loop_context(&parent, Value(loop_value.clone()))?;
            let map_value = self
                .return_expression
                .borrow()
                .expressions
                .get(RETURN_EXPRESSION)
                .unwrap()
                .borrow()
                .expression
                .eval(ctx);

            match map_value {
                Ok(val) => result.push(val),
                Err(err) => {
                    if let RuntimeErrorEnum::RuntimeFieldNotFound(_, field) = &err.error {
                        let missing =
                            missing_for_type(&element_type, Some(field.as_str()), &parent)?;
                        result.push(missing);
                    } else {
                        return Err(err);
                    }
                }
            }
        }

        Ok(Array(ArrayValue::PrimitivesArray {
            values: result,
            item_type: element_type.clone(),
        }))
    }

    fn iterate_objects(
        &self,
        values: Vec<Rc<RefCell<ExecutionContext>>>,
        parent: Rc<RefCell<ExecutionContext>>,
    ) -> Result<ValueEnum, RuntimeError> {
        let mut result: Vec<ValueEnum> = Vec::new();

        let element_type = match self.return_type.clone()? {
            ValueType::ListType(item_type) => item_type
                .as_ref()
                .map(|inner| (**inner).clone())
                .unwrap_or(ValueType::UndefinedType),
            err => {
                return RuntimeError::eval_error(format!(
                    "Cannot iterate through non list type `{}`",
                    err
                ))
                .into();
            }
        };

        for ctx_ref in values {
            let loop_value = ValueEnum::Reference(Rc::clone(&ctx_ref));
            let ctx = self.create_in_loop_context(&parent, Value(loop_value.clone()))?;
            let map_value = self
                .return_expression
                .borrow()
                .expressions
                .get(RETURN_EXPRESSION)
                .unwrap()
                .borrow()
                .expression
                .eval(ctx);

            match map_value {
                Ok(val) => result.push(val),
                Err(err) => {
                    if let RuntimeErrorEnum::RuntimeFieldNotFound(_, field) = &err.error {
                        let missing =
                            missing_for_type(&element_type, Some(field.as_str()), &parent)?;
                        result.push(missing);
                    } else {
                        return Err(err);
                    }
                }
            }
        }

        Ok(Array(ArrayValue::PrimitivesArray {
            values: result,
            item_type: element_type.clone(),
        }))
    }

    fn iterate_range(
        &self,
        values: Range<Integer>,
        parent: Rc<RefCell<ExecutionContext>>,
    ) -> Result<ValueEnum, RuntimeError> {
        let mut result: Vec<ValueEnum> = Vec::new();

        for value in values {
            let ctx = self.create_in_loop_context(&parent, Value(ValueEnum::from(value)))?;
            //@Todo way too complex
            let map_value = self
                .return_expression
                .borrow()
                .expressions
                .get(RETURN_EXPRESSION)
                .unwrap()
                .borrow()
                .expression
                .eval(ctx);
            //@Todo return values only, not tokens
            result.push(map_value?);
        }

        Ok(Array(ArrayValue::PrimitivesArray {
            values: result,
            item_type: ValueType::NumberType,
        }))
    }
}

impl EvaluatableExpression for ForFunction {
    fn eval(&self, context: Rc<RefCell<ExecutionContext>>) -> Result<ValueEnum, RuntimeError> {
        match self.in_expression.eval(Rc::clone(&context))? {
            Array(ArrayValue::PrimitivesArray { values, .. }) => {
                self.iterate_values(values, Rc::clone(&context))
            }
            Array(ArrayValue::ObjectsArray { values, .. }) => {
                self.iterate_objects(values, Rc::clone(&context))
            }
            RangeValue(range) => self.iterate_range(range, Rc::clone(&context)),
            other => {
                RuntimeError::eval_error(format!("Cannot iterate {}", other.get_type())).into()
            }
        }
    }
}

impl StaticLink for ForFunction {
    fn link(&mut self, ctx: Rc<RefCell<ContextObject>>) -> Link<ValueType> {
        if !is_linked(&self.return_type) {
            let list_type = self.in_expression.link(Rc::clone(&ctx))?;

            let item_type = match list_type {
                ValueType::ListType(list_item_type) => list_item_type
                    .as_ref()
                    .map(|inner| (**inner).clone())
                    .map(flatten_list_type_for_for)
                    .unwrap_or(ValueType::UndefinedType),
                ValueType::RangeType => ValueType::NumberType,
                _ => {
                    return LinkingError::other_error(format!(
                        "Cannot iterate through non list type `{}`",
                        list_type
                    ))
                    .into();
                }
            };

            let parameter_type = ComplexTypeRef::from_value_type(item_type);
            let for_parameter =
                FormalParameter::with_type_ref(self.in_loop_variable.clone(), parameter_type);

            self.return_expression
                .borrow_mut()
                .parameters
                .push(for_parameter);
            self.return_expression.borrow_mut().node =
                NodeData::new(NodeDataEnum::Internal(Rc::downgrade(&ctx)));

            // @Todo: link_parts will fail with unknown field if return_expression refers list item field, for example:
            // for item in [{a:1},{}] return item.a
            // technically, then field_type must be of a type "a", but item.a can be accessed only if list_item_type is object
            link_parts(Rc::clone(&self.return_expression))?;

            let field_type = self
                .return_expression
                .borrow()
                .expressions
                .get(RETURN_EXPRESSION)
                .unwrap()
                .borrow()
                .field_type
                .clone()?;

            self.return_type = Ok(ValueType::ListType(Some(Box::new(field_type))));
        }

        self.return_type.clone()
    }
}
