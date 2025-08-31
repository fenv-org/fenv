use semver::Version;
use serde::{Deserialize, Serialize};

/// Represents a fenv release version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FenvRelease {
    /// Semantic version string, e.g. "0.2.0"
    pub version: String,
}

impl FenvRelease {
    /// Creates a new FenvRelease with validated semantic version.
    ///
    /// # Arguments
    /// * `version` - A valid semantic version string (e.g. "1.2.3")
    ///
    /// # Returns
    /// * `Some(FenvRelease)` if version is valid
    /// * `None` if version is invalid
    pub fn new(version: &str) -> Option<Self> {
        if Version::parse(version).is_ok() {
            Some(Self {
                version: version.to_string(),
            })
        } else {
            None
        }
    }

    /// Creates a FenvRelease from a version string or tag (e.g. "v0.2.0" or "0.2.0").
    pub fn from_version_or_tag(version_or_tag: &str) -> Option<Self> {
        let v = version_or_tag.trim_start_matches('v');
        Self::new(v)
    }

    /// Returns the parsed semver::Version.
    ///
    /// This method is safe to call because FenvRelease can only be created
    /// with valid semantic versions.
    pub fn semver(&self) -> Version {
        Version::parse(&self.version).unwrap()
    }

    /// Returns true if self is newer than other.
    pub fn is_newer_than(&self, other: &FenvRelease) -> bool {
        self.semver() > other.semver()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_version_or_tag_valid() {
        let r1 = FenvRelease::from_version_or_tag("v1.2.3");
        let r2 = FenvRelease::from_version_or_tag("1.2.3");
        assert_eq!(
            r1,
            Some(FenvRelease {
                version: "1.2.3".to_string()
            })
        );
        assert_eq!(
            r2,
            Some(FenvRelease {
                version: "1.2.3".to_string()
            })
        );
    }

    #[test]
    fn test_from_version_or_tag_invalid() {
        assert_eq!(FenvRelease::from_version_or_tag("not-a-version"), None);
        assert_eq!(FenvRelease::from_version_or_tag("v1.2"), None);
    }

    #[test]
    fn test_new() {
        let r = FenvRelease::new("2.0.0").unwrap();
        assert_eq!(r.version, "2.0.0");
        assert_eq!(FenvRelease::new("not-a-version"), None);
    }

    #[test]
    fn test_semver() {
        let r = FenvRelease::from_version_or_tag("v2.0.0").unwrap();
        let semver = r.semver();
        assert_eq!(semver, Version::parse("2.0.0").unwrap());
    }

    #[test]
    fn test_is_newer_than() {
        let old = FenvRelease::from_version_or_tag("v1.0.0").unwrap();
        let new = FenvRelease::from_version_or_tag("v2.0.0").unwrap();
        assert!(new.is_newer_than(&old));
        assert!(!old.is_newer_than(&new));
        // same version
        let same = FenvRelease::from_version_or_tag("1.0.0").unwrap();
        assert!(!old.is_newer_than(&same));
    }
}
