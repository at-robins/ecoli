//! This module parses es data.

use std::{fs::File, io::BufReader, path::Path};

use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    error::ApplicationError,
    esdata::{
        EsContainer,
        sf01group::{Sf01GenericGroup, Sf01Groups},
        sf01header::Sf01Header,
    },
    essource::{sf01groupsource::Sf01GroupSource, sf01headersource::Sf01HeaderSource},
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
    let path = path.as_ref();
    log::info!("Compiling {} with SF01 flavour.", path.display());
    log::info!("Compiling header...");
    let start_time_header = std::time::Instant::now();
    let header_path = path.join(Sf01HeaderSource::SOURCE_FILE_NAME);
    let header_file = BufReader::new(File::open(&header_path).map_err(|err| {
        ApplicationError::from(err)
            .chain(format!("The SF01 header file \"{}\" could not be opened.", path.display()))
    })?);
    let header_source: Sf01HeaderSource = yaml_serde::from_reader(header_file).map_err(|err| {
        ApplicationError::from(err)
            .chain(format!("The SF01 header file \"{}\" could not be parsed.", path.display()))
    })?;

    let header_compiled: Sf01Header = Sf01Header::from_source(&header_source).map_err(|err| {
        ApplicationError::from(err).chain(format!(
            "The SF01 header file \"{}\" could not be converted from source representation.",
            path.display()
        ))
    })?;
    log::info!(
        "Compiled header in {} ms.",
        (std::time::Instant::now() - start_time_header).as_millis()
    );
    log::info!("Compiling groups...");
    let start_time_groups = std::time::Instant::now();
    let mut groups: Vec<Sf01GenericGroup> = Vec::new();
    let entries: Vec<Result<walkdir::DirEntry, walkdir::Error>> =
        walkdir::WalkDir::new(path).into_iter().collect();
    let group_results: Vec<Result<Option<Sf01GenericGroup>, ApplicationError>> =
        entries.into_par_iter().map(entry_to_group).collect();

    for group_result in group_results {
        if let Some(group) = group_result? {
            groups.push(group);
        }
    }
    log::info!(
        "Compiled groups in {} ms.",
        (std::time::Instant::now() - start_time_groups).as_millis()
    );
    Ok(EsContainer::Sf01Container {
        id: header_source.id().to_string(),
        header: header_compiled,
        groups: Sf01Groups::new(groups),
    })
}

/// Helper function to allow parallelisation.
fn entry_to_group(
    entry: Result<walkdir::DirEntry, walkdir::Error>,
) -> Result<Option<Sf01GenericGroup>, ApplicationError> {
    let entry = entry.map_err(|err| {
        ApplicationError::from(err)
            .chain(format!("Failed to read entry while inspecting the source directory."))
    })?;
    if let Some(file_extionsion) = entry.path().extension()
        && file_extionsion == Sf01GroupSource::GROUP_SOURCE_FILE_EXTENSION
    {
        let group_file = BufReader::new(File::open(entry.path()).map_err(|err| {
            ApplicationError::from(err).chain(format!(
                "The SF01 group file \"{}\" could not be opened.",
                entry.path().display()
            ))
        })?);
        log::info!("Compiling group {}...", entry.path().display());
        let start_time_group = std::time::Instant::now();
        let group_source: Sf01GroupSource = yaml_serde::from_reader(group_file).map_err(|err| {
            ApplicationError::from(err).chain(format!(
                "The SF01 group file \"{}\" could not be parsed.",
                entry.path().display()
            ))
        })?;
        log::info!(
            "Compiled group {} in {} ms.",
            entry.path().display(),
            (std::time::Instant::now() - start_time_group).as_millis()
        );
        Ok(Some(Sf01GenericGroup::from_source(group_source)?))
    } else {
        Ok(None)
    }
}
