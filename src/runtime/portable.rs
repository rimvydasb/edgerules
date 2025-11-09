use crate::ast::context::context_object::ContextObject;
use crate::ast::context::context_object_builder::ContextObjectBuilder;
use crate::ast::context::duplicate_name_error::DuplicateNameError;
use crate::ast::metaphors::functions::FunctionDefinition;
use crate::ast::sequence::CollectionExpression;
use crate::ast::token::ExpressionEnum;
use crate::ast::token::{ComplexTypeRef, DefinitionEnum, UserTypeBody};
use crate::link::node_data::Node;
#[cfg(feature = "mutable_decision_service")]
use crate::runtime::decision_service::DecisionService;
use crate::runtime::edge_rules::{ContextUpdateErrorEnum, EdgeRulesModel, EvalError, ParseErrors};
use crate::tokenizer::parser;
use crate::typesystem::errors::{ParseErrorEnum, RuntimeError};
use crate::typesystem::types::number::NumberEnum;
use crate::typesystem::values::ValueEnum;
use serde_json::{Map, Number, Value};
use std::fmt::{Display, Formatter};

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
    fn from(value: PortableError) -> Self {
        value.message
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

#[cfg(feature = "mutable_decision_service")]
pub struct DecisionServiceController {
    service: DecisionService,
}

#[cfg(feature = "mutable_decision_service")]
impl DecisionServiceController {
    pub fn from_portable(portable: &Value) -> Result<Self, PortableError> {
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

    pub fn model_snapshot(&mut self) -> Result<Value, PortableError> {
        let model = self.service.get_model();
        let snapshot = {
            let borrowed = model.borrow();
            serialize_model(&borrowed)?
        };
        Ok(snapshot)
    }

    pub fn set_entry(&mut self, path: &str, payload: &Value) -> Result<Value, PortableError> {
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

    pub fn get_entry(&mut self, path: &str) -> Result<Value, PortableError> {
        let model = self.service.get_model();
        let entry = {
            let borrowed = model.borrow();
            get_portable_entry(&borrowed, path)?
        };
        Ok(entry)
    }

    pub fn service(&mut self) -> &mut DecisionService {
        &mut self.service
    }
}

pub fn model_from_portable(portable: &Value) -> Result<EdgeRulesModel, PortableError> {
    let map = portable
        .as_object()
        .ok_or_else(|| PortableError::new("Portable context must be an object"))?;

    let mut model = EdgeRulesModel::new();
    for (name, value) in map {
        if name.starts_with('@') {
            continue;
        }

        match classify_entry(value) {
            PortableKind::Function(def_map) => {
                let definition = parse_function_definition(name, def_map)?;
                apply_function(&mut model, definition)?;
            }
            PortableKind::Type(def_map) => {
                let body = parse_type_definition(def_map)?;
                model.set_user_type(name, body)?;
            }
            PortableKind::Context(ctx_map) => {
                let expr = parse_static_object(ctx_map)?;
                model.set_expression(name, expr)?;
            }
            PortableKind::Expression(raw) => {
                let expr = parse_expression_value(raw)?;
                model.set_expression(name, expr)?;
            }
        }
    }

    Ok(model)
}

pub fn serialize_model(model: &EdgeRulesModel) -> Result<Value, PortableError> {
    serialize_builder(&model.ast_root)
}

pub fn apply_portable_entry(
    model: &mut EdgeRulesModel,
    path: &str,
    payload: &Value,
) -> Result<(), PortableError> {
    match classify_entry(payload) {
        PortableKind::Function(def_map) => {
            let (context_path, function_name) = split_path(path)?;
            let definition = parse_function_definition(&function_name, def_map)?;
            apply_function_with_path(model, context_path, definition)?;
        }
        PortableKind::Type(def_map) => {
            let body = parse_type_definition(def_map)?;
            model.set_user_type(path, body)?;
        }
        PortableKind::Context(ctx_map) => {
            let expr = parse_static_object(ctx_map)?;
            model.set_expression(path, expr)?;
        }
        PortableKind::Expression(raw) => {
            let expr = parse_expression_value(raw)?;
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

pub fn get_portable_entry(model: &EdgeRulesModel, path: &str) -> Result<Value, PortableError> {
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

enum PortableKind<'a> {
    Function(&'a Map<String, Value>),
    Type(&'a Map<String, Value>),
    Context(&'a Map<String, Value>),
    Expression(&'a Value),
}

fn classify_entry(value: &Value) -> PortableKind<'_> {
    if let Some(map) = value.as_object() {
        if let Some(Value::String(kind)) = map.get("@type") {
            match kind.as_str() {
                "function" => return PortableKind::Function(map),
                "type" => return PortableKind::Type(map),
                _ => {}
            }
        }
        return PortableKind::Context(map);
    }

    PortableKind::Expression(value)
}

fn parse_function_definition(
    name: &str,
    map: &Map<String, Value>,
) -> Result<FunctionDefinition, PortableError> {
    let mut parameters = Vec::new();
    if let Some(params) = map.get("@parameters") {
        let params_map = params
            .as_object()
            .ok_or_else(|| PortableError::new("@parameters must be an object"))?;
        for (param_name, param_type) in params_map {
            let tref = match param_type {
                Value::String(text) => Some(parser::parse_type(text)),
                other => {
                    return Err(PortableError::new(format!(
                        "Invalid parameter type for '{}': {}",
                        param_name, other
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

    let body_builder = parse_context_builder(map, true)?;
    let body = body_builder.build();
    FunctionDefinition::build(name.to_string(), parameters, body).map_err(PortableError::from)
}

fn parse_type_definition(map: &Map<String, Value>) -> Result<UserTypeBody, PortableError> {
    if let Some(Value::String(reference)) = map.get("@ref") {
        return Ok(UserTypeBody::TypeRef(parse_angle_type(reference)?));
    }

    let builder = parse_type_body(map)?;
    Ok(UserTypeBody::TypeObject(builder.build()))
}

fn parse_type_body(map: &Map<String, Value>) -> Result<ContextObjectBuilder, PortableError> {
    let mut builder = ContextObjectBuilder::new();
    for (name, value) in map {
        if name.starts_with('@') {
            continue;
        }

        match value {
            Value::String(text) => {
                let tref = parse_angle_type(text)?;
                builder.add_expression(name, ExpressionEnum::TypePlaceholder(tref))?;
            }
            Value::Object(nested) => {
                if let Some(Value::String(kind)) = nested.get("@type") {
                    if kind == "type" {
                        let nested_builder = parse_type_body(nested)?;
                        builder.add_expression(
                            name,
                            ExpressionEnum::StaticObject(nested_builder.build()),
                        )?;
                        continue;
                    }
                }

                let nested_expr = parse_static_object(nested)?;
                builder.add_expression(name, nested_expr)?;
            }
            _ => {
                return Err(PortableError::new(format!(
                    "Type field '{}' must be a string or object",
                    name
                )))
            }
        }
    }

    Ok(builder)
}

fn parse_static_object(map: &Map<String, Value>) -> Result<ExpressionEnum, PortableError> {
    let builder = parse_context_builder(map, false)?;
    Ok(ExpressionEnum::StaticObject(builder.build()))
}

fn parse_context_builder(
    map: &Map<String, Value>,
    skip_metadata: bool,
) -> Result<ContextObjectBuilder, PortableError> {
    let mut builder = ContextObjectBuilder::new();
    for (name, value) in map {
        if skip_metadata && name.starts_with('@') {
            continue;
        }

        match classify_entry(value) {
            PortableKind::Function(def_map) => {
                let definition = parse_function_definition(name, def_map)?;
                builder.add_definition(DefinitionEnum::UserFunction(definition))?;
            }
            PortableKind::Type(def_map) => {
                let body = parse_type_definition(def_map)?;
                builder.set_user_type_definition(name.clone(), body);
            }
            PortableKind::Context(nested) => {
                let expr = parse_static_object(nested)?;
                builder.add_expression(name, expr)?;
            }
            PortableKind::Expression(raw) => {
                let expr = parse_expression_value(raw)?;
                builder.add_expression(name, expr)?;
            }
        }
    }

    Ok(builder)
}

fn parse_expression_value(value: &Value) -> Result<ExpressionEnum, PortableError> {
    match value {
        Value::Null => Err(PortableError::new(
            "null is not supported in EdgeRules Portable",
        )),
        Value::Bool(flag) => Ok(ExpressionEnum::from(*flag)),
        Value::Number(number) => {
            if let Some(int_value) = number.as_i64() {
                Ok(ExpressionEnum::from(ValueEnum::from(int_value)))
            } else if let Some(float_value) = number.as_f64() {
                Ok(ExpressionEnum::from(ValueEnum::from(float_value)))
            } else {
                Err(PortableError::new("Unsupported numeric literal"))
            }
        }
        Value::String(text) => Ok(EdgeRulesModel::parse_expression(text)?),
        Value::Array(items) => {
            let mut expressions = Vec::with_capacity(items.len());
            for item in items {
                expressions.push(parse_expression_value(item)?);
            }
            Ok(ExpressionEnum::Collection(CollectionExpression::build(
                expressions,
            )))
        }
        Value::Object(map) => parse_static_object(map),
    }
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
    if segments.iter().any(|segment| segment.is_empty()) {
        return Err(PortableError::new(format!("Invalid path '{}'", path)));
    }

    let name = segments.pop().expect("path has at least one segment");
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

fn serialize_builder(builder: &ContextObjectBuilder) -> Result<Value, PortableError> {
    let mut map = Map::new();

    for (name, body) in builder.user_type_entries() {
        map.insert(name, serialize_type_body(&body)?);
    }

    for name in builder.get_field_names() {
        if let Some(expr) = builder.get_expression(name) {
            map.insert(
                name.to_string(),
                serialize_expression(&expr.borrow().expression)?,
            );
            continue;
        }

        if let Some(child) = builder.get_child_context(name) {
            map.insert(
                name.to_string(),
                Value::Object(context_to_map(&child.borrow())?),
            );
            continue;
        }

        if let Some(function) = builder.get_user_function(name) {
            map.insert(
                name.to_string(),
                serialize_function(&function.borrow().function_definition)?,
            );
        }
    }

    Ok(Value::Object(map))
}

fn serialize_expression(expr: &ExpressionEnum) -> Result<Value, PortableError> {
    match expr {
        ExpressionEnum::Value(value) => serialize_value(value),
        ExpressionEnum::StaticObject(ctx) => Ok(Value::Object(context_to_map(&ctx.borrow())?)),
        _ => Ok(Value::String(expr.to_string())),
    }
}

fn serialize_value(value: &ValueEnum) -> Result<Value, PortableError> {
    match value {
        ValueEnum::BooleanValue(flag) => Ok(Value::Bool(*flag)),
        ValueEnum::NumberValue(number) => match number {
            NumberEnum::Int(int_value) => Ok(Value::Number(Number::from(*int_value))),
            NumberEnum::Real(real_value) => Number::from_f64(*real_value)
                .map(Value::Number)
                .ok_or_else(|| PortableError::new("Invalid floating point literal")),
            NumberEnum::SV(_) => Ok(Value::String(value.to_string())),
        },
        ValueEnum::StringValue(_) => Ok(Value::String(value.to_string())),
        ValueEnum::Array(_) | ValueEnum::Reference(_) => Ok(Value::String(value.to_string())),
        _ => Ok(Value::String(value.to_string())),
    }
}

fn serialize_type_body(body: &UserTypeBody) -> Result<Value, PortableError> {
    match body {
        UserTypeBody::TypeRef(reference) => Ok(Value::String(format!("<{}>", reference))),
        UserTypeBody::TypeObject(ctx) => {
            let mut map = context_to_map(&ctx.borrow())?;
            map.insert("@type".to_string(), Value::String("type".to_string()));
            Ok(Value::Object(map))
        }
    }
}

fn serialize_function(definition: &FunctionDefinition) -> Result<Value, PortableError> {
    let mut map = context_to_map(&definition.body.borrow())?;
    map.insert("@type".to_string(), Value::String("function".to_string()));
    if !definition.arguments.is_empty() {
        let mut params = Map::new();
        for param in &definition.arguments {
            params.insert(param.name.clone(), Value::String(param.to_string()));
        }
        map.insert("@parameters".to_string(), Value::Object(params));
    }
    Ok(Value::Object(map))
}

fn context_to_map(ctx: &ContextObject) -> Result<Map<String, Value>, PortableError> {
    let mut map = Map::new();

    for &name in ctx.get_field_names().iter() {
        if let Some(expr) = ctx.expressions.get(name) {
            map.insert(
                name.to_string(),
                serialize_expression(&expr.borrow().expression)?,
            );
            continue;
        }

        if let Some(child) = ctx.node().get_child(name) {
            map.insert(
                name.to_string(),
                Value::Object(context_to_map(&child.borrow())?),
            );
            continue;
        }

        if let Some(function) = ctx.metaphors.get(name) {
            map.insert(
                name.to_string(),
                serialize_function(&function.borrow().function_definition)?,
            );
        }
    }

    for (type_name, body) in &ctx.defined_types {
        map.insert(type_name.clone(), serialize_type_body(body)?);
    }

    Ok(map)
}
