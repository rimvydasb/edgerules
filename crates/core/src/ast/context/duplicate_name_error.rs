use std::fmt::{Display, Formatter};

#[cfg(not(target_arch = "wasm32"))]
use std::error::Error;

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum NameKind {
    Field,
    Function,
    UserType,
}

impl NameKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            NameKind::Field => "field",
            NameKind::Function => "function",
            NameKind::UserType => "user type",
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(Clone, PartialEq, Eq)]
pub struct DuplicateNameError {
    pub kind: NameKind,
    pub name: String,
}

impl DuplicateNameError {
    pub fn new(kind: NameKind, name: impl Into<String>) -> Self {
        DuplicateNameError {
            kind,
            name: name.into(),
        }
    }
}

impl Display for DuplicateNameError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Duplicate {} '{}'", self.kind.as_str(), self.name)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Error for DuplicateNameError {}
