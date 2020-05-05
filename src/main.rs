#![allow(dead_code)]

mod cli;

use async_std::task;
use std::path::Path;
use canvas_file_sync::download::downloads::Downloads;
use canvas_file_sync::download::data::FileTree;
use std::error::Error;
use crate::cli::Args;

async fn async_main() -> Result<(), Box<dyn Error>> {
    let path: &Path = ".".as_ref();
    let mut downloads = Downloads::new(path.to_owned())?;
    let file_tree: FileTree = serde_json::from_str("")?;
    downloads.add_file_tree(file_tree)?;
    downloads.download().await?;
    Ok(())
}

fn main2() {
    task::block_on(async {
        async_main().await.unwrap();
    })
}

#[paw::main]
fn main(args: Args) -> std::io::Result<()> {
    println!("{:?}", args);
    Ok(())
}
