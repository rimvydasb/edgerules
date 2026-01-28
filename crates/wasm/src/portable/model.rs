use crate::portable::error::{PortableError, PortableObjectKey, SchemaViolationType};
use crate::utils::{get_prop, is_object, set_prop};
use edge_rules::ast::context::context_object::ContextObject;
use edge_rules::ast::context::context_object_builder::ContextObjectBuilder;
use edge_rules::ast::context::context_object_type::FormalParameter;
use edge_rules::ast::context::function_context::RETURN_EXPRESSION;
use edge_rules::ast::context::metadata::Metadata;
use edge_rules::ast::metaphors::functions::{FunctionDefinition, InlineFunctionDefinition, UserFunctionDefinition};
use edge_rules::ast::metaphors::metaphor::UserFunction;
use edge_rules::ast::sequence::CollectionExpression;
use edge_rules::ast::token::{ComplexTypeRef, DefinitionEnum, ExpressionEnum, UserTypeBody};
use edge_rules::ast::user_function_call::UserFunctionCall;
use edge_rules::link::node_data::Node;
use edge_rules::runtime::edge_rules::{ContextQueryErrorEnum, EdgeRulesModel, InvocationSpec};
use edge_rules::tokenizer::parser;
use edge_rules::tokenizer::utils::CharStream;
use edge_rules::typesystem::types::number::NumberEnum;
use edge_rules::typesystem::values::ValueEnum;
use edge_rules::utils::intern_field_name;
use js_sys::{Array, Object};
use rust_decimal::prelude::ToPrimitive;
use wasm_bindgen::{JsCast, JsValue};

pub fn model_from_portable(portable: &JsValue) -> Result<EdgeRulesModel, PortableError> {
    if !is_object(portable) {
        return Err(PortableError::SchemaViolation(PortableObjectKey::Root, SchemaViolationType::InvalidFieldType));
    }
    let mut model = EdgeRulesModel::new();
    let object: Object = portable.clone().unchecked_into();
    let keys = Object::keys(&object);

    let metadata = extract_metadata(portable);
    if !metadata.is_empty() {
        model.ast_root.set_metadata(metadata);
    }

    for i in 0..keys.length() {
        let name = keys.get(i).as_string().unwrap_or_default();
        if name.starts_with('@') {
            continue;
        }
        let value = get_prop(portable, &name).unwrap_or(JsValue::UNDEFINED);
        match classify_entry(&value) {
            PortableKind::Function(def_obj) => {
                let definition = parse_function_definition(&name, &def_obj)?;
                let def_enum = match definition {
                    UserFunctionDefinition::Function(f) => DefinitionEnum::UserFunction(f),
                    UserFunctionDefinition::Inline(i) => DefinitionEnum::InlineUserFunction(i),
                };
                model.ast_root.add_definition(def_enum)?;
            }
            PortableKind::Invocation(inv_obj) => {
                let spec = parse_invocation_spec(&inv_obj)?;
                model.set_invocation(&name, spec)?;
            }
            PortableKind::Type(def_obj) => {
                let body = parse_type_definition(&def_obj)?;
                model.set_user_type(&name, body)?;
            }
            PortableKind::Context(ctx_obj) => {
                let expr = parse_static_object(&ctx_obj)?;
                model.set_expression(&name, expr)?;
            }
            PortableKind::Expression(raw) => {
                let expr = parse_expression_value(&raw)?;
                model.set_expression(&name, expr)?;
            }
        }
    }
    Ok(model)
}

pub fn serialize_model(model: &EdgeRulesModel) -> Result<JsValue, PortableError> {
    serialize_builder(&model.ast_root)
}

fn extract_metadata(obj: &JsValue) -> Metadata {
    let mut m = Metadata::default();
    if let Some(v) = get_prop(obj, "@version").and_then(|v| v.as_string()) {
        m.version = Some(v);
    }
    if let Some(v) = get_prop(obj, "@model_name").and_then(|v| v.as_string()) {
        m.model_name = Some(v);
    }
    m
}

fn attach_metadata(obj: &JsValue, metadata: Option<&Metadata>) -> Result<(), PortableError> {
    if let Some(m) = metadata {
        if let Some(v) = &m.version {
            let _ = set_prop(obj, "@version", &JsValue::from_str(v));
        }
        if let Some(v) = &m.model_name {
            let _ = set_prop(obj, "@model_name", &JsValue::from_str(v));
        }
    }
    Ok(())
}

fn set_portable_prop(
    obj: &JsValue,
    key: &str,
    value: &JsValue,
    error_key: PortableObjectKey,
) -> Result<(), PortableError> {
    set_prop(obj, key, value)
        .map_err(|_| PortableError::SerializationError(error_key, SchemaViolationType::NotSupported))
        .map(|_| ())
}

fn set_custom_prop(obj: &JsValue, key: &str, value: &JsValue) -> Result<(), PortableError> {
    set_portable_prop(obj, key, value, PortableObjectKey::Custom(key.to_string()))
}

pub fn apply_portable_entry(model: &mut EdgeRulesModel, path: &str, payload: &JsValue) -> Result<(), PortableError> {
    match classify_entry(payload) {
        PortableKind::Function(def_obj) => {
            let (context_path, function_name) = split_path(path)?;
            if let PathTarget::ArrayElement(_, _) = parse_path_target(&function_name)? {
                return Err(PortableError::SchemaViolation(
                    PortableObjectKey::Function,
                    SchemaViolationType::InvalidFieldType,
                ));
            }
            let definition = parse_function_definition(&function_name, &def_obj)?;
            apply_function_with_path(model, context_path, definition)?;
        }
        PortableKind::Invocation(inv_obj) => {
            let (_context_path, name) = split_path(path)?;
            if let PathTarget::ArrayElement(_, _) = parse_path_target(&name)? {
                return Err(PortableError::SchemaViolation(
                    PortableObjectKey::Invocation,
                    SchemaViolationType::InvalidFieldType,
                ));
            }
            let spec = parse_invocation_spec(&inv_obj)?;
            model.set_invocation(path, spec)?;
        }
        PortableKind::Type(def_obj) => {
            let (_context_path, name) = split_path(path)?;
            if let PathTarget::ArrayElement(_, _) = parse_path_target(&name)? {
                return Err(PortableError::SchemaViolation(
                    PortableObjectKey::Type,
                    SchemaViolationType::InvalidFieldType,
                ));
            }
            let body = parse_type_definition(&def_obj)?;
            model.set_user_type(path, body)?;
        }
        PortableKind::Context(ctx_obj) => {
            let (context_path, name) = split_path(path)?;
            match parse_path_target(&name)? {
                PathTarget::Field => {
                    let expr = parse_static_object(&ctx_obj)?;
                    model.set_expression(path, expr)?;
                }
                PathTarget::ArrayElement(array_name, index) => {
                    let full_array_path = join_path(context_path, &array_name);
                    let expr = parse_static_object(&ctx_obj)?;
                    set_array_element(model, &full_array_path, index, expr)?;
                }
            }
        }
        PortableKind::Expression(raw) => {
            let (context_path, name) = split_path(path)?;
            match parse_path_target(&name)? {
                PathTarget::Field => {
                    let expr = parse_expression_value(&raw)?;
                    model.set_expression(path, expr)?;
                }
                PathTarget::ArrayElement(array_name, index) => {
                    let full_array_path = join_path(context_path, &array_name);
                    let expr = parse_expression_value(&raw)?;
                    set_array_element(model, &full_array_path, index, expr)?;
                }
            }
        }
    }
    Ok(())
}

pub fn remove_portable_entry(model: &mut EdgeRulesModel, path: &str) -> Result<(), PortableError> {
    let (context_path, name) = split_path(path)?;
    if let PathTarget::ArrayElement(array_name, index) = parse_path_target(&name)? {
        let full_array_path = join_path(context_path, &array_name);
        let entry = model.get_expression(&full_array_path).map_err(PortableError::from)?;
        let mut borrowed = entry.borrow_mut();

        match &mut borrowed.expression {
            ExpressionEnum::Collection(col) => {
                if index >= col.elements.len() {
                    return Err(PortableError::from(ContextQueryErrorEnum::WrongFieldPathError(Some(format!(
                        "Index {} out of bounds",
                        index
                    )))));
                }
                col.elements.remove(index);
                return Ok(());
            }
            _ => {
                return Err(PortableError::from(ContextQueryErrorEnum::WrongFieldPathError(Some(format!(
                    "Field '{}' is not an array",
                    full_array_path
                )))));
            }
        }
    }

    match model.get_user_type(path) {
        Ok(_) => {
            model.remove_user_type(path)?;
            return Ok(());
        }
        Err(ContextQueryErrorEnum::EntryNotFoundError(_)) => {}
        Err(err) => return Err(PortableError::from(err)),
    }

    match model.get_user_function(path) {
        Ok(_) => {
            model.remove_user_function(path)?;
            return Ok(());
        }
        Err(ContextQueryErrorEnum::EntryNotFoundError(_)) => {}
        Err(err) => return Err(PortableError::from(err)),
    }

    match model.get_expression(path) {
        Ok(_) => {
            model.remove_expression(path)?;
            return Ok(());
        }
        Err(ContextQueryErrorEnum::EntryNotFoundError(_)) => {}
        Err(err) => return Err(PortableError::from(err)),
    }

    Err(ContextQueryErrorEnum::EntryNotFoundError(path.to_string()).into())
}

pub fn get_portable_entry(model: &EdgeRulesModel, path: &str) -> Result<JsValue, PortableError> {
    let (context_path, name) = split_path(path)?;
    if let PathTarget::ArrayElement(array_name, index) = parse_path_target(&name)? {
        let full_array_path = join_path(context_path, &array_name);
        let entry = model.get_expression(&full_array_path).map_err(PortableError::from)?;
        let borrowed = entry.borrow();

        match &borrowed.expression {
            ExpressionEnum::Collection(col) => {
                if index >= col.elements.len() {
                    return Err(PortableError::from(ContextQueryErrorEnum::WrongFieldPathError(Some(format!(
                        "Index {} out of bounds",
                        index
                    )))));
                }
                return serialize_expression(&col.elements[index]);
            }
            _ => {
                return Err(PortableError::from(ContextQueryErrorEnum::WrongFieldPathError(Some(format!(
                    "Field '{}' is not an array",
                    full_array_path
                )))));
            }
        }
    }

    match model.get_user_type(path) {
        Ok(body) => return serialize_type_body(&body),
        Err(ContextQueryErrorEnum::EntryNotFoundError(_)) => {}
        Err(err) => return Err(PortableError::from(err)),
    }

    match model.get_user_function(path) {
        Ok(function) => return serialize_function(&function.borrow().function_definition),
        Err(ContextQueryErrorEnum::EntryNotFoundError(_)) => {}
        Err(err) => return Err(PortableError::from(err)),
    }

    match model.get_expression(path) {
        Ok(expression) => return serialize_expression(&expression.borrow().expression),
        Err(ContextQueryErrorEnum::EntryNotFoundError(_)) => {}
        Err(err) => return Err(PortableError::from(err)),
    }

    Err(ContextQueryErrorEnum::EntryNotFoundError(path.to_string()).into())
}

fn join_path(context: Option<Vec<String>>, name: &str) -> String {
    match context {
        Some(parts) => format!("{}.{}", parts.join("."), name),
        None => name.to_string(),
    }
}

fn set_array_element(
    model: &mut EdgeRulesModel,
    array_path: &str,
    index: usize,
    value: ExpressionEnum,
) -> Result<(), PortableError> {
    let entry = model.get_expression(array_path).map_err(PortableError::from)?;
    let mut borrowed = entry.borrow_mut();
    match &mut borrowed.expression {
        ExpressionEnum::Collection(col) => {
            let len = col.elements.len();
            if index > len {
                return Err(PortableError::from(ContextQueryErrorEnum::WrongFieldPathError(Some(format!(
                    "Index {} is out of bounds for array of length {} (no gaps allowed)",
                    index, len
                )))));
            } else if index == len {
                col.elements.push(value);
            } else {
                col.elements[index] = value;
            }
            Ok(())
        }
        _ => Err(PortableError::from(ContextQueryErrorEnum::WrongFieldPathError(Some(format!(
            "Field '{}' is not an array",
            array_path
        ))))),
    }
}

enum PathTarget {
    Field,
    ArrayElement(String, usize),
}

fn parse_path_target(name: &str) -> Result<PathTarget, ContextQueryErrorEnum> {
    if name.ends_with(']') {
        if let Some(start_bracket) = name.rfind('[') {
            let array_name = name[..start_bracket].trim();
            let index_str = name[start_bracket + 1..name.len() - 1].trim();

            if array_name.is_empty() {
                return Err(ContextQueryErrorEnum::WrongFieldPathError(Some(format!(
                    "Invalid array path '{}'",
                    name
                ))));
            }

            let index = index_str
                .parse::<usize>()
                .map_err(|_| ContextQueryErrorEnum::WrongFieldPathError(Some(format!("Invalid array index '{}'", name))))?;

            return Ok(PathTarget::ArrayElement(array_name.to_string(), index));
        }
    }

    Ok(PathTarget::Field)
}

enum PortableKind {
    Function(JsValue),
    Type(JsValue),
    Invocation(JsValue),
    Context(JsValue),
    Expression(JsValue),
}
fn classify_entry(value: &JsValue) -> PortableKind {
    if is_object(value) {
        if let Some(kind) = get_prop(value, "@type").and_then(|v| v.as_string()) {
            match kind.as_str() {
                "function" => return PortableKind::Function(value.clone()),
                "type" => return PortableKind::Type(value.clone()),
                "invocation" => return PortableKind::Invocation(value.clone()),
                _ => {}
            }
        }
        return PortableKind::Context(value.clone());
    }
    PortableKind::Expression(value.clone())
}

fn object_field_iter(obj: &JsValue) -> Vec<(String, JsValue)> {
    if !is_object(obj) {
        return Vec::new();
    }
    let object: Object = obj.clone().unchecked_into();
    let keys = Object::keys(&object);
    let mut out = Vec::with_capacity(keys.length() as usize);
    for i in 0..keys.length() {
        let k = keys.get(i).as_string().unwrap_or_default();
        if let Some(v) = get_prop(obj, &k) {
            out.push((k, v));
        }
    }
    out
}

fn parse_function_definition(name: &str, obj: &JsValue) -> Result<UserFunctionDefinition, PortableError> {
    let mut parameters = Vec::new();
    if let Some(params) = get_prop(obj, PortableObjectKey::Parameters.as_str()) {
        if !is_object(&params) {
            return Err(PortableError::SchemaViolation(
                PortableObjectKey::Parameters,
                SchemaViolationType::InvalidFieldType,
            ));
        }
        for (param_name, param_type) in object_field_iter(&params) {
            let tref = match param_type.as_string() {
                Some(text) => parser::parse_type(&text),
                None if param_type.is_null() || param_type.is_undefined() => ComplexTypeRef::undefined(),
                None => {
                    return Err(PortableError::SchemaViolation(
                        PortableObjectKey::Custom(param_name),
                        SchemaViolationType::InvalidFieldType,
                    ))
                }
            };
            parameters.push(FormalParameter::with_type_ref(param_name.to_string(), tref));
        }
    }
    let builder = parse_context_builder(obj, true)?;

    // Inline collapse: single return field, no children, no functions
    let return_name = intern_field_name("return");
    if builder.get_field_names().len() == 1
        && builder.get_child_context(return_name).is_none()
        && builder.get_user_function(return_name).is_none()
        && builder.get_expression(return_name).is_some()
    {
        let expr = {
            let expr_ref = builder.get_expression(return_name).expect("checked above");
            let mut borrowed = expr_ref.borrow_mut();
            std::mem::replace(&mut borrowed.expression, ExpressionEnum::Value(ValueEnum::BooleanValue(false)))
        };
        let inline =
            InlineFunctionDefinition::build(name.to_string(), parameters, expr).map_err(PortableError::from)?;
        return Ok(UserFunctionDefinition::Inline(inline));
    }

    let body = builder.build();
    let function = FunctionDefinition::build(name.to_string(), parameters, body).map_err(PortableError::from)?;
    Ok(UserFunctionDefinition::Function(function))
}

fn parse_type_definition(obj: &JsValue) -> Result<UserTypeBody, PortableError> {
    if let Some(reference) = get_prop(obj, "@ref").and_then(|v| v.as_string()) {
        return Ok(UserTypeBody::TypeRef(parse_angle_type(&reference)?));
    }
    let builder = parse_type_body(obj)?;
    Ok(UserTypeBody::TypeObject(builder.build()))
}

fn parse_type_body(obj: &JsValue) -> Result<ContextObjectBuilder, PortableError> {
    let mut builder = ContextObjectBuilder::new();

    let metadata = extract_metadata(obj);
    if !metadata.is_empty() {
        builder.set_metadata(metadata);
    }

    for (name, value) in object_field_iter(obj) {
        if name.starts_with('@') {
            continue;
        }
        match classify_entry(&value) {
            PortableKind::Type(nested) => {
                let nested_builder = parse_type_body(&nested)?;
                builder.add_expression(&name, ExpressionEnum::StaticObject(nested_builder.build()))?;
            }
            PortableKind::Invocation(_) => {
                return Err(PortableError::SchemaViolation(
                    PortableObjectKey::Invocation,
                    SchemaViolationType::NotSupported,
                ));
            }
            PortableKind::Context(nested_ctx) => {
                let nested_expr = parse_static_object(&nested_ctx)?;
                builder.add_expression(&name, nested_expr)?;
            }
            PortableKind::Function(_) | PortableKind::Expression(_) => {
                if let Some(text) = value.as_string() {
                    let tref = parse_angle_type(&text)?;
                    builder.add_expression(&name, ExpressionEnum::TypePlaceholder(tref))?;
                } else {
                    return Err(PortableError::SchemaViolation(
                        PortableObjectKey::Custom(name),
                        SchemaViolationType::InvalidFieldType,
                    ));
                }
            }
        }
    }
    Ok(builder)
}

fn parse_static_object(obj: &JsValue) -> Result<ExpressionEnum, PortableError> {
    let builder = parse_context_builder(obj, false)?;
    Ok(ExpressionEnum::StaticObject(builder.build()))
}

fn parse_context_builder(obj: &JsValue, skip_metadata: bool) -> Result<ContextObjectBuilder, PortableError> {
    let mut builder = ContextObjectBuilder::new();

    let metadata = extract_metadata(obj);
    if !metadata.is_empty() {
        builder.set_metadata(metadata);
    }

    for (name, value) in object_field_iter(obj) {
        if skip_metadata && name.starts_with('@') {
            continue;
        }
        // Also skip metadata fields if we just extracted them
        if !skip_metadata && (name == "@version" || name == "@model_name") {
            continue;
        }

        match classify_entry(&value) {
            PortableKind::Function(def_obj) => {
                let definition = parse_function_definition(&name, &def_obj)?;
                let def_enum = match definition {
                    UserFunctionDefinition::Function(f) => DefinitionEnum::UserFunction(f),
                    UserFunctionDefinition::Inline(i) => DefinitionEnum::InlineUserFunction(i),
                };
                builder.add_definition(def_enum)?;
            }
            PortableKind::Invocation(inv_obj) => {
                let expr = parse_invocation_expression(&inv_obj)?;
                builder.add_expression(&name, expr)?;
            }
            PortableKind::Type(def_obj) => {
                let body = parse_type_definition(&def_obj)?;
                builder.set_user_type_definition(name.clone(), body);
            }
            PortableKind::Context(nested) => {
                let expr = parse_static_object(&nested)?;
                builder.add_expression(&name, expr)?;
            }
            PortableKind::Expression(raw) => {
                let expr = parse_expression_value(&raw)?;
                builder.add_expression(&name, expr)?;
            }
        }
    }
    Ok(builder)
}

fn parse_expression_value(value: &JsValue) -> Result<ExpressionEnum, PortableError> {
    if value.is_null() || value.is_undefined() {
        return Err(PortableError::SchemaViolation(PortableObjectKey::Root, SchemaViolationType::InvalidFieldType));
    }
    if let Some(flag) = value.as_bool() {
        return Ok(ExpressionEnum::from(flag));
    }
    if let Some(text) = value.as_string() {
        return Ok(EdgeRulesModel::parse_expression(&text)?);
    }
    if let Some(number) = value.as_f64() {
        if number.fract() == 0.0 {
            return Ok(ExpressionEnum::from(ValueEnum::from(number as i64)));
        }
        return Ok(ExpressionEnum::from(ValueEnum::from(number)));
    }
    if Array::is_array(value) {
        let arr: Array = value.clone().unchecked_into();
        let mut expressions = Vec::with_capacity(arr.length() as usize);
        for i in 0..arr.length() {
            expressions.push(parse_expression_value(&arr.get(i))?);
        }
        return Ok(ExpressionEnum::Collection(CollectionExpression::build(expressions)));
    }
    if is_object(value) {
        return parse_static_object(value);
    }
    Err(PortableError::SchemaViolation(PortableObjectKey::Root, SchemaViolationType::InvalidFieldType))
}

fn parse_invocation_spec(obj: &JsValue) -> Result<InvocationSpec, PortableError> {
    if !is_object(obj) {
        return Err(PortableError::SchemaViolation(
            PortableObjectKey::Invocation,
            SchemaViolationType::InvalidFieldType,
        ));
    }
    let Some(method) = get_prop(obj, PortableObjectKey::Method.as_str()).and_then(|v| v.as_string()) else {
        return Err(PortableError::SchemaViolation(
            PortableObjectKey::Method,
            SchemaViolationType::MissingRequiredField,
        ));
    };
    let trimmed_method = method.trim();
    if trimmed_method.is_empty() {
        return Err(PortableError::SchemaViolation(PortableObjectKey::Method, SchemaViolationType::Empty));
    }
    let arguments = match get_prop(obj, "@arguments") {
        None => default_invocation_arguments()?,
        Some(arg_list) => parse_invocation_arguments(&arg_list)?,
    };
    Ok(InvocationSpec { method_path: trimmed_method.to_string(), arguments })
}

fn parse_invocation_arguments(args: &JsValue) -> Result<Vec<ExpressionEnum>, PortableError> {
    if !Array::is_array(args) {
        return Err(PortableError::SchemaViolation(
            PortableObjectKey::Arguments,
            SchemaViolationType::InvalidFieldType,
        ));
    }
    let arr: Array = args.clone().unchecked_into();
    let mut out = Vec::with_capacity(arr.length() as usize);
    for i in 0..arr.length() {
        out.push(parse_expression_value(&arr.get(i))?);
    }
    Ok(out)
}

fn default_invocation_arguments() -> Result<Vec<ExpressionEnum>, PortableError> {
    Ok(vec![EdgeRulesModel::parse_expression("request")?])
}

fn parse_invocation_expression(obj: &JsValue) -> Result<ExpressionEnum, PortableError> {
    let spec = parse_invocation_spec(obj)?;
    Ok(ExpressionEnum::from(UserFunctionCall::new(spec.method_path, spec.arguments)))
}

fn parse_angle_type(text: &str) -> Result<ComplexTypeRef, PortableError> {
    let trimmed = text.trim();
    if !trimmed.starts_with('<') {
        return Err(PortableError::SchemaViolation(PortableObjectKey::Ref, SchemaViolationType::InvalidFormat));
    }
    // Remove leading <, keep trailing > as parser expects it
    let inner_with_closing = &trimmed[1..];
    let mut stream = CharStream::new(inner_with_closing);
    parser::parse_complex_type_in_angle(&mut stream)
        .map_err(|_| PortableError::SchemaViolation(PortableObjectKey::Ref, SchemaViolationType::InvalidFormat))
}

fn split_path(path: &str) -> Result<(Option<Vec<String>>, String), ContextQueryErrorEnum> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(ContextQueryErrorEnum::WrongFieldPathError(None));
    }
    let mut segments: Vec<String> = trimmed.split('.').map(|s| s.trim().to_string()).collect();
    if segments.iter().any(|s| s.is_empty()) {
        return Err(ContextQueryErrorEnum::WrongFieldPathError(Some(path.to_string())));
    }
    let name = segments.pop().unwrap();
    if segments.is_empty() {
        return Ok((None, name));
    }
    Ok((Some(segments), name))
}

fn apply_function_with_path(
    model: &mut EdgeRulesModel,
    context: Option<Vec<String>>,
    definition: UserFunctionDefinition,
) -> Result<(), PortableError> {
    if let Some(path) = context {
        let segments: Vec<&str> = path.iter().map(|s| s.as_str()).collect();
        model.set_user_function(definition, Some(segments))?;
    } else {
        model.set_user_function(definition, None)?;
    }
    Ok(())
}

fn serialize_builder(builder: &ContextObjectBuilder) -> Result<JsValue, PortableError> {
    let map = context_builder_to_object(builder)?;
    attach_metadata(&map, builder.get_metadata())?;
    Ok(map)
}

fn serialize_expression(expr: &ExpressionEnum) -> Result<JsValue, PortableError> {
    match expr {
        ExpressionEnum::Value(value) => serialize_value(value),
        ExpressionEnum::StaticObject(ctx) => context_to_object(&ctx.borrow()),
        _ => {
            if let Some(call) = expr.as_user_function_call() {
                return serialize_invocation_call(call);
            }
            Ok(JsValue::from_str(&expr.to_string()))
        }
    }
}

fn serialize_invocation_call(call: &UserFunctionCall) -> Result<JsValue, PortableError> {
    let obj = Object::new();
    set_portable_prop(&obj, "@type", &JsValue::from_str("invocation"), PortableObjectKey::Type)?;
    set_portable_prop(&obj, "@method", &JsValue::from_str(&call.name), PortableObjectKey::Method)?;
    if !call.args.is_empty() {
        let args = Array::new();
        for arg in &call.args {
            args.push(&serialize_expression(arg)?);
        }
        set_portable_prop(&obj, "@arguments", &args, PortableObjectKey::Arguments)?;
    }
    Ok(JsValue::from(obj))
}
fn serialize_value(value: &ValueEnum) -> Result<JsValue, PortableError> {
    match value {
        ValueEnum::BooleanValue(flag) => Ok(JsValue::from_bool(*flag)),
        ValueEnum::NumberValue(number) => match number {
            NumberEnum::Int(i) => Ok(JsValue::from_f64(*i as f64)),
            NumberEnum::Real(r) => Ok(JsValue::from_f64(r.to_f64().unwrap_or(0.0))),
            NumberEnum::SV(_) => Ok(JsValue::from_str(&value.to_string())),
        },
        ValueEnum::StringValue(_) => Ok(JsValue::from_str(&value.to_string())),
        ValueEnum::Array(_) | ValueEnum::Reference(_) => Ok(JsValue::from_str(&value.to_string())),
        _ => Ok(JsValue::from_str(&value.to_string())),
    }
}
fn serialize_type_body(body: &UserTypeBody) -> Result<JsValue, PortableError> {
    match body {
        UserTypeBody::TypeRef(reference) => {
            let obj = Object::new();
            set_portable_prop(&obj, "@type", &JsValue::from_str("type"), PortableObjectKey::Type)?;
            set_portable_prop(
                &obj,
                "@ref",
                &JsValue::from_str(&format!("<{}>", reference)),
                PortableObjectKey::Ref,
            )?;
            Ok(JsValue::from(obj))
        }
        UserTypeBody::TypeObject(ctx) => {
            let obj = context_to_object(&ctx.borrow())?;
            set_portable_prop(&obj, "@type", &JsValue::from_str("type"), PortableObjectKey::Type)?;
            Ok(obj)
        }
    }
}
fn serialize_function(definition: &UserFunctionDefinition) -> Result<JsValue, PortableError> {
    let body = definition.get_body().map_err(PortableError::from)?;
    let body_obj = context_to_object(&body.borrow())?;
    let params = definition.get_parameters();

    set_portable_prop(&body_obj, "@type", &JsValue::from_str("function"), PortableObjectKey::Type)?;
    if !params.is_empty() {
        let params_obj = Object::new();
        for param in params.iter() {
            let type_val = if param.parameter_type.is_undefined() {
                JsValue::NULL
            } else {
                JsValue::from_str(&param.parameter_type.to_string())
            };
            set_custom_prop(&params_obj, &param.name, &type_val)?;
        }
        set_portable_prop(&body_obj, "@parameters", &params_obj, PortableObjectKey::Parameters)?;
    }
    Ok(body_obj)
}

fn context_builder_to_object(builder: &ContextObjectBuilder) -> Result<JsValue, PortableError> {
    let obj = Object::new();
    for (name, body) in builder.user_type_entries() {
        set_custom_prop(&obj, &name, &serialize_type_body(&body)?)?;
    }
    for name in builder.get_field_names() {
        if let Some(expr) = builder.get_expression(name) {
            set_custom_prop(&obj, name, &serialize_expression(&expr.borrow().expression)?)?;
            continue;
        }
        if let Some(child) = builder.get_child_context(name) {
            set_custom_prop(&obj, name, &context_to_object(&child.borrow())?)?;
            continue;
        }
        if let Some(function) = builder.get_user_function(name) {
            set_custom_prop(&obj, name, &serialize_function(&function.borrow().function_definition)?)?;
        }
    }
    Ok(JsValue::from(obj))
}

fn context_to_object(ctx: &ContextObject) -> Result<JsValue, PortableError> {
    let obj = Object::new();

    attach_metadata(&obj, ctx.metadata.as_ref())?;

    for &name in ctx.get_field_names().iter() {
        if let Some(expr) = ctx.expressions.get(name) {
            let key = if name == RETURN_EXPRESSION { "return" } else { name };
            set_custom_prop(&obj, key, &serialize_expression(&expr.borrow().expression)?)?;
            continue;
        }
        if let Some(child) = ctx.node().get_child(name) {
            set_custom_prop(&obj, name, &context_to_object(&child.borrow())?)?;
            continue;
        }
        if let Some(function) = ctx.metaphors.get(name) {
            set_custom_prop(&obj, name, &serialize_function(&function.borrow().function_definition)?)?;
        }
    }
    for (type_name, body) in &ctx.defined_types {
        set_custom_prop(&obj, type_name, &serialize_type_body(body)?)?;
    }
    Ok(JsValue::from(obj))
}

#[cfg(test)]
mod tests {
    use super::*;
    use edge_rules::ast::sequence::CollectionExpression;
    use edge_rules::typesystem::values::ValueEnum;

    #[test]
    fn test_parse_path_target_field() {
        let target = parse_path_target("someField").unwrap();
        match target {
            PathTarget::Field => {}
            _ => panic!("Expected Field target"),
        }
    }

    #[test]
    fn test_parse_path_target_array() {
        let res = parse_path_target("rules[0]").unwrap();
        match res {
            PathTarget::ArrayElement(name, idx) => {
                assert_eq!(name, "rules");
                assert_eq!(idx, 0);
            }
            _ => panic!("Expected ArrayElement"),
        }

        let res = parse_path_target("items[10]").unwrap();
        match res {
            PathTarget::ArrayElement(name, idx) => {
                assert_eq!(name, "items");
                assert_eq!(idx, 10);
            }
            _ => panic!("Expected ArrayElement"),
        }
    }

    #[test]
    fn test_parse_path_target_invalid() {
        assert!(parse_path_target("rules[").is_err());
        assert!(parse_path_target("rules[a]").is_err());
        assert!(parse_path_target("rules[-1]").is_err());
        assert!(parse_path_target("rules[]").is_err());
        assert!(parse_path_target("[0]").is_err()); // empty array name
    }

    #[test]
    fn test_set_array_element_append() {
        let mut model = EdgeRulesModel::new();
        // Setup array
        let col = ExpressionEnum::Collection(CollectionExpression::build(vec![]));
        model.set_expression("list", col).unwrap();

        // Append [0]
        let val = ExpressionEnum::from(ValueEnum::from(10));
        set_array_element(&mut model, "list", 0, val).unwrap();

        // Check
        let entry = model.get_expression("list").unwrap();
        let borrowed = entry.borrow();
        if let ExpressionEnum::Collection(c) = &borrowed.expression {
            assert_eq!(c.elements.len(), 1);
        } else {
            panic!("Not a collection");
        }
    }

    #[test]
    fn test_set_array_element_gap_error() {
        let mut model = EdgeRulesModel::new();
        let col = ExpressionEnum::Collection(CollectionExpression::build(vec![]));
        model.set_expression("list", col).unwrap();

        // Try [1] on empty -> error
        let val = ExpressionEnum::from(ValueEnum::from(10));
        let err = set_array_element(&mut model, "list", 1, val);
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("out of bounds"));
    }

    #[test]
    fn test_set_array_element_overwrite() {
        let mut model = EdgeRulesModel::new();
        let val1 = ExpressionEnum::from(ValueEnum::from(10));
        let col = ExpressionEnum::Collection(CollectionExpression::build(vec![val1]));
        model.set_expression("list", col).unwrap();

        // Overwrite [0]
        let val2 = ExpressionEnum::from(ValueEnum::from(20));
        set_array_element(&mut model, "list", 0, val2).unwrap();

        let entry = model.get_expression("list").unwrap();
        let borrowed = entry.borrow();
        if let ExpressionEnum::Collection(c) = &borrowed.expression {
            assert_eq!(c.elements.len(), 1);
        }
    }

    #[test]
    fn test_parse_angle_type_with_default() {
        let tref = parse_angle_type("<number, 10>").unwrap();
        assert_eq!(tref.to_string(), "number, 10");

        let tref_str = parse_angle_type("<string, 'foo'>").unwrap();
        assert_eq!(tref_str.to_string(), "string, 'foo'");

        let tref_bool = parse_angle_type("<boolean, true>").unwrap();
        assert_eq!(tref_bool.to_string(), "boolean, true");
    }
}
