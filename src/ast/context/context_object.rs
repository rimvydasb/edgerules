use crate::ast::context::context_object_type::{EObjectContent, FormalParameter};
use crate::ast::context::duplicate_name_error::{DuplicateNameError, NameKind};
use crate::ast::metaphors::functions::FunctionDefinition;
use crate::ast::token::ExpressionEnum;
use crate::ast::token::{ComplexTypeRef, UserTypeBody};
use crate::ast::Link;
use crate::link::linker;
use crate::link::node_data::{ContentHolder, Node, NodeData, NodeDataEnum};
use crate::typesystem::errors::LinkingError;
use crate::typesystem::types::{ToSchema, ValueType};
use crate::utils::intern_field_name;
use log::trace;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
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
    pub function_definition: FunctionDefinition,
    pub field_type: Link<ValueType>,
}

impl From<FunctionDefinition> for MethodEntry {
    fn from(value: FunctionDefinition) -> Self {
        MethodEntry {
            function_definition: value,
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
    /// fields can also be referenced by variables in various places in AST. This is why it is Rc.
    pub expressions: HashMap<&'static str, Rc<RefCell<ExpressionEntry>>>,
    /// metaphors are reference counted because they are linked to UserFunctionCall
    pub metaphors: HashMap<&'static str, Rc<RefCell<MethodEntry>>>,
    /// node.childs, expressions and metaphors have names
    pub all_field_names: Vec<&'static str>,
    pub field_name_set: HashSet<&'static str>,
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
            Ok(EObjectContent::UserFunctionRef(Rc::clone(content)))
        } else if let Some(parameter) = self.parameters.iter().find(|p| p.name == name) {
            Ok(EObjectContent::Definition(parameter.value_type.clone()))
        } else {
            LinkingError::field_not_found(self.node.node_type.to_string().as_str(), name).into()
        }
    }

    fn get_field_names(&self) -> Vec<&'static str> {
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

    pub fn get_field_names(&self) -> Vec<&'static str> {
        self.all_field_names.clone()
    }

    pub fn add_field_name(&mut self, field_name: &'static str) {
        if self.field_name_set.contains(field_name) {
            return;
        }

        self.field_name_set.insert(field_name);
        self.all_field_names.push(field_name);
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
                let resolve_in = |ctx: &ContextObject| -> Link<Option<ValueType>> {
                    if let Some(def) = ctx.defined_types.get(name) {
                        let vt = match def {
                            UserTypeBody::TypeRef(inner) => ctx.resolve_type_ref(inner)?,
                            UserTypeBody::TypeObject(obj) => {
                                linker::link_parts(Rc::clone(obj))?;
                                ValueType::ObjectType(Rc::clone(obj))
                            }
                        };
                        Ok(Some(vt))
                    } else {
                        Ok(None)
                    }
                };

                if let Some(vt) = resolve_in(self)? {
                    return Ok(vt);
                }

                let mut cur = self.node().node_type.get_parent();
                while let Some(parent) = cur {
                    if let Some(vt) = resolve_in(&parent.borrow())? {
                        return Ok(vt);
                    }
                    cur = parent.borrow().node().node_type.get_parent();
                }

                LinkingError::other_error(format!("Unknown type '{}'", name)).into()
            }
            ComplexTypeRef::List(inner) => Ok(ValueType::ListType(Some(Box::new(
                self.resolve_type_ref(inner)?,
            )))),
        }
    }

    fn alias_in_map(
        map: &HashMap<String, UserTypeBody>,
        target: &Rc<RefCell<ContextObject>>,
    ) -> Option<String> {
        map.iter().find_map(|(name, body)| match body {
            UserTypeBody::TypeObject(obj) if Rc::ptr_eq(obj, target) => Some(name.clone()),
            _ => None,
        })
    }

    fn find_alias_for_object(&self, target: &Rc<RefCell<ContextObject>>) -> Option<String> {
        if let Some(name) = Self::alias_in_map(&self.defined_types, target) {
            return Some(name);
        }

        let mut current = self.node().node_type.get_parent();
        while let Some(parent_rc) = current {
            let (alias, next_parent) = {
                let parent = parent_rc.borrow();
                let found = Self::alias_in_map(&parent.defined_types, target);
                let next = parent.node().node_type.get_parent();
                (found, next)
            };
            if let Some(name) = alias {
                return Some(name);
            }
            current = next_parent;
        }

        None
    }

    fn format_value_type(&self, value_type: &ValueType) -> String {
        match value_type {
            ValueType::ObjectType(obj) => self
                .find_alias_for_object(obj)
                .unwrap_or_else(|| obj.borrow().to_schema()),
            ValueType::ListType(Some(inner)) => {
                format!("{}[]", self.format_value_type(inner.as_ref()))
            }
            ValueType::ListType(None) => "[]".to_string(),
            _ => value_type.to_string(),
        }
    }

    pub fn remove_field(&mut self, field_name: &str) -> bool {
        let Some(&interned) = self.field_name_set.get(field_name) else {
            return false;
        };

        self.field_name_set.remove(interned);
        self.all_field_names.retain(|&field| field != interned);
        self.expressions.remove(interned);
        self.metaphors.remove(interned);
        self.node().get_childs().borrow_mut().remove(interned);

        true
    }

    pub fn add_expression_field(
        parent: &Rc<RefCell<ContextObject>>,
        field_name: &str,
        expression: ExpressionEnum,
    ) -> Result<(), DuplicateNameError> {
        let interned = intern_field_name(field_name);
        match expression {
            ExpressionEnum::StaticObject(obj) => {
                {
                    let mut parent_mut = parent.borrow_mut();
                    parent_mut.insert_field_name(interned, NameKind::Field)?;
                    parent_mut.node().add_child(interned, Rc::clone(&obj));
                }
                obj.borrow_mut().mut_node().node_type =
                    NodeDataEnum::Child(interned, Rc::downgrade(parent));
                Ok(())
            }
            other => {
                let mut parent_mut = parent.borrow_mut();
                parent_mut.insert_field_name(interned, NameKind::Field)?;
                parent_mut
                    .expressions
                    .insert(interned, ExpressionEntry::from(other).into());
                Ok(())
            }
        }
    }

    fn ensure_name_unique(
        &self,
        field_name: &'static str,
        kind: NameKind,
    ) -> Result<(), DuplicateNameError> {
        if self.field_name_set.contains(field_name) {
            return Err(DuplicateNameError::new(kind, field_name));
        }

        Ok(())
    }

    fn insert_field_name(
        &mut self,
        field_name: &'static str,
        kind: NameKind,
    ) -> Result<(), DuplicateNameError> {
        self.ensure_name_unique(field_name, kind)?;
        self.field_name_set.insert(field_name);
        self.all_field_names.push(field_name);
        Ok(())
    }
}

impl ToSchema for ContextObject {
    fn to_schema(&self) -> String {
        let mut lines: Vec<String> = Vec::new();

        let mut type_entries: Vec<_> = self.defined_types.iter().collect();
        type_entries.sort_by(|(left, _), (right, _)| left.cmp(right));

        for (name, body) in type_entries {
            lines.push(format!("{}: {}", name, body.to_schema()));
        }

        for name in self.all_field_names.iter() {
            let content = self.get(name).unwrap();
            match content {
                EObjectContent::ExpressionRef(entry) => {
                    let entry_ref = entry.borrow();
                    match &entry_ref.field_type {
                        Ok(field_type) => {
                            let formatted = self.format_value_type(field_type);
                            lines.push(format!("{}: {}", name, formatted));
                        }
                        Err(err) => lines.push(format!("{}: {}", name, err)),
                    }
                }
                EObjectContent::ObjectRef(entry) => {
                    lines.push(format!("{}: {}", name, entry.borrow().to_schema()));
                }
                _ => {}
            }
        }

        format!("{{{}}}", lines.join("; "))
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
    use crate::typesystem::types::ToSchema;

    use crate::utils::test::init_logger;

    type E = ExpressionEnum;

    #[test]
    fn test_nesting() -> Result<(), EvalError> {
        init_logger();

        info!(">>> test_nesting()");

        let mut builder = ContextObjectBuilder::new();
        builder.add_expression("a", E::from(1.0))?;
        builder.add_expression("b", E::from(2.0))?;

        let child_instance;

        {
            let mut child = ContextObjectBuilder::new();
            child.add_expression("x", E::from("Hello"))?;
            child.add_expression("y", expr("a + b")?)?;
            child.add_definition(DefinitionEnum::UserFunction(FunctionDefinition::build(
                "income".to_string(),
                vec![],
                ContextObjectBuilder::new().build(),
            )?))?;
            let instance = child.build();
            child_instance = Rc::clone(&instance);
            builder.add_expression("c", ExpressionEnum::StaticObject(instance))?;
        }

        let ctx = builder.build();

        link_parts(Rc::clone(&ctx))?;

        assert_eq!(
            ctx.borrow().to_string(),
            "{a: 1; b: 2; c: {x: 'Hello'; y: a + b; income() : {}}}"
        );
        assert_eq!(
            ctx.borrow().to_schema(),
            "{a: number; b: number; c: {x: string; y: number}}"
        );

        assert_eq!(ctx.borrow().get("a")?.to_string(), "1");
        assert_eq!(ctx.borrow().get("b")?.to_string(), "2");
        assert!(ctx.borrow().get("x").is_err());
        assert_eq!(
            ctx.borrow().get("c")?.to_string(),
            "{x: 'Hello'; y: a + b; income() : {}}"
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
