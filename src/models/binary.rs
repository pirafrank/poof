//! An installed binary having a repo name (in the format <USER>/<REPO>)
//! and a version is a 'binary'.

use crate::utils::semver::*;
use semver::Version;

use super::repostring::RepoString;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Binary {
    name: RepoString,
    version: Version,
}

// allowing dead code for the sake of having a complete set
// of function available for the Binary struct.
#[allow(dead_code)]
impl Binary {
    /// Creates a new Binary instance with the given name and version.
    pub fn new(name: String, version: Version) -> Self {
        Self {
            name: RepoString(name),
            version,
        }
    }

    /// Creates a new Binary instance with the given name and version as a string.
    pub fn new_as_string(name: String, version_str: &str) -> Self {
        let version: Option<Version> = version_str.strip_v().to_version();
        Self {
            name: RepoString(name),
            version: version.unwrap(),
        }
    }

    /// Returns a reference to the name of the binary.
    pub fn get_name(&self) -> &String {
        &self.name
    }

    /// Returns a reference to the version of the binary.
    pub fn get_version(&self) -> &Version {
        &self.version
    }

    /// Sets the name of the binary.
    pub fn set_name(&mut self, name: String) {
        self.name = RepoString(name);
    }

    /// Sets the version of the binary.
    pub fn set_version(&mut self, version: Version) {
        self.version = version;
    }
}
