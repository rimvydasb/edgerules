use crate::ast::context::context_object::ContextObject;
use crate::ast::context::context_object_type::EObjectContent;
use crate::ast::context::context_object_type::EObjectContent::{
    ConstantValue, ExpressionRef, MetaphorRef,
};
use crate::link::node_data::{ContentHolder, Node, NodeData, NodeDataEnum};
use crate::typesystem::errors::{LinkingError, RuntimeError};
use crate::typesystem::types::{TypedValue, ValueType};
use crate::typesystem::values::ValueEnum;
use crate::typesystem::values::ValueEnum::Reference;
use crate::utils::{intern_field_name, Line, Lines};
use log::trace;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::rc::{Rc, Weak};

/// ---
/// @TODO: https://doc.rust-lang.org/book/ch15-04-rc.html
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub node: NodeData<ExecutionContext>,
    /// There could be multiple execution contexts that wrap a single object at a given time.
    /// To simplify the code, it will stay mutable.
    /// - @Todo: consider using non mutable or clarify why it is mutable
    pub object: Rc<RefCell<ContextObject>>,
    /// limitations: context variable cannot be reference, or error, or object. It must be a primitive value
    pub context_variable: Option<ValueEnum>,
    /// This flag can be set by any method that performs or ensures that context is really fully evaluated so full evaluation will not be repeated.
    pub promise_eval_all: bool,
    /// stack can be constantly updated. accessed via API
    stack: RefCell<HashMap<&'static str, Result<ValueEnum, RuntimeError>>>,
    /// Weak self pointer to allow building parent links from methods that only have &self
    self_ref: Weak<RefCell<ExecutionContext>>,
}

impl Display for ExecutionContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.print_object(f)
    }
}

impl PartialEq for ExecutionContext {
    fn eq(&self, other: &Self) -> bool {
        self.object == other.object && self.node == other.node && self.stack == other.stack
    }
}

impl From<ExecutionContext> for Rc<RefCell<ExecutionContext>> {
    fn from(val: ExecutionContext) -> Self {
        Rc::new(RefCell::new(val))
    }
}

impl Node<ExecutionContext> for ExecutionContext {
    fn node(&self) -> &NodeData<ExecutionContext> {
        &self.node
    }

    fn mut_node(&mut self) -> &mut NodeData<ExecutionContext> {
        &mut self.node
    }
}

// @Todo: LinkingError should nopt be returned in execution context stuff, think of more templating
impl ContentHolder<ExecutionContext> for ExecutionContext {
    fn get(&self, name: &str) -> Result<EObjectContent<ExecutionContext>, LinkingError> {
        match self.stack.borrow().get(name) {
            None => {}
            Some(Err(err)) => {
                return LinkingError::other_error(err.to_string()).into();
            }
            Some(Ok(Reference(value))) => {
                return Ok(EObjectContent::ObjectRef(Rc::clone(value)));
            }
            Some(Ok(value)) => {
                return Ok(ConstantValue(value.clone()));
            }
        }

        if let Some(child) = self.node.get_child(name) {
            return Ok(EObjectContent::ObjectRef(child));
        }

        match self.object.borrow().get(name)? {
            EObjectContent::ObjectRef(object) => {
                trace!(
                    "Creating new child execution context for get: {}",
                    object.borrow().node().node_type
                );
                let new_child = self.create_orphan_context(intern_field_name(name), object);
                Ok(EObjectContent::ObjectRef(new_child))
            }
            ConstantValue(value) => Ok(ConstantValue(value)),
            ExpressionRef(value) => Ok(ExpressionRef(value)),
            MetaphorRef(value) => Ok(MetaphorRef(value)),
            EObjectContent::Definition(definition) => LinkingError::other_error(format!(
                "Definition {} is not supported in execution context",
                definition
            ))
            .into(),
        }
    }

    fn get_field_names(&self) -> Vec<&'static str> {
        self.object.borrow().get_field_names()
    }
}

impl TypedValue for ExecutionContext {
    fn get_type(&self) -> ValueType {
        ValueType::ObjectType(Rc::clone(&self.object))
    }
}

impl ExecutionContext {
    pub fn create_isolated_context(
        static_context: Rc<RefCell<ContextObject>>,
    ) -> Rc<RefCell<ExecutionContext>> {
        Self {
            object: static_context,
            stack: RefCell::new(HashMap::new()),
            context_variable: None,
            node: NodeData::new(NodeDataEnum::Isolated()),
            promise_eval_all: false,
            self_ref: Weak::new(),
        }
        .into_rc()
    }

    pub fn create_root_context(
        static_context: Rc<RefCell<ContextObject>>,
    ) -> Rc<RefCell<ExecutionContext>> {
        Self {
            object: static_context,
            stack: RefCell::new(HashMap::new()),
            context_variable: None,
            node: NodeData::new(NodeDataEnum::Root()),
            promise_eval_all: false,
            self_ref: Weak::new(),
        }
        .into_rc()
    }

    pub fn create_orphan_context(
        &self,
        assigned_to_field: &'static str,
        static_context: Rc<RefCell<ContextObject>>,
    ) -> Rc<RefCell<ExecutionContext>> {
        let new_child = Self {
            object: static_context,
            stack: RefCell::new(HashMap::new()),
            context_variable: None,
            node: NodeData::new(NodeDataEnum::Child(assigned_to_field, Weak::new())),
            promise_eval_all: false,
            self_ref: Weak::new(),
        }
        .into_rc();

        // Attach to this parent so that parent links are set and browsing can walk up
        if let Some(parent_rc) = self.self_ref.upgrade() {
            NodeData::attach_child(&parent_rc, &new_child);
        } else {
            // Fallback: keep previous behavior
            self.node()
                .add_child(assigned_to_field, Rc::clone(&new_child));
        }
        new_child
    }

    /// - Child context can refer parent context and get all it's owned fields. However, parent cannot refer to the child, because path or position of the child is not clear.
    /// - Child is also not added to the parent stack.
    pub fn create_temp_child_context(
        parent: Rc<RefCell<ExecutionContext>>,
        static_context: Rc<RefCell<ContextObject>>,
    ) -> Rc<RefCell<ExecutionContext>> {
        Self {
            object: static_context,
            stack: RefCell::new(HashMap::new()),
            context_variable: None,
            node: NodeData::new(NodeDataEnum::Internal(Rc::downgrade(&parent))),
            promise_eval_all: false,
            self_ref: Weak::new(),
        }
        .into_rc()
    }

    pub fn get_context_variable(&self) -> Result<ValueEnum, RuntimeError> {
        if let Some(value) = &self.context_variable {
            Ok(value.clone())
        } else {
            RuntimeError::eval_error(format!(
                "Context variable not set for {}",
                self.object.borrow().node.node_type
            ))
            .into()
        }
    }

    pub fn into_rc(self) -> Rc<RefCell<ExecutionContext>> {
        let rc = Rc::new(RefCell::new(self));
        let weak = Rc::downgrade(&rc);
        rc.borrow_mut().self_ref = weak;
        rc
    }

    pub fn to_code(&self) -> String {
        let mut lines = Lines::new();

        self.to_code_accumulate(&mut lines);

        lines.to_string()
    }

    fn to_code_accumulate(&self, lines: &mut Lines) {
        {
            let mut line = Line::new();
            match &self.node().node_type {
                NodeDataEnum::Child(name, _) => {
                    line.add(name).add(": {");
                }
                NodeDataEnum::Internal(_) => {
                    line.add("#child").add(": {");
                }
                NodeDataEnum::Isolated() | NodeDataEnum::Root() => {
                    line.add("{");
                }
            }

            trace!(
                "to_code_accumulate: {}, stack: {}",
                self.node().node_type,
                self.stack.borrow().len()
            );
            lines.add(line);
        }

        lines.tab();

        for field_name in &self.object.borrow().get_field_names() {
            match self.get(field_name) {
                Ok(field) => {
                    match field {
                        ConstantValue(value) => {
                            lines.add_str(format!("{}: {}", field_name, value).as_str());
                        }
                        ExpressionRef(expression) => {
                            lines.add_str(
                                format!("{}: {}", field_name, expression.borrow().expression)
                                    .as_str(),
                            );
                        }
                        MetaphorRef(_) => {
                            // skip
                        }
                        EObjectContent::ObjectRef(ref object) => {
                            let result_reference = Rc::clone(object);
                            (*result_reference).borrow().to_code_accumulate(lines);
                        }
                        EObjectContent::Definition(_) => {
                            // skip
                        }
                    }
                }
                Err(err) => {
                    lines.add_str(err.to_string().as_str());
                }
            }
        }

        lines.back();
        lines.add_str("}");
    }

    pub fn stack_insert(&self, field_name: &'static str, value: Result<ValueEnum, RuntimeError>) {
        if let Ok(Reference(child_ctx)) = &value {
            if let Some(parent) = self.self_ref.upgrade() {
                {
                    let mut child = child_ctx.borrow_mut();
                    child.mut_node().node_type = NodeDataEnum::Child(field_name, Weak::new());
                }
                NodeData::attach_child(&parent, child_ctx);
            }
        }
        self.stack.borrow_mut().insert(field_name, value);
    }

    pub fn eval_all_fields(ctx: &Rc<RefCell<ExecutionContext>>) -> Result<(), RuntimeError> {
        if ctx.borrow().promise_eval_all {
            return Ok(());
        }

        ctx.borrow_mut().promise_eval_all = true;

        let field_names = ctx.borrow().object.borrow().get_field_names();

        for name in field_names {
            match ctx.borrow().get(name)? {
                EObjectContent::ExpressionRef(expression) => {
                    ctx.borrow().node().lock_field(name)?;
                    let value = expression.borrow().expression.eval(Rc::clone(ctx));
                    ctx.borrow().stack_insert(name, value);
                    ctx.borrow().node().unlock_field(name);
                }
                EObjectContent::ObjectRef(reference) => {
                    NodeData::attach_child(ctx, &reference);
                    ExecutionContext::eval_all_fields(&reference)?;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[cfg(test)]
pub mod test {
    use crate::ast::context::context_object_builder::ContextObjectBuilder;
    use crate::ast::metaphors::functions::FunctionDefinition;
    use crate::ast::token::DefinitionEnum::Metaphor as MetaphorDef;
    use crate::ast::token::ExpressionEnum;
    use log::info;
    use std::rc::Rc;

    use crate::link::linker::link_parts;
    use crate::link::node_data::ContentHolder;
    use crate::runtime::edge_rules::{expr, EvalError};
    use crate::runtime::execution_context::ExecutionContext;
    use crate::typesystem::types::TypedValue;
    use crate::typesystem::values::ValueEnum;
    use crate::utils::test::init_logger;

    type E = ExpressionEnum;

    #[test]
    fn test_nesting() -> Result<(), EvalError> {
        init_logger();

        info!(">>> test_nesting()");

        let mut builder = ContextObjectBuilder::new();
        builder.add_expression("a", E::from(1.0))?;
        builder.add_expression("b", E::from(2.0))?;

        {
            let mut child = ContextObjectBuilder::new();
            child.add_expression("x", E::from("Hello"))?;
            child.add_expression("y", expr("a + b")?)?;
            child.add_definition(MetaphorDef(
                FunctionDefinition::build(
                    "income".to_string(),
                    vec![],
                    ContextObjectBuilder::new().build(),
                )?
                .into(),
            ))?;
            builder.add_expression("c", ExpressionEnum::StaticObject(child.build()))?;
        }

        let ctx = builder.build();

        link_parts(Rc::clone(&ctx))?;

        let ex = ExecutionContext::create_root_context(ctx);

        ex.borrow().stack_insert("a", Ok(ValueEnum::from(88.0)));
        ex.borrow().stack_insert("b", Ok(ValueEnum::from(99.0)));

        assert_eq!(ex.borrow().get("a")?.to_string(), "88");
        assert_eq!(ex.borrow().get("b")?.to_string(), "99");
        assert!(ex.borrow().get("x").is_err());
        assert_eq!(
            ex.borrow().to_string(),
            "{a: 88; b: 99; c: {x: 'Hello'; y: a + b; income() : {}}}"
        );
        assert_eq!(
            ex.borrow().get_type().to_string(),
            "{a: number; b: number; c: {x: string; y: number}}"
        );
        assert_eq!(
            ex.borrow().get("c")?.to_string(),
            "{x: 'Hello'; y: a + b; income() : {}}"
        );

        // @Todo: update tests
        // {
        //     let result = linker::find_variable(Rc::clone(&ex), "a")?;
        //     assert_eq!(result.to_string(), "88");
        //     assert_eq!(result.get_type().to_string(), "number");
        // }
        //
        // {
        //     let result = linker::find_path(Rc::clone(&ex), vec!["c","x"])?;
        //     assert_eq!(result.to_string(), "'Hello'");
        //     assert_eq!(result.get_type().to_string(), "string");
        // }
        //
        // {
        //     let result = linker::find_path(Rc::clone(&ex), vec!["c","y"])?;
        //     assert_eq!(result.to_string(), "(a + b)");
        //     assert_eq!(result.get_type().to_string(), "number");
        // }

        Ok(())
    }
}
