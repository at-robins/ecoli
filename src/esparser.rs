//! This module parses es data.

use std::{fs::File, io::{BufReader, Read}, path::Path};

use crate::{
    error::ApplicationError, esdata::{EsContainer, EsEntityIO, sf01group::{Sf01GenericGroup, Sf01Groups}, sf01header::Sf01Header},
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
    log::info!("Parsing {} with SF01 flavour.", path.as_ref().display());
    if let Some(file_name) = path.as_ref().file_stem()
        && path.as_ref().is_file()
    {
        let mut file = BufReader::new(File::open(&path).map_err(|err| {
            ApplicationError::from(err)
                .chain(format!("The input file \"{}\" could not be opened.", path.as_ref().display()))
        })?);
        // Reading the whole file into memory makes the parsing 10x faster.
        // File sizes should be less than 2 GB, so the increase in speed
        // is worth the memory trade-off.
        let mut file_buf = Vec::new();
        file.read_to_end(&mut file_buf)?;
        let mut file = std::io::Cursor::new(file_buf);
        log::info!("Parsing header...");
        let mut start_time = std::time::Instant::now();
        let header = Sf01Header::read(&mut file)?;
        log::info!("Parsed header in {} ms.", (std::time::Instant::now() - start_time).as_millis());
        log::info!("Parsing groups...");
        start_time = std::time::Instant::now();
        let groups = Sf01Groups::read(&mut file)?;
        log::info!("Parsed groups in {} ms.", (std::time::Instant::now() - start_time).as_millis());
        Ok(EsContainer::Sf01Container { id: file_name.to_string_lossy().to_string(), header })
    } else {
        return Err(ApplicationError::new(
            crate::error::ApplicationErrorType::InputDataError,
            format!("Input path \"{}\" does not point to a file.", path.as_ref().display()),
        ));
    }
}
