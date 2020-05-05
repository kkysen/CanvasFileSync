#![feature(move_ref_pattern)]
#![allow(dead_code)]

use std::path::PathBuf;
use crate::download::data::CanvasBase;
use std::error::Error;

pub mod api;
pub mod download;
mod util;

pub struct CanvasFileSync {
    pub dir: PathBuf,
    pub skip_git: bool,
}

#[derive(Debug)]
pub struct AddUser {
    pub access_token: String,
    pub search: String,
}

impl CanvasFileSync {
    pub fn add_user<F>(&self, add_user: AddUser, select_canvas: F) -> Result<(), Box<dyn Error>>
        where F: FnOnce(Vec<CanvasBase>) -> Result<CanvasBase, Box<dyn Error>> {
        dbg!(add_user);
        select_canvas(vec![])?;
        todo!()
    }
    
    pub fn sync(&self) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}
