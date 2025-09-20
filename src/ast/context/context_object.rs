use crate::ast::context::context_object_type::{EObjectContent, FormalParameter};
use crate::ast::metaphors::builtin::BuiltinMetaphor;
use crate::ast::token::ExpressionEnum;
use crate::ast::token::{ComplexTypeRef, UserTypeBody};
use crate::ast::Link;
use crate::link::node_data::{ContentHolder, Node, NodeData};
use crate::typesystem::errors::LinkingError;
use crate::typesystem::types::ValueType;
use log::trace;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub struct ExpressionEntry {
    pub expression: ExpressionEnum,
    pub field_type: Link<ValueType>,
}

impl From<ExpressionEnum> for ExpressionEntry {
    fn from(expression: ExpressionEnum) -> Self {
        ExpressionEntry {
            expression,
            field_type: LinkingError::not_linked().into(),
        }
    }
}

impl From<ExpressionEntry> for Rc<RefCell<ExpressionEntry>> {
    fn from(val: ExpressionEntry) -> Self {
        Rc::new(RefCell::new(val))
    }
}

#[derive(Debug)]
pub struct MethodEntry {
    pub metaphor: BuiltinMetaphor,
    pub field_type: Link<ValueType>,
}

impl From<BuiltinMetaphor> for MethodEntry {
    fn from(value: BuiltinMetaphor) -> Self {
        MethodEntry {
            metaphor: value,
            field_type: LinkingError::not_linked().into(),
        }
    }
}

impl From<MethodEntry> for Rc<RefCell<MethodEntry>> {
    fn from(val: MethodEntry) -> Self {
        Rc::new(RefCell::new(val))
    }
}

/// *Main considerations:*
/// - Context Object can have an instance that holds the data into stack: this one is ExecutionContext.
/// - Context Object is a Type itself
#[derive(Debug, Clone)]
pub struct ContextObject {
    /// fields can also referenced by variables in various places in AST. This is why it is Rc.
    pub expressions: HashMap<String, Rc<RefCell<ExpressionEntry>>>,
    /// metaphors are reference counted because they are linked to UserFunctionCall
    pub metaphors: HashMap<String, Rc<RefCell<MethodEntry>>>,
    /// node.childs, expressions and metaphors have names
    pub all_field_names: Vec<String>,
    /// context object can be treated as a function body, so it can have parameters
    pub parameters: Vec<FormalParameter>,

    /// User-defined type aliases within this context
    pub defined_types: HashMap<String, UserTypeBody>,

    pub node: NodeData<ContextObject>,

    pub context_type: Option<ValueType>,
}

impl Node<ContextObject> for ContextObject {
    fn node(&self) -> &NodeData<ContextObject> {
        &self.node
    }

    fn mut_node(&mut self) -> &mut NodeData<ContextObject> {
        &mut self.node
    }
}

impl ContentHolder<ContextObject> for ContextObject {
    /// Technically object content, but additional casting is done:
    /// returned reference is not assigned to the object itself, so it must be done outside
    fn get(&self, name: &str) -> Result<EObjectContent<ContextObject>, LinkingError> {
        trace!("get {}.{}", self.node().node_type, name);
        if let Some(content) = self.expressions.get(name) {
            Ok(EObjectContent::ExpressionRef(Rc::clone(content)))
        } else if let Some(ctx) = self.node().get_child(name) {
            Ok(EObjectContent::ObjectRef(ctx))
        } else if let Some(content) = self.metaphors.get(name) {
            Ok(EObjectContent::MetaphorRef(Rc::clone(content)))
        } else if let Some(parameter) = self.parameters.iter().find(|p| p.name == name) {
            Ok(EObjectContent::Definition(parameter.value_type.clone()))
        } else {
            LinkingError::field_not_found(self.node.node_type.to_string().as_str(), name).into()
        }
    }

    fn get_field_names(&self) -> Vec<String> {
        self.get_field_names()
    }
}

// @Todo: must evaluate types as well
impl PartialEq for ContextObject {
    fn eq(&self, other: &Self) -> bool {
        self.node() == other.node() && self.all_field_names == other.all_field_names
    }
}

impl Display for ContextObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        ContextObject::print_object(self, f)
    }
}

impl ContextObject {
    pub fn into_rc(self) -> Rc<RefCell<ContextObject>> {
        Rc::new(RefCell::new(self))
    }

    pub fn get_field_names(&self) -> Vec<String> {
        self.all_field_names.clone()
    }

    pub fn size(&self) -> usize {
        self.all_field_names.len()
    }

    pub fn get_function(&self, name: &str) -> Option<Rc<RefCell<MethodEntry>>> {
        Some(Rc::clone(self.metaphors.get(name)?))
    }

    pub fn resolve_type_ref(&self, tref: &ComplexTypeRef) -> Result<ValueType, LinkingError> {
        match tref {
            ComplexTypeRef::Primitive(vt) => Ok(vt.clone()),
            ComplexTypeRef::Alias(name) => {
                // walk up parents if not found locally
                let mut cur: Option<Rc<RefCell<ContextObject>>> = None;
                // Simple closure to lookup in a given context
                let mut lookup = |ctx: &ContextObject| -> Option<ValueType> {
                    ctx.defined_types.get(name).map(|def| match def {
                        UserTypeBody::TypeRef(inner) => ctx.resolve_type_ref(inner).unwrap_or(ValueType::UndefinedType),
                        UserTypeBody::TypeObject(obj) => ValueType::ObjectType(Rc::clone(obj)),
                    })
                };

                if let Some(vt) = lookup(self) {
                    return Ok(vt);
                }

                cur = self.node().node_type.get_parent();
                while let Some(parent) = cur {
                    if let Some(vt) = lookup(&parent.borrow()) {
                        return Ok(vt);
                    }
                    cur = parent.borrow().node().node_type.get_parent();
                }

                LinkingError::other_error(format!("Unknown type '{}'", name)).into()
            }
            ComplexTypeRef::List(inner) => Ok(ValueType::ListType(Box::new(self.resolve_type_ref(inner)?))),
        }
    }

    pub fn to_type_string(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        for name in self.all_field_names.iter() {
            let content = self.get(name).unwrap();
            match content {
                EObjectContent::ExpressionRef(entry) => match &entry.borrow().field_type {
                    Ok(field_type) => {
                        lines.push(format!("{}: {}", name, field_type));
                    }
                    Err(err) => {
                        lines.push(format!("{}: {}", name, err));
                    }
                },
                EObjectContent::ObjectRef(entry) => {
                    lines.push(format!("{}: {}", name, entry.borrow().to_type_string()));
                }
                _ => {}
            }
        }

        format!("Type<{}>", lines.join(", "))
    }
}

// ---
// Context object test cases
// ---
#[cfg(test)]
pub mod test {
    use log::info;
    use std::rc::Rc;

    use crate::ast::context::context_object_builder::ContextObjectBuilder;
    use crate::ast::metaphors::functions::FunctionDefinition;
    use crate::ast::token::DefinitionEnum;
    use crate::ast::token::ExpressionEnum;
    use crate::link::linker::{get_till_root, link_parts};
    use crate::link::node_data::ContentHolder;
    use crate::runtime::edge_rules::{expr, EvalError};

    use crate::utils::test::init_logger;

    type E = ExpressionEnum;

    #[test]
    fn test_nesting() -> Result<(), EvalError> {
        init_logger();

        info!(">>> test_nesting()");

        let mut builder = ContextObjectBuilder::new();
        builder.add_expression("a", E::from(1.0));
        builder.add_expression("b", E::from(2.0));

        let child_instance;

        {
            let mut child = ContextObjectBuilder::new();
            child.add_expression("x", E::from("Hello"));
            child.add_expression("y", expr("a + b")?);
            child.add_definition(DefinitionEnum::Metaphor(
                FunctionDefinition::build(
                    vec![],
                    "income".to_string(),
                    vec![],
                    ContextObjectBuilder::new().build(),
                )
                .into(),
            ));
            let instance = child.build();
            child_instance = Rc::clone(&instance);
            builder.add_expression("c", ExpressionEnum::StaticObject(instance));
        }

        let ctx = builder.build();

        link_parts(Rc::clone(&ctx))?;

        assert_eq!(
            ctx.borrow().to_string(),
            "{a : 1; b : 2; c : {x : 'Hello'; y : a + b; income() : {}}}"
        );
        assert_eq!(
            ctx.borrow().to_type_string(),
            "Type<a: number, b: number, c: Type<x: string, y: number>>"
        );

        assert_eq!(ctx.borrow().get("a")?.to_string(), "1");
        assert_eq!(ctx.borrow().get("b")?.to_string(), "2");
        assert!(ctx.borrow().get("x").is_err());
        assert_eq!(
            ctx.borrow().get("c")?.to_string(),
            "{x : 'Hello'; y : a + b; income() : {}}"
        );

        assert_eq!(
            get_till_root(Rc::clone(&ctx), "a")
                .unwrap()
                .content
                .to_string(),
            "1"
        );
        assert_eq!(
            get_till_root(Rc::clone(&child_instance), "a")
                .unwrap()
                .content
                .to_string(),
            "1"
        );
        assert_eq!(
            get_till_root(Rc::clone(&child_instance), "x")
                .unwrap()
                .content
                .to_string(),
            "'Hello'"
        );

        info!(">>> test_nesting() linking");

        Ok(())
    }
}
