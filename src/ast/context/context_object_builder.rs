use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use log::trace;
use crate::ast::context::context_object::{ContextObject, ExpressionEntry, MethodEntry};
use crate::ast::context::context_object_type::FormalParameter;
use crate::ast::token::{DefinitionEnum, ExpressionEnum};
use crate::ast::token::DefinitionEnum::MetaphorDefinition;
use crate::link::node_data::{Node, NodeData, NodeDataEnum};
use crate::typesystem::types::ValueType;

/// ---
/// **ContextObjectBuilder**
/// - Builds Execution Context Object and gets dismissed after building.

pub struct ContextObjectBuilder {
    fields: HashMap<String, Rc<RefCell<ExpressionEntry>>>,
    metaphors: HashMap<String, Rc<RefCell<MethodEntry>>>,
    childs: HashMap<String, Rc<RefCell<ContextObject>>>,
    field_names: Vec<String>,
    context_type: Option<ValueType>,
    parameters: Vec<FormalParameter>,
    node_type: NodeDataEnum<ContextObject>,
}

impl ContextObjectBuilder {
    pub fn new() -> Self {
        ContextObjectBuilder {
            fields: HashMap::new(),
            metaphors: HashMap::new(),
            childs: HashMap::new(),
            field_names: Vec::new(),
            context_type: None,
            node_type: NodeDataEnum::Root(),
            parameters: Vec::new(),
        }
    }

    pub fn new_internal(parent: Rc<RefCell<ContextObject>>) -> Self {
        ContextObjectBuilder {
            fields: HashMap::new(),
            metaphors: HashMap::new(),
            childs: HashMap::new(),
            field_names: Vec::new(),
            context_type: None,
            node_type: NodeDataEnum::Internal(Rc::downgrade(&parent)),
            parameters: Vec::new(),
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
        self.field_names.push(field_name.to_string());

        if let ExpressionEnum::StaticObject(obj) = &field {
            // No need to assign parent now, it is done later in build
            self.childs.insert(field_name.to_string(), Rc::clone(obj));
            return self;
        }

        trace!(">>> inserting field {:?}",field_name);
        self.fields.insert(field_name.to_string(), ExpressionEntry::from(field).into());

        self
    }

    pub fn add_definition(&mut self, field: DefinitionEnum) {
        match field {
            MetaphorDefinition(metaphor) => {
                trace!(">>> inserting function {:?}",metaphor.get_name());
                self.field_names.push(metaphor.get_name());
                self.metaphors.insert(metaphor.get_name(), MethodEntry::from(metaphor).into());
            }
        }
    }

    pub fn set_context_type(&mut self, context_type: ValueType) {
        self.context_type = Some(context_type);
    }

    pub fn append(&mut self, another: Rc<RefCell<ContextObject>>) {
        for (key, value) in another.borrow().expressions.iter() {
            self.fields.insert(key.clone(), Rc::clone(value));
        }

        for (key, value) in another.borrow().metaphors.iter() {
            self.metaphors.insert(key.clone(), Rc::clone(value));
        }

        for (key, value) in another.borrow().node().get_childs().borrow().iter() {
            if let Some(existing_child) = self.childs.get(key) {
                let another_child = another.borrow().node.get_child(key).unwrap();
                Self::merge(Rc::clone(existing_child), another_child);
                continue;
            }

            self.childs.insert(key.clone(), Rc::clone(value));
        }

        // Merge metaphors by name
        for (key, value) in another.borrow().metaphors.iter() {
            self.metaphors.insert(key.clone(), Rc::clone(value));
        }

        // Update field_names and deduplicate them
        self.field_names.extend(another.borrow().get_field_names());
        self.field_names.sort_unstable();
        self.field_names.dedup();
    }

    pub fn merge(target: Rc<RefCell<ContextObject>>, another: Rc<RefCell<ContextObject>>) {
        for (key, value) in another.borrow().expressions.iter() {
            target.borrow_mut().expressions.insert(key.clone(), Rc::clone(value));
        }

        for (key, value) in another.borrow().metaphors.iter() {
            target.borrow_mut().metaphors.insert(key.clone(), Rc::clone(value));
        }

        for (key, value) in another.borrow().node().get_childs().borrow().iter() {
            if let Some(existing_child) = target.borrow().node.get_child(key) {
                let another_child = another.borrow().node.get_child(key).unwrap();
                Self::merge(existing_child, another_child);
                continue;
            }

            target.borrow().node().add_child(key.clone(), Rc::clone(value));
        }

        // Merge metaphors by name
        for (key, value) in another.borrow().metaphors.iter() {
            target.borrow_mut().metaphors.insert(key.clone(), Rc::clone(value));
        }

        // Update field_names and deduplicate them
        target.borrow_mut().all_field_names.extend(another.borrow().get_field_names());
        target.borrow_mut().all_field_names.sort_unstable();
        target.borrow_mut().all_field_names.dedup();
    }

    pub fn get_field_names(&self) -> Vec<String> {
        self.field_names.clone()
    }

    pub fn build(self) -> Rc<RefCell<ContextObject>> {
        let obj = ContextObject {
            expressions: self.fields,
            metaphors: self.metaphors,
            all_field_names: self.field_names,
            node: NodeData::new_fixed(self.childs, self.node_type),
            parameters: self.parameters,
            context_type: self.context_type,
        };

        let ctx = Rc::new(RefCell::new(obj));

        ctx.borrow().node().get_childs().borrow().iter().for_each(|(name, child)| {
            child.borrow_mut().node.node_type = NodeDataEnum::Child(name.clone(), Rc::downgrade(&ctx));
        });

        ctx
    }
}
