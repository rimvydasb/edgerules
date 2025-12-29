#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(Clone, PartialEq, Default)]
pub struct Metadata {
    pub version: Option<String>,
    pub model_name: Option<String>,
}

impl Metadata {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.version.is_none() && self.model_name.is_none()
    }
}
