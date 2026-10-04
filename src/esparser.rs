//! This module parses es data.

use std::{fs::File, path::Path};

use crate::error::ApplicationError;

pub fn parse_es<T: AsRef<Path>>(path: T) -> Result<(), ApplicationError> {
    log::info!("Parsing {}.", path.as_ref().display());
    let file = File::open(&path).map_err(|err| {
        ApplicationError::from(err)
            .chain(format!("The input file \"{}\" could not be opened.", path.as_ref().display()))
    })?;
    Ok(())
}
