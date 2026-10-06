//! This module models es data.

use std::{
    io::{Read},
    path::Path,
};

use crate::{error::ApplicationError, esdata::{sf01decomp::{sf01_compile, sf01_decompile}, sf01header::Sf01Header}};

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

/// A container for the underlying data structure.
pub enum EsContainer {
    Sf01Container { id: String, header: Sf01Header },
}

impl EsContainer {
    /// Compiles the container into an artifact.
    /// 
    /// # Parameters
    /// 
    /// * `artifact_directory` - the directory to compile the container to
    pub fn compile<P: AsRef<Path>>(&self, artifact_directory: P) -> Result<(), ApplicationError> {
        match self {
            EsContainer::Sf01Container { id, header } => {
                sf01_compile(artifact_directory, id, header)
            },
        }
    }

    /// Decompiles the container into a source project.
    /// 
    /// # Parameters
    /// 
    /// * `source_directory` - the directory to decompile the container into
    pub fn decompile<P: AsRef<Path>>(&self, source_directory: P) -> Result<(), ApplicationError> {
        match self {
            EsContainer::Sf01Container { id, header } => sf01_decompile(source_directory, id, header),
        }
    }
}

pub mod sf01header;
pub mod sf01decomp;
