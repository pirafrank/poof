use anyhow::bail;
use anyhow::Error;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Slug(pub String);

// allowing dead code for the sake of having a complete set
// of function available for the Slug struct.
#[allow(dead_code)]
impl Slug {
    // Create a new Slug from a "user/repo" string with validation
    pub fn new(repo_slug: &str) -> Result<Self, Error> {
        let mut parts = repo_slug.split('/');
        match (parts.next(), parts.next(), parts.next()) {
            (Some(user), Some(repo), None) => Ok(Slug::from_parts(user, repo)?),
            _ => bail!("Invalid slug format: {}", repo_slug),
        }
    }

    // Create a new Slug from separate username and repository name
    pub fn from_parts(user: &str, repo: &str) -> Result<Self, Error> {
        let user = user.trim();
        let repo = repo.trim();
        if user.is_empty() || repo.is_empty() {
            bail!("Invalid slug format: {}/{}", user, repo)
        }
        Ok(Slug(format!("{}/{}", user, repo)))
    }

    // Get the underlying String
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Return a String representation of Slug.
impl std::fmt::Display for Slug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Implement From traits for easy conversion
impl TryFrom<String> for Slug {
    type Error = Error;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Slug::new(&s)
    }
}

impl TryFrom<&str> for Slug {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Slug::new(s)
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
        assert_eq!(slug.unwrap().as_str(), "user/repo");
    }

    #[test]
    fn test_slug_new_invalid() {
        // Empty parts
        let slug = Slug::new("");
        assert!(slug.is_err());

        // Missing repo
        let slug = Slug::new("user");
        assert!(slug.is_err());

        // Too many parts
        let slug = Slug::new("user/repo/extra");
        assert!(slug.is_err());
    }

    #[test]
    fn test_slug_from_parts() {
        let slug = Slug::from_parts("user", "repo");
        assert_eq!(slug.unwrap().as_str(), "user/repo");
    }

    #[test]
    fn test_slug_from_parts_invalid() {
        let slug = Slug::from_parts("", "repo");
        assert!(slug.is_err());

        let slug = Slug::from_parts("user", "");
        assert!(slug.is_err());
    }

    #[test]
    fn test_slug_as_str() {
        let slug = Slug::new("user/repo").unwrap();
        assert_eq!(slug.as_str(), "user/repo");
    }

    #[test]
    fn test_slug_display() {
        let slug = Slug::new("user/repo").unwrap();
        assert_eq!(format!("{}", slug), "user/repo");
    }

    #[test]
    fn test_slug_try_from_string() {
        let s = "user/repo".to_string();
        let slug: Result<Slug, _> = s.try_into();
        assert_eq!(slug.unwrap().as_str(), "user/repo");
    }

    #[test]
    fn test_slug_try_from_str() {
        let slug: Result<Slug, _> = "user/repo".try_into();
        assert_eq!(slug.unwrap().as_str(), "user/repo");
    }

    #[test]
    fn test_slug_partial_eq() {
        let slug1 = Slug::new("user/repo").unwrap();
        let slug2 = Slug::new("user/repo").unwrap();
        let slug3 = Slug::new("other/repo").unwrap();
        assert_eq!(slug1, slug2);
        assert_ne!(slug1, slug3);
    }

    #[test]
    fn test_slug_deref() {
        let slug = Slug::new("user/repo").unwrap();
        assert_eq!(slug.len(), 9);
        assert!(slug.contains("user"));
    }

    #[test]
    fn test_slug_trimming() {
        // Whitespace-only parts should be rejected
        assert!(Slug::new(" / ").is_err());
        assert!(Slug::new("user/   ").is_err());
        assert!(Slug::new("   /repo").is_err());
        assert!(Slug::from_parts(" ", "repo").is_err());
        assert!(Slug::from_parts("user", " ").is_err());

        // Valid slug with whitespace should be accepted and cleaned up
        let slug = Slug::new("  user  /  repo  ").unwrap();
        assert_eq!(slug.as_str(), "user/repo");
    }
}
