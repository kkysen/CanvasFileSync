use crate::api::core::{CoreApi, SurfResult, PerPage};
use crate::api::course::{Course, MaybeCourse};
use crate::api::module::Module;

mod core;
mod link;
mod course;
mod module;

pub struct Api {
    api: CoreApi,
}

impl Api {
    pub fn new(root_url: String, access_token: String) -> Api {
        Api {
            api: CoreApi::new(root_url, access_token),
        }
    }
    
    pub async fn courses(&self) -> SurfResult<impl Iterator<Item = Course>> {
        self.api
            .get_filtered_list::<PerPage, MaybeCourse, Course>("courses", &PerPage {per_page: 100})
            .await
    }
    
    pub async fn modules(&self, course: &Course) -> SurfResult<impl Iterator<Item = Module>> {
        let modules = self.api
            .get_list(course.modules_endpoint().as_str(), &PerPage {per_page: 100})
            .await?
            .into_iter();
        Ok(modules)
    }
    
}
