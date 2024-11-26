use std::{ffi::OsString, path::PathBuf};

use crate::Exo;

#[derive(Debug)]
pub struct MDFile {
    pub(crate) path: PathBuf,
    pub(crate) chapter: OsString,
    pub(crate) parsed_exo: Option<Result<Exo, String>>, // exo can be parsed or not
}
