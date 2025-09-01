use crate::typesystem::errors::{ErrorStack, LinkingError, LinkingErrorEnum};

pub mod token;
pub mod selections;
pub mod expression;
pub mod utils;
pub mod operators;
pub mod annotations;
pub mod metaphors;
pub mod variable;
pub mod context;
pub mod sequence;
pub mod ifthenelse;
pub mod foreach;
pub mod user_function_call;
pub mod functions;

//----------------------------------------------------------------------------------------------

pub type Link<T> = Result<T, LinkingError>;

pub fn is_linked<T>(link: &Link<T>) -> bool {
    if let Err(err) = link {
        err.get_error_type() != &LinkingErrorEnum::NotLinkedYet
    } else {
        true
    }
}
