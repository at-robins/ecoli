//! This module models sf01 header data.

use std::{
    borrow::Borrow,
    io::{Cursor, Read},
};

use getset::{CopyGetters, Getters};

use crate::{
    error::{ApplicationError, ApplicationErrorType},
    esdata::{
        EsEntityIO,
        sf01record::{Sf01GenericRecord, sf01headerrecords::Sf01HeaderRecord},
    },
    essource::sf01headersource::{Sf01HeaderFlagsSource, Sf01HeaderSource},
    utils::{NullTerminatedString, is_bit_set_u32, set_bit_u32},
};

/// The sf01 header ID.
const SF01_HEADER_ID: [u8; 4] = [84, 69, 83, 52];
/// The default file extension of a non-master file.
const FILE_EXTENSION_NO_MASTER: &str = "esp";
/// The default file extension of a master file.
const FILE_EXTENSION_MASTER: &str = "esm";

#[derive(Debug, CopyGetters, Getters)]
pub struct Sf01Header {
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
    #[getset(get = "pub")]
    author: Option<NullTerminatedString>,
    #[getset(get = "pub")]
    description: Option<NullTerminatedString>,
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
                    "Failed to parse input data as SF01 header record. ID sequence {:?} does not match {:?}.",
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

        let mut author = None;
        let mut description = None;
        if size > 0 {
            let mut payload_buffer: Vec<u8> = vec![0; size as usize];
            reader.read_exact(&mut payload_buffer)?;

            let mut payload_reader = Cursor::new(payload_buffer);

            while payload_reader.position() < size as u64 {
                let record = Sf01GenericRecord::read(&mut payload_reader)
                    .map_err(|err| err.chain("Failed to parse header record."))?;
                if record.id() == Sf01HeaderRecord::Author.record_id_binary() {
                    author = Some(NullTerminatedString::from_binary(record.take_payload())?)
                } else if record.id() == Sf01HeaderRecord::Description.record_id_binary() {
                    description = Some(NullTerminatedString::from_binary(record.take_payload())?)
                }
            }
        }

        Ok(Self {
            flags,
            form_id,
            version_control_1,
            form_version,
            version_control_2,
            author,
            description,
        })
    }

    fn serialise(&self) -> Vec<u8> {
        let records = self.serialse_records(todo!(), todo!());
        let mut serialised_record = Vec::with_capacity(24 + records.len());
        serialised_record.extend_from_slice(&SF01_HEADER_ID);
        serialised_record.extend((records.len() as u32).to_le_bytes());
        serialised_record.extend(self.flags.flags.to_le_bytes());
        serialised_record.extend(self.form_id.to_le_bytes());
        serialised_record.extend(self.version_control_1.to_le_bytes());
        serialised_record.extend(self.form_version.to_le_bytes());
        serialised_record.extend(self.version_control_2.to_le_bytes());
        serialised_record.extend(records);
        serialised_record
    }
}

impl Sf01Header {
    fn serialse_records(&self, record_count: u32, next_object_id: u32) -> Vec<u8> {
        let mut records = Vec::new();

        let mut header_payload = Sf01HeaderRecord::RECORD_FIELD_HEADER_VERSION.to_vec();
        header_payload.extend(record_count.to_le_bytes());
        header_payload.extend(next_object_id.to_le_bytes());

        records.push(Sf01GenericRecord::new(
            Sf01HeaderRecord::Header.record_id_binary(),
            header_payload,
        ));

        if let Some(author) = self.author() {
            records.push(Sf01GenericRecord::new(
                Sf01HeaderRecord::Author.record_id_binary(),
                author.serialise(),
            ));
        } else {
            records.push(Sf01GenericRecord::new(
                Sf01HeaderRecord::Author.record_id_binary(),
                Sf01HeaderRecord::default_author().serialise(),
            ));
        }
        if let Some(description) = self.description() {
            records.push(Sf01GenericRecord::new(
                Sf01HeaderRecord::Description.record_id_binary(),
                description.serialise(),
            ));
        }

        let mut serialised_records = Vec::new();
        for record in records {
            serialised_records.extend(record.serialise());
        }
        serialised_records
    }

    pub fn from_source<T: Borrow<Sf01HeaderSource>>(source: T) -> Result<Self, ApplicationError> {
        let source = source.borrow();

        Ok(Self {
            flags: Sf01HeaderFlags::from_source(source.flags()),
            form_id: source.form_id().unwrap_or(0),
            version_control_1: source.version_control_1().unwrap_or(0),
            form_version: source.form_version().unwrap_or(582),
            version_control_2: source.version_control_2().unwrap_or(0),
            author: source.author().clone(),
            description: source.description().clone(),
        })
    }

    /// Returns the default file extension for compilation.
    pub fn get_file_extension(&self) -> &str {
        if self.flags().full_master() || self.flags().small_master() || self.flags().medium_master()
        {
            FILE_EXTENSION_MASTER
        } else {
            FILE_EXTENSION_NO_MASTER
        }
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

    pub fn from_source<T: Borrow<Sf01HeaderFlagsSource>>(source: T) -> Self {
        let source = source.borrow();
        Self::from(source.base_flags())
            .set_full_master(source.full_master())
            .set_medium_master(source.medium_master())
            .set_small_master(source.small_master())
            .set_localised(source.localised())
            .set_blueprint(source.blueprint())
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
