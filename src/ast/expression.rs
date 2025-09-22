use crate::ast::context::context_object::ContextObject;
use crate::ast::token::ExpressionEnum::*;
use crate::ast::token::*;
use crate::ast::variable::VariableLink;
use crate::ast::Link;
use crate::link::node_data::ContentHolder;
use crate::runtime::execution_context::ExecutionContext;
use crate::typesystem::errors::{ErrorStack, LinkingError, RuntimeError};
use crate::typesystem::types::number::NumberEnum::Int;
use crate::typesystem::types::ValueType;
use crate::typesystem::values::ValueEnum;
use crate::typesystem::values::ValueEnum::{NumberValue, RangeValue, Reference};
use crate::*;
use log::{error, trace};
use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::ops::Range;
use std::rc::Rc;

pub trait StaticLink: Display + Debug {
    fn link(&mut self, ctx: Rc<RefCell<ContextObject>>) -> Link<ValueType>;
}

fn missing_for_type(ty: &ValueType, ctx: &Rc<RefCell<ExecutionContext>>) -> ValueEnum {
    use crate::typesystem::types::number::NumberEnum;
    use crate::typesystem::types::string::StringEnum;
    use crate::typesystem::values::ValueOrSv::Sv;
    // use crate::typesystem::values::DurationValue;
    use crate::typesystem::types::SpecialValueEnum as SV;
    use crate::typesystem::values::ValueEnum as V;

    match ty {
        ValueType::NumberType => V::NumberValue(NumberEnum::SV(SV::Missing)),
        ValueType::StringType => V::StringValue(StringEnum::SV(SV::Missing)),
        ValueType::BooleanType => V::StringValue(StringEnum::SV(SV::Missing)),
        ValueType::DateType => V::DateValue(Sv(SV::Missing)),
        ValueType::TimeType => V::TimeValue(Sv(SV::Missing)),
        ValueType::DateTimeType => V::DateTimeValue(Sv(SV::Missing)),
        ValueType::DurationType => V::DurationValue(Sv(SV::Missing)),
        ValueType::ListType(inner) => V::Array(vec![], ValueType::ListType(inner.clone())),
        ValueType::ObjectType(obj) => {
            // Build empty object filled with missing values for each field
            let mut builder =
                crate::ast::context::context_object_builder::ContextObjectBuilder::new();
            for name in obj.borrow().get_field_names() {
                if let Ok(content) = obj.borrow().get(&name) {
                    match content {
                        crate::ast::context::context_object_type::EObjectContent::ExpressionRef(
                            entry,
                        ) => {
                            // Determine field type via placeholder if present
                            let fty = match &entry.borrow().expression {
                                TypePlaceholder(tref) => obj.borrow().resolve_type_ref(tref).ok(),
                                _ => None,
                            }
                            .unwrap_or(ValueType::UndefinedType);
                            builder.add_expression(&name, missing_for_type(&fty, ctx).into());
                        }
                        crate::ast::context::context_object_type::EObjectContent::ObjectRef(o) => {
                            let inner = missing_for_type(&ValueType::ObjectType(o.clone()), ctx);
                            builder.add_expression(&name, inner.into());
                        }
                        _ => {}
                    }
                }
            }
            let static_obj = builder.build();
            let exec = ExecutionContext::create_temp_child_context(Rc::clone(ctx), static_obj);
            V::Reference(exec)
        }
        ValueType::RangeType | ValueType::UndefinedType => {
            V::StringValue(StringEnum::SV(SV::Missing))
        }
    }
}

fn cast_value_to_type(
    value: ValueEnum,
    target: ValueType,
    ctx: Rc<RefCell<ExecutionContext>>, // used for building child contexts
) -> Result<ValueEnum, RuntimeError> {
    use crate::typesystem::values::ValueEnum as V;
    match target {
        ValueType::NumberType
        | ValueType::StringType
        | ValueType::BooleanType
        | ValueType::DateType
        | ValueType::TimeType
        | ValueType::DateTimeType
        | ValueType::DurationType
        | ValueType::RangeType
        | ValueType::UndefinedType => Ok(value),
        ValueType::ListType(inner) => match value {
            V::Array(items, _) => {
                // rewrap items; no element-wise casting for now
                Ok(V::Array(items, ValueType::ListType(inner)))
            }
            other => Ok(other),
        },
        ValueType::ObjectType(schema) => {
            // Expect source to be an object reference
            let src_exec = match value {
                V::Reference(r) => r,
                _ => {
                    // cannot shape non-object to object; build missing object
                    return Ok(missing_for_type(&ValueType::ObjectType(schema), &ctx));
                }
            };

            let mut builder =
                crate::ast::context::context_object_builder::ContextObjectBuilder::new();
            for name in schema.borrow().get_field_names() {
                if let Ok(content) = schema.borrow().get(&name) {
                    match content {
                        crate::ast::context::context_object_type::EObjectContent::ExpressionRef(
                            entry,
                        ) => {
                            // Attempt to resolve expected field type
                            let mut expected_ty = match entry.borrow().field_type.clone() {
                                Ok(t) => t,
                                Err(_) => ValueType::UndefinedType,
                            };
                            // If expression is a TypePlaceholder, try to resolve from schema
                            if let crate::ast::token::ExpressionEnum::TypePlaceholder(tref) =
                                &entry.borrow().expression
                            {
                                expected_ty = schema
                                    .borrow()
                                    .resolve_type_ref(tref)
                                    .unwrap_or(ValueType::UndefinedType);
                            }
                            // Get source field value and cast if possible
                            let casted = match src_exec.borrow().get(&name) {
                                Ok(crate::ast::context::context_object_type::EObjectContent::ObjectRef(obj_exec)) => {
                                    // nested object; reuse reference
                                    cast_value_to_type(V::Reference(obj_exec), expected_ty.clone(), Rc::clone(&ctx))?
                                }
                                Ok(crate::ast::context::context_object_type::EObjectContent::ExpressionRef(src_entry)) => {
                                    let v = src_entry.borrow().expression.eval(Rc::clone(&src_exec))?;
                                    cast_value_to_type(v, expected_ty.clone(), Rc::clone(&ctx))?
                                }
                                Ok(crate::ast::context::context_object_type::EObjectContent::ConstantValue(v)) => {
                                    cast_value_to_type(v, expected_ty.clone(), Rc::clone(&ctx))?
                                }
                                Ok(_) => missing_for_type(&expected_ty, &ctx),
                                Err(_) => missing_for_type(&expected_ty, &ctx),
                            };
                            builder.add_expression(&name, casted.into());
                        }
                        crate::ast::context::context_object_type::EObjectContent::ObjectRef(
                            obj,
                        ) => {
                            // create empty shaped nested object
                            let val = missing_for_type(&ValueType::ObjectType(obj.clone()), &ctx);
                            builder.add_expression(&name, val.into());
                        }
                        _ => {}
                    }
                }
            }
            let shaped = builder.build();
            let ref_val = ExecutionContext::create_temp_child_context(Rc::clone(&ctx), shaped);
            Ok(V::Reference(ref_val))
        }
    }
}

#[derive(Debug)]
pub struct CastCall {
    expression: ExpressionEnum,
    target_ref: ComplexTypeRef,
    target_type: Link<ValueType>,
}

impl CastCall {
    pub fn new(expression: ExpressionEnum, target_ref: ComplexTypeRef) -> Self {
        CastCall {
            expression,
            target_ref,
            target_type: LinkingError::not_linked().into(),
        }
    }
}

impl StaticLink for CastCall {
    fn link(&mut self, ctx: Rc<RefCell<ContextObject>>) -> Link<ValueType> {
        self.expression.link(Rc::clone(&ctx))?;
        let resolved = ctx.borrow().resolve_type_ref(&self.target_ref)?;
        self.target_type = Ok(resolved.clone());
        Ok(resolved)
    }
}

impl Display for CastCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} as {})", self.expression, self.target_ref)
    }
}

pub trait EvaluatableExpression: StaticLink {
    fn eval(&self, context: Rc<RefCell<ExecutionContext>>) -> Result<ValueEnum, RuntimeError>;
}

impl EvaluatableExpression for CastCall {
    fn eval(&self, context: Rc<RefCell<ExecutionContext>>) -> Result<ValueEnum, RuntimeError> {
        let target_type = match &self.target_type {
            Ok(ty) => ty.clone(),
            Err(link_err) => {
                let err = link_err
                    .clone()
                    .with_context(|| format!("Evaluating cast `{}`", self));
                return Err(RuntimeError::from(err));
            }
        };

        let value = self.expression.eval(Rc::clone(&context))?;
        cast_value_to_type(value, target_type, context)
    }
}

impl<T> From<T> for ExpressionEnum
where
    T: EvaluatableExpression + Sized + 'static,
{
    fn from(expression: T) -> Self {
        FunctionCall(Box::new(expression))
    }
}

impl ExpressionEnum {
    pub fn eval(&self, context: Rc<RefCell<ExecutionContext>>) -> Result<ValueEnum, RuntimeError> {
        let trace_context = Rc::clone(&context);
        let eval_result = match self {
            Variable(variable) => variable.eval(context),
            ContextVariable => {
                trace!(">>> evaluating context variable");
                context.borrow().get_context_variable()
            }
            Operator(operator) => operator.eval(context),
            FunctionCall(function) => function.eval(context),
            Selection(selection) => selection.eval(context),
            Filter(filter) => filter.eval(context),
            ObjectField(field_name, _obj) => RuntimeError::eval_error(format!(
                "ObjectField evaluation is deprecated. Still used by {:?}",
                field_name
            ))
            .into(),
            Value(value) => Ok(value.clone()),
            Collection(elements) => elements.eval(context),
            RangeExpression(left, right) => {
                match (left.eval(Rc::clone(&context))?, right.eval(context)?) {
                    (NumberValue(Int(left_number)), NumberValue(Int(right_number))) => {
                        let range = Range {
                            start: left_number,
                            end: right_number + 1,
                        };

                        Ok(RangeValue(range))
                    }
                    _ => RuntimeError::eval_error("Range is not a valid number".to_string()).into(),
                }
            }
            StaticObject(object) => {
                let reference = ExecutionContext::create_temp_child_context(
                    Rc::clone(&context),
                    object.clone(),
                );
                Ok(Reference(reference))
            }
            TypePlaceholder(_tref) => {
                // BLOCKED: no external context hookup; always Missing as per spec
                Ok(ValueEnum::StringValue(
                    crate::typesystem::types::string::StringEnum::from(
                        crate::typesystem::types::SpecialValueEnum::Missing,
                    ),
                ))
            }
        };

        if let Err(error) = eval_result {
            //let error_str = error.get_error_type().to_string();
            error!(">                   `{:?}`", error.get_error_type());
            let with_context = error.with_context(|| {
                format!(
                    "Error evaluating `{}.{}`",
                    trace_context.borrow().object.borrow().node.node_type,
                    self
                )
            });
            return Err(with_context);
        }

        eval_result
    }

    pub fn variable(_literal: &str) -> ExpressionEnum {
        let path: Vec<&str> = _literal.split('.').collect();
        let path = VariableLink::new_unlinked_path(path.iter().map(|s| String::from(*s)).collect());
        Variable(path)
    }
}

impl PartialEq for ExpressionEnum {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (StaticObject(a), StaticObject(b)) => a == b,
            (Value(a), Value(b)) => a == b,
            (Variable(a), Variable(b)) => a == b,
            (ObjectField(name_a, expr_a), ObjectField(name_b, expr_b)) => {
                name_a == name_b && expr_a == expr_b
            }
            (RangeExpression(l1, r1), RangeExpression(l2, r2)) => l1 == l2 && r1 == r2,
            (ContextVariable, ContextVariable) => true,

            // Variants below contain trait objects or structs without PartialEq.
            // Provide a conservative fallback until/if richer semantics are needed.
            (Operator(_), Operator(_)) => false,
            (FunctionCall(_), FunctionCall(_)) => false,
            (Filter(_), Filter(_)) => false,
            (Selection(_), Selection(_)) => false,
            (Collection(_), Collection(_)) => false,

            _ => false,
        }
    }
}
