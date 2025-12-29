use edge_rules::ast::context::context_object::ContextObject;
use edge_rules::ast::context::context_object_type::EObjectContent;
use edge_rules::ast::context::context_object_type::FormalParameter;
use edge_rules::ast::context::function_context::RETURN_EXPRESSION;
use edge_rules::ast::expression::EvaluatableExpression;
use edge_rules::ast::foreach::ForFunction;
use edge_rules::ast::functions::function_types::{BinaryFunction, MultiFunction, UnaryFunction};
use edge_rules::ast::ifthenelse::IfThenElseFunction;
use edge_rules::ast::operators::comparators::{ComparatorEnum, ComparatorOperator};
use edge_rules::ast::operators::logical_operators::{LogicalOperator, LogicalOperatorEnum};
use edge_rules::ast::operators::math_operators::{
    MathOperator, MathOperatorEnum, NegationOperator,
};
use edge_rules::ast::selections::{ExpressionFilter, FieldSelection};
use edge_rules::ast::sequence::CollectionExpression;
use edge_rules::ast::token::ExpressionEnum;
use edge_rules::ast::user_function_call::UserFunctionCall;
use edge_rules::ast::variable::VariableLink;
use edge_rules::link::node_data::{ContentHolder, Node};
use edge_rules::runtime::edge_rules::EdgeRulesModel;
use edge_rules::runtime::execution_context::ExecutionContext;
use edge_rules::typesystem::types::number::NumberEnum;
use edge_rules::typesystem::types::string::StringEnum;
use edge_rules::typesystem::values::{ArrayValue, ValueEnum};
use std::any::Any;

pub trait ToJs {
    fn to_js(&self) -> String;
}

fn escape_js_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\'' => escaped.push_str("\\'"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{2028}' => escaped.push_str("\\u2028"),
            '\u{2029}' => escaped.push_str("\\u2029"),
            c if c.is_control() => escaped.push_str(&format!("\\x{:02X}", c as u32)),
            c => escaped.push(c),
        }
    }
    escaped
}

fn quote_str(value: &str) -> String {
    format!("\"{}\"", escape_js_string(value))
}

fn quote_key(value: &str) -> String {
    quote_str(value)
}

fn sanitize_identifier(name: &str) -> String {
    let mut out = String::new();
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "_ctx".to_string()
    } else {
        out
    }
}

fn resolve_in_scopes(name: &str, scope: Option<&str>, fallback_scope: Option<&str>) -> String {
    let key = quote_key(name);
    let mut parts = Vec::new();
    if let Some(scope_name) = scope {
        parts.push(format!("{scope_name}?.[{key}]"));
    }
    if let Some(fallback) = fallback_scope {
        parts.push(format!("{fallback}?.[{key}]"));
    }
    parts.push(format!("globalThis?.[{key}]"));
    format!("({})", parts.join(" ?? "))
}

fn render_number(number: &NumberEnum) -> String {
    match number {
        NumberEnum::Int(int) => int.to_string(),
        NumberEnum::Real(real) => real.to_string(),
        NumberEnum::SV(sv) => quote_str(&sv.to_string()),
    }
}

fn render_string_enum(value: &StringEnum) -> String {
    match value {
        StringEnum::String(value) => quote_str(value),
        StringEnum::Char(ch) => quote_str(&ch.to_string()),
        StringEnum::SV(sv) => quote_str(&sv.to_string()),
    }
}

fn render_value(value: &ValueEnum, scope: Option<&str>, fallback_scope: Option<&str>) -> String {
    match value {
        ValueEnum::NumberValue(num) => render_number(num),
        ValueEnum::BooleanValue(flag) => flag.to_string(),
        ValueEnum::StringValue(s) => render_string_enum(s),
        ValueEnum::DateValue(v) => quote_str(&ValueEnum::DateValue(v.clone()).to_string()),
        ValueEnum::TimeValue(v) => quote_str(&ValueEnum::TimeValue(v.clone()).to_string()),
        ValueEnum::DateTimeValue(v) => quote_str(&ValueEnum::DateTimeValue(v.clone()).to_string()),
        ValueEnum::DurationValue(v) => quote_str(&ValueEnum::DurationValue(v.clone()).to_string()),
        ValueEnum::PeriodValue(v) => quote_str(&ValueEnum::PeriodValue(v.clone()).to_string()),
        ValueEnum::Array(array) => render_array(array, scope, fallback_scope),
        ValueEnum::Reference(ctx) => render_execution_context(&ctx.borrow(), None, None),
        ValueEnum::RangeValue(range) => {
            format!(
                "({{start: {}, end: {}}})",
                range.start,
                range.end.saturating_sub(1)
            )
        }
        ValueEnum::TypeValue(value_type) => quote_str(&value_type.to_string()),
    }
}

fn render_array(array: &ArrayValue, scope: Option<&str>, fallback_scope: Option<&str>) -> String {
    match array {
        ArrayValue::EmptyUntyped => "[]".to_string(),
        ArrayValue::PrimitivesArray { values, .. } => {
            let mut parts = Vec::with_capacity(values.len());
            for value in values {
                parts.push(render_value(value, scope, fallback_scope));
            }
            format!("[{}]", parts.join(", "))
        }
        ArrayValue::ObjectsArray { values, .. } => {
            let mut parts = Vec::with_capacity(values.len());
            for (idx, ctx) in values.iter().enumerate() {
                let scope_name = format!("obj{}", idx);
                parts.push(render_execution_context(
                    &ctx.borrow(),
                    Some(&scope_name),
                    fallback_scope,
                ));
            }
            format!("[{}]", parts.join(", "))
        }
    }
}

fn render_variable(
    variable: &VariableLink,
    scope: Option<&str>,
    fallback_scope: Option<&str>,
) -> String {
    let key = variable
        .path
        .iter()
        .map(|segment| quote_key(segment))
        .collect::<Vec<_>>();

    let build_path = |root: &str| {
        let mut path = root.to_string();
        for segment in &key {
            path.push_str("?.[");
            path.push_str(segment);
            path.push(']');
        }
        path
    };

    let mut candidates = Vec::new();
    if let Some(scope_name) = scope {
        candidates.push(build_path(scope_name));
    }
    if let Some(fallback) = fallback_scope {
        candidates.push(build_path(fallback));
    }
    candidates.push(build_path("globalThis"));

    format!("({})", candidates.join(" ?? "))
}

fn render_math_operator(
    op: &MathOperator,
    scope: Option<&str>,
    fallback_scope: Option<&str>,
) -> Option<String> {
    let symbol = match op.data.operator {
        MathOperatorEnum::Addition => "+",
        MathOperatorEnum::Subtraction => "-",
        MathOperatorEnum::Multiplication => "*",
        MathOperatorEnum::Division => "/",
        MathOperatorEnum::Power => "**",
        MathOperatorEnum::Modulus => "%",
    };
    let left = render_expression(&op.data.left, scope, fallback_scope);
    let right = render_expression(&op.data.right, scope, fallback_scope);
    Some(format!("({} {} {})", left, symbol, right))
}

fn render_comparator(
    op: &ComparatorOperator,
    scope: Option<&str>,
    fallback_scope: Option<&str>,
) -> Option<String> {
    let symbol = match op.data.operator {
        ComparatorEnum::Equals => "===",
        ComparatorEnum::NotEquals => "!==",
        ComparatorEnum::Less => "<",
        ComparatorEnum::Greater => ">",
        ComparatorEnum::LessEquals => "<=",
        ComparatorEnum::GreaterEquals => ">=",
    };
    let left = render_expression(&op.data.left, scope, fallback_scope);
    let right = render_expression(&op.data.right, scope, fallback_scope);
    Some(format!("({} {} {})", left, symbol, right))
}

fn render_logical(
    op: &LogicalOperator,
    scope: Option<&str>,
    fallback_scope: Option<&str>,
) -> Option<String> {
    let symbol = match op.data.operator {
        LogicalOperatorEnum::And => "&&",
        LogicalOperatorEnum::Or => "||",
        LogicalOperatorEnum::Xor => "^",
        LogicalOperatorEnum::Not => "!",
    };

    let left = render_expression(&op.data.left, scope, fallback_scope);
    let right = render_expression(&op.data.right, scope, fallback_scope);

    if matches!(op.data.operator, LogicalOperatorEnum::Not) {
        Some(format!("(!{})", right))
    } else {
        Some(format!("({} {} {})", left, symbol, right))
    }
}

fn render_function_call(
    expr: &dyn EvaluatableExpression,
    scope: Option<&str>,
    fallback_scope: Option<&str>,
) -> Option<String> {
    let any_ref = expr as &dyn Any;

    if let Some(call) = any_ref.downcast_ref::<UserFunctionCall>() {
        let mut args = Vec::with_capacity(call.args.len());
        for arg in &call.args {
            args.push(render_expression(arg, scope, fallback_scope));
        }
        let target = resolve_in_scopes(call.name.as_str(), scope, fallback_scope);
        return Some(format!("{}({})", target, args.join(", ")));
    }

    if let Some(binary) = any_ref.downcast_ref::<BinaryFunction>() {
        let left = render_expression(&binary.left, scope, fallback_scope);
        let right = render_expression(&binary.right, scope, fallback_scope);
        return Some(format!("{}({}, {})", binary.definition.name, left, right));
    }

    if let Some(unary) = any_ref.downcast_ref::<UnaryFunction>() {
        let arg = render_expression(&unary.arg, scope, fallback_scope);
        return Some(format!("{}({})", unary.definition.name, arg));
    }

    if let Some(multi) = any_ref.downcast_ref::<MultiFunction>() {
        let mut args = Vec::with_capacity(multi.args.len());
        for arg in &multi.args {
            args.push(render_expression(arg, scope, fallback_scope));
        }
        return Some(format!("{}({})", multi.definition.name, args.join(", ")));
    }

    if let Some(ifelse) = any_ref.downcast_ref::<IfThenElseFunction>() {
        let condition = render_expression(&ifelse.condition, scope, fallback_scope);
        let then_js = render_expression(&ifelse.then_expression, scope, fallback_scope);
        let else_js = render_expression(&ifelse.else_expression, scope, fallback_scope);
        return Some(format!("({} ? {} : {})", condition, then_js, else_js));
    }

    if let Some(for_fn) = any_ref.downcast_ref::<ForFunction>() {
        return Some(render_for_function(for_fn, scope, fallback_scope));
    }

    None
}

fn render_expression(
    expr: &ExpressionEnum,
    scope: Option<&str>,
    fallback_scope: Option<&str>,
) -> String {
    match expr {
        ExpressionEnum::Value(value) => render_value(value, scope, fallback_scope),
        ExpressionEnum::Variable(var) => render_variable(var, scope, fallback_scope),
        ExpressionEnum::ContextVariable => scope.unwrap_or("it").to_string(),
        ExpressionEnum::Operator(op) => {
            let any_ref = op.as_ref() as &dyn Any;
            if let Some(math) = any_ref.downcast_ref::<MathOperator>() {
                render_math_operator(math, scope, fallback_scope)
            } else if let Some(comparator) = any_ref.downcast_ref::<ComparatorOperator>() {
                render_comparator(comparator, scope, fallback_scope)
            } else if let Some(logical) = any_ref.downcast_ref::<LogicalOperator>() {
                render_logical(logical, scope, fallback_scope)
            } else if let Some(negation) = any_ref.downcast_ref::<NegationOperator>() {
                let left = render_expression(&negation.left, scope, fallback_scope);
                Some(format!("(-{})", left))
            } else {
                None
            }
            .unwrap_or_else(|| quote_str(&expr.to_string()))
        }
        ExpressionEnum::FunctionCall(func) => {
            render_function_call(func.as_ref(), scope, fallback_scope)
                .unwrap_or_else(|| quote_str(&expr.to_string()))
        }
        ExpressionEnum::Selection(selection) => render_selection(selection, scope, fallback_scope),
        ExpressionEnum::Filter(filter) => render_filter(filter, scope, fallback_scope),
        ExpressionEnum::Collection(collection) => {
            render_collection(collection, scope, fallback_scope)
        }
        ExpressionEnum::RangeExpression(left, right) => {
            format!(
                "({{start: {}, end: {}}})",
                render_expression(left, scope, fallback_scope),
                render_expression(right, scope, fallback_scope)
            )
        }
        ExpressionEnum::StaticObject(obj) => render_context_object(&obj.borrow(), "ctx", None),
        ExpressionEnum::ObjectField(name, right) => {
            format!(
                "({{{}: {}}})",
                quote_key(name),
                render_expression(right, scope, fallback_scope)
            )
        }
        ExpressionEnum::TypePlaceholder(tref) => quote_str(&format!("<{}>", tref)),
    }
}

fn render_collection(
    collection: &CollectionExpression,
    scope: Option<&str>,
    fallback_scope: Option<&str>,
) -> String {
    let mut parts = Vec::with_capacity(collection.elements.len());
    for element in &collection.elements {
        parts.push(render_expression(element, scope, fallback_scope));
    }
    format!("[{}]", parts.join(", "))
}

fn render_filter(
    filter: &ExpressionFilter,
    scope: Option<&str>,
    fallback_scope: Option<&str>,
) -> String {
    let source_js = render_expression(&filter.source, scope, fallback_scope);
    let method_js = render_expression(&filter.method, Some("it"), scope);
    format!(
        concat!(
            "(() => {{\n",
            "    const source = {};\n",
            "    if (!Array.isArray(source)) {{ return source; }}\n",
            "    if (source.length === 0) {{ return []; }}\n",
            "    const compute = (it, index) => ({});\n",
            "    const probe = compute(source[0], 0);\n",
            "    if (typeof probe === \"number\") {{\n",
            "        const idx = Math.trunc(probe);\n",
            "        return idx >= 0 && idx < source.length ? source[idx] : undefined;\n",
            "    }}\n",
            "    return source.filter((item, index) => !!compute(item, index));\n",
            "}})()"
        ),
        source_js, method_js
    )
}

fn render_for_function(
    for_fn: &ForFunction,
    scope: Option<&str>,
    fallback_scope: Option<&str>,
) -> String {
    let source_js = render_expression(&for_fn.in_expression, scope, fallback_scope);
    let loop_scope = format!("loop_{}", sanitize_identifier(&for_fn.in_loop_variable));
    let return_js = {
        let ret_ctx = for_fn.return_expression.borrow();
        let return_entry = ret_ctx
            .expressions
            .get(RETURN_EXPRESSION)
            .expect("return expression must exist")
            .borrow();
        render_expression(
            &return_entry.expression,
            Some(&loop_scope),
            scope.or(fallback_scope),
        )
    };
    let loop_var = quote_key(&for_fn.in_loop_variable);
    format!(
        concat!(
            "(() => {{\n",
            "    const source = {};\n",
            "    if (Array.isArray(source)) {{\n",
            "        return source.map((it, index) => {{\n",
            "            const {} = {{}};\n",
            "            {}[{}] = it;\n",
            "            return {};\n",
            "        }});\n",
            "    }}\n",
            "    if (source && typeof source.start === \"number\" && typeof source.end === \"number\") {{\n",
            "        const out = [];\n",
            "        for (let i = source.start; i <= source.end; i++) {{\n",
            "            const {} = {{}};\n",
            "            {}[{}] = i;\n",
            "            out.push({});\n",
            "        }}\n",
            "        return out;\n",
            "    }}\n",
            "    return source;\n",
            "}})()"
        ),
        source_js,
        loop_scope,
        loop_scope,
        loop_var,
        return_js,
        loop_scope,
        loop_scope,
        loop_var,
        return_js
    )
}

fn render_selection(
    selection: &FieldSelection,
    scope: Option<&str>,
    fallback_scope: Option<&str>,
) -> String {
    let source_js = render_expression(&selection.source, scope, fallback_scope);
    let mut accessors = String::new();
    for segment in &selection.method.path {
        accessors.push_str("?.[");
        accessors.push_str(&quote_key(segment));
        accessors.push(']');
    }
    format!(
        "(() => {{\n    const source = {};\n    return source{};\n}})()",
        source_js, accessors
    )
}

fn render_function_definition_args(args: &[FormalParameter]) -> String {
    let mut params = Vec::with_capacity(args.len());
    for arg in args {
        params.push(quote_key(arg.name.as_str()));
    }
    format!("[{}]", params.join(", "))
}

fn render_context_object(obj: &ContextObject, scope: &str, parent_scope: Option<&str>) -> String {
    let mut lines = Vec::new();
    lines.push(format!("const {} = {{}};", scope));

    for name in obj.get_field_names() {
        if let Some(expr_entry) = obj.expressions.get(name) {
            let expr_js =
                render_expression(&expr_entry.borrow().expression, Some(scope), parent_scope);
            lines.push(format!("{}[{}] = {};", scope, quote_key(name), expr_js));
            continue;
        }

        if let Some(child) = obj.node().get_child(name) {
            let nested_scope = format!("{}_{}", scope, sanitize_identifier(name));
            let nested_js = render_context_object(&child.borrow(), &nested_scope, Some(scope));
            lines.push(format!("const {} = {};", nested_scope, nested_js));
            lines.push(format!(
                "{}[{}] = {};",
                scope,
                quote_key(name),
                nested_scope
            ));
            continue;
        }

        if let Some(method) = obj.metaphors.get(name) {
            let def = &method.borrow().function_definition;
            let args_js = render_function_definition_args(&def.arguments);
            let body_js = render_context_object(
                &def.body.borrow(),
                &format!("{}_{}", scope, "fn"),
                Some(scope),
            );
            lines.push(format!(
                "{}[{}] = {{ name: {}, args: {}, body: {} }};",
                scope,
                quote_key(name),
                quote_str(def.name.as_str()),
                args_js,
                body_js
            ));
            continue;
        }
    }

    lines.push(format!("return {};", scope));
    format!("(() => {{\n    {}\n}})()", lines.join("\n    "))
}

fn render_execution_context(
    ctx: &ExecutionContext,
    scope: Option<&str>,
    parent_scope: Option<&str>,
) -> String {
    let scope_name = scope.unwrap_or("ctx");
    let mut lines = Vec::new();
    lines.push(format!("const {} = {{}};", scope_name));

    for name in ctx.get_field_names() {
        match ctx.get(name) {
            Ok(EObjectContent::ConstantValue(value)) => {
                let value_js = render_value(&value, Some(scope_name), parent_scope);
                lines.push(format!(
                    "{}[{}] = {};",
                    scope_name,
                    quote_key(name),
                    value_js
                ));
            }
            Ok(EObjectContent::ExpressionRef(expr)) => {
                let expr_js =
                    render_expression(&expr.borrow().expression, Some(scope_name), parent_scope);
                lines.push(format!(
                    "{}[{}] = {};",
                    scope_name,
                    quote_key(name),
                    expr_js
                ));
            }
            Ok(EObjectContent::UserFunctionRef(method)) => {
                let def = &method.borrow().function_definition;
                let fn_var = format!(
                    "{}_fn_{}",
                    scope_name,
                    sanitize_identifier(def.name.as_str())
                );
                let arg_scope = format!("{}_args", fn_var);
                let body_scope = format!("{}_body", fn_var);

                lines.push(format!("const {} = (...__args) => {{", fn_var));
                lines.push(format!("    const {} = {{}};", arg_scope));
                for (idx, arg) in def.arguments.iter().enumerate() {
                    let arg_ident = sanitize_identifier(arg.name.as_str());
                    lines.push(format!("    const {} = __args[{}];", arg_ident, idx));
                    lines.push(format!(
                        "    {}[{}] = {};",
                        arg_scope,
                        quote_key(arg.name.as_str()),
                        arg_ident
                    ));
                }
                let body_js =
                    render_context_object(&def.body.borrow(), &body_scope, Some(&arg_scope));
                lines.push(format!("    const body = {};", body_js));
                lines.push("    return body;".to_string());
                lines.push("};".to_string());
                lines.push(format!("{}[{}] = {};", scope_name, quote_key(name), fn_var));
                lines.push(format!("globalThis[{}] = {};", quote_key(name), fn_var));
            }
            Ok(EObjectContent::ObjectRef(obj)) => {
                let nested_scope = format!("{}_{}", scope_name, sanitize_identifier(name));
                let nested_js =
                    render_execution_context(&obj.borrow(), Some(&nested_scope), Some(scope_name));
                lines.push(format!("const {} = {};", nested_scope, nested_js));
                lines.push(format!(
                    "{}[{}] = {};",
                    scope_name,
                    quote_key(name),
                    nested_scope
                ));
            }
            Ok(EObjectContent::Definition(definition)) => {
                lines.push(format!(
                    "{}[{}] = {{ $type: {} }};",
                    scope_name,
                    quote_key(name),
                    quote_str(&definition.to_string())
                ));
            }
            Err(err) => {
                lines.push(format!(
                    "{}[{}] = (() => {{ throw new Error({}); }})();",
                    scope_name,
                    quote_key(name),
                    quote_str(&err.to_string())
                ));
            }
        }
    }

    lines.push(format!("return {};", scope_name));
    format!("(() => {{\n    {}\n}})()", lines.join("\n    "))
}

impl ToJs for ValueEnum {
    fn to_js(&self) -> String {
        render_value(self, None, None)
    }
}

impl ToJs for ExpressionEnum {
    fn to_js(&self) -> String {
        render_expression(self, None, None)
    }
}

impl ToJs for ContextObject {
    fn to_js(&self) -> String {
        render_context_object(self, "ctx", None)
    }
}

impl ToJs for ExecutionContext {
    fn to_js(&self) -> String {
        render_execution_context(self, None, None)
    }
}

impl ToJs for ArrayValue {
    fn to_js(&self) -> String {
        render_array(self, None, None)
    }
}

impl ToJs for VariableLink {
    fn to_js(&self) -> String {
        render_variable(self, None, None)
    }
}

impl ToJs for CollectionExpression {
    fn to_js(&self) -> String {
        render_collection(self, None, None)
    }
}

impl ToJs for ExpressionFilter {
    fn to_js(&self) -> String {
        render_filter(self, None, None)
    }
}

impl ToJs for FieldSelection {
    fn to_js(&self) -> String {
        render_selection(self, None, None)
    }
}

pub fn to_js_model(model: &mut EdgeRulesModel) -> Result<String, String> {
    let runtime = model.to_runtime_snapshot().map_err(|err| err.to_string())?;
    let js_model = {
        let ctx_ref = runtime.context.borrow();
        render_execution_context(&ctx_ref, None, None)
    };
    Ok(js_model)
}

pub fn to_js_expression(expr: &ExpressionEnum) -> String {
    render_expression(expr, None, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use edge_rules::runtime::edge_rules::EdgeRulesModel;
    use edge_rules::typesystem::types::number::NumberEnum;
    use edge_rules::typesystem::values::ValueEnum;

    #[test]
    fn renders_primitives() {
        assert_eq!(ValueEnum::NumberValue(NumberEnum::from(5)).to_js(), "5");
        assert_eq!(
            ValueEnum::StringValue(StringEnum::String("hi\"there".into())).to_js(),
            "\"hi\\\"there\""
        );
    }

    #[test]
    fn renders_math_expression() {
        let expr = EdgeRulesModel::parse_expression("2 + 3").expect("parse expression");
        let js = expr.to_js();
        assert!(js.contains("+"));
    }

    #[test]
    fn renders_filter_expression() {
        let expr =
            EdgeRulesModel::parse_expression("[1,2,3][...>1]").expect("parse filter expression");
        let js = expr.to_js();
        assert!(js.contains("Array.isArray(source)"));
        assert!(js.contains("filter("));
    }

    #[test]
    fn renders_selection_expression() {
        let expr =
            EdgeRulesModel::parse_expression("[1,2,3][...>1].length").expect("parse selection");
        let js = expr.to_js();
        assert!(js.contains("return source"));
        assert!(js.contains("?.[\"length\"]"));
    }

    #[test]
    fn escapes_control_characters() {
        let expr = EdgeRulesModel::parse_expression("\"hi\nworld\"").expect("parse expression");
        assert_eq!(expr.to_js(), "\"hi\\nworld\"");
    }

    #[test]
    fn renders_nested_context_object() {
        let mut model = EdgeRulesModel::new();
        model
            .append_source(
                r#"
                {
                    user: { name: "Dana"; age: 30 }
                    label: user.name
                    nextYear: user.age + 1
                }
                "#,
            )
            .expect("parse model");
        let js = to_js_model(&mut model).expect("to js model");
        assert!(js.contains("user"));
        assert!(js.contains("nextYear"));
        assert!(js.contains("ctx_user"));
        assert!(js.contains("ctx_user[\"age\"]"));
    }

    #[test]
    fn renders_array_and_selection_nodes() {
        let expr =
            EdgeRulesModel::parse_expression("[{a: 1}][0].a").expect("parse collection selection");
        assert!(expr.to_js().contains("source.filter"));

        if let ExpressionEnum::Selection(selection) = &expr {
            assert!(selection.to_js().contains("return source"));
        }
    }

    #[test]
    fn renders_filter_node_directly() {
        let expr = EdgeRulesModel::parse_expression("[1,2,3][...>1]").expect("parse filter");
        if let ExpressionEnum::Filter(filter) = &expr {
            let js = filter.to_js();
            assert!(js.contains("Array.isArray(source)"));
            assert!(js.contains("compute("));
        }
    }
}
