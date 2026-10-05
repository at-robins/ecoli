//! This module models sf01 header data.

use std::io::Read;

use crate::{
    error::{ApplicationError, ApplicationErrorType},
    utils::{is_bit_set_u32, set_bit_u32},
};

/// The sf01 header ID.
const SF01_HEADER_ID: [u8; 4] = [84, 69, 83, 52];

#[derive(Debug)]
pub struct Sf01Header {
    size: u32,
    flags: Sf01HeaderFlags,
    form_id: u32,
    version_control_1: u32,
    form_version: u16,
    version_control_2: u16,
}

impl Sf01Header {
    pub fn read<T: Read>(reader: &mut T) -> Result<Self, ApplicationError> {
        let mut buffer_32: [u8; 4] = [0; 4];
        let mut buffer_16: [u8; 2] = [0; 2];

        reader.read_exact(&mut buffer_32).map_err(|err| {
            ApplicationError::from(err).chain("Failed to read SF01 header ID.")
        })?;
        if buffer_32 != SF01_HEADER_ID {
            return Err(ApplicationError::new(
                ApplicationErrorType::InputDataError,
                format!(
                    "Failed to parse input data as SF01 header record. ID sequence {:?} does not mathc {:?}.",
                    buffer_32, SF01_HEADER_ID
                ),
            ));
        }

        reader
            .read_exact(&mut buffer_32)
            .map_err(|err| ApplicationError::from(err).chain("Failed to read SF01 header size."))?;
        let size = u32::from_le_bytes(buffer_32);

        reader.read_exact(&mut buffer_32).map_err(|err| {
            ApplicationError::from(err).chain("Failed to read SF01 header flags.")
        })?;
        let flags = u32::from_le_bytes(buffer_32).into();

        reader.read_exact(&mut buffer_32).map_err(|err| {
            ApplicationError::from(err).chain("Failed to read SF01 header form ID.")
        })?;
        let form_id = u32::from_le_bytes(buffer_32);

        reader.read_exact(&mut buffer_32).map_err(|err| {
            ApplicationError::from(err).chain("Failed to read SF01 header version control 1.")
        })?;
        let version_control_1 = u32::from_le_bytes(buffer_32);

        reader.read_exact(&mut buffer_16).map_err(|err| {
            ApplicationError::from(err).chain("Failed to read SF01 header form version.")
        })?;
        let form_version = u16::from_le_bytes(buffer_16);

        reader.read_exact(&mut buffer_16).map_err(|err| {
            ApplicationError::from(err).chain("Failed to read SF01 header version control 2.")
        })?;
        let version_control_2 = u16::from_le_bytes(buffer_16);

        Ok(Self {
            size,
            flags,
            form_id,
            version_control_1,
            form_version,
            version_control_2,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Sf01HeaderFlags {
    flags: u32,
}

impl Sf01HeaderFlags {
    const INDEX_MASTER_FULL: u32 = 0;
    const INDEX_LOCALISED: u32 = 7;
    const INDEX_MASTER_MEDIUM: u32 = 8;
    const INDEX_MASTER_SMALL: u32 = 10;
    const INDEX_BLUEPRINT: u32 = 11;

    /// Returns true if the full master flag is set.
    pub fn full_master(&self) -> bool {
        is_bit_set_u32(self.flags, Self::INDEX_MASTER_FULL)
    }

    /// Returns true if the medium master flag is set.
    pub fn medium_master(&self) -> bool {
        is_bit_set_u32(self.flags, Self::INDEX_MASTER_MEDIUM)
    }

    /// Returns true if the small master flag is set.
    pub fn small_master(&self) -> bool {
        is_bit_set_u32(self.flags, Self::INDEX_MASTER_SMALL)
    }

    /// Returns true if the localised flag is set.
    pub fn localised(&self) -> bool {
        is_bit_set_u32(self.flags, Self::INDEX_LOCALISED)
    }

    /// Returns true if the blueprint flag is set.
    pub fn blueprint(&self) -> bool {
        is_bit_set_u32(self.flags, Self::INDEX_BLUEPRINT)
    }

    /// Returns the header flags with the full master flag set to the specified value.
    ///
    /// # Parameters
    ///
    /// * `value` - the bit value to set
    pub fn set_full_master(&self, value: bool) -> Self {
        set_bit_u32(self.flags, Self::INDEX_MASTER_FULL, value).into()
    }

    /// Returns the header flags with the medium master flag set to the specified value.
    ///
    /// # Parameters
    ///
    /// * `value` - the bit value to set
    pub fn set_medium_master(&self, value: bool) -> Self {
        set_bit_u32(self.flags, Self::INDEX_MASTER_MEDIUM, value).into()
    }

    /// Returns the header flags with the small master flag set to the specified value.
    ///
    /// # Parameters
    ///
    /// * `value` - the bit value to set
    pub fn set_small_master(&self, value: bool) -> Self {
        set_bit_u32(self.flags, Self::INDEX_MASTER_SMALL, value).into()
    }

    /// Returns the header flags with the localised flag set to the specified value.
    ///
    /// # Parameters
    ///
    /// * `value` - the bit value to set
    pub fn set_localised(&self, value: bool) -> Self {
        set_bit_u32(self.flags, Self::INDEX_LOCALISED, value).into()
    }

    /// Returns the header flags with the blueprint flag set to the specified value.
    ///
    /// # Parameters
    ///
    /// * `value` - the bit value to set
    pub fn set_blueprint(&self, value: bool) -> Self {
        set_bit_u32(self.flags, Self::INDEX_BLUEPRINT, value).into()
    }
}

impl From<u32> for Sf01HeaderFlags {
    fn from(flags: u32) -> Self {
        Sf01HeaderFlags { flags }
    }
}
