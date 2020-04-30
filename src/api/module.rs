use crate::api::core::Id;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Module {
    pub id: Id,
    pub name: String,
    pub items: Vec<ModuleItem>,
}

impl Module {
    //    pub fn items_endpoint(&self) -> String {
    //
    //    }
}

pub struct ModuleItem {
    pub id: Id,
    pub url: String,
}

pub struct File {
    pub id: Id,
}
