// use crate::api::core::{CoreApi, PerPage, SurfResult};
// use crate::api::course::{Course, MaybeCourse};
// use crate::api::module::Module;
// use crate::api::query::courses::CoursesAllCourses;
// use crate::api::query::modules::ModulesCourseModulesConnection;
// use crate::api::query::Courses;

pub mod data;
pub mod downloads;
mod download;
mod diff;

// mod core;
// mod course;
// mod link;
// mod module;
// mod query;

// pub struct Api {
//     api: CoreApi,
// }
//
// impl Api {
//     pub fn new(root_url: String, access_token: String) -> Api {
//         Api {
//             api: CoreApi::new(root_url, access_token),
//         }
//     }
//
//     pub fn root_url(&self) -> &str {
//         self.api.root_url.as_str()
//     }
//
//     pub fn access_token(&self) -> &str {
//         self.api.access_token()
//     }
//
//     pub fn authorization(&self) -> &str {
//         self.api.auth.as_str()
//     }
//
//     pub async fn courses(&self) -> SurfResult<impl Iterator<Item = Course>> {
//         self.api
//             .get_filtered_list::<PerPage, MaybeCourse, Course>(
//                 "courses",
//                 &PerPage { per_page: 100 },
//             )
//             .await
//     }
//
//     pub async fn modules(&self, course: &Course) -> SurfResult<impl Iterator<Item = Module>> {
//         let modules = self
//             .api
//             .get_list(
//                 course.modules_endpoint().as_str(),
//                 &PerPage { per_page: 100 },
//             )
//             .await?
//             .into_iter();
//         Ok(modules)
//     }
//
//     pub async fn all_courses(&self) -> SurfResult<Vec<CoursesAllCourses>> {
//         let courses = self
//             .api
//             .query::<query::Courses>(&query::courses::Variables {})
//             .await?
//             .data?
//             .all_courses?;
//         Ok(courses)
//     }
//
//     pub async fn modules_for_course(&self, course: &CoursesAllCourses) -> SurfResult<()> {
//         let modules = self
//             .api
//             .query::<query::Modules>(&query::modules::Variables {
//                 course_id: course.id,
//             })
//             .await?
//             .data?
//             .course?
//             .modules_connection?;
//         if modules.page_info.has_next_page {
//             return Box::new(Err(""));
//         }
//         let modules = modules
//             .nodes?
//             .into_iter()
//             .map(|it| )
//         }
//         }
//     }
// }
