use crate::conversion::traits::ToJs;
use crate::portable::error::PortableError;
use crate::portable::model::{
    apply_portable_entry, get_portable_entry, model_from_portable, remove_portable_entry, serialize_model,
};
use edge_rules::runtime::decision_service::DecisionService;
use edge_rules::typesystem::types::ValueType;
use edge_rules::typesystem::values::ValueEnum;
use wasm_bindgen::JsValue;

pub struct DecisionServiceController {
    service: DecisionService,
}

impl DecisionServiceController {
    pub fn from_portable(portable: &JsValue) -> Result<Self, PortableError> {
        let model = model_from_portable(portable)?;
        let service = DecisionService::from_model(model)?;
        Ok(Self { service })
    }

    pub fn from_source(source: &str) -> Result<Self, PortableError> {
        let service = DecisionService::from_source(source).map_err(PortableError::from)?;
        Ok(Self { service })
    }

    pub fn execute(&mut self, method: &str, args: Option<Vec<ValueEnum>>) -> Result<ValueEnum, PortableError> {
        Ok(self.service.execute(method, args)?)
    }

    pub fn model_snapshot(&mut self) -> Result<JsValue, PortableError> {
        let model = self.service.get_model();
        let snap = {
            let borrowed = model.borrow();
            serialize_model(&borrowed)?
        };

        // Add @schema metadata
        self.service.ensure_linked()?;
        let vt = self.service.get_linked_type("*")?;
        let schema = vt.to_js().map_err(PortableError::from)?;
        crate::utils::set_prop(&snap, "@schema", &schema)
            .map_err(|_| PortableError::DecisionServiceError("FailedToSetSchema".to_string()))?;

        Ok(snap)
    }

    pub fn set_entry(&mut self, path: &str, payload: &JsValue) -> Result<JsValue, PortableError> {
        let model = self.service.get_model();
        {
            let mut borrowed = model.borrow_mut();
            apply_portable_entry(&mut borrowed, path, payload)?;
        }
        self.service.ensure_linked()?;
        let updated = {
            let borrowed = model.borrow();
            get_portable_entry(&borrowed, path)?
        };
        Ok(updated)
    }

    pub fn remove_entry(&mut self, path: &str) -> Result<(), PortableError> {
        let model = self.service.get_model();
        {
            let mut borrowed = model.borrow_mut();
            remove_portable_entry(&mut borrowed, path)?;
        }
        self.service.ensure_linked()?;
        Ok(())
    }

    pub fn get_entry(&mut self, path: &str) -> Result<JsValue, PortableError> {
        if path == "*" {
            return self.model_snapshot();
        }
        let model = self.service.get_model();
        let val = {
            let borrowed = model.borrow();
            get_portable_entry(&borrowed, path)?
        };

        if crate::utils::is_object(&val) {
            self.service.ensure_linked()?;
            if let Ok(vt) = self.service.get_linked_type(path) {
                let schema = vt.to_js().map_err(PortableError::from)?;
                let _ = crate::utils::set_prop(&val, "@schema", &schema);
            }
        }

        Ok(val)
    }

    pub fn rename_entry(&mut self, old_path: &str, new_path: &str) -> Result<(), PortableError> {
        let model = self.service.get_model();
        {
            let mut borrowed = model.borrow_mut();
            borrowed.rename_entry(old_path, new_path).map_err(PortableError::from)?;
        }
        self.service.ensure_linked()?;
        Ok(())
    }

    pub fn get_entry_type(&mut self, path: &str) -> Result<ValueType, PortableError> {
        self.service.ensure_linked()?;
        self.service.get_linked_type(path).map_err(PortableError::from)
    }
}
