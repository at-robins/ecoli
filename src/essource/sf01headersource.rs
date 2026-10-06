//! This module models sf01 header soruce code.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::esdata::sf01header::{Sf01Header, Sf01HeaderFlags};

#[derive(Debug, Deserialize, Serialize)]
pub struct Sf01HeaderSource {
    flags: Sf01HeaderFlagsSource,
    form_id: Option<u32>,
    version_control_1: Option<u32>,
    form_version: Option<u16>,
    version_control_2: Option<u16>,
}

impl From<Sf01Header> for Sf01HeaderSource {
    fn from(value: Sf01Header) -> Self {
        Self {
            flags: value.flags().into(),
            form_id: Some(value.form_id()),
            version_control_1: Some(value.version_control_1()),
            form_version: Some(value.form_version()),
            version_control_2: Some(value.version_control_2()),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Sf01HeaderFlagsSource {
    base_flags: Option<u32>,
    full_master: bool,
    medium_master: bool,
    small_master: bool,
    localised: bool,
    blueprint: bool,
}

impl Sf01HeaderSource {
    pub fn serialise<T: AsRef<Path>>(&self, target_dir: T) {
        std::fs::create_dir_all(&target_dir).unwrap();

        yaml_serde::to_writer(
            std::fs::File::create(target_dir.as_ref().join("header.yaml")).unwrap(),
            self,
        );
    }
}

impl From<Sf01HeaderFlags> for Sf01HeaderFlagsSource {
    fn from(value: Sf01HeaderFlags) -> Self {
        Self {
            base_flags: Some(value.flags()),
            full_master: value.full_master(),
            medium_master: value.medium_master(),
            small_master: value.small_master(),
            localised: value.localised(),
            blueprint: value.blueprint(),
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
