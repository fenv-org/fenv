use super::path_like::PathLike;
use crate::{model::remote_flutter_sdk::RemoteFlutterSdk, util::chrono_wrapper::Clock};
use anyhow::Context;
use chrono::{DateTime, Duration};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct RemoteSdkListCache {
    expires_at: String,
    list: Vec<RemoteFlutterSdk>,
}

pub fn lookup_cached_list(
    cache_file: &PathLike,
    clock: &Box<dyn Clock>,
) -> Option<Vec<RemoteFlutterSdk>> {
    let content = cache_file.read_to_string().ok()?;
    let cache = serde_json::from_str::<RemoteSdkListCache>(&content).ok()?;
    if is_cache_expired(&cache, clock) {
        return None;
    }
    Some(cache.list)
}

/// Cache expiration in seconds.
///
/// For now, 5 minutes.
const CACHE_EXPIRATION: i64 = 5 * 60;

/// Stores the given `list` of the remote flutter SDKs to `cache_file`.
///
/// The cached list will be expired in 5 minutes.
pub fn cache_list(
    cache_file: &PathLike,
    list: &[RemoteFlutterSdk],
    clock: &Box<dyn Clock>,
) -> anyhow::Result<()> {
    if let Some(parent) = &cache_file.parent() {
        if !parent.is_dir() {
            parent
                .create_dir_all()
                .with_context(|| format!("Failed to create cache directory: {parent}"))?;
        }
    }

    let cache = RemoteSdkListCache {
        expires_at: (clock.utc_now() + Duration::seconds(CACHE_EXPIRATION)).to_rfc3339(),
        list: list.to_vec(),
    };
    cache_file
        .write(serde_json::to_string_pretty(&cache)?)
        .with_context(|| format!("Failed to write cache file: {cache_file}"))?;
    anyhow::Ok(())
}

fn is_cache_expired(cache: &RemoteSdkListCache, clock: &Box<dyn Clock>) -> bool {
    let expires_at = match DateTime::parse_from_rfc3339(&cache.expires_at) {
        Ok(expires_at) => expires_at,
        Err(_) => return false,
    };
    expires_at < clock.utc_now()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        context::FenvContext,
        model::{flutter_version::FlutterVersion, remote_flutter_sdk::GitRefsKind},
        service::macros::test_with_context,
    };
    use chrono::Utc;

    struct FakeClock {
        now: chrono::DateTime<chrono::Utc>,
    }

    impl FakeClock {
        fn new() -> Self {
            Self { now: Utc::now() }
        }

        fn from(now: &str) -> Self {
            Self {
                now: chrono::DateTime::parse_from_rfc3339(now).unwrap().into(),
            }
        }
    }

    impl Clock for FakeClock {
        fn utc_now(&self) -> chrono::DateTime<chrono::Utc> {
            self.now
        }
    }

    fn bake_sample() -> Vec<RemoteFlutterSdk> {
        fn remote_flutter_sdk_with_kind(
            kind: GitRefsKind,
            sha: &str,
            short: &str,
            long: &str,
        ) -> RemoteFlutterSdk {
            RemoteFlutterSdk {
                kind,
                sha: sha.to_string(),
                short: short.to_string(),
                long: long.to_string(),
            }
        }

        fn tag_of(major: u8, minor: u8, patch: u8, hotfix: u8) -> GitRefsKind {
            GitRefsKind::Tag(FlutterVersion::new(major, minor, patch, hotfix))
        }

        fn head_of(branch: &str) -> GitRefsKind {
            GitRefsKind::Head(branch.to_string())
        }

        vec![
            remote_flutter_sdk_with_kind(
                tag_of(0, 0, 6, 0),
                "dc4ca8db838a81a27672101b94b4679b3c8c9305",
                "0.0.6",
                "refs/tags/0.0.6",
            ),
            remote_flutter_sdk_with_kind(
                tag_of(0, 0, 10, 0),
                "3b6d84b083956df5652b2fe34c68287579e9b73d",
                "0.0.10",
                "refs/tags/0.0.10",
            ),
            remote_flutter_sdk_with_kind(
                tag_of(0, 0, 24, 0),
                "788f01f90dfa49642188a65ecda708ac0f4071d7",
                "v0.0.24",
                "refs/tags/v0.0.24",
            ),
            remote_flutter_sdk_with_kind(
                tag_of(1, 1, 9, 0),
                "1407091bfb5bb535630d4d596541817737da8412",
                "1.1.9",
                "refs/tags/v1.1.9",
            ),
            remote_flutter_sdk_with_kind(
                tag_of(1, 5, 4, 2),
                "7a4c33425ddd78c54aba07d86f3f9a4a0051769b",
                "v1.5.4-hotfix.2",
                "refs/tags/v1.5.4-hotfix.2",
            ),
            remote_flutter_sdk_with_kind(
                tag_of(1, 12, 13, 9),
                "f139b11009aeb8ed2a3a3aa8b0066e482709dde3",
                "v1.12.13+hotfix.9",
                "refs/tags/v1.12.13+hotfix.9",
            ),
            remote_flutter_sdk_with_kind(
                tag_of(2, 0, 0, 0),
                "60bd88df915880d23877bfc1602e8ddcf4c4dd2a",
                "2.0.0",
                "refs/tags/2.0.0",
            ),
            remote_flutter_sdk_with_kind(
                tag_of(3, 7, 12, 0),
                "4d9e56e694b656610ab87fcf2efbcd226e0ed8cf",
                "3.7.12",
                "refs/tags/3.7.12",
            ),
            remote_flutter_sdk_with_kind(
                head_of("dev"),
                "d6260f127fe3f88c98231243b387b48448479bff",
                "dev",
                "refs/heads/dev",
            ),
            remote_flutter_sdk_with_kind(
                head_of("beta"),
                "d11aff97d2df15a076d285f6ad18da75c0d75ddd",
                "beta",
                "refs/heads/beta",
            ),
            remote_flutter_sdk_with_kind(
                head_of("master"),
                "d2646cf87d6415562e5ac425ec88cc56f3889ff7",
                "master",
                "refs/heads/master",
            ),
            remote_flutter_sdk_with_kind(
                head_of("stable"),
                "4d9e56e694b656610ab87fcf2efbcd226e0ed8cf",
                "stable",
                "refs/heads/stable",
            ),
        ]
    }

    const BAKED_SAMPLE_JSON: &str = r#"{
  "expires_at": "2020-01-01T00:05:00+00:00",
  "list": [
    {
      "kind": {
        "Tag": {
          "major": 0,
          "minor": 0,
          "patch": 6,
          "hotfix": 0
        }
      },
      "sha": "dc4ca8db838a81a27672101b94b4679b3c8c9305",
      "short": "0.0.6",
      "long": "refs/tags/0.0.6"
    },
    {
      "kind": {
        "Tag": {
          "major": 0,
          "minor": 0,
          "patch": 10,
          "hotfix": 0
        }
      },
      "sha": "3b6d84b083956df5652b2fe34c68287579e9b73d",
      "short": "0.0.10",
      "long": "refs/tags/0.0.10"
    },
    {
      "kind": {
        "Tag": {
          "major": 0,
          "minor": 0,
          "patch": 24,
          "hotfix": 0
        }
      },
      "sha": "788f01f90dfa49642188a65ecda708ac0f4071d7",
      "short": "v0.0.24",
      "long": "refs/tags/v0.0.24"
    },
    {
      "kind": {
        "Tag": {
          "major": 1,
          "minor": 1,
          "patch": 9,
          "hotfix": 0
        }
      },
      "sha": "1407091bfb5bb535630d4d596541817737da8412",
      "short": "1.1.9",
      "long": "refs/tags/v1.1.9"
    },
    {
      "kind": {
        "Tag": {
          "major": 1,
          "minor": 5,
          "patch": 4,
          "hotfix": 2
        }
      },
      "sha": "7a4c33425ddd78c54aba07d86f3f9a4a0051769b",
      "short": "v1.5.4-hotfix.2",
      "long": "refs/tags/v1.5.4-hotfix.2"
    },
    {
      "kind": {
        "Tag": {
          "major": 1,
          "minor": 12,
          "patch": 13,
          "hotfix": 9
        }
      },
      "sha": "f139b11009aeb8ed2a3a3aa8b0066e482709dde3",
      "short": "v1.12.13+hotfix.9",
      "long": "refs/tags/v1.12.13+hotfix.9"
    },
    {
      "kind": {
        "Tag": {
          "major": 2,
          "minor": 0,
          "patch": 0,
          "hotfix": 0
        }
      },
      "sha": "60bd88df915880d23877bfc1602e8ddcf4c4dd2a",
      "short": "2.0.0",
      "long": "refs/tags/2.0.0"
    },
    {
      "kind": {
        "Tag": {
          "major": 3,
          "minor": 7,
          "patch": 12,
          "hotfix": 0
        }
      },
      "sha": "4d9e56e694b656610ab87fcf2efbcd226e0ed8cf",
      "short": "3.7.12",
      "long": "refs/tags/3.7.12"
    },
    {
      "kind": {
        "Head": "dev"
      },
      "sha": "d6260f127fe3f88c98231243b387b48448479bff",
      "short": "dev",
      "long": "refs/heads/dev"
    },
    {
      "kind": {
        "Head": "beta"
      },
      "sha": "d11aff97d2df15a076d285f6ad18da75c0d75ddd",
      "short": "beta",
      "long": "refs/heads/beta"
    },
    {
      "kind": {
        "Head": "master"
      },
      "sha": "d2646cf87d6415562e5ac425ec88cc56f3889ff7",
      "short": "master",
      "long": "refs/heads/master"
    },
    {
      "kind": {
        "Head": "stable"
      },
      "sha": "4d9e56e694b656610ab87fcf2efbcd226e0ed8cf",
      "short": "stable",
      "long": "refs/heads/stable"
    }
  ]
}"#;

    #[test]
    fn test_lookup_cached_list_returns_list_when_file_exists_and_not_expired() {
        test_with_context(|context| {
            // setup
            let clock: Box<dyn Clock> = Box::new(FakeClock::from("2020-01-01T00:05:00+00:00"));
            let cache_file = context.fenv_cache().join(".remote_list");
            cache_file.write(BAKED_SAMPLE_JSON).unwrap();

            // execution
            let actual = lookup_cached_list(&cache_file, &clock).unwrap();

            // validation
            assert_eq!(bake_sample(), actual)
        });
    }

    #[test]
    fn test_lookup_cached_list_returns_none_when_file_exists_and_but_expired() {
        test_with_context(|context| {
            // setup
            let clock: Box<dyn Clock> = Box::new(FakeClock::from("2020-01-01T00:05:01+00:00"));
            let cache_file = context.fenv_cache().join(".remote_list");
            cache_file.write(BAKED_SAMPLE_JSON).unwrap();

            // execution && validation
            assert!(lookup_cached_list(&cache_file, &clock).is_none());
        });
    }

    #[test]
    fn test_lookup_cached_list_returns_none_when_no_file_exists() {
        let clock: Box<dyn Clock> = Box::new(FakeClock::new());
        assert!(lookup_cached_list(&PathLike::from("/does/not/exist"), &clock).is_none())
    }

    #[test]
    fn test_lookup_cached_list_returns_none_when_not_valid_json() {
        test_with_context(|context| {
            // setup
            let clock: Box<dyn Clock> = Box::new(FakeClock::new());
            let cache_file = context.fenv_cache().join(".remote_list");
            cache_file.write(r#"{"not_valid": "format"}"#).unwrap();

            // execution & validation
            assert!(lookup_cached_list(&cache_file, &clock).is_none())
        });
    }

    #[test]
    fn test_cache_list() {
        test_with_context(|context| {
            // setup
            let clock: Box<dyn Clock> = Box::new(FakeClock::from("2020-01-01T00:00:00+00:00"));
            let list = bake_sample();
            let cache_file = context.fenv_cache().join(".remote_list");

            // execution
            cache_list(&cache_file, &list, &clock).unwrap();

            // validation
            assert_eq!(BAKED_SAMPLE_JSON, cache_file.read_to_string().unwrap(),)
        });
    }

    #[test]
    fn test_cache_list_when_parent_not_exists() {
        test_with_context(|context| {
            // setup
            let clock: Box<dyn Clock> = Box::new(FakeClock::from("2020-01-01T00:00:00+00:00"));
            let list = bake_sample();
            let cache_file = context.home().join("parent/.remote_list");

            // execution
            cache_list(&cache_file, &list, &clock).unwrap();

            // validation
            assert_eq!(BAKED_SAMPLE_JSON, cache_file.read_to_string().unwrap(),)
        });
    }

    #[test]
    fn test_cache_list_fails_when_cannot_create_parent_directory() {
        test_with_context(|context| {
            // setup
            let clock: Box<dyn Clock> = Box::new(FakeClock::new());
            let cache_file = context.home().join("not_directory/.remote_list");
            let parent_file = context.home().join("not_directory");
            // intentionally create a file.
            parent_file.create_file().unwrap();

            // execution
            let actual = cache_list(&cache_file, &vec![], &clock);

            // validation
            assert!(actual.is_err());
        });
    }
}
