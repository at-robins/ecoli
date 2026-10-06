//! This module models sf01 header data.

use std::io::Read;

use getset::CopyGetters;

use crate::{
    error::{ApplicationError, ApplicationErrorType},
    esdata::EsEntityIO,
    utils::{is_bit_set_u32, set_bit_u32},
};

/// The sf01 header ID.
const SF01_HEADER_ID: [u8; 4] = [84, 69, 83, 52];

#[derive(Debug, CopyGetters)]
pub struct Sf01Header {
    #[getset(get_copy = "pub")]
    size: u32,
    #[getset(get_copy = "pub")]
    flags: Sf01HeaderFlags,
    #[getset(get_copy = "pub")]
    form_id: u32,
    #[getset(get_copy = "pub")]
    version_control_1: u32,
    #[getset(get_copy = "pub")]
    form_version: u16,
    #[getset(get_copy = "pub")]
    version_control_2: u16,
}

impl EsEntityIO for Sf01Header {
    fn read<T: Read>(reader: &mut T) -> Result<Self, ApplicationError> {
        let mut buffer_32: [u8; 4] = [0; 4];
        let mut buffer_16: [u8; 2] = [0; 2];

        reader
            .read_exact(&mut buffer_32)
            .map_err(|err| ApplicationError::from(err).chain("Failed to read SF01 header ID."))?;
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

    fn serialise(&self) -> Vec<u8> {
        let mut serialised_record = Vec::with_capacity(24);
        serialised_record.extend_from_slice(&SF01_HEADER_ID);
        serialised_record.extend(self.size.to_le_bytes());
        serialised_record.extend(self.flags.flags.to_le_bytes());
        serialised_record.extend(self.form_id.to_le_bytes());
        serialised_record.extend(self.version_control_1.to_le_bytes());
        serialised_record.extend(self.form_version.to_le_bytes());
        serialised_record.extend(self.version_control_2.to_le_bytes());
        serialised_record
    }
}

#[derive(Debug, CopyGetters, Clone, Copy)]
pub struct Sf01HeaderFlags {
    #[getset(get_copy = "pub")]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sf01_headers_read_serialise() {
        let test_data: Vec<u8> = vec![
            0x54, 0x45, 0x53, 0x34, 0xa5, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46, 0x02, 0x00, 0x00,
        ];
        assert_eq!(
            test_data,
            Sf01Header::read(&mut (test_data.as_slice()))
                .unwrap()
                .serialise()
        );
    }
}
