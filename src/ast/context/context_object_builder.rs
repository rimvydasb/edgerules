use crate::ast::context::context_object::{ContextObject, ExpressionEntry, MethodEntry};
use crate::ast::context::context_object_type::FormalParameter;
use crate::ast::metaphors::metaphor::Metaphor;
use crate::ast::token::DefinitionEnum::Metaphor as MetaphorDef;
use crate::ast::token::{DefinitionEnum, ExpressionEnum, UserTypeBody};
use crate::link::node_data::{Node, NodeData, NodeDataEnum};
use crate::typesystem::types::ValueType;
use crate::utils::intern_field_name;
use log::trace;
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
            node_type: NodeDataEnum::Internal(Rc::downgrade(&parent)),
            parameters: Vec::new(),
            defined_types: HashMap::new(),
        }
    }

    pub fn set_parameters(&mut self, parameters: Vec<FormalParameter>) -> &mut Self {
        self.parameters = parameters;
        self
    }

    // @Todo: check if field is not duplicated
    // @Todo: optimize by inserting by a number, not a field name
    // @Todo: return an error and propogate it to the top
    pub fn add_expression(&mut self, field_name: &str, field: ExpressionEnum) -> &mut Self {
        let field_name = intern_field_name(field_name);
        self.insert_field_name(field_name);

        if let ExpressionEnum::StaticObject(obj) = &field {
            // No need to assign parent now, it is done later in build
            self.childs.insert(field_name, Rc::clone(obj));
            return self;
        }

        trace!(">>> inserting field {:?}", field_name);
        self.fields
            .insert(field_name, ExpressionEntry::from(field).into());

        self
    }

    pub fn add_definition(&mut self, field: DefinitionEnum) {
        match field {
            MetaphorDef(m) => {
                let name = m.get_name();
                trace!(">>> inserting function {:?}", name);
                let interned = intern_field_name(name.as_str());
                self.insert_field_name(interned);
                self.metaphors.insert(interned, MethodEntry::from(m).into());
            }
            DefinitionEnum::UserType(t) => {
                self.defined_types.insert(t.name, t.body);
            }
        }
    }

    pub fn set_context_type(&mut self, context_type: ValueType) {
        self.context_type = Some(context_type);
    }

    pub fn append(&mut self, another: Rc<RefCell<ContextObject>>) {
        for (key, value) in another.borrow().expressions.iter() {
            self.fields.insert(*key, Rc::clone(value));
        }

        for (key, value) in another.borrow().metaphors.iter() {
            self.metaphors.insert(*key, Rc::clone(value));
        }

        for (key, value) in another.borrow().node().get_childs().borrow().iter() {
            if let Some(existing_child) = self.childs.get(key) {
                let another_child = another.borrow().node.get_child(key).unwrap();
                Self::merge(Rc::clone(existing_child), another_child);
                continue;
            }

            self.childs.insert(*key, Rc::clone(value));
        }

        // Merge metaphors by name
        for (key, value) in another.borrow().metaphors.iter() {
            self.metaphors.insert(*key, Rc::clone(value));
        }

        for (key, value) in another.borrow().defined_types.iter() {
            self.defined_types.insert(key.clone(), value.clone());
        }

        // Update field_names and deduplicate them
        for field_name in another.borrow().get_field_names() {
            self.insert_field_name(field_name);
        }
    }

    pub fn merge(target: Rc<RefCell<ContextObject>>, another: Rc<RefCell<ContextObject>>) {
        for (key, value) in another.borrow().expressions.iter() {
            target
                .borrow_mut()
                .expressions
                .insert(*key, Rc::clone(value));
        }

        for (key, value) in another.borrow().metaphors.iter() {
            target.borrow_mut().metaphors.insert(*key, Rc::clone(value));
        }

        for (key, value) in another.borrow().node().get_childs().borrow().iter() {
            if let Some(existing_child) = target.borrow().node.get_child(key) {
                let another_child = another.borrow().node.get_child(key).unwrap();
                Self::merge(existing_child, another_child);
                continue;
            }

            target.borrow().node().add_child(*key, Rc::clone(value));
        }

        // Merge metaphors by name
        for (key, value) in another.borrow().metaphors.iter() {
            target.borrow_mut().metaphors.insert(*key, Rc::clone(value));
        }

        // Update field_names and deduplicate them
        {
            let mut target_ref = target.borrow_mut();
            for field_name in another.borrow().get_field_names() {
                target_ref.add_field_name(field_name);
            }
        }
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
        };

        let ctx = Rc::new(RefCell::new(obj));

        ctx.borrow()
            .node()
            .get_childs()
            .borrow()
            .iter()
            .for_each(|(name, child)| {
                child.borrow_mut().node.node_type = NodeDataEnum::Child(*name, Rc::downgrade(&ctx));
            });

        ctx
    }

    fn insert_field_name(&mut self, field_name: &'static str) {
        if self.field_name_set.contains(field_name) {
            return;
        }

        self.field_name_set.insert(field_name);
        self.field_names.push(field_name);
    }
}
