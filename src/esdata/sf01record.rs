//! This module models sf01 records data.

use std::{
    borrow::Borrow,
    io::{Cursor, Read, Seek},
};

use getset::{CopyGetters, Getters};

use crate::{
    error::{ApplicationError, ApplicationErrorType},
    esdata::EsEntityIO,
    essource::sf01recordsource::Sf01GenericRecordSource,
    utils::{is_bit_set_u32, set_bit_u32},
};

/// The SF01 size extension ID.
const SF01_RECORD_ID_EXTENDED_SIZE: [u8; 4] = [88, 88, 88, 88];

#[derive(Debug, CopyGetters, Getters)]
pub struct Sf01GenericRecord {
    #[getset(get_copy = "pub")]
    id: [u8; 4],
    #[getset(get = "pub")]
    payload: Vec<u8>,
}

impl Sf01GenericRecord {
    pub fn new(id: [u8; 4], payload: Vec<u8>) -> Self {
        Sf01GenericRecord { id, payload }
    }

    pub fn from_source<T: Borrow<Sf01GenericRecordSource>>(
        source: T,
    ) -> Result<Self, ApplicationError> {
        let source = source.borrow();
        let payload = base122_rs::decode(source.content()).map_err(|err| {
            ApplicationError::new(ApplicationErrorType::IOError, err)
                .chain(format!("Failed to decode binary source data: {:?}", source))
        })?;
        let id = match source.class().as_bytes().try_into() {
            Ok(value) => value,
            Err(_) => source
                .class()
                .parse::<u32>()
                .map_err(|err| {
                    ApplicationError::from(err)
                        .chain(format!("Failed to parse {} as record ID.", source.class()))
                })?
                .to_le_bytes(),
        };
        Ok(Self { id, payload })
    }

    /// Takes the payload of the generic record.
    pub fn take_payload(self) -> Vec<u8> {
        self.payload
    }

    // Returns the payload size in bytes.
    pub fn size(&self) -> u32 {
        self.payload().len() as u32
    }
}

impl EsEntityIO for Sf01GenericRecord {
    fn read<T: Read + Seek>(reader: &mut T) -> Result<Self, ApplicationError> {
        let mut buffer_32: [u8; 4] = [0; 4];
        let mut buffer_16: [u8; 2] = [0; 2];

        reader
            .read_exact(&mut buffer_32)
            .map_err(|err| ApplicationError::from(err).chain("Failed to read SF01 record ID."))?;
        reader
            .read_exact(&mut buffer_16)
            .map_err(|err| ApplicationError::from(err).chain("Failed to read SF01 record size."))?;
        let (id, size) = if buffer_32 == SF01_RECORD_ID_EXTENDED_SIZE {
            // Use the extended record size parsing.

            // Parses extended size payload.
            reader.read_exact(&mut buffer_32).map_err(|err| {
                ApplicationError::from(err).chain("Failed to read SF01 extended size record ID.")
            })?;
            let extended_size = u32::from_le_bytes(buffer_32);
            // Parses actual record.
            reader.read_exact(&mut buffer_32).map_err(|err| {
                ApplicationError::from(err).chain("Failed to read SF01 extended size record size.")
            })?;
            // Ignores the 0 size value of the actual record.
            reader.read_exact(&mut buffer_16).map_err(|err| {
                ApplicationError::from(err).chain("Failed to read SF01 record size.")
            })?;

            (buffer_32, extended_size)
        } else {
            (buffer_32, u16::from_le_bytes(buffer_16) as u32)
        };
        let mut payload: Vec<u8> = vec![0; size as usize];
        reader.read_exact(&mut payload)?;

        Ok(Self { id, payload })
    }

    fn serialise(&self) -> Vec<u8> {
        let mut serialised_record = Vec::new();
        if self.size() > u16::MAX as u32 {
            // Use extended size format.
            serialised_record.extend_from_slice(&SF01_RECORD_ID_EXTENDED_SIZE);
            serialised_record.extend(4u16.to_le_bytes());
            serialised_record.extend(self.size().to_le_bytes());
            serialised_record.extend(self.id());
            serialised_record.extend(0u16.to_le_bytes());
        } else {
            serialised_record.extend(self.id());
            serialised_record.extend((self.size() as u16).to_le_bytes());
        }
        serialised_record.extend(self.payload());

        serialised_record
    }
}

impl Default for Sf01GenericRecord {
    fn default() -> Self {
        Self {
            id: Default::default(),
            payload: Default::default(),
        }
    }
}

/// A generic top level record that might contain nested structures.
#[derive(Debug, CopyGetters, Getters, PartialEq, Eq)]
pub struct Sf01GenericEntity {
    #[getset(get_copy = "pub")]
    type_id: [u8; 4],
    #[getset(get_copy = "pub")]
    flags: Sf01EntityFlags,
    #[getset(get_copy = "pub")]
    form_id: u32,
    #[getset(get_copy = "pub")]
    version_control_1: u32,
    #[getset(get_copy = "pub")]
    form_version: u16,
    #[getset(get_copy = "pub")]
    version_control_2: u16,
    #[getset(get = "pub")]
    /// The raw binary payload. Might be compressed.
    /// For accessing decompressed data use [`Self::payload`].
    payload_raw: Vec<u8>,
}

impl Sf01GenericEntity {
    /// Returns the entities payload.
    /// Decompresses the payload if compressed.
    pub fn payload(&self) -> Result<EntityPayload<'_>, ApplicationError> {
        if self.flags().compressed() {
            // The first 2 bytes are the size of the decompressed data.
            let uncompressed_payload_size = u32::from_le_bytes(
                self.payload_raw()
                    .get(0..4)
                    .ok_or(ApplicationError::new(
                        ApplicationErrorType::InputDataError,
                        format!("The payload of entity {:?} is to short for decompression.", self),
                    ))?
                    .try_into()
                    .unwrap(),
            );
            let compressed_payload: &[u8] = self
                .payload_raw()
                .get(4..self.payload_raw().len())
                .ok_or(ApplicationError::new(
                    ApplicationErrorType::InputDataError,
                    format!("The payload of entity {:?} is too short for decompression.", self),
                ))?;
            let mut uncompressed_payload = Vec::with_capacity(uncompressed_payload_size as usize);
            libflate::zlib::Decoder::new(compressed_payload)
                .map_err(|err| {
                    ApplicationError::from(err)
                        .chain(format!("The payload of entity {:?} cannot be decompressed.", self))
                })?
                .read_to_end(&mut uncompressed_payload)?;
            Ok(EntityPayload::Decompressed(uncompressed_payload))
        } else {
            Ok(EntityPayload::Uncompressed(self.payload_raw()))
        }
    }

    /// Returns all records contained within the entities payload.
    pub fn records(&self) -> Result<Vec<Sf01GenericRecord>, ApplicationError> {
        let payload = self
            .payload()
            .map_err(|err| err.chain("Failed to get entity payload for record parsing."))?;
        let mut payload_reader = Cursor::new(match &payload {
            super::sf01record::EntityPayload::Uncompressed(items) => items.as_slice(),
            super::sf01record::EntityPayload::Decompressed(items) => items.as_slice(),
        });
        let payload_size = payload.size() as u64;

        let mut records = Vec::new();

        while payload_reader.position() < payload_size {
            records.push(
                Sf01GenericRecord::read(&mut payload_reader)
                    .map_err(|err| err.chain("Failed to parse record."))?,
            );
        }
        Ok(records)
    }
}

impl EsEntityIO for Sf01GenericEntity {
    fn read<T: Read>(reader: &mut T) -> Result<Self, ApplicationError> {
        let mut type_id: [u8; 4] = [0; 4];
        let mut buffer_32: [u8; 4] = [0; 4];
        let mut buffer_16: [u8; 2] = [0; 2];

        reader.read_exact(&mut type_id).map_err(|err| {
            ApplicationError::from(err).chain("Failed to read SF01 entity type ID.")
        })?;

        reader
            .read_exact(&mut buffer_32)
            .map_err(|err| ApplicationError::from(err).chain("Failed to read SF01 entity size."))?;
        let size = u32::from_le_bytes(buffer_32);

        reader.read_exact(&mut buffer_32).map_err(|err| {
            ApplicationError::from(err).chain("Failed to read SF01 entity flags.")
        })?;
        let flags = u32::from_le_bytes(buffer_32).into();

        reader.read_exact(&mut buffer_32).map_err(|err| {
            ApplicationError::from(err).chain("Failed to read SF01 entity form ID.")
        })?;
        let form_id = u32::from_le_bytes(buffer_32);

        reader.read_exact(&mut buffer_32).map_err(|err| {
            ApplicationError::from(err).chain("Failed to read SF01 entity version control 1.")
        })?;
        let version_control_1 = u32::from_le_bytes(buffer_32);

        reader.read_exact(&mut buffer_16).map_err(|err| {
            ApplicationError::from(err).chain("Failed to read SF01 entity form version.")
        })?;
        let form_version = u16::from_le_bytes(buffer_16);

        reader.read_exact(&mut buffer_16).map_err(|err| {
            ApplicationError::from(err).chain("Failed to read SF01 entity version control 2.")
        })?;
        let version_control_2 = u16::from_le_bytes(buffer_16);

        let mut payload_raw: Vec<u8> = vec![0; size as usize];
        reader.read_exact(&mut payload_raw).map_err(|err| {
            ApplicationError::from(err).chain("Failed to read SF01 entity raw payload.")
        })?;

        Ok(Self {
            type_id,
            flags,
            form_id,
            version_control_1,
            form_version,
            version_control_2,
            payload_raw,
        })
    }

    fn serialise(&self) -> Vec<u8> {
        let mut serialised_entity = Vec::with_capacity(24 + self.payload_raw().len());
        serialised_entity.extend(self.type_id());
        serialised_entity.extend((self.payload_raw().len() as u32).to_le_bytes());
        serialised_entity.extend(self.flags.flags().to_le_bytes());
        serialised_entity.extend(self.form_id.to_le_bytes());
        serialised_entity.extend(self.version_control_1.to_le_bytes());
        serialised_entity.extend(self.form_version.to_le_bytes());
        serialised_entity.extend(self.version_control_2.to_le_bytes());
        serialised_entity.extend(self.payload_raw());
        serialised_entity
    }
}

/// An entities payload either a reference to the
/// uncompressed data or the decompressed data itself.
pub enum EntityPayload<'a> {
    Uncompressed(&'a Vec<u8>),
    Decompressed(Vec<u8>),
}

impl<'a> EntityPayload<'a> {
    /// The payloads uncompressed size.
    pub fn size(&self) -> usize {
        match self {
            EntityPayload::Uncompressed(items) => items.len(),
            EntityPayload::Decompressed(items) => items.len(),
        }
    }
}

#[derive(Debug, CopyGetters, Clone, Copy, PartialEq, Eq)]
pub struct Sf01EntityFlags {
    #[getset(get_copy = "pub")]
    flags: u32,
}

impl Sf01EntityFlags {
    const INDEX_COMPRESSED: u32 = 18;

    /// Returns true if the compressed flag is set.
    pub fn compressed(&self) -> bool {
        is_bit_set_u32(self.flags, Self::INDEX_COMPRESSED)
    }

    /// Returns the entity flags with the compressed flag set to the specified value.
    ///
    /// # Parameters
    ///
    /// * `value` - the bit value to set
    pub fn set_compressed(&self, value: bool) -> Self {
        set_bit_u32(self.flags, Self::INDEX_COMPRESSED, value).into()
    }
}

impl From<u32> for Sf01EntityFlags {
    fn from(flags: u32) -> Self {
        Sf01EntityFlags { flags }
    }
}

#[derive(Debug, CopyGetters, Clone, Copy)]
/// A generic 32 bit flag collection.
pub struct Sf01GenericFlags {
    #[getset(get_copy = "pub")]
    flags: u32,
}

impl Sf01GenericFlags {
    /// Checks if the specified bit is set.
    /// Returns [`None`] if the requested bit is out of range.
    pub fn is_set(&self, bit_index: u32) -> Option<bool> {
        if bit_index < u32::BITS {
            Some(is_bit_set_u32(self.flags(), bit_index))
        } else {
            None
        }
    }
}

impl From<u32> for Sf01GenericFlags {
    fn from(flags: u32) -> Self {
        Self { flags }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
/// An entity ID / group label type.
pub enum Sf01EntityType {
    /// A cell entity.
    Cell,
    /// A worldspace entity.
    Worldspace,
}

impl Sf01EntityType {
    const TYPE_ID_CELL: &str = "CELL";
    const TYPE_ID_WORLDSPACE: &str = "WRLD";

    /// Returns the human readable entity ID.
    pub fn record_id(&self) -> &str {
        match self {
            Self::Cell => Self::TYPE_ID_CELL,
            Self::Worldspace => Self::TYPE_ID_WORLDSPACE,
        }
    }

    /// Returns the entity ID as fixed binary representation.
    pub fn record_id_binary(&self) -> [u8; 4] {
        self.record_id().as_bytes().try_into().unwrap()
    }

    pub fn from_binary(binary_data: &[u8]) -> Option<Self> {
        Some(if binary_data == Self::Cell.record_id_binary() {
            Self::Cell
        } else if binary_data == Self::Worldspace.record_id_binary() {
            Self::Worldspace
        } else {
            return None;
        })
    }
}

pub mod sf01cell;
pub mod sf01headerrecords;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {}
}
