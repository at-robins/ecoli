//! This module models sf01 header soruce code.

use std::{borrow::Borrow, path::Path};

use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

use crate::{
    error::ApplicationError,
    esdata::sf01header::{Sf01Header, Sf01HeaderFlags},
    essource::sf01recordsource::Sf01GenericRecordSource,
};

#[derive(Debug, Getters, CopyGetters, Deserialize, Serialize)]
pub struct Sf01HeaderSource {
    #[getset(get = "pub")]
    id: String,
    #[getset(get = "pub")]
    #[serde(default)]
    flags: Sf01HeaderFlagsSource,
    #[getset(get_copy = "pub")]
    form_id: Option<u32>,
    #[getset(get_copy = "pub")]
    version_control_1: Option<u32>,
    #[getset(get_copy = "pub")]
    form_version: Option<u16>,
    #[getset(get_copy = "pub")]
    version_control_2: Option<u16>,
    #[getset(get = "pub")]
    #[serde(default)]
    records: Vec<Sf01GenericRecordSource>,
}

impl Sf01HeaderSource {
    /// Name of the source file name.
    pub const SOURCE_FILE_NAME: &str = "header.yaml";

    pub fn from_compiled<S: Into<String>, H: Borrow<Sf01Header>>(id: S, header: H) -> Self {
        let header = header.borrow();
        Self {
            id: id.into(),
            flags: header.flags().into(),
            form_id: Some(header.form_id()),
            version_control_1: Some(header.version_control_1()),
            form_version: Some(header.form_version()),
            version_control_2: Some(header.version_control_2()),
            records: header
                .records()
                .into_iter()
                .map(Sf01GenericRecordSource::from_compiled)
                .collect(),
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

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Sf01HeaderFlagsSource {
    base_flags: Option<u32>,
    full_master: Option<bool>,
    medium_master: Option<bool>,
    small_master: Option<bool>,
    localised: Option<bool>,
    blueprint: Option<bool>,
}

impl Sf01HeaderFlagsSource {
    pub fn base_flags(&self) -> u32 {
        self.base_flags.unwrap_or(0)
    }

    pub fn full_master(&self) -> bool {
        self.full_master.unwrap_or(false)
    }

    pub fn medium_master(&self) -> bool {
        self.medium_master.unwrap_or(false)
    }

    pub fn small_master(&self) -> bool {
        self.small_master.unwrap_or(false)
    }

    pub fn localised(&self) -> bool {
        self.localised.unwrap_or(false)
    }

    pub fn blueprint(&self) -> bool {
        self.blueprint.unwrap_or(false)
    }
}

impl From<Sf01HeaderFlags> for Sf01HeaderFlagsSource {
    fn from(value: Sf01HeaderFlags) -> Self {
        Self {
            base_flags: Some(value.flags()),
            full_master: Some(value.full_master()),
            medium_master: Some(value.medium_master()),
            small_master: Some(value.small_master()),
            localised: Some(value.localised()),
            blueprint: Some(value.blueprint()),
        }
    }
}

impl Default for Sf01HeaderFlagsSource {
    fn default() -> Self {
        Sf01HeaderFlagsSource {
            base_flags: Some(0),
            full_master: None,
            medium_master: None,
            small_master: None,
            localised: None,
            blueprint: None,
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_serialise() {
//         let test_data = Sf01HeaderFlagsSource {
//             base_flags: 32,
//             full_master: true,
//             medium_master: false,
//             small_master: false,
//             localised: true,
//             blueprint: false,
//         };
//         test_data.serialise(".");
//     }
// }
