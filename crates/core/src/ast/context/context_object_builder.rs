use crate::ast::context::context_object::{ContextObject, ExpressionEntry, MethodEntry};
use crate::ast::context::context_resolver::resolve_context_path;
use crate::ast::context::context_object_type::FormalParameter;
use crate::ast::context::duplicate_name_error::{DuplicateNameError, NameKind};
use crate::ast::context::metadata::Metadata;
use crate::ast::metaphors::metaphor::UserFunction;
use crate::ast::token::DefinitionEnum::UserFunction as UserFunctionDef;
use crate::ast::token::{DefinitionEnum, ExpressionEnum, UserTypeBody};
use crate::link::node_data::{Node, NodeData, NodeDataEnum};
use crate::typesystem::types::ValueType;
use crate::utils::intern_field_name;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// ---
/// **ContextObjectBuilder**
/// - Builds Execution Context Object and gets dismissed after building.
pub struct ContextObjectBuilder {
    fields: HashMap<&'static str, Rc<RefCell<ExpressionEntry>>>,
    metaphors: HashMap<&'static str, Rc<RefCell<MethodEntry>>>,
    childs: HashMap<&'static str, Rc<RefCell<ContextObject>>>,
    field_names: Vec<&'static str>,
    field_name_set: HashSet<&'static str>,
    context_type: Option<ValueType>,
    parameters: Vec<FormalParameter>,
    node_type: NodeDataEnum<ContextObject>,
    defined_types: HashMap<String, UserTypeBody>,
    metadata: Option<Metadata>,
}

impl Default for ContextObjectBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextObjectBuilder {
    pub fn new() -> Self {
        ContextObjectBuilder {
            fields: HashMap::new(),
            metaphors: HashMap::new(),
            childs: HashMap::new(),
            field_names: Vec::new(),
            field_name_set: HashSet::new(),
            context_type: None,
            node_type: NodeDataEnum::Root(),
            parameters: Vec::new(),
            defined_types: HashMap::new(),
            metadata: None,
        }
    }

    pub fn new_internal(parent: Rc<RefCell<ContextObject>>) -> Self {
        ContextObjectBuilder {
            fields: HashMap::new(),
            metaphors: HashMap::new(),
            childs: HashMap::new(),
            field_names: Vec::new(),
            field_name_set: HashSet::new(),
            context_type: None,
            node_type: NodeDataEnum::Internal(Rc::downgrade(&parent), None),
            parameters: Vec::new(),
            defined_types: HashMap::new(),
            metadata: None,
        }
    }

    pub fn set_parameters(&mut self, parameters: Vec<FormalParameter>) -> &mut Self {
        self.parameters = parameters;
        self
    }

    pub fn set_metadata(&mut self, metadata: Metadata) -> &mut Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn get_metadata(&self) -> Option<&Metadata> {
        self.metadata.as_ref()
    }

    // @Todo: check if field is not duplicated
    // @Todo: optimize by inserting by a number, not a field name
    // @Todo: return an error and propagate it to the top
    pub fn add_expression(
        &mut self,
        field_name: &str,
        field: ExpressionEnum,
    ) -> Result<&mut Self, DuplicateNameError> {
        let field_name = intern_field_name(field_name);
        self.insert_field_name(field_name, NameKind::Field)?;

        if let ExpressionEnum::StaticObject(obj) = &field {
            // No need to assign parent now, it is done later in build
            self.childs.insert(field_name, Rc::clone(obj));
            return Ok(self);
        }

        trace!(">>> inserting field {:?}", field_name);
        self.fields
            .insert(field_name, ExpressionEntry::from(field).into());

        Ok(self)
    }

    pub fn set_expression(
        &mut self,
        field_name: &str,
        expression: ExpressionEnum,
    ) -> Result<(), DuplicateNameError> {
        self.remove_field(field_name);
        self.add_expression(field_name, expression).map(|_| ())
    }

    pub fn add_definition(
        &mut self,
        field: DefinitionEnum,
    ) -> Result<&mut Self, DuplicateNameError> {
        match field {
            UserFunctionDef(m) => {
                let name = m.get_name();
                trace!(">>> inserting function {:?}", name);
                let interned = intern_field_name(name.as_str());
                self.insert_field_name(interned, NameKind::Function)?;
                self.metaphors.insert(interned, MethodEntry::from(m).into());
            }
            DefinitionEnum::UserType(t) => {
                self.insert_type_definition(t.name, t.body)?;
            }
        }
        Ok(self)
    }

    pub fn set_context_type(&mut self, context_type: ValueType) {
        self.context_type = Some(context_type);
    }

    /// Appends another ContextObject into this builder.
    /// - Fails if there are duplicate field names.
    pub fn append(
        &mut self,
        another: Rc<RefCell<ContextObject>>,
    ) -> Result<&mut Self, DuplicateNameError> {
        let borrowed = another.borrow();
        let other_names = borrowed.get_field_names();

        for &name in &other_names {
            let kind = if borrowed.metaphors.contains_key(&name) {
                NameKind::Function
            } else {
                NameKind::Field
            };

            self.ensure_name_unique(name, kind)?;
        }

        let childs_ref = borrowed.node().get_childs();
        let childs_ref = childs_ref.borrow();

        for name in other_names {
            if let Some(field) = borrowed.expressions.get(name) {
                self.insert_field_name(name, NameKind::Field)?;
                self.fields.insert(name, Rc::clone(field));
                continue;
            }

            if let Some(child) = childs_ref.get(name) {
                self.insert_field_name(name, NameKind::Field)?;
                self.childs.insert(name, Rc::clone(child));
                continue;
            }

            if let Some(method) = borrowed.metaphors.get(name) {
                self.insert_field_name(name, NameKind::Function)?;
                self.metaphors.insert(name, Rc::clone(method));
            }
        }

        for (key, value) in borrowed.defined_types.iter() {
            self.insert_type_definition(key.clone(), value.clone())?;
        }

        if self.metadata.is_none() {
            self.metadata = borrowed.metadata.clone();
        }

        Ok(self)
    }

    pub fn merge_context_object(
        &mut self,
        object: Rc<RefCell<ContextObject>>,
    ) -> Result<(), DuplicateNameError> {
        self.append(object).map(|_| ())
    }

    pub fn append_if_missing(
        &mut self,
        another: Rc<RefCell<ContextObject>>,
    ) -> Result<&mut Self, DuplicateNameError> {
        let borrowed = another.borrow();
        let childs_ref = borrowed.node().get_childs();
        let childs_ref = childs_ref.borrow();

        for &name in borrowed.get_field_names().iter() {
            if self.field_name_set.contains(name) {
                continue;
            }

            if borrowed.metaphors.contains_key(&name) {
                self.insert_field_name(name, NameKind::Function)?;
                if let Some(method) = borrowed.metaphors.get(&name) {
                    self.metaphors.insert(name, Rc::clone(method));
                }
                continue;
            }

            self.insert_field_name(name, NameKind::Field)?;

            if let Some(field) = borrowed.expressions.get(name) {
                self.fields.insert(name, Rc::clone(field));
                continue;
            }

            if let Some(child) = childs_ref.get(name) {
                self.childs.insert(name, Rc::clone(child));
                continue;
            }
        }

        for (key, value) in borrowed.defined_types.iter() {
            self.defined_types
                .entry(key.clone())
                .or_insert_with(|| value.clone());
        }

        Ok(self)
    }

    pub fn get_child_context(&self, name: &str) -> Option<Rc<RefCell<ContextObject>>> {
        self.childs.get(name).cloned()
    }

    pub fn get_expression(&self, name: &str) -> Option<Rc<RefCell<ExpressionEntry>>> {
        self.fields.get(name).cloned()
    }

    pub fn get_user_function(&self, name: &str) -> Option<Rc<RefCell<MethodEntry>>> {
        self.metaphors.get(name).cloned()
    }

    pub fn get_user_type(&self, name: &str) -> Option<UserTypeBody> {
        self.defined_types.get(name).cloned()
    }

    pub fn user_type_entries(&self) -> Vec<(String, UserTypeBody)> {
        self.defined_types
            .iter()
            .map(|(name, body)| (name.clone(), body.clone()))
            .collect()
    }

    pub fn set_user_type_definition(&mut self, name: String, body: UserTypeBody) {
        self.defined_types.insert(name, body);
    }

    pub fn remove_user_type_definition(&mut self, name: &str) -> bool {
        self.defined_types.remove(name).is_some()
    }

    pub fn resolve_context(&self, path_segments: &[&str]) -> Option<Rc<RefCell<ContextObject>>> {
        if path_segments.is_empty() {
            return None;
        }

        let current = self.get_child_context(path_segments[0])?;
        resolve_context_path(current, &path_segments[1..])
    }

    pub fn remove_field(&mut self, name: &str) -> bool {
        let Some(&interned) = self.field_name_set.get(name) else {
            return false;
        };

        self.field_name_set.remove(interned);
        self.field_names.retain(|&field| field != interned);
        self.fields.remove(interned);
        self.metaphors.remove(interned);
        self.childs.remove(interned);

        true
    }

    pub fn get_field_names(&self) -> Vec<&'static str> {
        self.field_names.clone()
    }

    pub fn build(self) -> Rc<RefCell<ContextObject>> {
        let obj = ContextObject {
            expressions: self.fields,
            metaphors: self.metaphors,
            all_field_names: self.field_names,
            field_name_set: self.field_name_set,
            node: NodeData::new_fixed(self.childs, self.node_type),
            parameters: self.parameters,
            context_type: self.context_type,
            defined_types: self.defined_types,
            metadata: self.metadata,
        };

        let ctx = Rc::new(RefCell::new(obj));

        {
            let child_map = ctx.borrow().node().get_childs();
            let borrowed = child_map.borrow();
            for (&name, child) in borrowed.iter() {
                child.borrow_mut().node.node_type = NodeDataEnum::Child(name, Rc::downgrade(&ctx));
            }
        }

        {
            let parent = Rc::downgrade(&ctx);
            let borrowed = ctx.borrow();
            for method in borrowed.metaphors.values() {
                let (body, alias) = {
                    let method_ref = method.borrow();
                    (
                        Rc::clone(&method_ref.function_definition.body),
                        intern_field_name(method_ref.function_definition.name.as_str()),
                    )
                };
                body.borrow_mut().node.node_type =
                    NodeDataEnum::Internal(parent.clone(), Some(alias));
            }
        }

        {
            let type_defs = ctx.borrow();
            for (name, body) in type_defs.defined_types.iter() {
                if let UserTypeBody::TypeObject(type_ctx) = body {
                    let alias = intern_field_name(name.as_str());
                    type_ctx.borrow_mut().node.node_type =
                        NodeDataEnum::Internal(Rc::downgrade(&ctx), Some(alias));
                }
            }
        }

        ctx
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
        self.field_names.push(field_name);

        Ok(())
    }

    fn insert_type_definition(
        &mut self,
        name: String,
        body: UserTypeBody,
    ) -> Result<(), DuplicateNameError> {
        if self.defined_types.contains_key(&name) {
            return Err(DuplicateNameError::new(NameKind::UserType, name));
        }

        self.defined_types.insert(name, body);
        Ok(())
    }
}
