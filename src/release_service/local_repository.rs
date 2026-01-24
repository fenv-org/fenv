use crate::context::FenvContext;
use crate::release_service::model::fenv_release::FenvRelease;
use crate::util::chrono_wrapper::Clock;
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Represents cached release information with expiration timestamps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedReleaseInfo {
    pub release: FenvRelease,
    pub html_url: String,
    pub published_at: String,
    /// Cache is invalid after this time (needs refresh from remote).
    pub cache_expires_at: DateTime<Utc>,
    /// Notification can be shown after this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_available_at: Option<DateTime<Utc>>,
}

const CACHE_EXPIRATION_DAYS: i64 = 7;
const NOTIFICATION_INTERVAL_HOURS: i64 = 24;

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

        // Check if cache has expired
        let now = clock.utc_now();
        if now > cached.cache_expires_at {
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

        // Preserve notification_available_at from existing cache if present
        let notification_available_at = self
            .load_cache_raw(context)
            .and_then(|c| c.notification_available_at);

        let cached_info = CachedReleaseInfo {
            release: release.clone(),
            html_url: html_url.to_string(),
            published_at: published_at.to_string(),
            cache_expires_at: clock.utc_now() + Duration::days(CACHE_EXPIRATION_DAYS),
            notification_available_at,
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

    /// Checks if an update notification should be shown.
    ///
    /// Returns true if:
    /// - No cache exists (first run)
    /// - No notification has been shown before
    /// - Current time is past `notification_available_at`
    pub fn should_show_notification(&self, context: &impl FenvContext, clock: &impl Clock) -> bool {
        let Some(cached) = self.load_cache_raw(context) else {
            return true;
        };

        let Some(notification_available_at) = cached.notification_available_at else {
            return true;
        };

        clock.utc_now() >= notification_available_at
    }

    /// Records that a notification was shown.
    ///
    /// Updates the cache file with `notification_available_at` set to now + 24 hours.
    /// If the cache file doesn't exist, this is a no-op (notification shown but no release cached).
    pub fn record_notification_shown(&self, context: &impl FenvContext, clock: &impl Clock) {
        let Some(mut cached) = self.load_cache_raw(context) else {
            return;
        };

        cached.notification_available_at =
            Some(clock.utc_now() + Duration::hours(NOTIFICATION_INTERVAL_HOURS));

        let cache_file = self.get_cache_file_path(context);
        if let Ok(json) = serde_json::to_string_pretty(&cached) {
            let _ = cache_file.write(json.as_bytes());
        }
    }

    /// Gets the path to the cache file.
    fn get_cache_file_path(&self, context: &impl FenvContext) -> crate::util::path_like::PathLike {
        context.fenv_cache().join(".fenv_latest_release")
    }

    /// Loads the cache file without checking expiration.
    /// Used internally to preserve data when updating the cache.
    fn load_cache_raw(&self, context: &impl FenvContext) -> Option<CachedReleaseInfo> {
        let cache_file = self.get_cache_file_path(context);
        if !cache_file.exists() {
            return None;
        }
        let contents = cache_file.read_to_string().ok()?;
        serde_json::from_str(&contents).ok()
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
        assert_eq!(cached.cache_expires_at, now + Duration::days(7));
        assert_eq!(cached.notification_available_at, None);
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

    #[test]
    fn test_should_show_notification_when_no_cache_exists() {
        let context = create_test_context();
        let clock = MockClock { now: Utc::now() };
        let repo = LocalReleaseRepository::new();

        assert!(repo.should_show_notification(&context, &clock));
    }

    #[test]
    fn test_should_show_notification_when_no_notification_recorded() {
        let context = create_test_context();
        let now = Utc::now();
        let clock = MockClock { now };
        let repo = LocalReleaseRepository::new();

        let release = FenvRelease::new("1.2.3").unwrap();
        repo.store_cached_release(
            &context,
            &clock,
            &release,
            "https://example.com",
            "2024-01-01T00:00:00Z",
        )
        .unwrap();

        // Cache exists but no notification recorded yet
        assert!(repo.should_show_notification(&context, &clock));
    }

    #[test]
    fn test_should_show_notification_when_notification_expired() {
        let context = create_test_context();
        let now = Utc::now();
        let repo = LocalReleaseRepository::new();

        let release = FenvRelease::new("1.2.3").unwrap();
        let old_clock = MockClock {
            now: now - Duration::hours(25),
        };

        // Store cache
        repo.store_cached_release(
            &context,
            &old_clock,
            &release,
            "https://example.com",
            "2024-01-01T00:00:00Z",
        )
        .unwrap();

        // Record notification with old time
        repo.record_notification_shown(&context, &old_clock);

        // Check with current time (25 hours later)
        let current_clock = MockClock { now };
        assert!(repo.should_show_notification(&context, &current_clock));
    }

    #[test]
    fn test_should_not_show_notification_when_not_yet_available() {
        let context = create_test_context();
        let now = Utc::now();
        let repo = LocalReleaseRepository::new();

        let release = FenvRelease::new("1.2.3").unwrap();
        let clock = MockClock { now };

        // Store cache and record notification
        repo.store_cached_release(
            &context,
            &clock,
            &release,
            "https://example.com",
            "2024-01-01T00:00:00Z",
        )
        .unwrap();
        repo.record_notification_shown(&context, &clock);

        // Check with current time (notification_available_at is now + 24 hours)
        assert!(!repo.should_show_notification(&context, &clock));

        // Check 1 hour later - still not available
        let later_clock = MockClock {
            now: now + Duration::hours(1),
        };
        assert!(!repo.should_show_notification(&context, &later_clock));
    }

    #[test]
    fn test_record_notification_shown() {
        let context = create_test_context();
        let now = Utc::now();
        let clock = MockClock { now };
        let repo = LocalReleaseRepository::new();

        let release = FenvRelease::new("1.2.3").unwrap();
        repo.store_cached_release(
            &context,
            &clock,
            &release,
            "https://example.com",
            "2024-01-01T00:00:00Z",
        )
        .unwrap();

        // Initially no notification recorded
        let cached = repo.load_cached_release(&context, &clock).unwrap();
        assert!(cached.notification_available_at.is_none());

        // Record notification
        repo.record_notification_shown(&context, &clock);

        // Verify notification_available_at is set to now + 24 hours
        let cached = repo.load_cached_release(&context, &clock).unwrap();
        assert_eq!(
            cached.notification_available_at,
            Some(now + Duration::hours(24))
        );
    }

    #[test]
    fn test_record_notification_preserves_release_cache() {
        let context = create_test_context();
        let now = Utc::now();
        let clock = MockClock { now };
        let repo = LocalReleaseRepository::new();

        let release = FenvRelease::new("1.2.3").unwrap();
        let html_url = "https://github.com/fenv-org/fenv/releases/tag/v1.2.3";
        let published_at = "2024-01-01T00:00:00Z";

        repo.store_cached_release(&context, &clock, &release, html_url, published_at)
            .unwrap();

        // Record notification
        repo.record_notification_shown(&context, &clock);

        // Verify all release data is preserved
        let cached = repo.load_cached_release(&context, &clock).unwrap();
        assert_eq!(cached.release, release);
        assert_eq!(cached.html_url, html_url);
        assert_eq!(cached.published_at, published_at);
        assert_eq!(cached.cache_expires_at, now + Duration::days(7));
        assert_eq!(
            cached.notification_available_at,
            Some(now + Duration::hours(24))
        );
    }

    #[test]
    fn test_store_release_preserves_notification_available_at() {
        let context = create_test_context();
        let now = Utc::now();
        let clock = MockClock { now };
        let repo = LocalReleaseRepository::new();

        let release1 = FenvRelease::new("1.2.3").unwrap();
        repo.store_cached_release(
            &context,
            &clock,
            &release1,
            "https://example.com/1",
            "2024-01-01T00:00:00Z",
        )
        .unwrap();

        // Record notification (sets notification_available_at to now + 24h)
        repo.record_notification_shown(&context, &clock);

        // Store new release info
        let later_clock = MockClock {
            now: now + Duration::hours(1),
        };
        let release2 = FenvRelease::new("1.3.0").unwrap();
        repo.store_cached_release(
            &context,
            &later_clock,
            &release2,
            "https://example.com/2",
            "2024-02-01T00:00:00Z",
        )
        .unwrap();

        // Verify notification_available_at is preserved
        let cached = repo.load_cached_release(&context, &later_clock).unwrap();
        assert_eq!(cached.release, release2);
        assert_eq!(cached.html_url, "https://example.com/2");
        // Original notification_available_at (now + 24h) is preserved
        assert_eq!(
            cached.notification_available_at,
            Some(now + Duration::hours(24))
        );
    }

    #[test]
    fn test_record_notification_noop_when_no_cache() {
        let context = create_test_context();
        let clock = MockClock { now: Utc::now() };
        let repo = LocalReleaseRepository::new();

        // Should not fail when no cache exists
        repo.record_notification_shown(&context, &clock);

        // Cache should still not exist
        assert!(repo.load_cached_release(&context, &clock).is_none());
    }
}
