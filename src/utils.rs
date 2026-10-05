//! This module contains utility functions.

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
}
