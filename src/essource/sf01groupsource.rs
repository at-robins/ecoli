//! This module models sf01 group soruce code.

use std::{borrow::Borrow, path::Path};

use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{
    error::{ApplicationError, ApplicationErrorType},
    esdata::{EsEntityIO, sf01group::Sf01GenericGroup},
};

#[skip_serializing_none]
#[derive(Debug, Getters, CopyGetters, Deserialize, Serialize)]
pub struct Sf01GroupSource {
    #[getset(get = "pub")]
    content: String,
}

impl Sf01GroupSource {
    pub fn from_compiled<G: Borrow<Sf01GenericGroup>>(group: G) -> Self {
        let group = group.borrow();
        Self {
            content: base122_rs::encode(&group.serialise()),
        }
    }

    /// Serialises the group file.
    ///
    /// # Parameters
    ///
    /// * `target_file` - the target file path
    pub fn serialise<T: AsRef<Path>>(&self, target_file: T) -> Result<(), ApplicationError> {
        let target_file = target_file.as_ref();
        std::fs::create_dir_all(target_file.parent().ok_or(ApplicationError::new(
            ApplicationErrorType::InternalError,
            format!(
                "Target file path for SF01 group source serialisation {} has no parent directory.",
                target_file.display()
            ),
        ))?)
        .map_err(|err| {
            ApplicationError::from(err).chain(format!(
                "Failed to create source directory to serialise SF01 group to {}.",
                target_file.display()
            ))
        })?;

        let group_file = std::fs::File::create(&target_file).map_err(|err| {
            ApplicationError::from(err).chain(format!(
                "Failed to create SF01 group source file at {}.",
                target_file.display()
            ))
        })?;
        yaml_serde::to_writer(group_file, self).map_err(|err| {
            ApplicationError::from(err).chain(format!(
                "Failed to serialise SF01 group source to {}.",
                target_file.display()
            ))
        })
    }
}
