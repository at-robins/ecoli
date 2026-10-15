//! This module models es data.

use std::{
    io::{Read, Seek},
    path::Path,
};

use crate::{
    error::ApplicationError,
    esdata::{
        sf01decomp::{sf01_compile, sf01_decompile},
        sf01group::Sf01Groups,
        sf01header::Sf01Header,
    },
};

/// An entity that can be serialised to deserialised from binary data.
pub trait EsEntityIO {
    /// Tries to read the entity from a [`Read`]er.
    ///
    /// # Parameters
    ///
    /// * ``reader` - the reader to parse data from
    fn read<T: Read + Seek>(reader: &mut T) -> Result<Self, ApplicationError>
    where
        Self: Sized;

    /// Serialises the entity to binary data.
    fn serialise(&self) -> Vec<u8>;
}

/// A container for the underlying data structure.
pub enum EsContainer {
    Sf01Container {
        id: String,
        header: Sf01Header,
        groups: Sf01Groups,
    },
}

impl EsContainer {
    /// Compiles the container into an artifact.
    ///
    /// # Parameters
    ///
    /// * `artifact_directory` - the directory to compile the container to
    pub fn compile<P: AsRef<Path>>(
        &mut self,
        artifact_directory: P,
    ) -> Result<(), ApplicationError> {
        match self {
            EsContainer::Sf01Container { id, header, groups } => {
                header.set_record_count(groups.record_count());
                header.set_interior_cell_count(groups.interior_cell_count().map_err(|err| {
                    err.chain(format!(
                        "Failed to set interior cell count during compilation of {}.",
                        artifact_directory.as_ref().display()
                    ))
                })?);
                header.set_dependencies(todo!());
                sf01_compile(artifact_directory, id, header, groups)
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
            EsContainer::Sf01Container { id, header, groups } => {
                sf01_decompile(source_directory, id, header, groups)
            },
        }
    }
}

pub mod sf01decomp;
pub mod sf01group;
pub mod sf01header;
pub mod sf01record;
