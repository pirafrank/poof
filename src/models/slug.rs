#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Slug(pub String);

// allowing dead code for the sake of having a complete set
// of function available for the Slug struct.
#[allow(dead_code)]
impl Slug {
    // Create a new Slug from a repo and a version
    pub fn new(repo: &str, version: &str) -> Self {
        Self(format!("{}/{}", repo.trim(), version.trim()))
    }

    // Create a new Slug from a String
    pub fn new_from_string(s: impl Into<String>) -> Self {
        Self(s.into())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slug_new() {
        let slug = Slug::new("user/repo");
        assert_eq!(slug.0, "user/repo");
    }

    #[test]
    fn test_slug_as_str() {
        let slug = Slug::new("user/repo");
        assert_eq!(slug.as_str(), "user/repo");
    }

    #[test]
    fn test_slug_get_username() {
        let slug = Slug::new("user/repo");
        assert_eq!(slug.get_username(), Some("user".to_string()));

        let no_slash = Slug::new("userrepo");
        assert_eq!(no_slash.get_username(), None);
    }

    #[test]
    fn test_slug_get_reponame() {
        let slug = Slug::new("user/repo");
        assert_eq!(slug.get_reponame(), Some("repo".to_string()));

        let no_slash = Slug::new("userrepo");
        assert_eq!(no_slash.get_reponame(), None);
    }

    #[test]
    fn test_slug_display() {
        let slug = Slug::new("user/repo");
        assert_eq!(format!("{}", slug), "user/repo");
    }

    #[test]
    fn test_slug_from_string() {
        let s = "user/repo".to_string();
        let slug = Slug::from(s);
        assert_eq!(slug.0, "user/repo");
    }

    #[test]
    fn test_slug_from_str() {
        let slug = Slug::from("user/repo");
        assert_eq!(slug.0, "user/repo");
    }

    #[test]
    fn test_slug_deref() {
        let slug = Slug::new("user/repo");
        assert_eq!(slug.len(), 9);
        assert!(slug.contains("user"));
    }

    #[test]
    fn test_slug_deref_mut() {
        let mut slug = Slug::new("user/repo");
        slug.push_str("/extra");
        assert_eq!(slug.0, "user/repo/extra");
    }

    #[test]
    fn test_slug_partial_eq() {
        let slug1 = Slug::new("user/repo");
        let slug2 = Slug::new("user/repo");
        let slug3 = Slug::new("other/repo");
        assert_eq!(slug1, slug2);
        assert_ne!(slug1, slug3);
    }
}
