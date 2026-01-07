use crate::ast::context::context_object::ContextObject;
use crate::link::node_data::Node;
use std::cell::RefCell;
use std::rc::Rc;

pub fn resolve_context_path(
    mut current: Rc<RefCell<ContextObject>>,
    path_segments: &[&str],
) -> Option<Rc<RefCell<ContextObject>>> {
    for segment in path_segments {
        let next = {
            let ctx = current.borrow();
            if let Some(child) = ctx.node().get_child(segment) {
                Some(child)
            } else if let Some(function) = ctx.get_function(segment) {
                Some(Rc::clone(&function.borrow().function_definition.body))
            } else {
                None
            }
        };
        match next {
            Some(child) => current = child,
            None => return None,
        }
    }
    Some(current)
}
