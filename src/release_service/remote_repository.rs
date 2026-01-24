use crate::external::github_client::GitHubClient;
use crate::release_service::model::fenv_release::FenvRelease;
use anyhow::Result;

/// Information about a fenv release fetched from GitHub.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FenvReleaseInfo {
    pub release: FenvRelease,
    pub html_url: String,
    pub published_at: String,
}

/// Repository for fetching release information from GitHub.
pub struct RemoteReleaseRepository;

impl Default for RemoteReleaseRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl RemoteReleaseRepository {
    /// Creates a new RemoteReleaseRepository.
    pub fn new() -> Self {
        Self
    }

    /// Fetches the latest release information from GitHub.
    ///
    /// # Arguments
    /// * `github_client` - GitHub client implementation for making API calls
    ///
    /// # Returns
    /// * `Ok(Some(FenvReleaseInfo))` if a release was found and parsed successfully
    /// * `Ok(None)` if no release was found (404)
    /// * `Err` if the API call failed or the version tag is invalid
    pub fn fetch_latest_release(
        &self,
        github_client: &impl GitHubClient,
    ) -> Result<Option<FenvReleaseInfo>> {
        let response = github_client.fetch_latest_release()?;

        let Some(release_response) = response else {
            return Ok(None);
        };

        // Parse the tag_name (which may be "v1.2.3" or "1.2.3") into a FenvRelease
        let release = FenvRelease::from_version_or_tag(&release_response.tag_name);

        match release {
            Some(rel) => Ok(Some(FenvReleaseInfo {
                release: rel,
                html_url: release_response.html_url,
                published_at: release_response.published_at,
            })),
            None => {
                // Invalid version tag - this is unexpected but not fatal
                // Return None to indicate no valid release found
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::external::github_client::{GitHubClient, GitHubReleaseResponse};
    use anyhow::anyhow;

    /// Loads and parses the latest_release_success.json test fixture.
    fn load_success_response() -> GitHubReleaseResponse {
        let json = include_str!("../../resources/test/github_api/latest_release_success.json");
        serde_json::from_str(json).expect("Failed to parse latest_release_success.json")
    }

    struct MockGitHubClient {
        response: Result<Option<GitHubReleaseResponse>>,
    }

    impl GitHubClient for MockGitHubClient {
        fn fetch_latest_release(&self) -> Result<Option<GitHubReleaseResponse>> {
            match &self.response {
                Ok(resp) => Ok(resp.clone()),
                Err(e) => Err(anyhow!("{}", e)),
            }
        }
    }

    #[test]
    fn test_fetch_latest_release_success_from_json_fixture() {
        let github_client = MockGitHubClient {
            response: Ok(Some(load_success_response())),
        };

        let repo = RemoteReleaseRepository::new();
        let result = repo.fetch_latest_release(&github_client).unwrap();

        assert!(result.is_some());
        let info = result.unwrap();
        // The fixture contains v0.2.0
        assert_eq!(info.release, FenvRelease::new("0.2.0").unwrap());
        assert_eq!(
            info.html_url,
            "https://github.com/fenv-org/fenv/releases/tag/v0.2.0"
        );
        assert_eq!(info.published_at, "2025-06-14T11:30:37Z");
    }

    #[test]
    fn test_fetch_latest_release_returns_none_when_no_release() {
        let github_client = MockGitHubClient {
            response: Ok(None),
        };

        let repo = RemoteReleaseRepository::new();
        let result = repo.fetch_latest_release(&github_client).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_fetch_latest_release_returns_none_for_invalid_version() {
        // Create a modified response with an invalid tag_name
        let mut response = load_success_response();
        response.tag_name = "not-a-version".to_string();

        let github_client = MockGitHubClient {
            response: Ok(Some(response)),
        };

        let repo = RemoteReleaseRepository::new();
        let result = repo.fetch_latest_release(&github_client).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_fetch_latest_release_propagates_errors() {
        let github_client = MockGitHubClient {
            response: Err(anyhow!("Network error")),
        };

        let repo = RemoteReleaseRepository::new();
        let result = repo.fetch_latest_release(&github_client);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Network error");
    }
}
