use std::rc::Rc;
use std::cell::RefCell;
use log::trace;
use std::fmt::{Debug, Display, Formatter};
use std::fmt;
use crate::ast::context::context_object::ContextObject;
use crate::ast::context::context_object_builder::ContextObjectBuilder;
use crate::ast::context::context_object_type::{EObjectContent, FormalParameter};
use crate::ast::context::context_object_type::EObjectContent::{ConstantValue, Definition, ExpressionRef, MetaphorRef, ObjectRef};
use crate::ast::token::ExpressionEnum;
use crate::link::node_data::{ContentHolder, Node, NodeData, NodeDataEnum};
use crate::runtime::execution_context::ExecutionContext;
use crate::typesystem::errors::{LinkingError, RuntimeError};
use crate::typesystem::types::{TypedValue, ValueType};
use crate::typesystem::values::ValueEnum;

/// Function context can be created as an internally scoped: that means no upper context browse is possible.
#[derive(Debug, Clone)]
pub struct AbstractFunctionContext<B: PartialEq + Debug> {
    /// The access to the parent context if available - it is available for an inline functions
    pub node: NodeData<ContextObject>,
    /// It can be a body of a function or a sinle expression for an inline function such as a loop
    pub body: Rc<RefCell<B>>,
    /// Context can have requirements to be executed. If context is a function body, then requirements are parameters
    pub parameters: Vec<FormalParameter>,
}

pub type FunctionContext = AbstractFunctionContext<ContextObject>;
//pub type InlineFunctionContext = AbstractFunctionContext<ExpressionEnum>;

impl<B: PartialEq + Debug> PartialEq for AbstractFunctionContext<B> {
    fn eq(&self, other: &Self) -> bool {
        self.body == other.body && self.node == other.node && self.parameters == other.parameters
    }
}

impl ContentHolder<ContextObject> for FunctionContext {
    fn get(&self, name: &str) -> Result<EObjectContent<ContextObject>, LinkingError> {
        let finding = self.parameters.iter().find(|field| field.name == name);

        if finding.is_some() {
            return Ok(Definition(finding.unwrap().value_type.clone()));
        }

        match self.body.borrow().get(name)? {
            ObjectRef(object) => Ok(ObjectRef(Rc::clone(&object))),
            ConstantValue(value) => Ok(ConstantValue(value)),
            ExpressionRef(value) => Ok(ExpressionRef(value)),
            MetaphorRef(value) => Ok(MetaphorRef(value)),
            Definition(definition) => Ok(Definition(definition)),
        }
    }

    fn get_field_names(&self) -> Vec<String> {
        self.body.borrow().get_field_names()
    }
}

impl TypedValue for FunctionContext {
    fn get_type(&self) -> ValueType {
        ValueType::ObjectType(Rc::clone(&self.body))
    }
}

impl Node<ContextObject> for FunctionContext {
    fn node(&self) -> &NodeData<ContextObject> {
        &self.node
    }

    fn mut_node(&mut self) -> &mut NodeData<ContextObject> {
        &mut self.node
    }
}

impl Display for FunctionContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.print_object(f)
    }
}

pub const RETURN_EXPRESSION: &str = "_return";

impl FunctionContext {
    pub fn create_for(body: Rc<RefCell<ContextObject>>, parameters: Vec<FormalParameter>) -> Self {
        Self {
            body,
            parameters,
            node: NodeData::new(NodeDataEnum::Isolated())
        }
    }

    pub fn create_inline_for(expression: ExpressionEnum, parameters: Vec<FormalParameter>, parent: Rc<RefCell<ContextObject>>) -> Self {
        let mut builder = ContextObjectBuilder::new_internal(Rc::clone(&parent));

        builder
            .set_parameters(parameters.clone())
            .add_expression(RETURN_EXPRESSION, expression);

        Self {
            body: builder.build(),
            parameters,
            node: NodeData::new(NodeDataEnum::Isolated())
        }
    }

    pub fn create_eval_context(&self, input: Vec<Result<ValueEnum, RuntimeError>>) -> Result<Rc<RefCell<ExecutionContext>>, RuntimeError> {
        let ctx = ExecutionContext::create_isolated_context(Rc::clone(&self.body));

        input.into_iter()
            .zip(self.parameters.iter())
            .for_each(|(value, arg)| {
                trace!("function {}(...) {} = {:?}",ctx.borrow().node().node_type, arg.name,&value);
                ctx.borrow().stack_insert(arg.name.clone(), value);
            });

        Ok(ctx)
    }
}

// impl InlineFunctionContext {
//     pub fn create_for(expression: Rc<RefCell<ExpressionEnum>>, parameters: Vec<FormalParameter>, parent: Rc<RefCell<ContextObject>>) -> Self {
//         let parent_name = parent.borrow().node().get_assigned_to_field();
//         let mut node = NodeData::new(Some(parent_name), 0);
//         node.parent = Rc::downgrade(&parent);
//
//         Self {
//             body: expression,
//             parameters,
//             node,
//         }
//     }
// }
//
// impl Display for InlineFunctionContext {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         write!(f, "{}", self.body.borrow())
//     }
// }
