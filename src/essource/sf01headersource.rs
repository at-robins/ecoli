//! This module models sf01 header soruce code.

use std::{borrow::Borrow, path::Path};

use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{
    error::ApplicationError,
    esdata::sf01header::{
        HEADER_FIELD_DEFAULT_FORM_ID, HEADER_FIELD_DEFAULT_FORM_VERSION,
        HEADER_FIELD_DEFAULT_VERSION_CONTROL_1, HEADER_FIELD_DEFAULT_VERSION_CONTROL_2, Sf01Header,
        Sf01HeaderFlags,
    },
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
    output_type: Option<OutputType>,
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
            form_id: if header.form_id() != HEADER_FIELD_DEFAULT_FORM_ID {
                Some(header.form_id())
            } else {
                None
            },
            version_control_1: if header.version_control_1()
                != HEADER_FIELD_DEFAULT_VERSION_CONTROL_1
            {
                Some(header.version_control_1())
            } else {
                None
            },
            form_version: if header.form_version() != HEADER_FIELD_DEFAULT_FORM_VERSION {
                Some(header.form_version())
            } else {
                None
            },
            version_control_2: if header.version_control_2()
                != HEADER_FIELD_DEFAULT_VERSION_CONTROL_2
            {
                Some(header.version_control_2())
            } else {
                None
            },
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
pub enum OutputType {
    #[serde(rename = "development")]
    Development,
    #[serde(rename = "full master")]
    MasterFull,
    #[serde(rename = "medium master")]
    MasterMedium,
    #[serde(rename = "small master")]
    MasterSmall,
    #[serde(rename = "blueprint")]
    MasterBlueprint,
}

impl From<Sf01HeaderFlags> for Option<OutputType> {
    fn from(flags: Sf01HeaderFlags) -> Self {
        Some(if flags.master() {
            if flags.blueprint() && !flags.medium_master() && !flags.small_master() {
                OutputType::MasterBlueprint
            } else if flags.small_master() && !flags.blueprint() && !flags.medium_master() {
                OutputType::MasterSmall
            } else if flags.medium_master() && !flags.blueprint() && !flags.small_master() {
                OutputType::MasterMedium
            } else if !flags.blueprint() && !flags.medium_master() && !flags.small_master() {
                OutputType::MasterFull
            } else {
                return None;
            }
        } else {
            OutputType::Development
        })
    }
}

impl std::fmt::Display for OutputType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            OutputType::Development => "development",
            OutputType::MasterFull => "full master",
            OutputType::MasterMedium => "medium master",
            OutputType::MasterSmall => "small master",
            OutputType::MasterBlueprint => "blueprint",
        };
        write!(f, "{}", name)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_serialise() {

//     }
// }
