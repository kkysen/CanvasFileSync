use async_std::task;
use canvas_file_sync::api::Api;

fn main() {
    let api = Api::new(
        "courseworks2.columbia.edu".into(),
        env!("ACCESS_TOKEN").into(),
    );
    task::block_on(async {
        api.courses()
            .await
            .unwrap()
            .map(|it| format!("{}, {}", it.id, it.name))
            .for_each(|it| println!("{}", it))
    });
}
