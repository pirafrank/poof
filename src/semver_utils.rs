use semver::Version;
use std::cmp::Ordering;

// allowing dead code for the sake of having a complete set
// of function available for the Asset struct.

/// Trait to extend Vec<String> with semantic version sorting
pub trait SemverSort {
    #[allow(dead_code)]
    /// Sorts the vector of strings in place using semantic versioning rules.
    fn sort_semver(&mut self);
}

impl SemverSort for Vec<String> {
    /// Sorts the vector of strings in place using semantic versioning rules.
    /// The sorting happens in place.
    /// If a version string is invalid, it will be sorted to the end of the list.
    fn sort_semver(&mut self) {
        self.sort_by(|a, b| {
            let v_a = Version::parse(a);
            let v_b = Version::parse(b);
            match (v_a, v_b) {
                (Ok(va), Ok(vb)) => va.cmp(&vb),
                (Ok(_), Err(_)) => Ordering::Less,
                (Err(_), Ok(_)) => Ordering::Greater,
                (Err(_), Err(_)) => a.cmp(b), // fallback to normal string sort if neither is valid
            }
        });
    }
}

pub trait SemverConversion {
    /// Converts a vector of strings to a vector of Version objects.
    fn to_version(&self) -> Vec<Version>;
}

impl SemverConversion for Vec<String> {
    /// Converts a vector of strings to a vector of Version objects.
    fn to_version(&self) -> Vec<Version> {
        let mut versions: Vec<Version> = Vec::new();
        for version_str in self {
            match Version::parse(version_str) {
                Ok(version) => versions.push(version),
                Err(_) => continue, // Invalid version string, skip it
            }
        }
        versions
    }
}

pub trait SemverVersionConversion {
    /// Converts a version String to an optional Version object.
    fn to_version(&self) -> Option<Version>;
}

impl SemverVersionConversion for String {
    /// Converts a version String to an optional Version object.
    fn to_version(&self) -> Option<Version> {
        Version::parse(self).ok()
    }
}

pub trait SemverStringConversion {
    /// Converts a vector of Version objects to a vector of strings.
    fn to_string(&self) -> Vec<String>;
}

impl SemverStringConversion for Vec<Version> {
    /// Converts a vector of Version objects to a vector of strings.
    fn to_string(&self) -> Vec<String> {
        self.iter().map(|v| v.to_string()).collect()
    }
}

pub trait SemverStringPrefix {
    /// Fixes the version strings in the vector by removing any leading 'v' or 'V'.
    /// Edits happen in place.
    fn strip_v_in_place(&mut self);

    /// Fixes the version strings in the vector by removing any leading 'v' or 'V'.
    /// It returns a new vector without modifying the original.
    fn strip_v(&self) -> Self;
}

impl SemverStringPrefix for Vec<String> {
    /// Strips the leading 'v' or 'V' from each version string in the vector.
    /// It modifies the vector in place.
    /// This is useful to avoid version parsing issues when using semver crate.
    fn strip_v_in_place(&mut self) {
        for version_str in self.iter_mut() {
            if version_str.starts_with(['v', 'V']) {
                *version_str = version_str[1..].to_string();
            }
        }
    }

    /// Strips the leading 'v' or 'V' from each version string in the vector.
    /// It returns a new vector without modifying the original.
    /// This is useful to avoid version parsing issues when using semver crate.
    fn strip_v(&self) -> Self {
        let mut new_vec = self.clone();
        new_vec.strip_v_in_place();
        new_vec
    }
}

impl SemverStringPrefix for String {
    /// Strips the leading 'v' or 'V' from the version string.
    /// It modifies the string in place.
    /// This is useful to avoid version parsing issues when using semver crate.
    fn strip_v_in_place(&mut self) {
        if self.starts_with(['v', 'V']) {
            *self = self[1..].to_string();
        }
    }

    /// Strips the leading 'v' or 'V' from the version string.
    /// It returns a new string without modifying the original.
    /// This is useful to avoid version parsing issues when using semver crate.
    fn strip_v(&self) -> Self {
        let mut new_str = self.clone();
        new_str.strip_v_in_place();
        new_str
    }
}
