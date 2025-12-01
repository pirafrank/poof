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

pub trait SemverArrayConversion {
    /// Converts a vector of strings to a vector of Version objects.
    fn to_version(&self) -> Vec<Version>;
}

impl SemverArrayConversion for Vec<String> {
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

impl SemverVersionConversion for &str {
    /// Converts a version String to an optional Version object.
    fn to_version(&self) -> Option<Version> {
        Version::parse(self).ok()
    }
}

pub trait SemverStringConversion {
    /// Converts a vector of Version objects to a vector of strings.
    fn to_string_vec(&self) -> Vec<String>;
}

impl SemverStringConversion for Vec<Version> {
    /// Converts a vector of Version objects to a vector of strings.
    fn to_string_vec(&self) -> Vec<String> {
        self.iter().map(|v| v.to_string()).collect()
    }
}

pub trait SemverStringPrefix {
    /// Fixes the version strings in the vector by removing any leading 'v' or 'V'.
    /// It returns a new vector without modifying the original.
    fn strip_v(&self) -> Self;
}

impl SemverStringPrefix for Vec<String> {
    /// Strips the leading 'v' or 'V' from each version string in the vector.
    /// It returns a new vector without modifying the original.
    /// This is useful to avoid version parsing issues when using semver crate.
    fn strip_v(&self) -> Self {
        let mut new_vec: Vec<String> = Vec::new();
        for version in self {
            if version.starts_with(['v', 'V']) {
                new_vec.push(version.clone()[1..].to_string());
            } else {
                new_vec.push(version.clone());
            }
        }
        new_vec
    }
}

impl SemverStringPrefix for String {
    /// Strips the leading 'v' or 'V' from the version string.
    /// It does not modify the original.
    /// This is useful to avoid version parsing issues when using semver crate.
    fn strip_v(&self) -> Self {
        let mut s: String = self.clone();
        if s.starts_with(['v', 'V']) {
            s.remove(0);
        }
        s
    }
}

impl SemverStringPrefix for &String {
    /// Strips the leading 'v' or 'V' from the version string.
    /// It does not modify the original.
    /// This is useful to avoid version parsing issues when using semver crate.
    fn strip_v(&self) -> Self {
        let s: &String = self;
        if s.starts_with(['v', 'V']) {
            s[1..].to_string();
        }
        s
    }
}

impl SemverStringPrefix for &str {
    /// Strips the leading 'v' or 'V' from the version string.
    /// It does not modify the original.
    /// This is useful to avoid version parsing issues when using semver crate.
    fn strip_v(&self) -> Self {
        if self.starts_with(['v', 'V']) {
            &self[1..]
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_semver_basic() {
        let mut versions = vec![
            "2.0.0".to_string(),
            "1.0.0".to_string(),
            "1.5.0".to_string(),
        ];
        versions.sort_semver();
        assert_eq!(versions, vec!["1.0.0", "1.5.0", "2.0.0"]);
    }

    #[test]
    fn test_sort_semver_with_prereleases() {
        let mut versions = vec![
            "1.0.0".to_string(),
            "1.0.0-alpha".to_string(),
            "1.0.0-beta1".to_string(),
            "1.0.0-alpha.2".to_string(),
        ];
        versions.sort_semver();
        assert_eq!(
            versions,
            vec!["1.0.0-alpha", "1.0.0-alpha.2", "1.0.0-beta1", "1.0.0"]
        );
    }

    #[test]
    fn test_sort_semver_with_invalid() {
        let mut versions = vec![
            "2.0.0".to_string(),
            "invalid".to_string(),
            "1.0.0".to_string(),
        ];
        versions.sort_semver();
        // Invalid versions should be sorted to the end
        assert_eq!(versions[0], "1.0.0");
        assert_eq!(versions[1], "2.0.0");
        assert_eq!(versions[2], "invalid");
    }

    #[test]
    fn test_sort_semver_all_invalid() {
        let mut versions = vec!["xyz".to_string(), "abc".to_string(), "def".to_string()];
        versions.sort_semver();
        // Should fallback to lexicographic sorting
        assert_eq!(versions, vec!["abc", "def", "xyz"]);
    }

    #[test]
    fn test_to_version_vec_basic() {
        let versions = vec![
            "1.0.0".to_string(),
            "2.0.0".to_string(),
            "3.0.0".to_string(),
        ];
        let version_objs = versions.to_version();
        assert_eq!(version_objs.len(), 3);
        assert_eq!(version_objs[0].to_string(), "1.0.0");
        assert_eq!(version_objs[1].to_string(), "2.0.0");
        assert_eq!(version_objs[2].to_string(), "3.0.0");
    }

    #[test]
    fn test_to_version_vec_with_invalid() {
        let versions = vec![
            "1.0.0".to_string(),
            "invalid".to_string(),
            "2.0.0".to_string(),
        ];
        let version_objs = versions.to_version();
        // Invalid versions should be skipped
        assert_eq!(version_objs.len(), 2);
        assert_eq!(version_objs[0].to_string(), "1.0.0");
        assert_eq!(version_objs[1].to_string(), "2.0.0");
    }

    #[test]
    fn test_to_version_string_types() {
        // Test both String and &str types with valid versions
        let owned = "1.2.3".to_string();
        let borrowed = "1.2.3";

        assert!(owned.to_version().is_some());
        assert_eq!(owned.to_version().unwrap().to_string(), "1.2.3");
        assert!(borrowed.to_version().is_some());
        assert_eq!(borrowed.to_version().unwrap().to_string(), "1.2.3");

        // Test invalid versions
        let invalid_owned = "invalid".to_string();
        let invalid_borrowed = "invalid";
        assert!(invalid_owned.to_version().is_none());
        assert!(invalid_borrowed.to_version().is_none());
    }

    #[test]
    fn test_to_string_vec() {
        let versions = vec![
            Version::parse("1.0.0").unwrap(),
            Version::parse("2.0.0").unwrap(),
            Version::parse("3.0.0").unwrap(),
        ];
        let strings = versions.to_string_vec();
        assert_eq!(strings, vec!["1.0.0", "2.0.0", "3.0.0"]);
    }

    #[test]
    fn test_strip_v() {
        // Test Vec<String>: lowercase, uppercase, no prefix, and edge cases
        let versions = vec![
            "v1.0.0".to_string(),
            "V2.0.0".to_string(),
            "3.0.0".to_string(),
            "version".to_string(), // Edge case: v prefix in word
        ];
        let stripped = versions.strip_v();
        assert_eq!(stripped, vec!["1.0.0", "2.0.0", "3.0.0", "ersion"]);

        // Test owned String: lowercase, uppercase, and no prefix
        assert_eq!("v1.2.3".to_string().strip_v(), "1.2.3");
        assert_eq!("V1.2.3".to_string().strip_v(), "1.2.3");
        assert_eq!("1.2.3".to_string().strip_v(), "1.2.3");

        // Test &str: lowercase, uppercase, and no prefix
        assert_eq!("v1.2.3".strip_v(), "1.2.3");
        assert_eq!("V1.2.3".strip_v(), "1.2.3");
        assert_eq!("1.2.3".strip_v(), "1.2.3");
    }

    #[test]
    fn test_integration_strip_sort_and_convert() {
        let versions = vec![
            "v2.0.0".to_string(),
            "V1.0.0".to_string(),
            "v1.5.0".to_string(),
        ];
        let mut stripped = versions.strip_v();
        stripped.sort_semver();
        assert_eq!(stripped, vec!["1.0.0", "1.5.0", "2.0.0"]);

        let version_objs = stripped.to_version();
        let back_to_strings = version_objs.to_string_vec();
        assert_eq!(back_to_strings, vec!["1.0.0", "1.5.0", "2.0.0"]);
    }
}
