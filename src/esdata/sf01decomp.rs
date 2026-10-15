//! This module contains SF01 related compilation and decompilation logic.

use std::{
    borrow::Borrow,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use crate::{
    error::ApplicationError,
    esdata::{EsEntityIO, sf01group::Sf01Groups, sf01header::Sf01Header},
    essource::{sf01groupsource::Sf01GroupSource, sf01headersource::Sf01HeaderSource},
};

/// Compiles the project.
///
/// # Parameters
///
/// * `artifact_directory` - the directory to compile the project to
/// * `id` - the ID to use as artifact name
/// * `header` - the header information
pub fn sf01_compile<S: AsRef<str>, P: AsRef<Path>, H: Borrow<Sf01Header>, G: Borrow<Sf01Groups>>(
    artifact_directory: P,
    id: S,
    header: H,
    groups: G,
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
    artifact
        .write_all(&groups.borrow().serialise())
        .map_err(|err| {
            ApplicationError::from(err)
                .chain(format!("Failed to write SF01 groups to {}.", artifact_path.display()))
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
pub fn sf01_decompile<
    S: AsRef<str>,
    P: AsRef<Path>,
    H: Borrow<Sf01Header>,
    G: Borrow<Sf01Groups>,
>(
    source_directory: P,
    id: S,
    header: H,
    groups: G,
) -> Result<(), ApplicationError> {
    std::fs::create_dir_all(&source_directory).map_err(|err| {
        ApplicationError::from(err).chain(format!(
            "Failed to create SF01 project directory {}.",
            source_directory.as_ref().display()
        ))
    })?;
    log::info!("Writing header source...");
    let start_time = std::time::Instant::now();
    Sf01HeaderSource::from_compiled(id.as_ref(), header.borrow())
        .serialise(&source_directory)
        .map_err(|err| {
            ApplicationError::from(err).chain(format!(
                "Failed to create SF01 header source file in {}.",
                source_directory.as_ref().display()
            ))
        })?;
    log::info!("Wrote header in {} ms.", (std::time::Instant::now() - start_time).as_millis());
    log::info!("Writing groups...");
    let start_time_groups = std::time::Instant::now();
    for group in groups.borrow().groups() {
        let mut file_path = source_directory
            .as_ref()
            .join(String::from_utf8_lossy(&group.label()).as_ref());
        file_path.set_extension(Sf01GroupSource::GROUP_SOURCE_FILE_EXTENSION);
        log::info!("Writing {}...", file_path.display());
        let start_time_group = std::time::Instant::now();

        Sf01GroupSource::from_compiled(group)
            .serialise(&file_path)
            .map_err(|err| {
                ApplicationError::from(err).chain(format!(
                    "Failed to create SF01 group source file at {}.",
                    file_path.display()
                ))
            })?;
        log::info!(
            "Wrote group in {} ms.",
            (std::time::Instant::now() - start_time_group).as_millis()
        );
    }
    log::info!(
        "Wrote groups in {} ms.",
        (std::time::Instant::now() - start_time_groups).as_millis()
    );
    Ok(())
}
