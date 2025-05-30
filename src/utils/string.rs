//!
//! This file contains utility functions for string manipulation.
//!

/// Get the position of a substring in a string where the string is split by a separator.
pub fn position_of_str_in_string(input: String, sep: &str, item: &str) -> i16 {
    let mut position: i16 = 0;
    let mut found = false;
    for i in input.split(sep) {
        if i == item {
            found = true;
            break;
        }
        position += 1;
    }
    if found {
        return position;
    };
    -1 // not found
}
