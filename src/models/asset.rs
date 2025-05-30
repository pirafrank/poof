//! An installed binary having a repo name (in the format <USER>/<REPO>)
//! and a list of versions is an 'asset'.

use crate::utils::semver::*;
use semver::Version;
use std::cmp::Ordering;

use super::repostring::RepoString;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Asset {
    name: RepoString,
    versions: Vec<Version>,
}

// allowing dead code for the sake of having a complete set
// of function available for the Asset struct.
#[allow(dead_code)]
impl Asset {
    /// Creates a new Asset instance with the given name and versions.
    pub fn new(name: String, versions: Vec<Version>) -> Self {
        Self {
            name: RepoString(name),
            versions,
        }
    }

    /// Creates a new Asset instance with the given name and versions as strings.
    pub fn new_as_string(name: String, versions_str: Vec<String>) -> Self {
        let mut versions = versions_str.strip_v().to_version();
        versions.sort();
        Self {
            name: RepoString(name),
            versions,
        }
    }

    /// Creates a new Asset instance with the given name and an empty version list.
    pub fn get_name(&self) -> &String {
        &self.name
    }

    /// Returns a reference to the vector of versions.
    pub fn get_versions(&self) -> &Vec<Version> {
        &self.versions
    }

    /// Sets the name of the asset.
    pub fn set_name(&mut self, name: String) {
        self.name = RepoString(name);
    }

    /// Sets the vector of versions.
    pub fn set_versions(&mut self, versions: Vec<Version>) {
        self.versions = versions;
    }

    /// Adds a version to the vector of versions.
    /// If the version already exists, it will not be added again.
    pub fn add_version(&mut self, version: Version) {
        if !self.versions.contains(&version) {
            self.versions.push(version);
            self.versions.sort();
        }
    }

    /// Adds a version to the vector of versions taking a string as input.
    /// If the version already exists, it will not be added again.
    pub fn add_version_as_string(&mut self, version_str: &str) {
        let stripped = version_str.to_string().strip_v();
        let version = stripped.to_version().unwrap();
        if !self.versions.contains(&version) {
            self.versions.push(version);
            self.versions.sort();
        }
    }

    /// Removes a version from the vector of versions.
    /// If the version does not exist, nothing happens.
    pub fn remove_version(&mut self, version: Version) {
        self.versions.retain(|v| v != &version);
    }

    /// Removes a version from the vector of versions taking a string as input.
    /// If the version string is invalid or the corresponding version does
    /// not exist, nothing happens.
    pub fn remove_version_as_string(&mut self, version_str: &str) {
        let stripped = version_str.to_string().strip_v();
        let version = stripped.to_version().unwrap();
        self.versions.retain(|v| v != &version);
    }

    /// Removes all versions from the vector of versions.
    pub fn clear_versions(&mut self) {
        self.versions.clear();
    }

    /// Checks if the vector of versions is empty.
    pub fn is_empty(&self) -> bool {
        self.versions.is_empty()
    }

    /// Returns the latest version as a string.
    /// If there are no versions, returns None.
    pub fn get_latest_version(&self) -> Option<String> {
        match self.versions.last() {
            Some(v) => {
                let v_str = v.to_string();
                Some(v_str)
            }
            None => None,
        }
    }

    /// Checks if the vector of versions contains a specific version
    pub fn contains_version(&self, version_str: &str) -> bool {
        let version = match Version::parse(version_str) {
            Ok(v) => v,
            Err(_) => return false, // Invalid version string, do nothing
        };
        self.versions.contains(&version)
    }

    /// Compare two Asset instances. Comparison is based on their names.
    pub fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }

    /// Sort the vector of versions in ascending order.
    /// This is useful for ensuring that the versions are in a consistent
    /// and semantic versioning order. Ordering is done in place.
    pub fn sort(&mut self) {
        self.versions.sort();
    }
}

/// Return a String representation of Asset.
impl std::fmt::Display for Asset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let versions_str = self
            .versions
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "{}: {}", self.name, versions_str)
    }
}

pub trait VecAssets {
    /// Sorts the vector of Asset instances in place based on their names.
    fn sort(&mut self);
}

impl VecAssets for Vec<Asset> {
    /// Sorts the vector of Asset instances in place based on their names.
    fn sort(&mut self) {
        self.sort_by(|a, b| a.cmp(b));
    }
}
