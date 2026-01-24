use crate::context::FenvContext;
use crate::util::chrono_wrapper::Clock;
use chrono::{DateTime, Duration, Utc};

const NOTIFICATION_CACHE_FILE: &str = ".fenv_last_notification";
const NOTIFICATION_INTERVAL_HOURS: i64 = 24;

pub fn should_show_notification(context: &impl FenvContext, clock: &impl Clock) -> bool {
    let cache_file = context.fenv_cache().join(NOTIFICATION_CACHE_FILE);

    if let Ok(content) = cache_file.read_to_string() {
        if let Ok(last_shown) = DateTime::parse_from_rfc3339(content.trim()) {
            let last_shown_utc: DateTime<Utc> = last_shown.into();
            if clock.utc_now() - last_shown_utc < Duration::hours(NOTIFICATION_INTERVAL_HOURS) {
                return false;
            }
        }
    }
    true
}

pub fn record_notification_shown(context: &impl FenvContext, clock: &impl Clock) {
    let cache_file = context.fenv_cache().join(NOTIFICATION_CACHE_FILE);
    let _ = cache_file.write(clock.utc_now().to_rfc3339().as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::RealFenvContext;
    use chrono::Utc;

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

        RealFenvContext::new(
            &temp_dir.path().join(".fenv").to_str().unwrap(),
            temp_dir.path().to_str().unwrap(),
            temp_dir.path().to_str().unwrap(),
            "/bin/bash",
            &temp_dir.path().join(".pub-cache").to_str().unwrap(),
            crate::context::OperatingSystem::Linux,
            crate::context::Architecture::X86_64,
            "1.0.0",
            true, // no_update_check: disable update checks during tests
        )
    }

    #[test]
    fn test_should_show_notification_when_no_cache_exists() {
        let context = create_test_context();
        let clock = MockClock { now: Utc::now() };

        assert!(should_show_notification(&context, &clock));
    }

    #[test]
    fn test_should_show_notification_when_cache_expired() {
        let context = create_test_context();
        let now = Utc::now();
        let old_clock = MockClock {
            now: now - Duration::hours(25),
        };

        // Record notification with old time
        record_notification_shown(&context, &old_clock);

        // Check with current time
        let current_clock = MockClock { now };
        assert!(should_show_notification(&context, &current_clock));
    }

    #[test]
    fn test_should_not_show_notification_when_cache_valid() {
        let context = create_test_context();
        let now = Utc::now();
        let recent_clock = MockClock {
            now: now - Duration::hours(1),
        };

        // Record notification with recent time
        record_notification_shown(&context, &recent_clock);

        // Check with current time
        let current_clock = MockClock { now };
        assert!(!should_show_notification(&context, &current_clock));
    }

    #[test]
    fn test_record_notification_shown() {
        let context = create_test_context();
        let clock = MockClock { now: Utc::now() };

        record_notification_shown(&context, &clock);

        let cache_file = context.fenv_cache().join(NOTIFICATION_CACHE_FILE);
        assert!(cache_file.exists());

        let content = cache_file.read_to_string().unwrap();
        let parsed = DateTime::parse_from_rfc3339(content.trim()).unwrap();
        let parsed_utc: DateTime<Utc> = parsed.into();

        // Should be very close to the clock time (within a second)
        assert!(
            (parsed_utc - clock.utc_now()).num_seconds().abs() < 1,
            "Expected parsed time to be close to clock time"
        );
    }
}
