use canvas_file_sync::download::data::{CanvasBase, IdName};
use std::error::Error;
use std::process::{Command, Stdio};
use std::io::Write;
use itertools::Itertools;
use std::fmt::Display;
use serde::export::Formatter;
use std::fmt;
use std::convert::{TryFrom, TryInto};
use canvas_file_sync::api::CoreApi;

struct DisplayCanvas {
    name: String,
    domain: String,
}

impl PartialEq<CanvasBase> for DisplayCanvas {
    fn eq(&self, other: &CanvasBase) -> bool {
        self.name == other.id.name && self.name == other.api.domain
    }
}

impl From<CanvasBase> for DisplayCanvas {
    fn from(canvas: CanvasBase) -> Self {
        let CanvasBase {
            id: IdName {
                id: _,
                name,
            },
            api: CoreApi {
                domain,
                authorization: _,
            },
        } = canvas;
        Self {
            name,
            domain,
        }
    }
}

// could use serde for this but more complicated

const SEP: &str = " & ";

impl Display for DisplayCanvas {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}", self.name, SEP, self.domain)
    }
}

impl TryFrom<&str> for DisplayCanvas {
    type Error = &'static str;
    
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut split = value.split(SEP);
        let name = split.next().ok_or("no name")?.into();
        let domain = split.next().ok_or("no domain")?.into();
        let this = Self {name, domain};
        Ok(this)
    }
}

pub fn select_canvas_using_skim(canvases: Vec<CanvasBase>) -> Result<CanvasBase, Box<dyn Error>> {
    let input = canvases.iter()
        .map(|it| format!("{}", it))
        .join("\0");
    let mut child = Command::new("sk")
        .args(&["--read0", "--print0", "--no-multi"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    // TODO handle when skim is not installed
    child.stdin
        .as_mut()
        .ok_or("skim has no stdin")?
        .write_all(input.as_bytes())?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        let error_message = std::str::from_utf8(&*output.stderr)?;
        return Err(error_message.into());
    }
    let chosen: DisplayCanvas = std::str::from_utf8(&*output.stdout)?
        .split('\0')
        .next().unwrap()
        .try_into()?;
    let chosen = canvases.into_iter()
        .find(|it| chosen == *it)
        .ok_or("nothing printed by skim")?;
    Ok(chosen)
}
