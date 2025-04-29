use lazy_static::lazy_static;
use regex::Regex;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct RepoString(pub String);

lazy_static! {
    static ref REPO_REGEX: Regex = Regex::new(r"^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$").unwrap();
}

// allowing dead code for the sake of having a complete set
// of function available for the Binary struct.
#[allow(dead_code)]
impl RepoString {
    // Create a new RepoString from a String
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    // Example method: count the number of specific characters
    pub fn count_char(&self, c: char) -> usize {
        self.0.chars().filter(|&ch| ch == c).count()
    }

    // Get the underlying String
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check if the RepoString is valid
    /// A valid RepoString is in the format <USER>/<REPO>
    pub fn is_valid(&self) -> bool {
        REPO_REGEX.is_match(&self.0)
    }

    /// Returns the username of repo, which is the first part of the RepoString
    /// before the first '/'
    pub fn get_username(&self) -> Option<String> {
        if self.is_valid() {
            let parts: Vec<&str> = self.0.split('/').collect();
            return Some(parts[0].to_string());
        }
        None
    }

    /// Returns the repository name, which is the second part of the RepoString
    /// after the first '/'
    pub fn get_reponame(&self) -> Option<String> {
        if self.is_valid() {
            let parts: Vec<&str> = self.0.split('/').collect();
            return Some(parts[1].to_string());
        }
        None
    }
}

// Return a String representation of RepoString.
impl std::fmt::Display for RepoString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Implement From traits for easy conversion
impl From<String> for RepoString {
    fn from(s: String) -> Self {
        RepoString(s)
    }
}

impl From<&str> for RepoString {
    fn from(s: &str) -> Self {
        RepoString(s.to_string())
    }
}

// Implement Deref to allow using String methods directly
impl std::ops::Deref for RepoString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for RepoString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
