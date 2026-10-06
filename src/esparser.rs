//! This module parses es data.

use std::{fs::File, io::BufReader, path::Path};

use crate::{
    error::ApplicationError,
    esdata::{EsContainer, EsEntityIO, sf01header::Sf01Header},
};

pub enum EsParser {
    SF01Parser,
}

impl EsParser {
    pub fn parse_es<T: AsRef<Path>>(&self, path: T) -> Result<EsContainer, ApplicationError> {
        match self {
            EsParser::SF01Parser => parse_sf01(path),
        }
    }
}

pub fn parse_sf01<T: AsRef<Path>>(path: T) -> Result<EsContainer, ApplicationError> {
    log::info!("Parsing {} with sf01 flavour.", path.as_ref().display());
    if let Some(file_name) = path.as_ref().file_stem()
        && path.as_ref().is_file()
    {
        let mut file = BufReader::new(File::open(&path).map_err(|err| {
            ApplicationError::from(err)
                .chain(format!("The input file \"{}\" could not be opened.", path.as_ref().display()))
        })?);
        let header = Sf01Header::read(&mut file)?;
        Ok(EsContainer::Sf01Container { id: file_name.to_string_lossy().to_string(), header })
    } else {
        return Err(ApplicationError::new(
            crate::error::ApplicationErrorType::InputDataError,
            format!("Input path {} does not point to a file.", path.as_ref().display()),
        ));
    }
}
