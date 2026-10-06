//! This module contains SF01 related compilation and decompilation logic.

use std::{
    borrow::Borrow,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use crate::{
    error::ApplicationError,
    esdata::{EsEntityIO, sf01header::Sf01Header},
    essource::sf01headersource::Sf01HeaderSource,
};

/// Compiles the project.
///
/// # Parameters
///
/// * `artifact_directory` - the directory to compile the project to
/// * `id` - the ID to use as artifact name
/// * `header` - the header information
pub fn sf01_compile<S: AsRef<str>, P: AsRef<Path>, H: Borrow<Sf01Header>>(
    artifact_directory: P,
    id: S,
    header: H,
) -> Result<(), ApplicationError> {
    std::fs::create_dir_all(&artifact_directory).map_err(|err| {
        ApplicationError::from(err).chain(format!(
            "Failed to create SF01 build directory {}.",
            artifact_directory.as_ref().display()
        ))
    })?;
    let mut artifact_path = artifact_directory.as_ref().join(id.as_ref());
    artifact_path.set_extension(header.borrow().get_file_extension());
    let mut artifact = BufWriter::new(File::create(&artifact_path).map_err(|err| {
        ApplicationError::from(err)
            .chain(format!("Failed to create SF01 artifact at {}.", artifact_path.display()))
    })?);
    artifact
        .write_all(&header.borrow().serialise())
        .map_err(|err| {
            ApplicationError::from(err)
                .chain(format!("Failed to write SF01 header to {}.", artifact_path.display()))
        })?;
    Ok(())
}

/// Decompiles the project.
///
/// # Parameters
///
/// * `source_directory` - the directory to decompile the container to
/// * `id` - the ID of the container
/// * `header` - the header information
pub fn sf01_decompile<S: AsRef<str>, P: AsRef<Path>, H: Borrow<Sf01Header>>(
    source_directory: P,
    id: S,
    header: H,
) -> Result<(), ApplicationError> {
    std::fs::create_dir_all(&source_directory).map_err(|err| {
        ApplicationError::from(err).chain(format!(
            "Failed to create SF01 project directory {}.",
            source_directory.as_ref().display()
        ))
    })?;
    Sf01HeaderSource::from_compiled(id.as_ref(), header.borrow())
        .serialise(&source_directory)
        .map_err(|err| {
            ApplicationError::from(err).chain(format!(
                "Failed to create SF01 header source file in {}.",
                source_directory.as_ref().display()
            ))
        })?;

    Ok(())
}
