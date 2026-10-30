//! This module provides identity and dependency data structures.

use crate::error::{ApplicationError, ApplicationErrorType};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// The type of dependency. Defines the number of possible dependencies of the specified type
/// and the maximum number of owned entities based on the ID scheme.
pub enum Sf01DependencyType {
    /// A full dependency / master. Can own the most entities, but is limited in numbers.
    Full,
    /// A medium dependency / master. Can own a medium amount of entities,
    /// and can be present in limited numbers.
    Medium,
    /// A small dependency / master. Can own only a small amount of entities,
    /// but can be present in large numbers.
    Small,
}

impl Sf01DependencyType {
    /// Extracts the dependency type from a raw entity ID.
    ///
    /// # Parameters
    ///
    /// * `id` - the raw entity ID
    pub fn from_id(id: u32) -> Self {
        let binary_id = id.to_le_bytes();
        if binary_id[3] == 254 {
            Self::Small
        } else if binary_id[3] == 253 {
            Self::Medium
        } else {
            Self::Full
        }
    }

    /// Returns the entity ID part of the full ID.
    ///
    /// # Parameters
    ///
    /// * `id` - the full entity ID
    pub fn id_part(&self, id: u32) -> u32 {
        self.id_mask() & id
    }

    /// Returns the dependency part of the full ID.
    ///
    /// # Parameters
    ///
    /// * `id` - the full entity ID
    pub fn dependency_part(&self, id: u32) -> u32 {
        self.dependency_mask() & id
    }

    /// Returns the entity ID part of the full ID by inferring the ID type.
    ///
    /// # Parameters
    ///
    /// * `id` - the full entity ID
    pub fn id_part_from_id(id: u32) -> u32 {
        Self::from_id(id).id_part(id)
    }

    /// Returns the dependency part of the full ID by inferring the ID type.
    ///
    /// # Parameters
    ///
    /// * `id` - the full entity ID
    pub fn dependency_part_from_id(id: u32) -> u32 {
        Self::from_id(id).dependency_part(id)
    }

    /// Returns a bitmask for the dependency part of an entitie's ID.
    pub fn dependency_mask(&self) -> u32 {
        match self {
            Sf01DependencyType::Full => 0b11111111_00000000_00000000_00000000,
            Sf01DependencyType::Medium => 0b11111111_11111111_00000000_00000000,
            Sf01DependencyType::Small => 0b11111111_11111111_11110000_00000000,
        }
    }

    /// Returns a bitmask for the ID part of an entitie's ID.
    pub fn id_mask(&self) -> u32 {
        0b11111111_11111111_11111111_11111111 ^ self.dependency_mask()
    }

    /// The load position index within the respective group.
    ///
    /// # Parameters
    ///
    /// * `id` - the full entity ID
    pub fn positon(&self, id: u32) -> u32 {
        match self {
            Sf01DependencyType::Full => id >> 24,
            Sf01DependencyType::Medium => (id << 8) >> 24,
            Sf01DependencyType::Small => (id << 8) >> 20,
        }
    }

    /// The load position index within the respective group by inferring the ID type.
    ///
    /// # Parameters
    ///
    /// * `id` - the full entity ID
    pub fn positon_from_id(id: u32) -> u32 {
        match Self::from_id(id) {
            Sf01DependencyType::Full => id >> 24,
            Sf01DependencyType::Medium => (id << 8) >> 24,
            Sf01DependencyType::Small => (id << 8) >> 20,
        }
    }

    /// Returns the maximum position value / index possible
    /// for the given dependency type.
    pub fn maximum_position(&self) -> u32 {
        match self {
            Sf01DependencyType::Full => 252,
            Sf01DependencyType::Medium => 255,
            Sf01DependencyType::Small => 4095,
        }
    }

    /// Constructs the full dependency part from the load position index.
    ///
    /// # Parameters
    ///
    /// * `position` - the position index to construct the dependency part from
    pub fn position_to_dependency_part(&self, position: u32) -> Result<u32, ApplicationError> {
        if position > self.maximum_position() {
            Err(ApplicationError::new(
                ApplicationErrorType::InternalError,
                format!(
                    "Position {} is larger than the maximum position value for  {:?}.",
                    position, self
                ),
            ))
        } else {
            Ok(match self {
                Sf01DependencyType::Full => position << 24,
                Sf01DependencyType::Medium => {
                    ((position << 16) & 0b00000000_11111111_00000000_00000000)
                        | 0b11111101_00000000_00000000_00000000
                },
                Sf01DependencyType::Small => {
                    ((position << 12) & 0b00000000_11111111_11110000_00000000)
                        | 0b11111110_00000000_00000000_00000000
                },
            })
        }
    }
}

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_type_dependency_mask() {
        assert_eq!(
            Sf01DependencyType::Full.dependency_mask(),
            0b11111111_00000000_00000000_00000000
        );
        assert_eq!(
            Sf01DependencyType::Medium.dependency_mask(),
            0b11111111_11111111_00000000_00000000
        );
        assert_eq!(
            Sf01DependencyType::Small.dependency_mask(),
            0b11111111_11111111_11110000_00000000
        );
    }

    #[test]
    fn test_dependency_type_id_mask() {
        assert_eq!(Sf01DependencyType::Full.id_mask(), 0b00000000_11111111_11111111_11111111);
        assert_eq!(Sf01DependencyType::Medium.id_mask(), 0b00000000_00000000_11111111_11111111);
        assert_eq!(Sf01DependencyType::Small.id_mask(), 0b00000000_00000000_00001111_11111111);
    }

    #[test]
    fn test_dependency_type_from_id() {
        assert_eq!(Sf01DependencyType::Full, Sf01DependencyType::from_id(0x00ABCDEF));
        assert_eq!(Sf01DependencyType::Full, Sf01DependencyType::from_id(0x99ABCDEF));
        assert_eq!(Sf01DependencyType::Medium, Sf01DependencyType::from_id(0xFDABCDEF));
        assert_eq!(Sf01DependencyType::Small, Sf01DependencyType::from_id(0xFEABCDEF));
    }

    #[test]
    fn test_dependency_type_id_part() {
        assert_eq!(Sf01DependencyType::Full.id_part(0x00ABCDEF), 0x00ABCDEF);
        assert_eq!(Sf01DependencyType::Full.id_part(0x99ABCDEF), 0x00ABCDEF);
        assert_eq!(Sf01DependencyType::Medium.id_part(0xFDABCDEF), 0x0000CDEF);
        assert_eq!(
            Sf01DependencyType::Small.id_part(0b11111110_01001011_11101000_00000010),
            0b00000000_00000000_00001000_00000010
        );
    }

    #[test]
    fn test_dependency_type_dependency_part() {
        assert_eq!(Sf01DependencyType::Full.dependency_part(0x00ABCDEF), 0x00000000);
        assert_eq!(Sf01DependencyType::Full.dependency_part(0x99ABCDEF), 0x99000000);
        assert_eq!(Sf01DependencyType::Medium.dependency_part(0xFDABCDEF), 0xFDAB0000);
        assert_eq!(
            Sf01DependencyType::Small.dependency_part(0b11111110_01001011_11101000_00000010),
            0b11111110_01001011_11100000_00000000
        );
    }

    #[test]
    fn test_dependency_type_id_part_from_id() {
        assert_eq!(Sf01DependencyType::id_part_from_id(0x00ABCDEF), 0x00ABCDEF);
        assert_eq!(Sf01DependencyType::id_part_from_id(0x99ABCDEF), 0x00ABCDEF);
        assert_eq!(Sf01DependencyType::id_part_from_id(0xFDABCDEF), 0x0000CDEF);
        assert_eq!(
            Sf01DependencyType::id_part_from_id(0b11111110_01001011_11101000_00000010),
            0b00000000_00000000_00001000_00000010
        );
    }

    #[test]
    fn test_dependency_type_dependency_part_from_id() {
        assert_eq!(Sf01DependencyType::dependency_part_from_id(0x00ABCDEF), 0x00000000);
        assert_eq!(Sf01DependencyType::dependency_part_from_id(0x99ABCDEF), 0x99000000);
        assert_eq!(Sf01DependencyType::dependency_part_from_id(0xFDABCDEF), 0xFDAB0000);
        assert_eq!(
            Sf01DependencyType::dependency_part_from_id(0b11111110_01001011_11101000_00000010),
            0b11111110_01001011_11100000_00000000
        );
    }

    #[test]
    fn test_dependency_type_position() {
        assert_eq!(Sf01DependencyType::Full.positon(0x00ABCDEF), 0);
        assert_eq!(Sf01DependencyType::Full.positon(0x99ABCDEF), 0x99);
        assert_eq!(Sf01DependencyType::Medium.positon(0xFDABCDEF), 0xAB);
        assert_eq!(
            Sf01DependencyType::Small.positon(0b11111110_01001011_11101000_00000010),
            0b01001011_1110
        );
    }

    #[test]
    fn test_dependency_type_position_from_id() {
        assert_eq!(Sf01DependencyType::positon_from_id(0x00ABCDEF), 0);
        assert_eq!(Sf01DependencyType::positon_from_id(0x99ABCDEF), 0x99);
        assert_eq!(Sf01DependencyType::positon_from_id(0xFDABCDEF), 0xAB);
        assert_eq!(
            Sf01DependencyType::positon_from_id(0b11111110_01001011_11101000_00000010),
            0b01001011_1110
        );
    }

    #[test]
    fn test_dependency_type_max_position_index() {
        assert_eq!(Sf01DependencyType::Full.maximum_position(), 2u32.pow(8) - 4);
        assert_eq!(Sf01DependencyType::Medium.maximum_position(), 2u32.pow(8) - 1);
        assert_eq!(Sf01DependencyType::Small.maximum_position(), 2u32.pow(12) - 1);
    }

    #[test]
    fn test_dependency_type_position_to_dependency_part() {
        assert_eq!(
            Sf01DependencyType::Full
                .position_to_dependency_part(0x00)
                .unwrap(),
            0x00000000
        );
        assert_eq!(
            Sf01DependencyType::Full
                .position_to_dependency_part(0x42)
                .unwrap(),
            0x42000000
        );
        assert_eq!(
            Sf01DependencyType::Medium
                .position_to_dependency_part(0xAB)
                .unwrap(),
            0xFDAB0000
        );
        assert_eq!(
            Sf01DependencyType::Small
                .position_to_dependency_part(0b01001011_1110)
                .unwrap(),
            0b11111110_01001011_11100000_00000000
        );
    }

    #[test]
    fn test_dependency_type_position_to_dependency_part_position_to_large() {
        // Full
        assert!(
            Sf01DependencyType::Full
                .position_to_dependency_part(0x00)
                .is_ok(),
        );
        assert!(
            Sf01DependencyType::Full
                .position_to_dependency_part(0x42)
                .is_ok(),
        );
        assert!(
            Sf01DependencyType::Full
                .position_to_dependency_part(0xFC)
                .is_ok(),
        );
        assert!(
            Sf01DependencyType::Full
                .position_to_dependency_part(0xFD)
                .is_err(),
        );
        assert!(
            Sf01DependencyType::Full
                .position_to_dependency_part(0xFE)
                .is_err(),
        );
        assert!(
            Sf01DependencyType::Full
                .position_to_dependency_part(0xFF)
                .is_err(),
        );
        assert!(
            Sf01DependencyType::Full
                .position_to_dependency_part(0xABC)
                .is_err(),
        );
        // Medium
        assert!(
            Sf01DependencyType::Medium
                .position_to_dependency_part(0x00)
                .is_ok(),
        );
        assert!(
            Sf01DependencyType::Medium
                .position_to_dependency_part(0x42)
                .is_ok(),
        );
        assert!(
            Sf01DependencyType::Medium
                .position_to_dependency_part(0xFC)
                .is_ok(),
        );
        assert!(
            Sf01DependencyType::Medium
                .position_to_dependency_part(0xFD)
                .is_ok(),
        );
        assert!(
            Sf01DependencyType::Medium
                .position_to_dependency_part(0xFE)
                .is_ok(),
        );
        assert!(
            Sf01DependencyType::Medium
                .position_to_dependency_part(0xFF)
                .is_ok(),
        );
        assert!(
            Sf01DependencyType::Medium
                .position_to_dependency_part(0x100)
                .is_err(),
        );
        assert!(
            Sf01DependencyType::Medium
                .position_to_dependency_part(0xABC)
                .is_err(),
        );
        // Small
        assert!(
            Sf01DependencyType::Small
                .position_to_dependency_part(0x00)
                .is_ok(),
        );
        assert!(
            Sf01DependencyType::Small
                .position_to_dependency_part(0x42)
                .is_ok(),
        );
        assert!(
            Sf01DependencyType::Small
                .position_to_dependency_part(0xFC)
                .is_ok(),
        );
        assert!(
            Sf01DependencyType::Small
                .position_to_dependency_part(0xFD)
                .is_ok(),
        );
        assert!(
            Sf01DependencyType::Small
                .position_to_dependency_part(0xFE)
                .is_ok(),
        );
        assert!(
            Sf01DependencyType::Small
                .position_to_dependency_part(0xFF)
                .is_ok(),
        );
        assert!(
            Sf01DependencyType::Small
                .position_to_dependency_part(0xABC)
                .is_ok(),
        );
        assert!(
            Sf01DependencyType::Small
                .position_to_dependency_part(0b11111111_1111)
                .is_ok(),
        );
        assert!(
            Sf01DependencyType::Small
                .position_to_dependency_part(0b1_00000000_0000)
                .is_err(),
        );
    }
}
