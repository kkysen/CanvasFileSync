use serde::Deserialize;
use async_std::task;
use canvas_file_sync::api::{Api, no_query};

type Id = u64;

#[derive(Debug, Deserialize)]
struct MaybeCourse {
    id: Id,
    name: Option<String>,
    // other fields not needed
}

#[derive(Debug)]
struct Course {
    id: Id,
    name: String,
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

fn main() {
    let api = Api::new(
        "courseworks2.columbia.edu".into(),
        env!("ACCESS_TOKEN").into(),
    );
    task::block_on(async {
        let courses: Vec<MaybeCourse> = api.get_list("courses", no_query()).await.unwrap();
        courses
            .into_iter()
            .filter_map(|it| it.into())
            .map(|it: Course| format!("{}, {}", it.id, it.name))
            .for_each(|it| println!("{}", it))
    });
}
