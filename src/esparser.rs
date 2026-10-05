//! This module parses es data.

use std::{fs::File, io::BufReader, path::Path};

use log::info;

use crate::{error::ApplicationError, esdata::sf01header::Sf01Header};

pub enum EsParser {
    SF01Parser,
}

impl EsParser {
    pub fn parse_es<T: AsRef<Path>>(&self, path: T) -> Result<(), ApplicationError> {
        match self {
            EsParser::SF01Parser => parse_sf01(path),
        }
    }
}

pub fn parse_sf01<T: AsRef<Path>>(path: T) -> Result<(), ApplicationError> {
    log::info!("Parsing {} with sf01 flavour.", path.as_ref().display());
    let mut file = BufReader::new(File::open(&path).map_err(|err| {
        ApplicationError::from(err)
            .chain(format!("The input file \"{}\" could not be opened.", path.as_ref().display()))
    })?);
    let header = Sf01Header::read(&mut file)?;
    info!("{:?}", header);
    Ok(())
}