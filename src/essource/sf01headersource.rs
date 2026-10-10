//! This module models sf01 header soruce code.

use std::{borrow::Borrow, path::Path};

use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{
    error::ApplicationError,
    esdata::sf01header::{Sf01Header, Sf01HeaderFlags},
    utils::NullTerminatedString,
};

#[skip_serializing_none]
#[derive(Debug, Getters, CopyGetters, Deserialize, Serialize)]
pub struct Sf01HeaderSource {
    #[getset(get = "pub")]
    id: String,
    #[getset(get_copy = "pub")]
    base_flags: Option<u32>,
    #[getset(get_copy = "pub")]
    localised: Option<bool>,
    #[getset(get_copy = "pub")]
    output_type: Option<Output_Type>,
    #[getset(get_copy = "pub")]
    form_id: Option<u32>,
    #[getset(get_copy = "pub")]
    version_control_1: Option<u32>,
    #[getset(get_copy = "pub")]
    form_version: Option<u16>,
    #[getset(get_copy = "pub")]
    version_control_2: Option<u16>,
    #[getset(get = "pub")]
    author: Option<NullTerminatedString>,
    #[getset(get = "pub")]
    description: Option<NullTerminatedString>,
}

impl Sf01HeaderSource {
    /// Name of the source file name.
    pub const SOURCE_FILE_NAME: &str = "header.yaml";

    pub fn from_compiled<S: Into<String>, H: Borrow<Sf01Header>>(id: S, header: H) -> Self {
        let header = header.borrow();
        let flags = header.flags();

        Self {
            id: id.into(),
            base_flags: if flags.is_non_canonical() {
                Some(flags.flags())
            } else {
                None
            },
            localised: if flags.localised() { Some(true) } else { None },
            output_type: flags.into(),
            form_id: Some(header.form_id()),
            version_control_1: Some(header.version_control_1()),
            form_version: Some(header.form_version()),
            version_control_2: Some(header.version_control_2()),
            author: header.author().clone(),
            description: header.description().clone(),
        }
    }

    /// Serialises the header file to the target directory.
    ///
    /// # Parameters
    ///
    /// * `target_dir` - the target directory
    pub fn serialise<T: AsRef<Path>>(&self, target_dir: T) -> Result<(), ApplicationError> {
        std::fs::create_dir_all(&target_dir).map_err(|err| {
            ApplicationError::from(err).chain(format!(
                "Failed to create source directory to serialise SF01 header to {}.",
                target_dir.as_ref().display()
            ))
        })?;

        let file_path = target_dir.as_ref().join(Self::SOURCE_FILE_NAME);
        let header_file = std::fs::File::create(&file_path).map_err(|err| {
            ApplicationError::from(err).chain(format!(
                "Failed to create SF01 header source file at {}.",
                file_path.display()
            ))
        })?;
        yaml_serde::to_writer(header_file, self).map_err(|err| {
            ApplicationError::from(err).chain(format!(
                "Failed to serialise SF01 header source to {}.",
                file_path.display()
            ))
        })
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Output_Type {
    DEVELOPMENT,
    MASTER_FULL,
    MASTER_MEDIUM,
    MASTER_SMALL,
    MASTER_BLUEPRINT,
}

impl From<Sf01HeaderFlags> for Option<Output_Type> {
    fn from(flags: Sf01HeaderFlags) -> Self {

        Some(if flags.master() {
            if flags.blueprint() && !flags.medium_master() && !flags.small_master() {
             Output_Type::MASTER_BLUEPRINT
            } else if flags.small_master() && !flags.blueprint() && !flags.medium_master() {
                Output_Type::MASTER_SMALL
            } else if flags.medium_master()  && !flags.blueprint() && !flags.small_master(){
                Output_Type::MASTER_MEDIUM
            } else if !flags.blueprint() && !flags.medium_master() && !flags.small_master() {
                Output_Type::MASTER_FULL
            } else {
                return None;
            }
        } else {
            Output_Type::DEVELOPMENT
        })
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_serialise() {

//     }
// }
