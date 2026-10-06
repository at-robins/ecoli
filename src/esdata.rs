//! This module models es data.

use std::io::Read;

use crate::{error::ApplicationError, esdata::sf01header::Sf01Header};

/// An entity that can be serialised to deserialised from binary data.
pub trait EsEntityIO {
    /// Tries to read the entity from a [`Read`]er.
    /// 
    /// # Parameters
    /// 
    /// * ``reader` - the reader to parse data from
    fn read<T: Read>(reader: &mut T) -> Result<Self, ApplicationError> where Self: Sized;

    /// Serialises the entity to binary data.
    fn serialise(&self) -> Vec<u8>;
}

pub trait EsDecompiler {

}

/// A container for the underlying data structure.
pub enum EsContainer {
    Sf01Container{id: String, header: Sf01Header,},
}

pub mod sf01header;