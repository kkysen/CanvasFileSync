use crate::api::core::Id;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Module {
    pub id: Id,
    pub name: String,
    pub items_url: String,
}

impl Module {
    pub fn items_endpoint(&self) -> String {
    
    }
}

pub struct ModuleItem {
    pub id: Id,
    pub title: String,
}
