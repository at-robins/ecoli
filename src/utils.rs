//! This module contains utility functions.

use crate::error::{ApplicationError, ApplicationErrorType};

/// Returns true if the bit at the specified index is set.
///
/// # Parameters
///
/// * `value` - the bit sequence
/// * `bit_index` - the index of the bit to check
///
/// # Panics
///
/// If the index is out of bounds.
pub fn is_bit_set_u32(value: u32, bit_index: u32) -> bool {
    if bit_index >= u32::BITS {
        panic!("Index {} is out of bounds for type u32.", bit_index);
    }
    ((value >> bit_index) & 1) == 1
}

/// Sets a bit at the specified index and returns the respective value.
///
/// # Parameters
///
/// * `value` - the base bit sequence
/// * `bit_index` - the index of the bit to set
/// * `bit` - the bit value to set
///
/// # Panics
///
/// If the index is out of bounds.
pub fn set_bit_u32(value: u32, bit_index: u32, bit: bool) -> u32 {
    if bit_index >= u32::BITS {
        panic!("Index {} is out of bounds for type u32.", bit_index);
    }
    if bit {
        value | (1 << bit_index)
    } else {
        value & (!(1 << bit_index))
    }
}

pub struct NullTerminatedString {
    value: String,
}

impl NullTerminatedString {
    const TERMINATION_CHARACTER: char = '\0';

    /// Creates a new NULL terminated string if the input string does not contain internal termination characters.
    ///
    /// # Paramerters
    ///
    /// * `value` - the value to convert to a NULL terminated string (might already be a NULL terminated string)
    pub fn new<T: Into<String>>(value: T) -> Result<Self, ApplicationError> {
        let mut value = value.into();
        // If this string is already NULL terminated, the termination character is removed,
        // before the subsequent check for invalid termination characters.
        if let Some(last_character) = value.chars().last()
            && last_character == Self::TERMINATION_CHARACTER
        {
            let _ = value.pop();
        }
        if value.contains(Self::TERMINATION_CHARACTER) {
            Err(ApplicationError::new(
                ApplicationErrorType::InputDataError,
                format!(
                    "The string {} contains termination character {}.",
                    value,
                    Self::TERMINATION_CHARACTER
                ),
            ))
        } else {
            Ok(Self { value })
        }
    }

    // Returns the actual internal string representation without the termination character.
    pub fn value(&self) -> &str {
        self.value.as_str()
    }

    /// Returns the UTF8 encoded string as byte vector including the termination character.
    pub fn serialise(&self) -> Vec<u8> {
        let mut termination_character_buffer = [0; 1];
        let mut serialised = Vec::new();
        serialised.extend_from_slice(self.value.as_bytes());
        serialised.extend_from_slice(
            Self::TERMINATION_CHARACTER
                .encode_utf8(&mut termination_character_buffer)
                .as_bytes(),
        );
        serialised
    }
}

impl Default for NullTerminatedString {
    fn default() -> Self {
        Self { value: Default::default() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_bit_set_u32() {
        for i in 0..u32::BITS {
            assert!(!is_bit_set_u32(0, i));
            if i == 0 {
                assert!(is_bit_set_u32(1, i));
            } else {
                assert!(!is_bit_set_u32(1, i))
            }
            if i == 31 || i == 15 || i == 13 {
                assert!(is_bit_set_u32(0b10000000000000001010000000000000, i));
            } else {
                assert!(!is_bit_set_u32(0b10000000000000001010000000000000, i))
            }
        }
    }

    #[test]
    #[should_panic]
    fn test_is_bit_set_u32_panic() {
        is_bit_set_u32(0, 32);
    }

    #[test]
    fn test_set_bit_u32() {
        assert_eq!(set_bit_u32(0, 0, true), 1);
        assert_eq!(set_bit_u32(1, 2, true), 5);
        assert_eq!(set_bit_u32(5, 0, false), 4);
        assert_eq!(set_bit_u32(5, 0, true), 5);
        assert_eq!(
            set_bit_u32(0b00000000000000001010000000000000, 31, true),
            0b10000000000000001010000000000000
        );
    }

    #[test]
    #[should_panic]
    fn test_set_bit_u32_panic() {
        set_bit_u32(0, 32, true);
    }

    #[test]
    fn test_null_terminated_string() {
        assert!(NullTerminatedString::new("test\0test").is_err());
        assert!(NullTerminatedString::new("\0testtest").is_err());
        assert!(NullTerminatedString::new("testtest\0\0").is_err());
        assert!(NullTerminatedString::new("\0\0").is_err());
        assert_eq!(NullTerminatedString::new("testtest\0").unwrap().value(), "testtest");
        assert_eq!(NullTerminatedString::new("testtest").unwrap().value(), "testtest");
        assert_eq!(NullTerminatedString::new("\0").unwrap().value(), "");
        assert_eq!(NullTerminatedString::new("").unwrap().value(), "");
    }

    #[test]
    fn test_null_terminated_string_serialise() {
        assert_eq!(
            NullTerminatedString::new("test").unwrap().serialise(),
            vec![116, 101, 115, 116, 0]
        );
        assert_eq!(
            NullTerminatedString::new("test\0").unwrap().serialise(),
            vec![116, 101, 115, 116, 0]
        );
        assert_eq!(
            NullTerminatedString::new("testñ").unwrap().serialise(),
            vec![116, 101, 115, 116, 195, 177, 0]
        );
        assert_eq!(
            NullTerminatedString::new("testñ\0").unwrap().serialise(),
            vec![116, 101, 115, 116, 195, 177, 0]
        );
    }
}
