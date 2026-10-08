//! This modules contains header related records.

use std::io::{Cursor, Read};

use crate::{
    error::{ApplicationError, ApplicationErrorType},
    esdata::sf01record::Sf01GenericRecord,
};

pub enum Sf01HeaderRecord {
    /// Record containing header information.
    Header {
        version: u32,
        record_count: u32,
        next_form_id: u32,
    },
    /// Internal record. Can be ignored for (de-)compilation.
    Offset(Sf01GenericRecord),
    /// Internal entity deletion record. Can be ignored for (de-)compilation.
    Deletion(Sf01GenericRecord),
    Author,
    Description,
    Master,
    Overrides,
    SCRN,
    TransientIDs {
        type_index: u32,
        ids: Vec<u32>,
    },
    /// Internal version control branch name record. Can be ignored for (de-)compilation.
    VersionControlBranchName(Sf01GenericRecord),
    /// Internal versioning. Can be ignored for (de-)compilation.
    InternalVersion(Sf01GenericRecord),
    /// Count of all CELL records with the is_interior_cell flag set that belong to the container itself and not one of its dependencies.
    InteriorCellCount,
    /// Internal version control change list record. Can be ignored for (de-)compilation.
    ChangeList(Sf01GenericRecord),
}

impl Sf01HeaderRecord {
    const RECORD_ID_HEADER: &str = "HEDR";
    const RECORD_ID_OFFSET: &str = "OFST";
    const RECORD_ID_DELETION: &str = "DELE";
    const RECORD_ID_AUTHOR: &str = "CNAM";
    const RECORD_ID_DESCRIPTION: &str = "SNAM";
    const RECORD_ID_MASTER: &str = "MAST";
    const RECORD_ID_OVERRIDES: &str = "ONAM";
    const RECORD_ID_SCRN: &str = "SCRN";
    const RECORD_ID_TRANSIENT_IDS: &str = "TNAM";
    const RECORD_ID_VERSION_CONTROL_BRANCH_NAME: &str = "BNAM";
    const RECORD_ID_INTERNAL_VERSION: &str = "INTV";
    const RECORD_ID_INTERIOR_CELL_COUNT: &str = "INCC";
    const RECORD_ID_CHANGE_LIST: &str = "CHGL";

    /// Returns the human readable record ID.
    pub fn record_id(&self) -> &str {
        match self {
            Sf01HeaderRecord::Header { .. } => Self::RECORD_ID_HEADER,
            Sf01HeaderRecord::Offset(_) => Self::RECORD_ID_OFFSET,
            Sf01HeaderRecord::Deletion(_) => Self::RECORD_ID_DELETION,
            Sf01HeaderRecord::Author => Self::RECORD_ID_AUTHOR,
            Sf01HeaderRecord::Description => Self::RECORD_ID_DESCRIPTION,
            Sf01HeaderRecord::Master => Self::RECORD_ID_MASTER,
            Sf01HeaderRecord::Overrides => Self::RECORD_ID_OVERRIDES,
            Sf01HeaderRecord::SCRN => Self::RECORD_ID_SCRN,
            Sf01HeaderRecord::TransientIDs { .. } => Self::RECORD_ID_TRANSIENT_IDS,
            Sf01HeaderRecord::VersionControlBranchName(_) => {
                Self::RECORD_ID_VERSION_CONTROL_BRANCH_NAME
            },
            Sf01HeaderRecord::InternalVersion(_) => Self::RECORD_ID_INTERNAL_VERSION,
            Sf01HeaderRecord::InteriorCellCount => Self::RECORD_ID_INTERIOR_CELL_COUNT,
            Sf01HeaderRecord::ChangeList(_) => Self::RECORD_ID_CHANGE_LIST,
        }
    }

    /// Returns the record ID as fixed binary representation.
    pub fn record_id_binary(&self) -> [u8; 4] {
        self.record_id().as_bytes().try_into().unwrap()
    }

    /// Tries to parse a generic record as header record.
    pub fn from_generic_record(
        generic_record: Sf01GenericRecord,
    ) -> Result<Result<Self, Sf01GenericRecord>, ApplicationError> {
        Ok(match str::from_utf8(&generic_record.id()) {
            Ok(record_id) => match record_id {
                Self::RECORD_ID_HEADER => Ok(Self::header_from_generic(generic_record)?),
                Self::RECORD_ID_DELETION => Ok(Sf01HeaderRecord::Deletion(generic_record)),
                Self::RECORD_ID_OFFSET => Ok(Sf01HeaderRecord::Offset(generic_record)),
                Self::RECORD_ID_VERSION_CONTROL_BRANCH_NAME => {
                    Ok(Sf01HeaderRecord::VersionControlBranchName(generic_record))
                },
                Self::RECORD_ID_INTERNAL_VERSION => {
                    Ok(Sf01HeaderRecord::InternalVersion(generic_record))
                },
                Self::RECORD_ID_CHANGE_LIST => Ok(Sf01HeaderRecord::ChangeList(generic_record)),
                _ => Err(generic_record),
            },
            Err(_) => Err(generic_record),
        })
    }

    fn header_from_generic(record: Sf01GenericRecord) -> Result<Self, ApplicationError> {
        if record.payload().len() == 12 {
            let payload = record.take_payload();
            Ok(Self::Header {
                version: u32::from_le_bytes(payload[0..4].try_into().unwrap()),
                record_count: u32::from_le_bytes(payload[4..8].try_into().unwrap()),
                next_form_id: u32::from_le_bytes(payload[8..12].try_into().unwrap()),
            })
        } else {
            Err(ApplicationError::new(
                ApplicationErrorType::InputDataError,
                format!(
                    "Cannot construct a SF01 header record from generic SF01 record {:?}.",
                    record
                ),
            ))
        }
    }

    fn transient_ids_from_generic(record: Sf01GenericRecord) -> Result<Self, ApplicationError> {
        let mut buffer_32: [u8; 4] = [0; 4];
        let raw_payload = record.take_payload();
        let payload_size = raw_payload.len() as u64;
        let mut payload = Cursor::new(raw_payload);

        payload.read_exact(&mut buffer_32).map_err(|err| {
            ApplicationError::from(err).chain("Failed to read SF01 TNAM type index.")
        })?;
        let type_index = u32::from_le_bytes(buffer_32);
        let mut ids = Vec::new();
        while payload.position() < payload_size {
            payload.read_exact(&mut buffer_32).map_err(|err| {
                ApplicationError::from(err).chain("Failed to read SF01 TNAM IDs.")
            })?;
            ids.push(u32::from_le_bytes(buffer_32));
        }
        Ok(Self::TransientIDs { type_index, ids })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_sf01_header_record_record_id_binary() {
        let possible_variants = vec![
            Sf01HeaderRecord::Header {
                version: 0,
                record_count: 0,
                next_form_id: 0,
            },
            Sf01HeaderRecord::Offset(Sf01GenericRecord::default()),
            Sf01HeaderRecord::Deletion(Sf01GenericRecord::default()),
            Sf01HeaderRecord::Author,
            Sf01HeaderRecord::Description,
            Sf01HeaderRecord::Master,
            Sf01HeaderRecord::Overrides,
            Sf01HeaderRecord::SCRN,
            Sf01HeaderRecord::TransientIDs {
                type_index: 0,
                ids: Vec::new(),
            },
            Sf01HeaderRecord::VersionControlBranchName(Sf01GenericRecord::default()),
            Sf01HeaderRecord::InternalVersion(Sf01GenericRecord::default()),
            Sf01HeaderRecord::InteriorCellCount,
            Sf01HeaderRecord::ChangeList(Sf01GenericRecord::default()),
        ];

        // Asserts that the unwrap works and tat all IDs are exactly 4 bytes.
        for variant in possible_variants {
            variant.record_id_binary();
        }
    }
}
