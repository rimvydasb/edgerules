use crate::ast::context::context_object::ContextObject;
use crate::ast::context::context_object_builder::ContextObjectBuilder;
use crate::ast::context::context_object_type::EObjectContent;
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
use crate::utils::intern_field_name;
use crate::*;
use log::{error, trace};
use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::ops::Range;
use std::rc::Rc;

pub trait StaticLink: Display + Debug {
    fn link(&mut self, ctx: Rc<RefCell<ContextObject>>) -> Link<ValueType>;
}

pub(crate) fn missing_for_type(
    ty: &ValueType,
    field_name: Option<&str>,
    ctx: &Rc<RefCell<ExecutionContext>>,
) -> Result<ValueEnum, RuntimeError> {
    use crate::typesystem::types::number::NumberEnum;
    use crate::typesystem::types::string::StringEnum;
    use crate::typesystem::values::ValueOrSv::Sv;
    // use crate::typesystem::values::DurationValue;
    use crate::typesystem::types::SpecialValueEnum as SV;
    use crate::typesystem::values::ValueEnum as V;

    match ty {
        ValueType::NumberType => Ok(NumberValue(NumberEnum::SV(SV::missing_for(field_name)))),
        ValueType::StringType => Ok(V::StringValue(StringEnum::SV(SV::missing_for(field_name)))),
        ValueType::BooleanType => Ok(V::StringValue(StringEnum::SV(SV::missing_for(field_name)))),
        ValueType::DateType => Ok(V::DateValue(Sv(SV::missing_for(field_name)))),
        ValueType::TimeType => Ok(V::TimeValue(Sv(SV::missing_for(field_name)))),
        ValueType::DateTimeType => Ok(V::DateTimeValue(Sv(SV::missing_for(field_name)))),
        ValueType::DurationType => Ok(V::DurationValue(Sv(SV::missing_for(field_name)))),
        ValueType::ListType(inner) => {
            let item_type = inner.as_ref().clone();
            Ok(V::Array(vec![], item_type))
        }
        ValueType::ObjectType(obj) => {
            // Build empty object filled with missing values for each field
            let mut builder = ContextObjectBuilder::new();
            for name in obj.borrow().get_field_names() {
                if let Ok(content) = obj.borrow().get(name) {
                    match content {
                        EObjectContent::ExpressionRef(entry) => {
                            // Determine field type via placeholder if present
                            let fty = match &entry.borrow().expression {
                                TypePlaceholder(tref) => obj.borrow().resolve_type_ref(tref).ok(),
                                _ => None,
                            }
                            .unwrap_or(ValueType::UndefinedType);
                            let child_origin_owned = field_name
                                .filter(|parent| !parent.is_empty())
                                .map(|parent| format!("{}.{}", parent, name));
                            let child_origin = child_origin_owned.as_deref().or(Some(name));
                            let default_value = missing_for_type(&fty, child_origin, ctx)?;
                            builder.add_expression(name, default_value.into())?;
                        }
                        EObjectContent::ObjectRef(o) => {
                            let child_origin_owned = field_name
                                .filter(|parent| !parent.is_empty())
                                .map(|parent| format!("{}.{}", parent, name));
                            let child_origin = child_origin_owned.as_deref().or(Some(name));
                            let inner = missing_for_type(
                                &ValueType::ObjectType(o.clone()),
                                child_origin,
                                ctx,
                            )?;
                            builder.add_expression(name, inner.into())?;
                        }
                        _ => {}
                    }
                }
            }
            let static_obj = builder.build();
            let exec = ExecutionContext::create_temp_child_context(Rc::clone(ctx), static_obj);
            Ok(Reference(exec))
        }
        ValueType::RangeType | ValueType::UndefinedType => {
            Ok(V::StringValue(StringEnum::SV(SV::missing_for(field_name))))
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
        ValueType::ListType(inner) => {
            let item_type = *inner;
            match value {
                V::Array(items, _) => {
                    let mut casted_items = Vec::with_capacity(items.len());
                    for item in items {
                        match item {
                            Ok(v) => {
                                let mapped =
                                    cast_value_to_type(v, item_type.clone(), Rc::clone(&ctx))?;
                                casted_items.push(Ok(mapped));
                            }
                            Err(err) => casted_items.push(Err(err)),
                        }
                    }
                    Ok(V::Array(casted_items, item_type))
                }
                other => {
                    let mapped = cast_value_to_type(other, item_type.clone(), Rc::clone(&ctx))?;
                    Ok(V::Array(vec![Ok(mapped)], item_type))
                }
            }
        }
        ValueType::ObjectType(schema) => {
            // Expect source to be an object reference
            let src_exec = match value {
                Reference(r) => r,
                _ => {
                    // cannot shape non-object to object; build missing object
                    return missing_for_type(&ValueType::ObjectType(schema), None, &ctx);
                }
            };

            let mut builder = ContextObjectBuilder::new();
            for name in schema.borrow().get_field_names() {
                if let Ok(content) = schema.borrow().get(name) {
                    match content {
                        EObjectContent::ExpressionRef(entry) => {
                            // Attempt to resolve expected field type
                            let mut expected_ty = entry
                                .borrow()
                                .field_type
                                .clone()
                                .unwrap_or_else(|_| ValueType::UndefinedType);
                            // If expression is a TypePlaceholder, try to resolve from schema
                            if let TypePlaceholder(tref) = &entry.borrow().expression {
                                expected_ty = schema
                                    .borrow()
                                    .resolve_type_ref(tref)
                                    .unwrap_or(ValueType::UndefinedType);
                            }
                            // Get source field value and cast if possible
                            let casted = match src_exec.borrow().get(name) {
                                Ok(EObjectContent::ObjectRef(obj_exec)) => {
                                    // nested object; reuse reference
                                    cast_value_to_type(
                                        V::Reference(obj_exec),
                                        expected_ty.clone(),
                                        Rc::clone(&ctx),
                                    )?
                                }
                                Ok(EObjectContent::ExpressionRef(src_entry)) => {
                                    let v =
                                        src_entry.borrow().expression.eval(Rc::clone(&src_exec))?;
                                    cast_value_to_type(v, expected_ty.clone(), Rc::clone(&ctx))?
                                }
                                Ok(EObjectContent::ConstantValue(v)) => {
                                    cast_value_to_type(v, expected_ty.clone(), Rc::clone(&ctx))?
                                }
                                Ok(_) => missing_for_type(&expected_ty, Some(name), &ctx)?,
                                Err(_) => missing_for_type(&expected_ty, Some(name), &ctx)?,
                            };
                            builder.add_expression(name, casted.into())?;
                        }
                        EObjectContent::ObjectRef(obj) => {
                            // create empty shaped nested object
                            let val = missing_for_type(
                                &ValueType::ObjectType(obj.clone()),
                                Some(name),
                                &ctx,
                            )?;
                            builder.add_expression(name, val.into())?;
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
                        typesystem::types::SpecialValueEnum::missing_for(None),
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
        let path: Vec<&'static str> = _literal
            .split('.')
            .map(|segment| intern_field_name(segment))
            .collect();
        Variable(VariableLink::new_interned_path(path))
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
