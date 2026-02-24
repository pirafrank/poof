use lazy_static::lazy_static;
use regex::Regex;
use semver::{BuildMetadata, Prerelease};
use std::cmp::Ordering;

lazy_static! {
    static ref SEMVER_REGEX: Regex =
        Regex::new(r"(?i)^(?:[a-z\-\s]*)(\d+)(?:\.(\d+))?(?:\.(\d+))?(?:[.\-](.*))?$").unwrap();
}

/// Parses a version string loosely, handling leading zeros, 'v'/'r' prefixes, etc.
pub fn parse_lenient(version_str: &str) -> Option<semver::Version> {
    // Try standard parse first
    if let Ok(v) = semver::Version::parse(version_str) {
        return Some(v);
    }

    // Try parsing with regex
    if let Some(captures) = SEMVER_REGEX.captures(version_str) {
        let major = captures.get(1)?.as_str().parse::<u64>().ok()?;
        let minor = captures
            .get(2)
            .map_or(0, |m| m.as_str().parse::<u64>().unwrap_or(0));
        let patch = captures
            .get(3)
            .map_or(0, |m| m.as_str().parse::<u64>().unwrap_or(0));

        let pre_str = captures.get(4).map(|m| m.as_str()).unwrap_or("");
        let pre = Prerelease::new(pre_str).ok()?;
        let build = BuildMetadata::default();

        Some(semver::Version {
            major,
            minor,
            patch,
            pre,
            build,
        })
    } else {
        None
    }
}

/// A wrapper around semver::Version that preserves the original string representation.
/// This is useful for non-standard versions like "r35" or "01.02.03" that would
/// otherwise be unsupported by semver::Version.
#[derive(Debug, Clone)]
pub struct RawVersion {
    pub original: String,
    pub version: Option<semver::Version>,
}

pub type Version = RawVersion;

impl RawVersion {
    /// Creates a new RawVersion from a string.
    /// It uses parse_lenient to attempt to parse the version.
    pub fn new(s: String) -> Self {
        Self {
            version: parse_lenient(&s),
            original: s,
        }
    }

    /// Parses a string into a RawVersion.
    /// The Result return type may be not need yet done to match
    /// the signature of Version::parse to allow drop-in replacement,
    /// while using a more lenient parsing.
    pub fn parse(s: &str) -> Result<Self, semver::Error> {
        let rv = Self::new(s.to_string());
        if rv.version.is_none() {
            // we use semver::Version::parse to get a valid semver error
            return Err(semver::Version::parse("invalid").unwrap_err());
        }
        Ok(rv)
    }
}

impl PartialEq for RawVersion {
    fn eq(&self, other: &Self) -> bool {
        match (&self.version, &other.version) {
            (Some(v_a), Some(v_b)) => v_a == v_b,
            (None, None) => self.original == other.original,
            _ => false,
        }
    }
}

impl Eq for RawVersion {}

impl PartialOrd for RawVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RawVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.version, &other.version) {
            (Some(v_a), Some(v_b)) => v_a.cmp(v_b),
            (Some(_), None) => Ordering::Less, // Valid versions come before invalid/unparseable
            (None, Some(_)) => Ordering::Greater,
            (None, None) => self.original.cmp(&other.original), // Fallback to string comparison
        }
    }
}

impl std::fmt::Display for RawVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.original)
    }
}

impl From<String> for RawVersion {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for RawVersion {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

// allowing dead code for the sake of having a complete set
// of function available for the Asset struct.

/// Trait to extend `Vec<String>` with semantic version sorting
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
            let v_a = parse_lenient(a);
            let v_b = parse_lenient(b);
            match (v_a, v_b) {
                (Some(va), Some(vb)) => va.cmp(&vb),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => a.cmp(b), // fallback to normal string sort if neither is valid
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
            versions.push(RawVersion::new(version_str.clone()));
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
        Some(RawVersion::new(self.clone()))
    }
}

impl SemverVersionConversion for &str {
    /// Converts a version &str to an optional Version object.
    fn to_version(&self) -> Option<Version> {
        Some(RawVersion::new(self.to_string()))
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
    fn test_sort_semver_with_zeros() {
        let mut versions = vec![
            "2.0.0".to_string(),
            "0.1.0".to_string(),
            "1.05.0".to_string(),
        ];
        versions.sort_semver();
        assert_eq!(versions, vec!["0.1.0", "1.05.0", "2.0.0"]);
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
        // All strings are converted to RawVersion now, even invalid ones
        assert_eq!(version_objs.len(), 3);
        assert_eq!(version_objs[0].to_string(), "1.0.0");
        assert_eq!(version_objs[1].to_string(), "invalid");
        assert_eq!(version_objs[2].to_string(), "2.0.0");
    }

    #[test]
    fn test_to_version_string_types_basic() {
        // Test both String and &str types with valid versions
        let owned = "1.2.3".to_string();
        let borrowed = "1.2.3";

        assert!(owned.to_version().is_some());
        assert_eq!(owned.to_version().unwrap().to_string(), "1.2.3");
        assert!(borrowed.to_version().is_some());
        assert_eq!(borrowed.to_version().unwrap().to_string(), "1.2.3");
    }

    #[test]
    fn test_to_version_string_types_with_invalid() {
        // Test invalid versions - they still create RawVersion but with None for version field
        let invalid_owned = "invalid".to_string();
        let invalid_borrowed = "invalid";
        assert!(invalid_owned.to_version().is_some());
        assert!(invalid_borrowed.to_version().is_some());
        // The original string is preserved even if parsing fails
        assert_eq!(invalid_owned.to_version().unwrap().to_string(), "invalid");
        assert_eq!(
            invalid_borrowed.to_version().unwrap().to_string(),
            "invalid"
        );
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
            "version".to_string(), // Edge case: v prefix in word. Collateral: won't "fix".
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

    #[test]
    fn test_parse_lenient_cases() {
        // Leading zeros
        assert_eq!(parse_lenient("01.02.03").unwrap().to_string(), "1.2.3");
        // r prefix
        assert_eq!(parse_lenient("r35").unwrap().to_string(), "35.0.0");
        // v prefix
        assert_eq!(parse_lenient("v1.2").unwrap().to_string(), "1.2.0");
        // release- prefix
        assert_eq!(parse_lenient("release-1.0").unwrap().to_string(), "1.0.0");
        // Prerelease
        assert_eq!(
            parse_lenient("1.0.0-beta.1").unwrap().to_string(),
            "1.0.0-beta.1"
        );
        // Missing patch
        assert_eq!(parse_lenient("1.2").unwrap().to_string(), "1.2.0");
        // Missing minor and patch
        assert_eq!(parse_lenient("1").unwrap().to_string(), "1.0.0");
    }

    #[test]
    fn test_sort_semver_lenient() {
        let mut versions = vec![
            "r35".to_string(),
            "r4".to_string(),
            "v1.0.0".to_string(),
            "0.1.0".to_string(),
        ];
        versions.sort_semver();
        // 0.1.0 (0.1.0)
        // v1.0.0 (1.0.0)
        // r4 (4.0.0)
        // r35 (35.0.0)
        assert_eq!(versions, vec!["0.1.0", "v1.0.0", "r4", "r35"]);
    }

    #[test]
    fn test_raw_version_ordering() {
        let mut versions = [
            RawVersion::new("r35".to_string()),
            RawVersion::new("r4".to_string()),
            RawVersion::new("v1.0.0".to_string()),
            RawVersion::new("0.1.0".to_string()),
        ];
        versions.sort();

        assert_eq!(versions[0].to_string(), "0.1.0");
        assert_eq!(versions[1].to_string(), "v1.0.0");
        assert_eq!(versions[2].to_string(), "r4");
        assert_eq!(versions[3].to_string(), "r35");
    }

    #[test]
    fn test_raw_version_display() {
        let v = RawVersion::new("01.02.03".to_string());
        assert_eq!(v.to_string(), "01.02.03");
        assert_eq!(v.version.unwrap().to_string(), "1.2.3");
    }
}
