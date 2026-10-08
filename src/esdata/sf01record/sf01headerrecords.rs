//! This modules contains header related records.

use crate::esdata::sf01record::Sf01GenericRecord;

pub enum Sf01HeaderRecord {
    /// Record containing header information.
    Header,
    /// Internal record. Can be ignored for (de-)compilation.
    Offset(Sf01GenericRecord),
    /// Internal entity deletion record. Can be ignored for (de-)compilation.
    Deletion(Sf01GenericRecord),
    Author,
    Description,
    Master,
    Overrides,
    SCRN,
    TransientIDs,
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
    /// Returns the human readable record ID.
    pub fn record_id(&self) -> &str {
        match self {
            Sf01HeaderRecord::Header => "HEDR",
            Sf01HeaderRecord::Offset(_) => "OFST",
            Sf01HeaderRecord::Deletion(_) => "DELE",
            Sf01HeaderRecord::Author => "CNAM",
            Sf01HeaderRecord::Description => "SNAM",
            Sf01HeaderRecord::Master => "MAST",
            Sf01HeaderRecord::Overrides => "ONAM",
            Sf01HeaderRecord::SCRN => "SCRN",
            Sf01HeaderRecord::TransientIDs => "TNAM",
            Sf01HeaderRecord::VersionControlBranchName(_) => "BNAM",
            Sf01HeaderRecord::InternalVersion(_) => "INTV",
            Sf01HeaderRecord::InteriorCellCount => "INCC",
            Sf01HeaderRecord::ChangeList(_) => "CHGL",
        }
    }

    /// Returns the record ID as fixed binary representation.
    pub fn record_id_binary(&self) -> [u8; 4] {
        self.record_id().as_bytes().try_into().unwrap()
    }

    // pub fn from_generic_record(generic_record: Sf01GenericRecord) -> Option<Self>{
    //     match generic_record.id() {

    //     }
    // }
}

// pub struct Sf01HeaderRecord {
//     version: u32,
//     record_count: u32,
//     next_form_id: u32,
// }

// impl Sf01HeaderRecord {
//     const RECORD_ID: [u8; 4] = [72, 69, 68, 82];

//     pub fn record_id() -> u32 {
//         u32::from_le_bytes(Self::RECORD_ID)
//     }
// }

// impl TryFrom<Sf01GenericRecord> for Sf01HeaderRecord {
//     type Error = ApplicationError;

//     fn try_from(record: Sf01GenericRecord) -> Result<Self, Self::Error> {
//         if record.id() == Self::record_id() && record.payload().len() == 12 {
//             let payload = record.take_payload();
//             Ok(Self {
//                 version: u32::from_le_bytes(payload[0..4].try_into().unwrap()),
//                 record_count: u32::from_le_bytes(payload[4..8].try_into().unwrap()),
//                 next_form_id: u32::from_le_bytes(payload[8..12].try_into().unwrap()),
//             })
//         } else {
//             Err(ApplicationError::new(
//                 ApplicationErrorType::InputDataError,
//                 format!(
//                     "Cannot construct a SF01 header record from generic SF01 record {:?}.",
//                     record
//                 ),
//             ))
//         }
//     }
// }

// pub struct Sf01TnamRecord {
//     /// The index of the record type the IDs reference.
//     type_index: u32,
//     ids: Vec<u32>,
// }

// impl Sf01TnamRecord {
//     const RECORD_ID: [u8; 4] = [84, 78, 65, 77];

//     pub fn record_id() -> u32 {
//         u32::from_le_bytes(Self::RECORD_ID)
//     }
// }

// impl TryFrom<Sf01GenericRecord> for Sf01TnamRecord {
//     type Error = ApplicationError;

//     fn try_from(record: Sf01GenericRecord) -> Result<Self, Self::Error> {
//         if record.id() == Self::record_id() && record.payload().len() == 12 {
//             let mut buffer_32: [u8; 4] = [0; 4];
//             let raw_payload = record.take_payload();
//             let payload_size = raw_payload.len() as u64;
//             let mut payload = Cursor::new(raw_payload);

//             payload.read_exact(&mut buffer_32).map_err(|err| {
//                 ApplicationError::from(err).chain("Failed to read SF01 TNAM type index.")
//             })?;
//             let type_index = u32::from_le_bytes(buffer_32);
//             let mut ids = Vec::new();
//             while payload.position() < payload_size {
//                 payload.read_exact(&mut buffer_32).map_err(|err| {
//                     ApplicationError::from(err).chain("Failed to read SF01 TNAM IDs.")
//                 })?;
//                 ids.push(u32::from_le_bytes(buffer_32));
//             }
//             Ok(Self { type_index, ids })
//         } else {
//             Err(ApplicationError::new(
//                 ApplicationErrorType::InputDataError,
//                 format!(
//                     "Cannot construct a SF01 TNAM record from generic SF01 record {:?}.",
//                     record
//                 ),
//             ))
//         }
//     }
// }

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_sf01_header_record_record_id_binary() {
        let possible_variants = vec![
            Sf01HeaderRecord::Header,
            Sf01HeaderRecord::Offset(Sf01GenericRecord::default()),
            Sf01HeaderRecord::Deletion(Sf01GenericRecord::default()),
            Sf01HeaderRecord::Author,
            Sf01HeaderRecord::Description,
            Sf01HeaderRecord::Master,
            Sf01HeaderRecord::Overrides,
            Sf01HeaderRecord::SCRN,
            Sf01HeaderRecord::TransientIDs,
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
