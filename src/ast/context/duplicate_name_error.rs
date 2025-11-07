use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

impl Error for DuplicateNameError {}
