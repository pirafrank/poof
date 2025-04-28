//!
//! This file contains utility functions for string manipulation.
//!

/// Get the position of a substring in a string where the string is split by a separator.
pub fn position_of_str_in_string(input: String, sep: &str, item: &str) -> u16 {
    let mut position: u16 = 0;
    for i in input.split(sep) {
        position += 1;
        if i == item {
            break;
        }
    }
    position
}
