#![cfg(all(target_arch = "wasm32", feature = "wasm"))]

use crate::ast::context::context_object::ContextObject;
use crate::ast::context::context_object_builder::ContextObjectBuilder;
use crate::ast::context::duplicate_name_error::DuplicateNameError;
use crate::ast::metaphors::functions::FunctionDefinition;
use crate::ast::sequence::CollectionExpression;
use crate::ast::token::ExpressionEnum;
use crate::ast::token::{ComplexTypeRef, DefinitionEnum, UserTypeBody};
use crate::link::node_data::Node;
// for node()
use crate::runtime::decision_service::DecisionService;
use crate::runtime::edge_rules::{ContextUpdateErrorEnum, EdgeRulesModel, EvalError, ParseErrors};
use crate::tokenizer::parser;
use crate::typesystem::errors::{ParseErrorEnum, RuntimeError};
use crate::typesystem::types::number::NumberEnum;
use crate::typesystem::values::ValueEnum;
use js_sys::{Array, Object, Reflect};
use std::fmt::{Display, Formatter};
use wasm_bindgen::JsCast;
// for unchecked_into
use wasm_bindgen::JsValue;

#[derive(Debug)]
pub struct PortableError {
    message: String,
}
impl PortableError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
    pub fn into_message(self) -> String {
        self.message
    }
}
impl Display for PortableError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl From<PortableError> for String {
    fn from(v: PortableError) -> Self {
        v.message
    }
}
impl From<ContextUpdateErrorEnum> for PortableError {
    fn from(err: ContextUpdateErrorEnum) -> Self {
        PortableError::from(ParseErrorEnum::from(err))
    }
}
impl From<DuplicateNameError> for PortableError {
    fn from(err: DuplicateNameError) -> Self {
        Self::new(err.to_string())
    }
}
impl From<ParseErrorEnum> for PortableError {
    fn from(err: ParseErrorEnum) -> Self {
        Self::new(err.to_string())
    }
}
impl From<ParseErrors> for PortableError {
    fn from(err: ParseErrors) -> Self {
        Self::new(err.to_string())
    }
}
impl From<EvalError> for PortableError {
    fn from(err: EvalError) -> Self {
        Self::new(err.to_string())
    }
}
impl From<RuntimeError> for PortableError {
    fn from(err: RuntimeError) -> Self {
        Self::new(err.to_string())
    }
}

pub struct DecisionServiceController {
    service: DecisionService,
}
impl DecisionServiceController {
    pub fn from_portable(portable: &JsValue) -> Result<Self, PortableError> {
        let model = model_from_portable(portable)?;
        let service = DecisionService::from_model(model)?;
        Ok(Self { service })
    }
    pub fn execute_value(
        &mut self,
        method: &str,
        request: ValueEnum,
    ) -> Result<ValueEnum, PortableError> {
        Ok(self.service.execute(method, request)?)
    }
    pub fn model_snapshot(&mut self) -> Result<JsValue, PortableError> {
        let model = self.service.get_model();
        let snap = {
            let borrowed = model.borrow();
            serialize_model(&borrowed)?
        };
        Ok(snap)
    }
    pub fn set_entry(&mut self, path: &str, payload: &JsValue) -> Result<JsValue, PortableError> {
        let model = self.service.get_model();
        {
            let mut borrowed = model.borrow_mut();
            apply_portable_entry(&mut borrowed, path, payload)?;
        }
        let updated = {
            let borrowed = model.borrow();
            get_portable_entry(&borrowed, path)?
        };
        Ok(updated)
    }
    pub fn remove_entry(&mut self, path: &str) -> Result<(), PortableError> {
        let model = self.service.get_model();
        {
            let mut borrowed = model.borrow_mut();
            remove_portable_entry(&mut borrowed, path)?;
        }
        Ok(())
    }
    pub fn get_entry(&mut self, path: &str) -> Result<JsValue, PortableError> {
        let model = self.service.get_model();
        let entry = {
            let borrowed = model.borrow();
            get_portable_entry(&borrowed, path)?
        };
        Ok(entry)
    }
}

fn is_object(v: &JsValue) -> bool {
    v.is_object() && !Array::is_array(v)
}
fn get_prop(obj: &JsValue, name: &str) -> Option<JsValue> {
    Reflect::get(obj, &JsValue::from_str(name))
        .ok()
        .filter(|v| !v.is_undefined())
}

pub fn model_from_portable(portable: &JsValue) -> Result<EdgeRulesModel, PortableError> {
    if !is_object(portable) {
        return Err(PortableError::new("Portable context must be an object"));
    }
    let mut model = EdgeRulesModel::new();
    let object: Object = portable.clone().unchecked_into();
    let keys = Object::keys(&object);
    for i in 0..keys.length() {
        let name = keys.get(i).as_string().unwrap_or_default();
        if name.starts_with('@') {
            continue;
        }
        let value = get_prop(portable, &name).unwrap_or(JsValue::UNDEFINED);
        match classify_entry(&value) {
            PortableKind::Function(def_obj) => {
                let definition = parse_function_definition(&name, &def_obj)?;
                apply_function(&mut model, definition)?;
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

pub fn apply_portable_entry(
    model: &mut EdgeRulesModel,
    path: &str,
    payload: &JsValue,
) -> Result<(), PortableError> {
    match classify_entry(payload) {
        PortableKind::Function(def_obj) => {
            let (context_path, function_name) = split_path(path)?;
            let definition = parse_function_definition(&function_name, &def_obj)?;
            apply_function_with_path(model, context_path, definition)?;
        }
        PortableKind::Type(def_obj) => {
            let body = parse_type_definition(&def_obj)?;
            model.set_user_type(path, body)?;
        }
        PortableKind::Context(ctx_obj) => {
            let expr = parse_static_object(&ctx_obj)?;
            model.set_expression(path, expr)?;
        }
        PortableKind::Expression(raw) => {
            let expr = parse_expression_value(&raw)?;
            model.set_expression(path, expr)?;
        }
    }
    Ok(())
}

pub fn remove_portable_entry(model: &mut EdgeRulesModel, path: &str) -> Result<(), PortableError> {
    if model.get_user_type(path).is_some() {
        model.remove_user_type(path)?;
        return Ok(());
    }
    if model.get_user_function(path).is_some() {
        model.remove_user_function(path)?;
        return Ok(());
    }
    if model.get_expression(path).is_some() {
        model.remove_expression(path)?;
        return Ok(());
    }
    Err(PortableError::new(format!(
        "Entry '{}' not found in decision service model",
        path
    )))
}

pub fn get_portable_entry(model: &EdgeRulesModel, path: &str) -> Result<JsValue, PortableError> {
    if let Some(body) = model.get_user_type(path) {
        return serialize_type_body(&body);
    }
    if let Some(function) = model.get_user_function(path) {
        return serialize_function(&function.borrow().function_definition);
    }
    if let Some(expression) = model.get_expression(path) {
        return serialize_expression(&expression.borrow().expression);
    }
    Err(PortableError::new(format!(
        "Entry '{}' not found in decision service model",
        path
    )))
}

enum PortableKind {
    Function(JsValue),
    Type(JsValue),
    Context(JsValue),
    Expression(JsValue),
}
fn classify_entry(value: &JsValue) -> PortableKind {
    if is_object(value) {
        if let Some(kind) = get_prop(value, "@type").and_then(|v| v.as_string()) {
            match kind.as_str() {
                "function" => return PortableKind::Function(value.clone()),
                "type" => return PortableKind::Type(value.clone()),
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

fn parse_function_definition(
    name: &str,
    obj: &JsValue,
) -> Result<FunctionDefinition, PortableError> {
    let mut parameters = Vec::new();
    if let Some(params) = get_prop(obj, "@parameters") {
        if !is_object(&params) {
            return Err(PortableError::new("@parameters must be an object"));
        }
        for (param_name, param_type) in object_field_iter(&params) {
            let tref = match param_type.as_string() {
                Some(text) => Some(parser::parse_type(&text)),
                None => {
                    return Err(PortableError::new(format!(
                        "Invalid parameter type for '{}': {:?}",
                        param_name, param_type
                    )))
                }
            };
            parameters.push(
                crate::ast::context::context_object_type::FormalParameter::with_type_ref(
                    param_name.to_string(),
                    tref,
                ),
            );
        }
    }
    let body_builder = parse_context_builder(obj, true)?;
    let body = body_builder.build();
    FunctionDefinition::build(name.to_string(), parameters, body).map_err(PortableError::from)
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
    for (name, value) in object_field_iter(obj) {
        if name.starts_with('@') {
            continue;
        }
        match classify_entry(&value) {
            PortableKind::Type(nested) => {
                let nested_builder = parse_type_body(&nested)?;
                builder
                    .add_expression(&name, ExpressionEnum::StaticObject(nested_builder.build()))?;
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
                    return Err(PortableError::new(format!(
                        "Type field '{}' must be a string or object",
                        name
                    )));
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

fn parse_context_builder(
    obj: &JsValue,
    skip_metadata: bool,
) -> Result<ContextObjectBuilder, PortableError> {
    let mut builder = ContextObjectBuilder::new();
    for (name, value) in object_field_iter(obj) {
        if skip_metadata && name.starts_with('@') {
            continue;
        }
        match classify_entry(&value) {
            PortableKind::Function(def_obj) => {
                let definition = parse_function_definition(&name, &def_obj)?;
                builder.add_definition(DefinitionEnum::UserFunction(definition))?;
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
        return Err(PortableError::new(
            "null/undefined not supported in EdgeRules Portable",
        ));
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
        return Ok(ExpressionEnum::Collection(CollectionExpression::build(
            expressions,
        )));
    }
    if is_object(value) {
        return parse_static_object(value);
    }
    Err(PortableError::new("Unsupported portable expression value"))
}

fn parse_angle_type(text: &str) -> Result<ComplexTypeRef, PortableError> {
    let trimmed = text.trim();
    if let Some(inner) = trimmed.strip_prefix('<').and_then(|v| v.strip_suffix('>')) {
        return Ok(parser::parse_type(inner.trim()));
    }
    Err(PortableError::new(format!(
        "Type reference '{}' must use <...> notation",
        text
    )))
}

fn split_path(path: &str) -> Result<(Option<Vec<String>>, String), PortableError> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(PortableError::new("Path cannot be empty"));
    }
    let mut segments: Vec<String> = trimmed.split('.').map(|s| s.trim().to_string()).collect();
    if segments.iter().any(|s| s.is_empty()) {
        return Err(PortableError::new(format!("Invalid path '{}'", path)));
    }
    let name = segments.pop().unwrap();
    if segments.is_empty() {
        return Ok((None, name));
    }
    Ok((Some(segments), name))
}

fn apply_function(
    model: &mut EdgeRulesModel,
    definition: FunctionDefinition,
) -> Result<(), PortableError> {
    model.set_user_function(definition, None)?;
    Ok(())
}
fn apply_function_with_path(
    model: &mut EdgeRulesModel,
    context: Option<Vec<String>>,
    definition: FunctionDefinition,
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
    Ok(map)
}
fn serialize_expression(expr: &ExpressionEnum) -> Result<JsValue, PortableError> {
    match expr {
        ExpressionEnum::Value(value) => serialize_value(value),
        ExpressionEnum::StaticObject(ctx) => context_to_object(&ctx.borrow()),
        _ => Ok(JsValue::from_str(&expr.to_string())),
    }
}
fn serialize_value(value: &ValueEnum) -> Result<JsValue, PortableError> {
    match value {
        ValueEnum::BooleanValue(flag) => Ok(JsValue::from_bool(*flag)),
        ValueEnum::NumberValue(number) => match number {
            NumberEnum::Int(i) => Ok(JsValue::from_f64(*i as f64)),
            NumberEnum::Real(r) => Ok(JsValue::from_f64(*r)),
            NumberEnum::SV(_) => Ok(JsValue::from_str(&value.to_string())),
        },
        ValueEnum::StringValue(_) => Ok(JsValue::from_str(&value.to_string())),
        ValueEnum::Array(_) | ValueEnum::Reference(_) => Ok(JsValue::from_str(&value.to_string())),
        _ => Ok(JsValue::from_str(&value.to_string())),
    }
}
fn serialize_type_body(body: &UserTypeBody) -> Result<JsValue, PortableError> {
    match body {
        UserTypeBody::TypeRef(reference) => Ok(JsValue::from_str(&format!("<{}>", reference))),
        UserTypeBody::TypeObject(ctx) => {
            let obj = context_to_object(&ctx.borrow())?;
            Reflect::set(
                &obj,
                &JsValue::from_str("@type"),
                &JsValue::from_str("type"),
            )
            .map_err(|_| PortableError::new("Failed to set type metadata"))?;
            Ok(obj)
        }
    }
}
fn serialize_function(definition: &FunctionDefinition) -> Result<JsValue, PortableError> {
    let obj = context_to_object(&definition.body.borrow())?;
    Reflect::set(
        &obj,
        &JsValue::from_str("@type"),
        &JsValue::from_str("function"),
    )
    .map_err(|_| PortableError::new("Failed to set function metadata"))?;
    if !definition.arguments.is_empty() {
        let params = Object::new();
        for param in &definition.arguments {
            Reflect::set(
                &params,
                &JsValue::from_str(&param.name),
                &JsValue::from_str(&param.to_string()),
            )
            .map_err(|_| PortableError::new("Failed to set parameter"))?;
        }
        Reflect::set(&obj, &JsValue::from_str("@parameters"), &params)
            .map_err(|_| PortableError::new("Failed to attach parameters"))?;
    }
    Ok(obj)
}

fn context_builder_to_object(builder: &ContextObjectBuilder) -> Result<JsValue, PortableError> {
    let obj = Object::new();
    for (name, body) in builder.user_type_entries() {
        Reflect::set(
            &obj,
            &JsValue::from_str(&name),
            &serialize_type_body(&body)?,
        )
        .map_err(|_| PortableError::new("Failed to set type entry"))?;
    }
    for name in builder.get_field_names() {
        if let Some(expr) = builder.get_expression(name) {
            Reflect::set(
                &obj,
                &JsValue::from_str(name),
                &serialize_expression(&expr.borrow().expression)?,
            )
            .map_err(|_| PortableError::new("Failed to set expression"))?;
            continue;
        }
        if let Some(child) = builder.get_child_context(name) {
            Reflect::set(
                &obj,
                &JsValue::from_str(name),
                &context_to_object(&child.borrow())?,
            )
            .map_err(|_| PortableError::new("Failed to set child context"))?;
            continue;
        }
        if let Some(function) = builder.get_user_function(name) {
            Reflect::set(
                &obj,
                &JsValue::from_str(name),
                &serialize_function(&function.borrow().function_definition)?,
            )
            .map_err(|_| PortableError::new("Failed to set function"))?;
        }
    }
    Ok(JsValue::from(obj))
}

fn context_to_object(ctx: &ContextObject) -> Result<JsValue, PortableError> {
    let obj = Object::new();
    for &name in ctx.get_field_names().iter() {
        if let Some(expr) = ctx.expressions.get(name) {
            Reflect::set(
                &obj,
                &JsValue::from_str(name),
                &serialize_expression(&expr.borrow().expression)?,
            )
            .map_err(|_| PortableError::new("Failed to set expression"))?;
            continue;
        }
        if let Some(child) = ctx.node().get_child(name) {
            Reflect::set(
                &obj,
                &JsValue::from_str(name),
                &context_to_object(&child.borrow())?,
            )
            .map_err(|_| PortableError::new("Failed to set child context"))?;
            continue;
        }
        if let Some(function) = ctx.metaphors.get(name) {
            Reflect::set(
                &obj,
                &JsValue::from_str(name),
                &serialize_function(&function.borrow().function_definition)?,
            )
            .map_err(|_| PortableError::new("Failed to set function"))?;
        }
    }
    for (type_name, body) in &ctx.defined_types {
        Reflect::set(
            &obj,
            &JsValue::from_str(type_name),
            &serialize_type_body(body)?,
        )
        .map_err(|_| PortableError::new("Failed to set local type"))?;
    }
    Ok(JsValue::from(obj))
}
