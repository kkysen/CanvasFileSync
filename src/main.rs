use serde::Deserialize;
use canvas_file_sync::Api;
use async_std::task;

type Id = u64;

#[derive(Debug, Deserialize)]
struct Course {
    id: Id,
    name: String,
    // other fields not needed
}

fn main() {
    let api = Api::new("courseworks2.columbia.edu".into(), "".into());
    task::block_on(async {
        let courses: Vec<Course> = api.get_list("courses", &"").await.unwrap();
        println!("{:?}", courses);
    });
    println!("Hello, world!");
}
