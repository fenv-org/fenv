use crate::context::FenvContext;
use crate::external::github_client::GitHubClient;
use crate::release_service::local_repository::LocalReleaseRepository;
use crate::release_service::model::fenv_release::FenvRelease;
use crate::release_service::remote_repository::RemoteReleaseRepository;
use crate::util::chrono_wrapper::Clock;
use anyhow::Result;

/// Information about available updates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateInfo {
    pub current_version: FenvRelease,
    pub latest_version: FenvRelease,
    pub html_url: String,
    pub is_update_available: bool,
}

/// Service for managing fenv release information and update checks.
pub struct ReleaseService<G: GitHubClient, C: Clock> {
    github_client: G,
    clock: C,
    local_repo: LocalReleaseRepository,
    remote_repo: RemoteReleaseRepository,
}

impl<G: GitHubClient, C: Clock> ReleaseService<G, C> {
    /// Creates a new ReleaseService.
    ///
    /// # Arguments
    /// * `github_client` - GitHub client for fetching release information
    /// * `clock` - Clock implementation for time-based operations
    pub fn new(github_client: G, clock: C) -> Self {
        Self {
            github_client,
            clock,
            local_repo: LocalReleaseRepository::new(),
            remote_repo: RemoteReleaseRepository::new(),
        }
    }

    /// Checks for updates using cached data when available.
    ///
    /// This method will use cached release information if it's available and valid.
    /// If the cache is missing or expired, it will fetch fresh data from GitHub.
    /// Network errors are handled gracefully by returning None.
    ///
    /// # Arguments
    /// * `context` - The fenv context
    ///
    /// # Returns
    /// * `Ok(Some(UpdateInfo))` if update check succeeded
    /// * `Ok(None)` if no release information is available or network error occurred
    /// * `Err` if current version parsing failed or other unexpected errors
    pub fn check_for_updates(&self, context: &impl FenvContext) -> Result<Option<UpdateInfo>> {
        log::debug!(
            "Checking for updates: current_version = {}",
            context.current_version()
        );

        // Parse current version
        let current_version = FenvRelease::from_version_or_tag(&context.current_version());
        let Some(current) = current_version else {
            // Current version is invalid - this shouldn't happen in production
            log::debug!("Invalid current version format, skipping update check");
            return Ok(None);
        };

        // Try to load from cache first
        if let Some(cached) = self.local_repo.load_cached_release(context, &self.clock) {
            log::debug!(
                "Cache hit: latest_version = {}, is_update_available = {}",
                cached.release.version,
                cached.release.is_newer_than(&current)
            );
            return Ok(Some(UpdateInfo {
                current_version: current.clone(),
                latest_version: cached.release.clone(),
                html_url: cached.html_url,
                is_update_available: cached.release.is_newer_than(&current),
            }));
        }

        log::debug!("Cache miss or expired, fetching from remote");
        // Cache miss or expired - fetch from remote
        self.fetch_and_cache_update_info(context, &current)
    }

    /// Forces a fresh check for updates, bypassing the cache.
    ///
    /// This method always fetches the latest release information from GitHub,
    /// ignoring any cached data. The fetched data will be cached for future use.
    /// Network errors are handled gracefully by returning None.
    ///
    /// # Arguments
    /// * `context` - The fenv context
    ///
    /// # Returns
    /// * `Ok(Some(UpdateInfo))` if update check succeeded
    /// * `Ok(None)` if no release information is available or network error occurred
    /// * `Err` if current version parsing failed or other unexpected errors
    pub fn force_check_for_updates(
        &self,
        context: &impl FenvContext,
    ) -> Result<Option<UpdateInfo>> {
        // Parse current version
        let current_version = FenvRelease::from_version_or_tag(&context.current_version());
        let Some(current) = current_version else {
            // Current version is invalid - this shouldn't happen in production
            return Ok(None);
        };

        self.fetch_and_cache_update_info(context, &current)
    }

    /// Fetches release information from GitHub and caches it.
    ///
    /// # Arguments
    /// * `context` - The fenv context
    /// * `current` - The current fenv version
    ///
    /// # Returns
    /// * `Ok(Some(UpdateInfo))` if fetch succeeded
    /// * `Ok(None)` if no release found or network error occurred
    fn fetch_and_cache_update_info(
        &self,
        context: &impl FenvContext,
        current: &FenvRelease,
    ) -> Result<Option<UpdateInfo>> {
        // Fetch from remote, handling errors gracefully
        let remote_result = self.remote_repo.fetch_latest_release(&self.github_client);

        let release_info = match remote_result {
            Ok(Some(info)) => {
                log::debug!(
                    "Fetched latest release from GitHub: version = {}",
                    info.release.version
                );
                info
            }
            Ok(None) => {
                log::debug!("No release found from GitHub");
                return Ok(None);
            }
            Err(e) => {
                // Network error or other issue - return None gracefully
                log::debug!("Failed to fetch release from GitHub: {}", e);
                return Ok(None);
            }
        };

        // Cache the result
        let _ = self.local_repo.store_cached_release(
            context,
            &self.clock,
            &release_info.release,
            &release_info.html_url,
            &release_info.published_at,
        );

        let is_update_available = release_info.release.is_newer_than(current);
        log::debug!(
            "Update check complete: current = {}, latest = {}, is_update_available = {}",
            current.version,
            release_info.release.version,
            is_update_available
        );

        Ok(Some(UpdateInfo {
            current_version: current.clone(),
            latest_version: release_info.release.clone(),
            html_url: release_info.html_url,
            is_update_available,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::RealFenvContext;
    use crate::external::github_client::{GitHubClient, GitHubReleaseResponse};
    use crate::util::chrono_wrapper::Clock;
    use anyhow::anyhow;
    use chrono::{DateTime, Utc};

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

    struct MockClock {
        now: DateTime<Utc>,
    }

    impl Clock for MockClock {
        fn utc_now(&self) -> DateTime<Utc> {
            self.now
        }
    }

    fn create_test_context(version: &str) -> RealFenvContext {
        let temp_dir = tempfile::tempdir().unwrap();

        RealFenvContext::new(
            &temp_dir.path().join(".fenv").to_str().unwrap(),
            temp_dir.path().to_str().unwrap(),
            temp_dir.path().to_str().unwrap(),
            "/bin/bash",
            &temp_dir.path().join(".pub-cache").to_str().unwrap(),
            crate::context::OperatingSystem::Linux,
            crate::context::Architecture::X86_64,
            version,
            true, // no_update_check: disable update checks during tests
        )
    }

    #[test]
    fn test_check_for_updates_no_update_available() {
        let github_client = MockGitHubClient {
            response: Ok(Some(GitHubReleaseResponse {
                tag_name: "v1.0.0".to_string(),
                name: "Release 1.0.0".to_string(),
                html_url: "https://github.com/fenv-org/fenv/releases/tag/v1.0.0".to_string(),
                published_at: "2024-01-01T00:00:00Z".to_string(),
            })),
        };
        let clock = MockClock {
            now: Utc::now(),
        };
        let service = ReleaseService::new(github_client, clock);
        let context = create_test_context("1.0.0");

        let result = service.check_for_updates(&context).unwrap();

        assert!(result.is_some());
        let update_info = result.unwrap();
        assert_eq!(update_info.current_version.version, "1.0.0");
        assert_eq!(update_info.latest_version.version, "1.0.0");
        assert!(!update_info.is_update_available);
    }

    #[test]
    fn test_check_for_updates_update_available() {
        let github_client = MockGitHubClient {
            response: Ok(Some(GitHubReleaseResponse {
                tag_name: "v2.0.0".to_string(),
                name: "Release 2.0.0".to_string(),
                html_url: "https://github.com/fenv-org/fenv/releases/tag/v2.0.0".to_string(),
                published_at: "2024-02-01T00:00:00Z".to_string(),
            })),
        };
        let clock = MockClock {
            now: Utc::now(),
        };
        let service = ReleaseService::new(github_client, clock);
        let context = create_test_context("1.0.0");

        let result = service.check_for_updates(&context).unwrap();

        assert!(result.is_some());
        let update_info = result.unwrap();
        assert_eq!(update_info.current_version.version, "1.0.0");
        assert_eq!(update_info.latest_version.version, "2.0.0");
        assert!(update_info.is_update_available);
        assert_eq!(
            update_info.html_url,
            "https://github.com/fenv-org/fenv/releases/tag/v2.0.0"
        );
    }

    #[test]
    fn test_check_for_updates_uses_cache() {
        let github_client = MockGitHubClient {
            response: Ok(Some(GitHubReleaseResponse {
                tag_name: "v2.0.0".to_string(),
                name: "Release 2.0.0".to_string(),
                html_url: "https://github.com/fenv-org/fenv/releases/tag/v2.0.0".to_string(),
                published_at: "2024-02-01T00:00:00Z".to_string(),
            })),
        };
        let clock = MockClock {
            now: Utc::now(),
        };
        let service = ReleaseService::new(github_client, clock);
        let context = create_test_context("1.0.0");

        // First call should fetch from remote and cache
        let result1 = service.check_for_updates(&context).unwrap();
        assert!(result1.is_some());

        // Create a new service with a client that would fail
        let failing_client = MockGitHubClient {
            response: Err(anyhow!("Should not be called")),
        };
        let clock2 = MockClock {
            now: Utc::now(),
        };
        let service2 = ReleaseService::new(failing_client, clock2);

        // Second call should use cache and not call the failing client
        let result2 = service2.check_for_updates(&context).unwrap();
        assert!(result2.is_some());
        let update_info = result2.unwrap();
        assert_eq!(update_info.latest_version.version, "2.0.0");
    }

    #[test]
    fn test_force_check_for_updates_bypasses_cache() {
        let github_client = MockGitHubClient {
            response: Ok(Some(GitHubReleaseResponse {
                tag_name: "v2.0.0".to_string(),
                name: "Release 2.0.0".to_string(),
                html_url: "https://github.com/fenv-org/fenv/releases/tag/v2.0.0".to_string(),
                published_at: "2024-02-01T00:00:00Z".to_string(),
            })),
        };
        let clock = MockClock {
            now: Utc::now(),
        };
        let service = ReleaseService::new(github_client, clock);
        let context = create_test_context("1.0.0");

        // Cache an old version
        let local_repo = LocalReleaseRepository::new();
        let old_release = FenvRelease::new("1.5.0").unwrap();
        let clock_for_cache = MockClock {
            now: Utc::now(),
        };
        local_repo
            .store_cached_release(
                &context,
                &clock_for_cache,
                &old_release,
                "https://old.url",
                "2024-01-01T00:00:00Z",
            )
            .unwrap();

        // Force check should bypass cache and get v2.0.0
        let result = service.force_check_for_updates(&context).unwrap();
        assert!(result.is_some());
        let update_info = result.unwrap();
        assert_eq!(update_info.latest_version.version, "2.0.0");
    }

    #[test]
    fn test_check_for_updates_handles_network_error_gracefully() {
        let github_client = MockGitHubClient {
            response: Err(anyhow!("Network error")),
        };
        let clock = MockClock {
            now: Utc::now(),
        };
        let service = ReleaseService::new(github_client, clock);
        let context = create_test_context("1.0.0");

        let result = service.check_for_updates(&context).unwrap();

        // Should return None instead of propagating the error
        assert!(result.is_none());
    }

    #[test]
    fn test_check_for_updates_handles_no_release_found() {
        let github_client = MockGitHubClient {
            response: Ok(None),
        };
        let clock = MockClock {
            now: Utc::now(),
        };
        let service = ReleaseService::new(github_client, clock);
        let context = create_test_context("1.0.0");

        let result = service.check_for_updates(&context).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_check_for_updates_handles_invalid_current_version() {
        let github_client = MockGitHubClient {
            response: Ok(Some(GitHubReleaseResponse {
                tag_name: "v2.0.0".to_string(),
                name: "Release 2.0.0".to_string(),
                html_url: "https://github.com/fenv-org/fenv/releases/tag/v2.0.0".to_string(),
                published_at: "2024-02-01T00:00:00Z".to_string(),
            })),
        };
        let clock = MockClock {
            now: Utc::now(),
        };
        let service = ReleaseService::new(github_client, clock);
        let context = create_test_context("not-a-version");

        let result = service.check_for_updates(&context).unwrap();

        // Should return None for invalid current version
        assert!(result.is_none());
    }
}
