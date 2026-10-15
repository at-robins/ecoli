//! This module models sf01 groups data.

use std::{
    borrow::Borrow,
    io::{Cursor, SeekFrom},
};

use getset::{CopyGetters, Getters};

use crate::{
    error::{ApplicationError, ApplicationErrorType}, esdata::{
        EsEntityIO, sf01record::{
            Sf01EntityType, Sf01GenericEntity, sf01cell::{CELL_CONTAINING_GROUPS, Sf01CellRecord},
        },
    }, essource::sf01groupsource::Sf01GroupSource,
};

/// The binary identifier of a group in serialised data (GRUP).
const BINARY_GROUP_ID: [u8; 4] = [71, 82, 85, 80];
/// The size in bytes of the descriptor for groups.
const GROUP_DESCRIPTOR_SIZE: u32 = 24;

#[derive(Debug, Getters)]
pub struct Sf01Groups {
    #[getset(get = "pub")]
    groups: Vec<Sf01GenericGroup>,
}

impl Sf01Groups {
    /// Creates a new group container.
    pub fn new(mut groups: Vec<Sf01GenericGroup>) -> Self {
        groups.sort();
        Self { groups }
    }

    /// Returns the total entity count.
    pub fn record_count(&self) -> u32 {
        self.groups()
            .iter()
            .map(Sf01GenericGroup::record_count)
            .sum()
    }

    /// Returns the total interior cell count.
    pub fn interior_cell_count(&self) -> Result<u32, ApplicationError> {
        let mut counter = 0;
        let binary_cell_group_labels: Vec<[u8; 4]> = CELL_CONTAINING_GROUPS
            .iter()
            .map(|cell_group| cell_group.record_id_binary())
            .collect();
        for group in self
            .groups()
            .iter()
            .filter(|group| binary_cell_group_labels.contains(&group.label()))
        {
            counter += group.interior_cell_count()?;
        }
        Ok(counter)
    }
}

impl EsEntityIO for Sf01Groups {
    fn read<T: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut T,
    ) -> Result<Self, ApplicationError>
    where
        Self: Sized,
    {
        let mut groups = Vec::new();
        let mut eof_test_buffer = [0u8];
        while reader.read(&mut eof_test_buffer)? > 0 {
            // Reset the EOF test read.
            reader.seek(SeekFrom::Current(-(eof_test_buffer.len() as i64)))?;
            groups.push(Sf01GenericGroup::read(reader)?);
        }
        Ok(Sf01Groups::new(groups))
    }

    fn serialise(&self) -> Vec<u8> {
        let mut serialised_groups = Vec::new();
        for group in self.groups() {
            serialised_groups.extend(group.serialise());
        }
        serialised_groups
    }
}

#[derive(Debug, CopyGetters, Getters, PartialEq, Eq)]
pub struct Sf01GenericGroup {
    #[getset(get_copy = "pub")]
    label: [u8; 4],
    #[getset(get_copy = "pub")]
    label_type: GroupLabelType,
    #[getset(get_copy = "pub")]
    /// 2 bytes timestamp and 6 unknwon bytes.
    unknown: u64,
    #[getset(get = "pub")]
    children: Vec<Sf01GroupChild>,
}

impl EsEntityIO for Sf01GenericGroup {
    fn read<T: std::io::prelude::Read + std::io::Seek>(
        reader: &mut T,
    ) -> Result<Self, ApplicationError>
    where
        Self: Sized,
    {
        let mut buffer_32: [u8; 4] = [0; 4];
        let mut buffer_64: [u8; 8] = [0; 8];
        reader.read_exact(&mut buffer_32).map_err(|err| {
            ApplicationError::from(err).chain("Failed to read SF01 group type ID.")
        })?;

        if buffer_32 != BINARY_GROUP_ID {
            return Err(ApplicationError::new(
                ApplicationErrorType::InputDataError,
                format!(
                    "Failed to parse input data as SF01 group. Type ID sequence {:?} does not match {:?}.",
                    buffer_32, BINARY_GROUP_ID
                ),
            ));
        }

        reader
            .read_exact(&mut buffer_32)
            .map_err(|err| ApplicationError::from(err).chain("Failed to read SF01 group size."))?;
        let size = u32::from_le_bytes(buffer_32);

        reader
            .read_exact(&mut buffer_32)
            .map_err(|err| ApplicationError::from(err).chain("Failed to read SF01 group label."))?;
        let label = buffer_32.clone();

        let label_type = reader
            .read_exact(&mut buffer_32)
            .map_err(|err| {
                ApplicationError::from(err).chain("Failed to read SF01 group label type.")
            })
            .and_then(|()| GroupLabelType::try_from(buffer_32))?;

        // 2 bytes timestamp and 6 unknwon bytes.
        let unknown = reader
            .read_exact(&mut buffer_64)
            .and_then(|()| Ok(u64::from_le_bytes(buffer_64)))
            .map_err(|err| ApplicationError::from(err).chain("Failed to read SF01 group ."))?;

        let payload_size = size
            .checked_sub(GROUP_DESCRIPTOR_SIZE)
            .ok_or(ApplicationError::new(
                ApplicationErrorType::InputDataError,
                format!(
                    "Size of SF01 group is {}, but a minimum size of {} is expected.",
                    size, GROUP_DESCRIPTOR_SIZE
                ),
            ))?;

        let mut children = Vec::new();
        let payload_start_position = reader.stream_position()?;
        while reader
            .stream_position()?
            .checked_sub(payload_start_position)
            .ok_or(ApplicationError::new(
                ApplicationErrorType::InternalError,
                "Reader position was moved unexpectedly while reading SF01 group payload.",
            ))?
            < payload_size as u64
        {
            children.push(Sf01GroupChild::read(reader)?);
        }

        Ok(Self {
            label,
            label_type,
            unknown,
            children,
        })
    }

    fn serialise(&self) -> Vec<u8> {
        let mut serialised_group = BINARY_GROUP_ID.to_vec();
        // Inserts a dummy size for later swapping.
        serialised_group.extend([0, 0, 0, 0]);
        serialised_group.extend(self.label());
        serialised_group.extend(self.label_type().serialise());
        serialised_group.extend(self.unknown().to_le_bytes());
        for child in self.children() {
            serialised_group.extend(child.serialise());
        }
        let mut size = (serialised_group.len() as u32).to_le_bytes();
        size.swap_with_slice(&mut serialised_group[4..8]);
        serialised_group
    }
}

impl Sf01GenericGroup {
    /// Returns the total entity count.
    pub fn record_count(&self) -> u32 {
        self.children.iter().map(Sf01GroupChild::record_count).sum()
    }

    /// Returns the total interior cell count.
    pub fn interior_cell_count(&self) -> Result<u32, ApplicationError> {
        let mut counter = 0;
        for child in &self.children {
            counter += child.interior_cell_count()?;
        }
        Ok(counter)
    }

    pub fn from_source<T: Borrow<Sf01GroupSource>>(source: T) -> Result<Self, ApplicationError> {
        let source = source.borrow();
        let payload = base122_rs::decode(source.content()).map_err(|err| {
            ApplicationError::new(ApplicationErrorType::IOError, err)
                .chain(format!("Failed to decode binary source data: {:?}", source))
        })?;
        Self::read(&mut Cursor::new(payload.as_slice())).map_err(|err| {
            err.chain(format!("Failed to parse binary source data as SF01 group: {:?}", payload))
        })
    }
}

impl PartialOrd for Sf01GenericGroup {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let label_self = Sf01EntityType::from_binary( &self.label());
        let label_other = Sf01EntityType::from_binary( &other.label());
        Some(if let Some(label_self) = label_self {
            if let Some(label_other) = label_other {
                label_self.cmp(&label_other)
            } else {
                core::cmp::Ordering::Greater
            }
        } else if label_other.is_some() {
            core::cmp::Ordering::Less
        } else {
            core::cmp::Ordering::Equal
        })
    }
}

impl Ord for Sf01GenericGroup {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Sf01GroupChild {
    Entity(Sf01GenericEntity),
    Group(Sf01GenericGroup),
}

impl Sf01GroupChild {
    /// Returns the total entity count.
    pub fn record_count(&self) -> u32 {
        match self {
            Sf01GroupChild::Entity(_) => 1,
            Sf01GroupChild::Group(sf01_generic_group) => sf01_generic_group.record_count(),
        }
    }

    /// Returns the total interior cell count.
    pub fn interior_cell_count(&self) -> Result<u32, ApplicationError> {
        match self {
            Sf01GroupChild::Entity(sf01_generic_entity) => {
                Sf01CellRecord::is_interior_cell(sf01_generic_entity)
                    .map(|value| if value { 1 } else { 0 })
            },
            Sf01GroupChild::Group(sf01_generic_group) => sf01_generic_group.interior_cell_count(),
        }
    }
}

impl EsEntityIO for Sf01GroupChild {
    fn read<T: std::io::prelude::Read + std::io::Seek>(
        reader: &mut T,
    ) -> Result<Self, ApplicationError>
    where
        Self: Sized,
    {
        let mut buffer_type_id: [u8; 4] = [0; 4];
        reader.read_exact(&mut buffer_type_id).map_err(|err| {
            ApplicationError::from(err).chain("Failed to read SF01 group or entity type ID.")
        })?;
        // Reset the reader position.
        reader.seek(SeekFrom::Current(-(buffer_type_id.len() as i64)))?;

        if buffer_type_id == BINARY_GROUP_ID {
            Sf01GenericGroup::read(reader).map(|group| Self::Group(group))
        } else {
            Sf01GenericEntity::read(reader).map(|entity| Self::Entity(entity))
        }
        .map_err(|err| err.chain("Failed to read SF01 group or entity."))
    }

    fn serialise(&self) -> Vec<u8> {
        match self {
            Sf01GroupChild::Entity(sf01_generic_entity) => sf01_generic_entity.serialise(),
            Sf01GroupChild::Group(sf01_generic_group) => sf01_generic_group.serialise(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupLabelType {
    // The type ID of contained records.
    RecordID,
    // The form ID of a parent world (WRLD) record.
    ParentWorldID,
    // Interior cell block number.
    InteriorCellBlockID,
    // Interior cell sub-block number.
    InteriorCellSubBlockID,
    // Exterior cell block / grid cooridnates (x: u16, y: u16).
    ExteriorCellBlockID,
    // Exterior cell sub-block / grid cooridnates (x: u16, y: u16).
    ExteriorSubCellBlockID,
    // From ID of parent cell (CELL).
    ParentCellID,
    // From ID of parent dialogue (DIAL).
    ParentDialogueID,
    // From ID of parent cell (CELL) for persistent child cells.
    ParentCellPersistentID,
    // From ID of parent cell (CELL) for temporary child cells.
    ParentCellTemporaryID,
    // From ID of parent cell (CELL) for distant visible child cells.
    ParentCellVisibleDistantID,
}

impl GroupLabelType {
    const NUMERICAL_RECORD_ID: u32 = 0;
    const NUMERICAL_PARENT_WORLD_ID: u32 = 1;
    const NUMERICAL_INTERIOR_CELL_BLOCK_ID: u32 = 2;
    const NUMERICAL_INTERIOR_CELL_SUB_BLOCK_ID: u32 = 3;
    const NUMERICAL_EXTERIOR_CELL_BLOCK_ID: u32 = 4;
    const NUMERICAL_EXTERIOR_SUB_CELL_BLOCK_ID: u32 = 5;
    const NUMERICAL_PARENT_CELL_ID: u32 = 6;
    const NUMERICAL_PARENT_DIALOGUE_ID: u32 = 7;
    const NUMERICAL_PARENT_CELL_PERSISTENT_ID: u32 = 8;
    const NUMERICAL_PARENT_CELL_TEMPORARY_ID: u32 = 9;
    const NUMERICAL_PARENT_CELL_VISIBLE_DISTANT_ID: u32 = 10;
}

impl GroupLabelType {
    pub fn serialise(&self) -> [u8; 4] {
        u32::from(self).to_le_bytes()
    }
}

impl From<GroupLabelType> for u32 {
    fn from(value: GroupLabelType) -> Self {
        u32::from(&value)
    }
}

impl From<&GroupLabelType> for u32 {
    fn from(value: &GroupLabelType) -> Self {
        match value {
            GroupLabelType::RecordID => GroupLabelType::NUMERICAL_RECORD_ID,
            GroupLabelType::ParentWorldID => GroupLabelType::NUMERICAL_PARENT_WORLD_ID,
            GroupLabelType::InteriorCellBlockID => GroupLabelType::NUMERICAL_INTERIOR_CELL_BLOCK_ID,
            GroupLabelType::InteriorCellSubBlockID => {
                GroupLabelType::NUMERICAL_INTERIOR_CELL_SUB_BLOCK_ID
            },
            GroupLabelType::ExteriorCellBlockID => GroupLabelType::NUMERICAL_EXTERIOR_CELL_BLOCK_ID,
            GroupLabelType::ExteriorSubCellBlockID => {
                GroupLabelType::NUMERICAL_EXTERIOR_SUB_CELL_BLOCK_ID
            },
            GroupLabelType::ParentCellID => GroupLabelType::NUMERICAL_PARENT_CELL_ID,
            GroupLabelType::ParentDialogueID => GroupLabelType::NUMERICAL_PARENT_DIALOGUE_ID,
            GroupLabelType::ParentCellPersistentID => {
                GroupLabelType::NUMERICAL_PARENT_CELL_PERSISTENT_ID
            },
            GroupLabelType::ParentCellTemporaryID => {
                GroupLabelType::NUMERICAL_PARENT_CELL_TEMPORARY_ID
            },
            GroupLabelType::ParentCellVisibleDistantID => {
                GroupLabelType::NUMERICAL_PARENT_CELL_VISIBLE_DISTANT_ID
            },
        }
    }
}

impl TryFrom<u32> for GroupLabelType {
    type Error = ApplicationError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            Self::NUMERICAL_RECORD_ID => Ok(Self::RecordID),
            Self::NUMERICAL_PARENT_WORLD_ID => Ok(Self::ParentWorldID),
            Self::NUMERICAL_INTERIOR_CELL_BLOCK_ID => Ok(Self::InteriorCellBlockID),
            Self::NUMERICAL_INTERIOR_CELL_SUB_BLOCK_ID => Ok(Self::InteriorCellBlockID),
            Self::NUMERICAL_EXTERIOR_CELL_BLOCK_ID => Ok(Self::ExteriorCellBlockID),
            Self::NUMERICAL_EXTERIOR_SUB_CELL_BLOCK_ID => Ok(Self::ExteriorSubCellBlockID),
            Self::NUMERICAL_PARENT_CELL_ID => Ok(Self::ParentCellID),
            Self::NUMERICAL_PARENT_DIALOGUE_ID => Ok(Self::ParentDialogueID),
            Self::NUMERICAL_PARENT_CELL_PERSISTENT_ID => Ok(Self::ParentCellPersistentID),
            Self::NUMERICAL_PARENT_CELL_TEMPORARY_ID => Ok(Self::ParentCellTemporaryID),
            Self::NUMERICAL_PARENT_CELL_VISIBLE_DISTANT_ID => Ok(Self::ParentCellVisibleDistantID),
            _ => Err(ApplicationError::new(
                ApplicationErrorType::InputDataError,
                format!("Cannot construct a group label type from {}.", value),
            )),
        }
    }
}

impl TryFrom<[u8; 4]> for GroupLabelType {
    type Error = ApplicationError;

    fn try_from(value: [u8; 4]) -> Result<Self, Self::Error> {
        Self::try_from(u32::from_le_bytes(value)).map_err(|err| {
            err.chain(format!("Cannot construct a group label type from {:?}.", value))
        })
    }
}
