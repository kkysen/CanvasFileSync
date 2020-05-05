mod select;

use std::path::PathBuf;
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::fmt;
use structopt::StructOpt;
use itertools::Itertools;
use canvas_file_sync::CanvasFileSync;
use crate::cli::select::select_canvas_using_skim;
use std::error::Error;

#[derive(Debug)]
pub struct CanvasParentDir(PathBuf);

impl From<&OsStr> for CanvasParentDir {
    fn from(s: &OsStr) -> Self {
        Self(s.into())
    }
}

impl Default for CanvasParentDir {
    fn default() -> Self {
        // TODO should this panic on None?
        // how else can I tell clap not to use a default_value if it fails
        let dir = dirs::document_dir()
            .expect("dirs::document_dir() failed");
        Self(dir)
    }
}

impl CanvasParentDir {
    fn into_canvas_dir(self) -> PathBuf {
        let mut dir = self.0;
        dir.push("CanvasFileSync");
        dir
    }
}

impl Display for CanvasParentDir {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.display().fmt(f)
    }
}

#[derive(StructOpt, Debug)]
pub struct AddUser {
    #[structopt(long, env = "CANVAS_ACCESS_TOKEN", hide_env_values = true)]
    access_token: String,
    search: Vec<String>,
}

#[derive(StructOpt, Debug)]
pub enum Command {
    Add(AddUser),
}

#[derive(StructOpt, Debug)]
#[structopt(
author = env ! ("CARGO_PKG_AUTHORS"),
about = env ! ("CARGO_PKG_DESCRIPTION"),
)]
pub struct Args {
    #[structopt(long, env = "CANVAS_PARENT_DIR", parse(from_os_str), default_value)]
    dir: CanvasParentDir,
    #[structopt(long)]
    skip_git: bool,
    #[structopt(subcommand)]
    command: Option<Command>,
}

impl From<AddUser> for canvas_file_sync::AddUser {
    fn from(it: AddUser) -> Self {
        let AddUser {
            access_token,
            search,
        } = it;
        let search = search.into_iter().join(" ");
        Self {
            access_token,
            search,
        }
    }
}

impl Args {
    pub fn run(self) -> Result<(), Box<dyn Error>> {
        let Self {
            dir,
            skip_git,
            command,
        } = self;
        let dir = dir.into_canvas_dir();
        let api = CanvasFileSync {
            dir,
            skip_git,
        };
        match command {
            Some(Command::Add(add_user)) =>
                api.add_user(add_user.into(), select_canvas_using_skim)?,
            None =>
                api.sync()?,
        }
        Ok(())
    }
}
