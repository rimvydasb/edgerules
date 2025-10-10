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
use crate::typesystem::types::{TypedValue, ValueType};
use crate::typesystem::values::ValueEnum::{NumberValue, RangeValue, Reference};
use crate::typesystem::values::{ArrayValue, ValueEnum};
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
        ValueType::ListType(inner) => match inner.as_ref() {
            Some(item_type) => Ok(V::Array(ArrayValue::PrimitivesArray {
                values: Vec::new(),
                item_type: (**item_type).clone(),
            })),
            None => Ok(V::Array(ArrayValue::EmptyUntyped)),
        },
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
    origin: Option<&str>,
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
            let desired_item_type = inner.as_ref().map(|boxed| (**boxed).clone());
            match value {
                V::Array(array) => match array {
                    ArrayValue::EmptyUntyped => {
                        if let Some(item_type) = desired_item_type {
                            Ok(V::Array(ArrayValue::PrimitivesArray {
                                values: Vec::new(),
                                item_type,
                            }))
                        } else {
                            Ok(V::Array(ArrayValue::EmptyUntyped))
                        }
                    }
                    ArrayValue::PrimitivesArray { values, item_type } => {
                        let target_item_type =
                            desired_item_type.clone().unwrap_or(item_type.clone());
                        let mut casted = Vec::with_capacity(values.len());
                        for v in values {
                            let mapped = cast_value_to_type(
                                v,
                                target_item_type.clone(),
                                Rc::clone(&ctx),
                                origin,
                            )?;
                            casted.push(mapped);
                        }
                        Ok(V::Array(ArrayValue::PrimitivesArray {
                            values: casted,
                            item_type: target_item_type,
                        }))
                    }
                    ArrayValue::ObjectsArray {
                        values,
                        object_type,
                    } => match desired_item_type.clone() {
                        Some(ValueType::ObjectType(target_object_type)) => {
                            let mut casted: Vec<Rc<RefCell<ExecutionContext>>> =
                                Vec::with_capacity(values.len());
                            for reference in values {
                                let casted_value = cast_value_to_type(
                                    V::Reference(Rc::clone(&reference)),
                                    ValueType::ObjectType(Rc::clone(&target_object_type)),
                                    Rc::clone(&ctx),
                                    origin,
                                )?;
                                if let V::Reference(obj_ref) = casted_value {
                                    casted.push(obj_ref);
                                } else {
                                    return RuntimeError::type_not_supported(
                                        casted_value.get_type(),
                                    )
                                    .into();
                                }
                            }
                            Ok(V::Array(ArrayValue::ObjectsArray {
                                values: casted,
                                object_type: target_object_type,
                            }))
                        }
                        Some(other_item_type) => {
                            let mut casted_values = Vec::with_capacity(values.len());
                            for reference in values {
                                let mapped = cast_value_to_type(
                                    V::Reference(reference),
                                    other_item_type.clone(),
                                    Rc::clone(&ctx),
                                    origin,
                                )?;
                                casted_values.push(mapped);
                            }
                            let final_item_type =
                                if matches!(other_item_type, ValueType::UndefinedType)
                                    && !casted_values.is_empty()
                                {
                                    casted_values[0].get_type()
                                } else {
                                    other_item_type
                                };
                            Ok(V::Array(ArrayValue::PrimitivesArray {
                                values: casted_values,
                                item_type: final_item_type,
                            }))
                        }
                        None => Ok(V::Array(ArrayValue::ObjectsArray {
                            values,
                            object_type,
                        })),
                    },
                },
                other => {
                    let target_item_type = desired_item_type
                        .clone()
                        .unwrap_or_else(|| other.get_type());
                    let mapped = cast_value_to_type(
                        other,
                        target_item_type.clone(),
                        Rc::clone(&ctx),
                        origin,
                    )?;
                    Ok(V::Array(ArrayValue::PrimitivesArray {
                        values: vec![mapped],
                        item_type: target_item_type,
                    }))
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
                let field_origin_owned = origin.map(|parent| {
                    if parent.is_empty() {
                        name.to_string()
                    } else {
                        format!("{}.{}", parent, name)
                    }
                });
                let field_origin = field_origin_owned.as_deref().or(Some(name));

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
                                        field_origin,
                                    )?
                                }
                                Ok(EObjectContent::ExpressionRef(src_entry)) => {
                                    let v =
                                        src_entry.borrow().expression.eval(Rc::clone(&src_exec))?;
                                    cast_value_to_type(
                                        v,
                                        expected_ty.clone(),
                                        Rc::clone(&ctx),
                                        field_origin,
                                    )?
                                }
                                Ok(EObjectContent::ConstantValue(v)) => cast_value_to_type(
                                    v,
                                    expected_ty.clone(),
                                    Rc::clone(&ctx),
                                    field_origin,
                                )?,
                                Ok(_) => missing_for_type(&expected_ty, field_origin, &ctx)?,
                                Err(_) => missing_for_type(&expected_ty, field_origin, &ctx)?,
                            };
                            builder.add_expression(name, casted.into())?;
                        }
                        EObjectContent::ObjectRef(obj) => {
                            // create empty shaped nested object
                            let val = missing_for_type(
                                &ValueType::ObjectType(obj.clone()),
                                field_origin,
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
        cast_value_to_type(value, target_type, context, None)
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
