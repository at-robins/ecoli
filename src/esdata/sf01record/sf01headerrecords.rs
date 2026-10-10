//! This modules contains header related records.

use crate::utils::NullTerminatedString;

pub enum Sf01HeaderRecord {
    /// Record containing header information.
    Header,
    /// Internal record. Can be ignored for (de-)compilation.
    Offset,
    /// Internal entity deletion record. Can be ignored for (de-)compilation.
    Deletion,
    Author,
    Description,
    Master,
    Overrides,
    SCRN,
    TransientIDs,
    /// Internal version control branch name record. Can be ignored for (de-)compilation.
    VersionControlBranchName,
    /// Internal versioning. Can be ignored for (de-)compilation.
    InternalVersion,
    /// Count of all CELL records with the is_interior_cell flag set that belong to the container itself and not one of its dependencies.
    InteriorCellCount,
    /// Internal version control change list record. Can be ignored for (de-)compilation.
    ChangeList,
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

    pub const RECORD_FIELD_HEADER_VERSION: [u8; 4] = [143, 194, 117, 63];
    pub const RECORD_FIELD_AUTHOR_DEFAULT: &str = "DEFAULT";

    /// Returns the human readable record ID.
    pub fn record_id(&self) -> &str {
        match self {
            Sf01HeaderRecord::Header => Self::RECORD_ID_HEADER,
            Sf01HeaderRecord::Offset => Self::RECORD_ID_OFFSET,
            Sf01HeaderRecord::Deletion => Self::RECORD_ID_DELETION,
            Sf01HeaderRecord::Author => Self::RECORD_ID_AUTHOR,
            Sf01HeaderRecord::Description => Self::RECORD_ID_DESCRIPTION,
            Sf01HeaderRecord::Master => Self::RECORD_ID_MASTER,
            Sf01HeaderRecord::Overrides => Self::RECORD_ID_OVERRIDES,
            Sf01HeaderRecord::SCRN => Self::RECORD_ID_SCRN,
            Sf01HeaderRecord::TransientIDs => Self::RECORD_ID_TRANSIENT_IDS,
            Sf01HeaderRecord::VersionControlBranchName => {
                Self::RECORD_ID_VERSION_CONTROL_BRANCH_NAME
            },
            Sf01HeaderRecord::InternalVersion => Self::RECORD_ID_INTERNAL_VERSION,
            Sf01HeaderRecord::InteriorCellCount => Self::RECORD_ID_INTERIOR_CELL_COUNT,
            Sf01HeaderRecord::ChangeList => Self::RECORD_ID_CHANGE_LIST,
        }
    }

    /// Returns the record ID as fixed binary representation.
    pub fn record_id_binary(&self) -> [u8; 4] {
        self.record_id().as_bytes().try_into().unwrap()
    }

    /// Returns the default author as [`NullTerminatedString`].
    pub fn default_author() -> NullTerminatedString {
        NullTerminatedString::new(Self::RECORD_FIELD_AUTHOR_DEFAULT).unwrap()
    }

//     fn transient_ids_from_generic(record: Sf01GenericRecord) -> Result<Self, ApplicationError> {
//         let mut buffer_32: [u8; 4] = [0; 4];
//         let raw_payload = record.take_payload();
//         let payload_size = raw_payload.len() as u64;
//         let mut payload = Cursor::new(raw_payload);

//         payload.read_exact(&mut buffer_32).map_err(|err| {
//             ApplicationError::from(err).chain("Failed to read SF01 TNAM type index.")
//         })?;
//         let type_index = u32::from_le_bytes(buffer_32);
//         let mut ids = Vec::new();
//         while payload.position() < payload_size {
//             payload.read_exact(&mut buffer_32).map_err(|err| {
//                 ApplicationError::from(err).chain("Failed to read SF01 TNAM IDs.")
//             })?;
//             ids.push(u32::from_le_bytes(buffer_32));
//         }
//         Ok(Self::TransientIDs { type_index, ids })
//     }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_sf01_header_record_record_id_binary() {
        let possible_variants = vec![
            Sf01HeaderRecord::Header,
            Sf01HeaderRecord::Offset,
            Sf01HeaderRecord::Deletion,
            Sf01HeaderRecord::Author,
            Sf01HeaderRecord::Description,
            Sf01HeaderRecord::Master,
            Sf01HeaderRecord::Overrides,
            Sf01HeaderRecord::SCRN,
            Sf01HeaderRecord::TransientIDs,
            Sf01HeaderRecord::VersionControlBranchName,
            Sf01HeaderRecord::InternalVersion,
            Sf01HeaderRecord::InteriorCellCount,
            Sf01HeaderRecord::ChangeList,
        ];

        // Asserts that the unwrap works and tat all IDs are exactly 4 bytes.
        for variant in possible_variants {
            variant.record_id_binary();
        }
    }

    #[test]
    fn test_sf01_header_record_default_author() {
        // Makes sure the unwrap does not panic.
        Sf01HeaderRecord::default_author();
    }
}
