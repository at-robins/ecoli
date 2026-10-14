//! This modules contains cell related records.

use std::borrow::Borrow;

use crate::{
    error::ApplicationError,
    esdata::sf01record::{Sf01EntityType, Sf01GenericEntity, Sf01GenericFlags},
};

/// All group / entity types that contain cell entities.
pub const CELL_CONTAINING_GROUPS: [Sf01EntityType; 2] = [Sf01EntityType::Cell, Sf01EntityType::Worldspace]; 

/// Records defined by CELL entities.
pub enum Sf01CellRecord {
    /// Record containing data flags.
    Data,
}

impl Sf01CellRecord {
    /// The ID of a data record.
    const RECORD_ID_DATA: &str = "DATA";

    /// The flag bit used to mark the entity as interior cell.
    pub const RECORD_FIELD_DATA_FLAG_IS_INTERIOR_CELL: u32 = 0;

    /// Returns the human readable record ID.
    pub fn record_id(&self) -> &str {
        match self {
            Sf01CellRecord::Data => Self::RECORD_ID_DATA,
        }
    }

    /// Returns the record ID as fixed binary representation.
    pub fn record_id_binary(&self) -> [u8; 4] {
        self.record_id().as_bytes().try_into().unwrap()
    }

    /// Returns `true` if the cell has the interior cell flag set.
    pub fn is_interior_cell<T: Borrow<Sf01GenericEntity>>(
        entity: T,
    ) -> Result<bool, ApplicationError> {
        let entity = entity.borrow();
        if entity.type_id() == Sf01EntityType::Cell.record_id_binary() {
            for record in entity.records()? {
                if record.id() == Sf01CellRecord::Data.record_id_binary() {
                    let flags = Sf01GenericFlags::from(u32::from_le_bytes(
                        record.payload().as_slice().try_into().map_err(|err| {
                            ApplicationError::from(err).chain(format!(
                                "Failed to parse entity payload as DATA flag: {:?}",
                                record.payload()
                            ))
                        })?,
                    ));
                    // Unwrap must work if the bit index was correctly specified.
                    return Ok(flags
                        .is_set(Self::RECORD_FIELD_DATA_FLAG_IS_INTERIOR_CELL)
                        .unwrap());
                }
            }
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
}
