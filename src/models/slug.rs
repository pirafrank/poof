#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Slug(pub String);

// allowing dead code for the sake of having a complete set
// of function available for the Slug struct.
#[allow(dead_code)]
impl Slug {
    // Create a new Slug from a String
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

    /// Returns the username of repo, which is the first part of the Slug
    /// before the first '/'
    pub fn get_username(&self) -> Option<String> {
        if let Some((username, _)) = self.0.split_once('/') {
            Some(username.to_string())
        } else {
            None
        }
    }

    /// Returns the repository name, which is the second part of the Slug
    /// after the first '/'
    pub fn get_reponame(&self) -> Option<String> {
        if let Some((_, reponame)) = self.0.split_once('/') {
            Some(reponame.to_string())
        } else {
            None
        }
    }
}

// Return a String representation of Slug.
impl std::fmt::Display for Slug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Implement From traits for easy conversion
impl From<String> for Slug {
    fn from(s: String) -> Self {
        Slug(s)
    }
}

impl From<&str> for Slug {
    fn from(s: &str) -> Self {
        Slug(s.to_string())
    }
}

// Implement Deref to allow using String methods directly
impl std::ops::Deref for Slug {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Slug {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
