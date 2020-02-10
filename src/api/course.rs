use crate::api::core::Id;
use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct MaybeCourse {
    id: Id,
    name: Option<String>,
    // other fields not needed
}

#[derive(Debug)]
pub struct Course {
    pub id: Id,
    pub name: String,
}

impl From<MaybeCourse> for Option<Course> {
    fn from(course: MaybeCourse) -> Self {
        let MaybeCourse {
            id,
            name,
        } = course;
        let name = name?;
        Some(Course {
            id,
            name,
        })
    }
}

impl Course {
    pub fn modules_endpoint(&self) -> String {
        format!("courses/{}/modules", self.id)
    }
}
