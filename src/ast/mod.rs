use crate::typesystem::errors::{ErrorStack, LinkingError, LinkingErrorEnum};

pub mod annotations;
pub mod context;
pub mod expression;
pub mod foreach;
pub mod functions;
pub mod ifthenelse;
pub mod metaphors;
pub mod operators;
pub mod selections;
pub mod sequence;
pub mod token;
pub mod user_function_call;
pub mod utils;
pub mod variable;

//----------------------------------------------------------------------------------------------

pub type Link<T> = Result<T, LinkingError>;

pub fn is_linked<T>(link: &Link<T>) -> bool {
    if let Err(err) = link {
        err.get_error_type() != &LinkingErrorEnum::NotLinkedYet
    } else {
        true
    }
}
