//! This module parses es data.

use std::{fs::File, io::BufReader, path::Path};

use crate::{
    error::ApplicationError,
    esdata::{EsContainer, sf01header::Sf01Header},
    essource::sf01headersource::Sf01HeaderSource,
};

pub enum EsCompiler {
    SF01Compiler,
}

impl EsCompiler {
    pub fn compile_es<T: AsRef<Path>>(&self, path: T) -> Result<EsContainer, ApplicationError> {
        match self {
            EsCompiler::SF01Compiler => compile_sf01(path),
        }
    }
}

pub fn compile_sf01<T: AsRef<Path>>(path: T) -> Result<EsContainer, ApplicationError> {
    log::info!("Compiling {} with SF01 flavour.", path.as_ref().display());
    let header_path = path.as_ref().join(Sf01HeaderSource::SOURCE_FILE_NAME);
    let header_file = BufReader::new(File::open(&header_path).map_err(|err| {
        ApplicationError::from(err).chain(format!(
            "The SF01 header file \"{}\" could not be opened.",
            path.as_ref().display()
        ))
    })?);
    let header_source: Sf01HeaderSource = yaml_serde::from_reader(header_file).map_err(|err| {
        ApplicationError::from(err).chain(format!(
            "The SF01 header file \"{}\" could not be parsed.",
            path.as_ref().display()
        ))
    })?;

    let header_compiled: Sf01Header = Sf01Header::from_source(&header_source).map_err(|err| {
        ApplicationError::from(err).chain(format!(
            "The SF01 header file \"{}\" could not be converted from source representation.",
            path.as_ref().display()
        ))
    })?;
    Ok(EsContainer::Sf01Container {
        id: header_source.id().to_string(),
        header: header_compiled,
    })
}
