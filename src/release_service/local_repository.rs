use crate::context::FenvContext;
use crate::release_service::model::fenv_release::FenvRelease;
use crate::util::chrono_wrapper::Clock;
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Represents cached release information with timestamp.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedReleaseInfo {
    pub release: FenvRelease,
    pub html_url: String,
    pub published_at: String,
    pub cached_at: DateTime<Utc>,
}

/// Repository for managing locally cached release information.
pub struct LocalReleaseRepository;

impl Default for LocalReleaseRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalReleaseRepository {
    /// Creates a new LocalReleaseRepository.
    pub fn new() -> Self {
        Self
    }

    /// Loads cached release information if it exists and is still valid.
    ///
    /// # Arguments
    /// * `context` - The fenv context containing cache directory path
    /// * `clock` - Clock implementation for getting current time
    ///
    /// # Returns
    /// * `Some(CachedReleaseInfo)` if cache exists and is valid (not expired)
    /// * `None` if cache doesn't exist, is invalid, or has expired
    pub fn load_cached_release(
        &self,
        context: &impl FenvContext,
        clock: &impl Clock,
    ) -> Option<CachedReleaseInfo> {
        let cache_file = self.get_cache_file_path(context);

        if !cache_file.exists() {
            return None;
        }

        let contents = cache_file.read_to_string().ok()?;
        let cached: CachedReleaseInfo = serde_json::from_str(&contents).ok()?;

        // Check if cache is expired (older than 7 days)
        let now = clock.utc_now();
        let cache_age = now.signed_duration_since(cached.cached_at);
        let cache_expiration = Duration::days(7);

        if cache_age > cache_expiration {
            return None;
        }

        Some(cached)
    }

    /// Stores release information to the local cache.
    ///
    /// # Arguments
    /// * `context` - The fenv context containing cache directory path
    /// * `clock` - Clock implementation for getting current time
    /// * `release` - The release version to cache
    /// * `html_url` - The GitHub release URL
    /// * `published_at` - The release publication timestamp
    ///
    /// # Returns
    /// * `Ok(())` if caching succeeded
    /// * `Err` if writing to cache failed
    pub fn store_cached_release(
        &self,
        context: &impl FenvContext,
        clock: &impl Clock,
        release: &FenvRelease,
        html_url: &str,
        published_at: &str,
    ) -> Result<()> {
        let cache_file = self.get_cache_file_path(context);

        let cached_info = CachedReleaseInfo {
            release: release.clone(),
            html_url: html_url.to_string(),
            published_at: published_at.to_string(),
            cached_at: clock.utc_now(),
        };

        let json = serde_json::to_string_pretty(&cached_info)
            .context("Failed to serialize release info")?;

        cache_file
            .write(json.as_bytes())
            .context("Failed to write cache file")?;

        Ok(())
    }

    /// Clears the cached release information.
    ///
    /// # Arguments
    /// * `context` - The fenv context containing cache directory path
    ///
    /// # Returns
    /// * `Ok(())` if cache was cleared successfully or didn't exist
    /// * `Err` if removal failed
    pub fn clear_cache(&self, context: &impl FenvContext) -> Result<()> {
        let cache_file = self.get_cache_file_path(context);
        cache_file
            .remove_file()
            .context("Failed to remove cache file")?;
        Ok(())
    }

    /// Gets the path to the cache file.
    fn get_cache_file_path(&self, context: &impl FenvContext) -> crate::util::path_like::PathLike {
        context.fenv_cache().join(".fenv_latest_release")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::RealFenvContext;
    use crate::util::chrono_wrapper::Clock;
    use chrono::{DateTime, Utc};
    use std::collections::HashMap;

    struct MockClock {
        now: DateTime<Utc>,
    }

    impl Clock for MockClock {
        fn utc_now(&self) -> DateTime<Utc> {
            self.now
        }
    }

    fn create_test_context() -> RealFenvContext {
        let temp_dir = tempfile::tempdir().unwrap();
        let env_map: HashMap<String, String> = [
            ("HOME", temp_dir.path().to_str().unwrap()),
            ("SHELL", "/bin/bash"),
            ("PWD", temp_dir.path().to_str().unwrap()),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

        RealFenvContext::from(&env_map, "linux", "x86_64").unwrap()
    }

    #[test]
    fn test_load_cached_release_returns_none_if_cache_does_not_exist() {
        let context = create_test_context();
        let clock = MockClock {
            now: Utc::now(),
        };
        let repo = LocalReleaseRepository::new();

        let result = repo.load_cached_release(&context, &clock);
        assert!(result.is_none());
    }

    #[test]
    fn test_store_and_load_cached_release() {
        let context = create_test_context();
        let now = Utc::now();
        let clock = MockClock { now };
        let repo = LocalReleaseRepository::new();

        let release = FenvRelease::new("1.2.3").unwrap();
        let html_url = "https://github.com/fenv-org/fenv/releases/tag/v1.2.3";
        let published_at = "2024-01-01T00:00:00Z";

        // Store the cache
        repo.store_cached_release(&context, &clock, &release, html_url, published_at)
            .unwrap();

        // Load the cache
        let cached = repo.load_cached_release(&context, &clock).unwrap();

        assert_eq!(cached.release, release);
        assert_eq!(cached.html_url, html_url);
        assert_eq!(cached.published_at, published_at);
        assert_eq!(cached.cached_at, now);
    }

    #[test]
    fn test_load_cached_release_returns_none_if_expired() {
        let context = create_test_context();
        let now = Utc::now();
        let old_clock = MockClock {
            now: now - Duration::days(8), // 8 days ago
        };
        let repo = LocalReleaseRepository::new();

        let release = FenvRelease::new("1.2.3").unwrap();
        let html_url = "https://github.com/fenv-org/fenv/releases/tag/v1.2.3";
        let published_at = "2024-01-01T00:00:00Z";

        // Store with old timestamp
        repo.store_cached_release(&context, &old_clock, &release, html_url, published_at)
            .unwrap();

        // Try to load with current time (8 days later)
        let new_clock = MockClock { now };
        let cached = repo.load_cached_release(&context, &new_clock);

        assert!(cached.is_none());
    }

    #[test]
    fn test_clear_cache() {
        let context = create_test_context();
        let clock = MockClock {
            now: Utc::now(),
        };
        let repo = LocalReleaseRepository::new();

        let release = FenvRelease::new("1.2.3").unwrap();
        let html_url = "https://github.com/fenv-org/fenv/releases/tag/v1.2.3";
        let published_at = "2024-01-01T00:00:00Z";

        // Store the cache
        repo.store_cached_release(&context, &clock, &release, html_url, published_at)
            .unwrap();

        // Verify it exists
        assert!(repo.load_cached_release(&context, &clock).is_some());

        // Clear the cache
        repo.clear_cache(&context).unwrap();

        // Verify it's gone
        assert!(repo.load_cached_release(&context, &clock).is_none());
    }

    #[test]
    fn test_clear_cache_when_no_cache_exists() {
        let context = create_test_context();
        let repo = LocalReleaseRepository::new();

        // Should not fail even if cache doesn't exist
        let result = repo.clear_cache(&context);
        assert!(result.is_ok());
    }
}
