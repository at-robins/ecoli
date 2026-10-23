//! This module provides identity and dependency data structures.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// The type of dependency. Defines the number of possible dependencies of the specified type
/// and the maximum number of owned entities based on the ID scheme.
pub enum Sf01DependencyType {
    /// A full dependency / master. Can own the most entities, but is limited in numbers.
    Full,
    /// A medium dependency / master. Can own a medium amount of entities,
    /// and can be present in medium numbers.
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
            Sf01DependencyType::Medium => 0b11111111_11110000_00000000_00000000,
            Sf01DependencyType::Small => 0b11111111_11111111_00000000_00000000,
        }
    }

    /// Returns a bitmask for the ID part of an entitie's ID.
    pub fn id_mask(&self) -> u32 {
        0b11111111_11111111_11111111_11111111 ^ self.dependency_mask()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_type_dependency_mask() {
        assert_eq!(Sf01DependencyType::Full.dependency_mask(), 0b11111111_00000000_00000000_00000000);
        assert_eq!(Sf01DependencyType::Medium.dependency_mask(), 0b11111111_11110000_00000000_00000000);
        assert_eq!(Sf01DependencyType::Small.dependency_mask(), 0b11111111_11111111_00000000_00000000);
    }

    #[test]
    fn test_dependency_type_id_mask() {
        assert_eq!(Sf01DependencyType::Full.id_mask(), 0b00000000_11111111_11111111_11111111);
        assert_eq!(Sf01DependencyType::Medium.id_mask(), 0b00000000_00001111_11111111_11111111);
        assert_eq!(Sf01DependencyType::Small.id_mask(), 0b00000000_00000000_11111111_11111111);
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
        assert_eq!(Sf01DependencyType::Medium.id_part(0b11111101_01001011_11101000_00000010), 0b00000000_00001011_11101000_00000010);
        assert_eq!(Sf01DependencyType::Small.id_part(0xFEABCDEF), 0x0000CDEF);
    }

    #[test]
    fn test_dependency_type_dependency_part() {
        assert_eq!(Sf01DependencyType::Full.dependency_part(0x00ABCDEF), 0x00000000);
        assert_eq!(Sf01DependencyType::Full.dependency_part(0x99ABCDEF), 0x99000000);
        assert_eq!(Sf01DependencyType::Medium.dependency_part(0b11111101_01001011_11101000_00000010), 0b11111101_01000000_00000000_00000000);
        assert_eq!(Sf01DependencyType::Small.dependency_part(0xFEABCDEF), 0xFEAB0000);
    }

    #[test]
    fn test_dependency_type_id_part_from_id() {
        assert_eq!(Sf01DependencyType::id_part_from_id(0x00ABCDEF), 0x00ABCDEF);
        assert_eq!(Sf01DependencyType::id_part_from_id(0x99ABCDEF), 0x00ABCDEF);
        assert_eq!(Sf01DependencyType::id_part_from_id(0b11111101_01001011_11101000_00000010), 0b00000000_00001011_11101000_00000010);
        assert_eq!(Sf01DependencyType::id_part_from_id(0xFEABCDEF), 0x0000CDEF);
    }

    #[test]
    fn test_dependency_type_dependency_part_from_id() {
        assert_eq!(Sf01DependencyType::dependency_part_from_id(0x00ABCDEF), 0x00000000);
        assert_eq!(Sf01DependencyType::dependency_part_from_id(0x99ABCDEF), 0x99000000);
        assert_eq!(Sf01DependencyType::dependency_part_from_id(0b11111101_01001011_11101000_00000010), 0b11111101_01000000_00000000_00000000);
        assert_eq!(Sf01DependencyType::dependency_part_from_id(0xFEABCDEF), 0xFEAB0000);
    }
}

