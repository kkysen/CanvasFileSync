use async_std::task;
// use canvas_file_sync::api::Api;
// use itertools::Itertools;

use std::path::Path;
use canvas_file_sync::api::downloads::Downloads;
use canvas_file_sync::api::data::Directory;

fn old_main() {
    // let api = Api::new(
    //     "courseworks2.columbia.edu".into(),
    //     env!("ACCESS_TOKEN").into(),
    // );
    // task::block_on(async {
    //     let courses = api.courses().await.unwrap().collect_vec();
    //     courses
    //         .iter()
    //         .map(|it| format!("{}, {}", it.id, it.name))
    //         .for_each(|it| println!("{}", it));
    //     let spanish = courses.iter().find(|it| it.id == 99186).unwrap();
    //     let modules = api.modules(spanish).await.unwrap().collect_vec();
    //     modules
    //         .iter()
    //         .map(|it| format!("{}, {}, {}", it.id, it.name, it.items_url))
    //         .for_each(|it| println!("{}", it))
    // });
    // println!("{}", api.authorization())
}

fn main() {
    let path: &Path = ".".as_ref();
    let mut downloads = Downloads::new(path.to_owned()).unwrap();
    let file_tree = Directory::from_dir(path).unwrap();
    downloads.add_file_tree(file_tree);
    task::block_on(async {
        downloads.download().await.unwrap();
    })
}
