//! This module models es data.

use std::{
    borrow::Borrow,
    fs::File,
    io::{BufWriter, Read, Write},
    path::Path,
};

use crate::{error::ApplicationError, esdata::sf01header::Sf01Header};

/// An entity that can be serialised to deserialised from binary data.
pub trait EsEntityIO {
    /// Tries to read the entity from a [`Read`]er.
    ///
    /// # Parameters
    ///
    /// * ``reader` - the reader to parse data from
    fn read<T: Read>(reader: &mut T) -> Result<Self, ApplicationError>
    where
        Self: Sized;

    /// Serialises the entity to binary data.
    fn serialise(&self) -> Vec<u8>;
}

pub trait EsDecompiler {}

/// A container for the underlying data structure.
pub enum EsContainer {
    Sf01Container { id: String, header: Sf01Header },
}

impl EsContainer {
    pub fn compile<P: AsRef<Path>>(&self, artifact_directory: P) -> Result<(), ApplicationError> {
        match self {
            EsContainer::Sf01Container { id, header } => {
                sf01_compile(artifact_directory, id, header)
            },
        }
    }

    pub fn decompile(&self) {
        match self {
            EsContainer::Sf01Container { id, header } => todo!(),
        }
    }
}

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

pub mod sf01header;
// pub mod sf01decomp;
