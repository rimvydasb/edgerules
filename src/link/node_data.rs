use crate::ast::context::context_object_type::EObjectContent;
use crate::ast::context::context_object_type::EObjectContent::{
    ConstantValue, ExpressionRef, ObjectRef, UserFunctionRef,
};
use crate::link::node_data::NodeDataEnum::{Child, Internal, Isolated, Root};
use crate::typesystem::errors::LinkingError;
use crate::typesystem::errors::LinkingErrorEnum::CyclicReference;
use crate::utils::bracket_unwrap;
use log::trace;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::rc::{Rc, Weak};

#[derive(Debug, Clone)]
pub enum NodeDataEnum<T: Debug + Node<T>> {
    /// This is a normal context child. Child can access parent context and parent context can access child. Used in:
    /// 1. Context child
    Child(&'static str, Weak<RefCell<T>>),
    /// internal content can reach parent context. Used in:
    /// 1. Loops
    /// 2. Inline functions (if supported)
    Internal(Weak<RefCell<T>>),
    /// Fully isolated - parent cannot access internals, and internals cannot access parent. Used in:
    /// 1. Function bodies
    Isolated(),
    /// Same as isolated, but for the root context. Assigned by Edge Rules
    Root(),
}

impl<T: Debug + Node<T>> Display for NodeDataEnum<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Child(name, parent) => match parent.upgrade() {
                None => {
                    // @Todo: put a debugger here and run "unhappy_unreachable_path" - try to trace calls, something is off there
                    write!(f, "OrphanedChild({})", name)
                }
                Some(parent) => {
                    write!(f, "{}.{}", parent.borrow().node().node_type, name)
                }
            },
            Isolated() => write!(f, "Isolated"),
            Internal(parent) => match parent.upgrade() {
                None => {
                    // this situation should enver happen:
                    write!(f, "OrphanedChild(#child)")
                }
                Some(parent) => {
                    // @Todo: run tests with coverage and see if this path is ever hit
                    write!(f, "{}.{}", parent.borrow().node().node_type, "#child")
                }
            },
            Root() => write!(f, "Root"),
        }
    }
}

impl<T: Debug + Node<T>> NodeDataEnum<T> {
    pub fn get_parent(&self) -> Option<Rc<RefCell<T>>> {
        match self {
            Child(_, parent) => parent.upgrade(),
            Internal(parent) => parent.upgrade(),
            _ => None,
        }
    }

    pub fn to_code(&self) -> String {
        match self {
            Child(name, _parent) => (*name).to_string(),
            Isolated() | Root() => String::new(),
            Internal(_) => "#child".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodeData<T: Debug + Node<T>> {
    /// The type of node: root, isolated, internal or child
    pub node_type: NodeDataEnum<T>,
    /// Simple placeholder to lock object fields from modification
    object_field_locks: RefCell<HashSet<&'static str>>,
    /// Usually child list is not modified byt execution context is an exception
    childs: RefCell<HashMap<&'static str, Rc<RefCell<T>>>>,
}

pub trait ContentHolder<T: Node<T>> {
    fn get(&self, name: &str) -> Result<EObjectContent<T>, LinkingError>;

    fn get_field_names(&self) -> Vec<&'static str>;

    fn print_object(&self, f: &mut Formatter) -> fmt::Result {
        trace!("print_object: {:?}", self.get_field_names());

        let mut lines: Vec<String> = Vec::new();

        for field_name in self.get_field_names().iter() {
            match self.get(field_name) {
                Ok(ExpressionRef(field)) => {
                    let value = bracket_unwrap(format!("{}", field.borrow().expression));
                    lines.push(format!("{}: {}", field_name, value));
                }
                Ok(UserFunctionRef(definition)) => {
                    lines.push(format!("{}", definition.borrow().function_definition));
                }
                Ok(ObjectRef(obj)) => {
                    lines.push(format!("{}: {}", field_name, obj.borrow()));
                }
                Ok(ConstantValue(value)) => {
                    lines.push(format!("{}: {}", field_name, value));
                }
                _ => {}
            }
        }

        write!(f, "{{{}}}", lines.join("; "))
    }
}

pub trait Node<T: Node<T>>: Display + Debug + Clone + ContentHolder<T> {
    fn node(&self) -> &NodeData<T>;

    fn mut_node(&mut self) -> &mut NodeData<T>;
}

impl<T: Node<T>> PartialEq for NodeData<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get_assigned_to_field() == other.get_assigned_to_field()
    }
}

impl<T: Node<T>> NodeData<T> {
    // pub fn new(assigned_to_field: Option<String>, capacity: usize) -> Self {
    //     let object_field_locks = RefCell::new(HashSet::with_capacity(capacity));
    //     let childs = HashMap::with_capacity(capacity);
    //
    //     Self {
    //         assigned_to_field,
    //         parent: Weak::new(),
    //         childs: RefCell::new(childs),
    //         object_field_locks,
    //     }
    // }
    //
    pub fn new_fixed(
        childs: HashMap<&'static str, Rc<RefCell<T>>>,
        node_type: NodeDataEnum<T>,
    ) -> Self {
        let object_field_locks = RefCell::new(HashSet::with_capacity(childs.len()));
        let childs = RefCell::new(childs);

        Self {
            node_type,
            childs,
            object_field_locks,
        }
    }

    pub fn new(node_type: NodeDataEnum<T>) -> Self {
        Self {
            node_type,
            childs: RefCell::new(HashMap::new()),
            object_field_locks: RefCell::new(HashSet::new()),
        }
    }

    pub fn get_assigned_to_field(&self) -> Option<&'static str> {
        match self.node_type {
            Child(name, _) => Some(name),
            _ => None,
        }
    }

    pub fn lock_field(&self, field: &'static str) -> Result<(), LinkingError> {
        trace!("lock_field: {}.{}", self.node_type, field);
        if self.is_field_locked(field) {
            return LinkingError::new(CyclicReference(
                self.node_type.to_string(),
                field.to_string(),
            ))
            .into();
        }
        self.object_field_locks.borrow_mut().insert(field);

        Ok(())
    }

    pub fn unlock_field(&self, field: &'static str) {
        trace!("unlock_field: {}.{}", self.node_type, field);
        self.object_field_locks.borrow_mut().remove(field);
    }

    pub fn is_field_locked(&self, field: &'static str) -> bool {
        self.object_field_locks.borrow().contains(field)
    }

    pub fn get_childs(&self) -> RefCell<HashMap<&'static str, Rc<RefCell<T>>>> {
        RefCell::clone(&self.childs)
    }

    pub fn get_child(&self, name: &str) -> Option<Rc<RefCell<T>>> {
        self.childs.borrow().get(name).cloned()
    }

    pub fn add_child(&self, name: &'static str, child: Rc<RefCell<T>>) {
        self.childs.borrow_mut().insert(name, child);
    }
    //
    // /// The attachment can be made for a child to access the parent.
    // /// However, if assigned_to_field is left None, then parent cannot access a child in browse methods
    // pub fn attach_to_parent(&mut self, parent: &Rc<RefCell<T>>, assigned_to_field: Option<String>) {
    //     self.parent = Rc::downgrade(parent);
    //     self.assigned_to_field = assigned_to_field;
    // }
    //
    pub fn attach_child(parent: &Rc<RefCell<T>>, child: &Rc<RefCell<T>>) {
        let name = match &child.borrow().node().node_type {
            Child(name, parent) => match parent.upgrade() {
                None => Some(*name),
                Some(_) => None,
            },
            _ => None,
        };

        if let Some(name) = name {
            parent.borrow().node().add_child(name, Rc::clone(child));
            child.borrow_mut().mut_node().node_type = Child(name, Rc::downgrade(parent));
        };
    }
}
