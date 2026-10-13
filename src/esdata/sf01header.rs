//! This module models sf01 header data.

use std::{
    borrow::Borrow,
    io::{Cursor, Read, Seek},
};

use getset::{CopyGetters, Getters};

use crate::{
    error::{ApplicationError, ApplicationErrorType},
    esdata::{
        EsEntityIO,
        sf01record::{
            Sf01GenericEntity, Sf01GenericRecord,
            sf01headerrecords::{Sf01HeaderRecord, Sf01MasterRecord},
        },
    },
    essource::sf01headersource::{OutputType, Sf01HeaderSource},
    utils::{NullTerminatedString, is_bit_set_u32, set_bit_u32, u32_vector_from_binary},
};

/// The sf01 header ID.
const SF01_HEADER_ID: [u8; 4] = [84, 69, 83, 52];
/// The default file extension of a non-master file.
const FILE_EXTENSION_NO_MASTER: &str = "esp";
/// The default file extension of a master file.
const FILE_EXTENSION_MASTER: &str = "esm";

pub const HEADER_FIELD_DEFAULT_FORM_ID: u32 = 0;
pub const HEADER_FIELD_DEFAULT_VERSION_CONTROL_1: u32 = 0;
pub const HEADER_FIELD_DEFAULT_VERSION_CONTROL_2: u16 = 0;
pub const HEADER_FIELD_DEFAULT_FORM_VERSION: u16 = 582;

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
    #[getset(get = "pub")]
    dependencies: Vec<Sf01MasterRecord>,
}

impl EsEntityIO for Sf01Header {
    fn read<T: Read + Seek>(reader: &mut T) -> Result<Self, ApplicationError> {
        let header_entity = Sf01GenericEntity::read(reader).map_err(|err| {
            ApplicationError::from(err).chain("Failed to read SF01 header entity.")
        })?;

        if header_entity.type_id() != SF01_HEADER_ID {
            return Err(ApplicationError::new(
                ApplicationErrorType::InputDataError,
                format!(
                    "Failed to parse input data as SF01 header entity. Type ID sequence {:?} does not match {:?}.",
                    header_entity.type_id(),
                    SF01_HEADER_ID
                ),
            ));
        }

        let payload_buffer = header_entity.payload();

        let mut author = None;
        let mut description = None;
        let mut masters = Vec::new();
        let mut overrides = Vec::new();
        if !payload_buffer.is_empty() {
            let payload_size = payload_buffer.len();

            let mut payload_reader = Cursor::new(payload_buffer);
            while payload_reader.position() < payload_size as u64 {
                let record = Sf01GenericRecord::read(&mut payload_reader)
                    .map_err(|err| err.chain("Failed to parse header record."))?;
                if record.id() == Sf01HeaderRecord::Author.record_id_binary() {
                    author = Some(NullTerminatedString::from_binary(record.take_payload())?)
                } else if record.id() == Sf01HeaderRecord::Description.record_id_binary() {
                    description = Some(NullTerminatedString::from_binary(record.take_payload())?)
                } else if record.id() == Sf01HeaderRecord::Master.record_id_binary() {
                    masters.push(NullTerminatedString::from_binary(record.take_payload())?);
                } else if record.id() == Sf01HeaderRecord::Overrides.record_id_binary() {
                    overrides.extend(u32_vector_from_binary(&record.take_payload())?);
                }
            }
        }

        Ok(Self {
            flags: header_entity.flags().into(),
            form_id: header_entity.form_id(),
            version_control_1: header_entity.version_control_1(),
            form_version: header_entity.form_version(),
            version_control_2: header_entity.version_control_2(),
            author,
            description,
            dependencies: Sf01MasterRecord::from_raw(masters, overrides),
        })
    }

    fn serialise(&self) -> Vec<u8> {
        let records = self.serialse_records(todo!());
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
    fn serialse_records(&self, group_entity_count: u32) -> Vec<u8> {
        let mut records = Vec::new();

        let mut header_payload = Sf01HeaderRecord::RECORD_FIELD_HEADER_VERSION.to_vec();
        // The header entity also counts.
        header_payload.extend((group_entity_count + 1).to_le_bytes());
        header_payload.extend(16779264u32.to_le_bytes());

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
        records.extend(Sf01MasterRecord::to_generic_records(self.dependencies()));

        let mut serialised_records = Vec::new();
        for record in records {
            serialised_records.extend(record.serialise());
        }
        serialised_records
    }

    pub fn from_source<T: Borrow<Sf01HeaderSource>>(source: T) -> Result<Self, ApplicationError> {
        let source = source.borrow();

        Ok(Self {
            flags: Sf01HeaderFlags::from_source(
                source.output_type(),
                source.base_flags(),
                source.localised().unwrap_or(false),
            ),
            form_id: source.form_id().unwrap_or(HEADER_FIELD_DEFAULT_FORM_ID),
            version_control_1: source
                .version_control_1()
                .unwrap_or(HEADER_FIELD_DEFAULT_VERSION_CONTROL_1),
            form_version: source
                .form_version()
                .unwrap_or(HEADER_FIELD_DEFAULT_FORM_VERSION),
            version_control_2: source
                .version_control_2()
                .unwrap_or(HEADER_FIELD_DEFAULT_VERSION_CONTROL_2),
            author: source.author().clone(),
            description: source.description().clone(),
            dependencies: Vec::new(),
        })
    }

    /// Returns the default file extension for compilation.
    pub fn get_file_extension(&self) -> &str {
        if self.flags().master() || self.flags().small_master() || self.flags().medium_master() {
            FILE_EXTENSION_MASTER
        } else {
            FILE_EXTENSION_NO_MASTER
        }
    }

    pub fn calculate_number_of_records() {
        // number_of_groups + number_of_group_records
    }

    pub fn set_dependencies(&mut self, dependencies: Vec<Sf01MasterRecord>) {
        self.dependencies = dependencies;
    }
}

#[derive(Debug, CopyGetters, Clone, Copy)]
pub struct Sf01HeaderFlags {
    #[getset(get_copy = "pub")]
    flags: u32,
}

impl Sf01HeaderFlags {
    const INDEX_MASTER: u32 = 0;
    const INDEX_LOCALISED: u32 = 7;
    const INDEX_MASTER_MEDIUM: u32 = 10;
    const INDEX_MASTER_SMALL: u32 = 8;
    const INDEX_BLUEPRINT: u32 = 11;

    /// Returns true if the master flag is set.
    pub fn master(&self) -> bool {
        is_bit_set_u32(self.flags, Self::INDEX_MASTER)
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

    /// Returns the header flags with the master flag set to the specified value.
    ///
    /// # Parameters
    ///
    /// * `value` - the bit value to set
    pub fn set_master(&self, value: bool) -> Self {
        set_bit_u32(self.flags, Self::INDEX_MASTER, value).into()
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

    /// Returns `true` if a non canonical bit has been set.
    pub fn is_non_canonical(&self) -> bool {
        self.set_blueprint(false)
            .set_localised(false)
            .set_master(false)
            .set_medium_master(false)
            .set_small_master(false)
            .flags()
            != 0
    }

    pub fn from_source(
        output_type: Option<OutputType>,
        base_flags: Option<u32>,
        localised: bool,
    ) -> Self {
        let flags = Self::from(base_flags.unwrap_or(0)).set_localised(localised);
        if let Some(ot) = output_type {
            match ot {
                OutputType::Development => flags
                    .set_master(false)
                    .set_medium_master(false)
                    .set_small_master(false)
                    .set_blueprint(false),
                OutputType::MasterFull => flags
                    .set_master(true)
                    .set_medium_master(false)
                    .set_small_master(false)
                    .set_blueprint(false),
                OutputType::MasterMedium => flags
                    .set_master(true)
                    .set_medium_master(true)
                    .set_small_master(false)
                    .set_blueprint(false),
                OutputType::MasterSmall => flags
                    .set_master(true)
                    .set_medium_master(false)
                    .set_small_master(true)
                    .set_blueprint(false),
                OutputType::MasterBlueprint => flags
                    .set_master(true)
                    .set_medium_master(false)
                    .set_small_master(false)
                    .set_blueprint(true),
            }
        } else {
            flags
        }
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
            Sf01Header::read(&mut Cursor::new(test_data.as_slice()))
                .unwrap()
                .serialise()
        );
    }
}
