//! This module models sf01 records data.

use std::{io::Read};

use getset::CopyGetters;

use crate::{
    error::{ApplicationError},
    esdata::EsEntityIO,
};

/// The sf01 header ID.
const SF01_RECORD_ID_EXTENDED_SIZE: [u8; 4] = [120, 120, 120, 120];

#[derive(Debug, CopyGetters)]
pub struct Sf01GenericRecord {
    #[getset(get_copy = "pub")]
    id: u32,
    #[getset(get_copy = "pub")]
    size: u32,
}

impl EsEntityIO for Sf01GenericRecord {
    fn read<T: Read>(reader: &mut T) -> Result<Self, ApplicationError> {
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
            reader
            .read_exact(&mut buffer_32)
            .map_err(|err| ApplicationError::from(err).chain("Failed to read SF01 extended size record ID."))?;
            let extended_size = u32::from_le_bytes(buffer_32);
            // Parses actual record.
            reader
            .read_exact(&mut buffer_32)
            .map_err(|err| ApplicationError::from(err).chain("Failed to read SF01 extended size record size."))?;
            let extended_id = u32::from_le_bytes(buffer_32);
            // Ignores the 0 size value of the actual record.
            reader
            .read_exact(&mut buffer_16)
            .map_err(|err| ApplicationError::from(err).chain("Failed to read SF01 record size."))?;

            (extended_id, extended_size)
        } else {
            (u32::from_le_bytes(buffer_32), u16::from_le_bytes(buffer_16) as u32)
        };


        Ok(Self {
            id,
            size,
        })
    }

    fn serialise(&self) -> Vec<u8> {
        let mut serialised_record = Vec::with_capacity(6);
        if self.size() > u16::MAX as u32 {
            // Use extended size format.
            serialised_record.extend_from_slice(&SF01_RECORD_ID_EXTENDED_SIZE);
            serialised_record.extend(4u16.to_le_bytes());
            serialised_record.extend(self.size.to_le_bytes());
            serialised_record.extend(self.id.to_le_bytes());
            serialised_record.extend(0u16.to_le_bytes());
        } else {
            serialised_record.extend(self.id.to_le_bytes());
            serialised_record.extend((self.size as u16).to_le_bytes());
        }

        serialised_record
    }
}

// impl Sf01Header {
//     pub fn from_source<T: Borrow<Sf01HeaderSource>>(source: T) -> Self {
//         let source = source.borrow();
//         Self {
//             // TODO: set actual payload size!
//             size: todo!(),
//             flags: Sf01HeaderFlags::from_source(source.flags()),
//             form_id: source.form_id().unwrap_or(0),
//             version_control_1: source.version_control_1().unwrap_or(0),
//             form_version: source.form_version().unwrap_or(582),
//             version_control_2: source.version_control_2().unwrap_or(0),
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        
    }
}
