use crate::{
    args::FenvLatestArgs,
    context::FenvContext,
    external::git_command::{GitCommand, GitCommandImpl},
    model::{
        flutter_sdk::FlutterSdk, local_flutter_sdk::LocalFlutterSdk,
        remote_flutter_sdk::RemoteFlutterSdk,
    },
    service::{
        list_remote::list_remote_service::FenvListRemoteService, service::Service,
        versions::versions_service::FenvVersionsService,
    },
};
use anyhow::bail;
use lazy_static::lazy_static;
use regex::Regex;
use std::result::Result::Ok;

pub struct FenvLatestService {
    pub args: FenvLatestArgs,
}

impl FenvLatestService {
    pub fn new(args: FenvLatestArgs) -> Self {
        Self { args }
    }

    pub fn latest<'a>(
        context: &impl FenvContext<'a>,
        prefix: &str,
    ) -> anyhow::Result<LocalFlutterSdk> {
        latest(context, prefix)
    }

    pub fn latest_remote<'a>(
        context: &impl FenvContext<'a>,
        prefix: &str,
    ) -> anyhow::Result<RemoteFlutterSdk> {
        latest_remote(context, prefix)
    }
}

impl Service for FenvLatestService {
    fn execute<'a>(
        &self,
        context: &impl FenvContext<'a>,
        stdout: &mut impl std::io::Write,
    ) -> anyhow::Result<()> {
        #[allow(deprecated)]
        let from_remote = self.args.from_remote || self.args.known;

        let version_or_channel: anyhow::Result<String> = if from_remote {
            let latest = latest_remote(context, &self.args.prefix);
            latest
                .map(|sdk| sdk.display_name())
                .map_err(|e| anyhow::anyhow!(e))
        } else {
            let latest = latest(context, &self.args.prefix);
            latest
                .map(|sdk| sdk.display_name())
                .map_err(|e| anyhow::anyhow!(e))
        };

        if version_or_channel.is_err() && self.args.quiet {
            Ok(())
        } else if let Ok(version_or_channel) = version_or_channel {
            writeln!(stdout, "{}", version_or_channel)?;
            Ok(())
        } else {
            version_or_channel.map(|_| ())
        }
    }
}

fn latest<'a>(context: &impl FenvContext<'a>, prefix: &str) -> anyhow::Result<LocalFlutterSdk> {
    let sdks = FenvVersionsService::list_installed_sdks(context)?;
    let filtered_sdks = matches_prefix(&sdks, &prefix);
    match filtered_sdks.last() {
        Some(sdk) => anyhow::Ok(sdk.to_owned()),
        None => bail!("Not found any matched flutter sdk version: `{prefix}`"),
    }
}

fn latest_remote<'a>(
    context: &impl FenvContext<'a>,
    prefix: &str,
) -> anyhow::Result<RemoteFlutterSdk> {
    let git_command: Box<dyn GitCommand> = Box::new(GitCommandImpl::new());
    let sdks = FenvListRemoteService::list_remote_sdks(context, &git_command)?;
    let filtered_sdks = matches_prefix(&sdks, &prefix);
    match filtered_sdks.last() {
        Some(sdk) => anyhow::Ok(sdk.to_owned()),
        None => bail!("Not found any matched flutter sdk version: `{prefix}`"),
    }
}

enum VersionFragments<'a> {
    Version(Vec<&'a str>),
    Channel(&'a str),
}

impl<'a> VersionFragments<'a> {
    fn parse(prefix: &'a str) -> Self {
        lazy_static! {
            static ref VERSION_PATTERN: Regex = Regex::new(r"^v?(\d.*)$").unwrap();
            static ref SPLITTER: Regex = Regex::new(r"((-|\+)hotfix)?\.").unwrap();
        }
        match VERSION_PATTERN.captures(prefix) {
            Some(captures) => {
                let version = captures.get(1).unwrap().as_str();
                let fragments: Vec<&str> = SPLITTER.split(version).collect();
                Self::Version(fragments)
            }
            None => Self::Channel(prefix),
        }
    }

    fn matches(&self, sdk: &impl FlutterSdk) -> bool {
        let version_or_channel = sdk.display_name();
        let sdk_fragments = VersionFragments::parse(&version_or_channel);
        match self {
            VersionFragments::Version(version_me) => match sdk_fragments {
                VersionFragments::Version(version_you) => {
                    if version_me.len() > version_you.len() {
                        false
                    } else {
                        version_you[..version_me.len()] == *version_me
                    }
                }
                VersionFragments::Channel(_) => false,
            },
            VersionFragments::Channel(channel_me) => match sdk_fragments {
                VersionFragments::Version(_) => false,
                VersionFragments::Channel(channel_you) => channel_you.starts_with(*channel_me),
            },
        }
    }
}

fn matches_prefix<T: FlutterSdk>(list: &[T], prefix: &str) -> Vec<T> {
    let fragments = VersionFragments::parse(prefix);
    list.to_vec()
        .into_iter()
        .filter(|sdk| fragments.matches(sdk))
        .collect()
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use crate::{service::macros::test_with_context, util::path_like::PathLike};

    fn setup_installed_versions<'a>(context: &impl FenvContext<'a>) {
        let versions = PathLike::from(&context.fenv_versions()[..]);
        versions.join("v1.0.0").create_dir_all().unwrap();
        versions.join("v1.1.0").create_dir_all().unwrap();
        versions.join("v1.3.14").create_dir_all().unwrap();
        versions.join("v1.4.5").create_dir_all().unwrap();
        versions.join("v1.4.5-hotfix.1").create_dir_all().unwrap();
        versions.join("v1.4.5-hotfix.2").create_dir_all().unwrap();
        versions.join("v1.4.9-hotfix.1").create_dir_all().unwrap();
        versions.join("v1.16.3").create_dir_all().unwrap();
        versions.join("1.17.5").create_dir_all().unwrap();
        versions.join("1.20.0").create_dir_all().unwrap();
        versions.join("1.20.4").create_dir_all().unwrap();
        versions.join("1.22.6").create_dir_all().unwrap();
        versions.join("3.0.0").create_dir_all().unwrap();
        versions.join("3.1.0").create_dir_all().unwrap();
        versions.join("3.1.10").create_dir_all().unwrap();
        versions.join("3.10.0").create_dir_all().unwrap();
        versions.join("3.10.9").create_dir_all().unwrap();
        versions.join("3.10.10").create_dir_all().unwrap();
        versions.join("stable").create_dir_all().unwrap();
        versions.join("master").create_dir_all().unwrap();
    }

    #[test]
    pub fn test_latest_find_v1() {
        test_with_context(|context| {
            setup_installed_versions(context);
            let args = FenvLatestArgs {
                from_remote: false,
                known: false,
                quiet: false,
                prefix: "v1".to_string(),
            };
            let service = FenvLatestService::new(args);

            // execution
            let mut stdout: Vec<u8> = Vec::new();
            service.execute(context, &mut stdout).unwrap();

            // validation
            assert_eq!("1.22.6\n", String::from_utf8(stdout).unwrap())
        });
    }

    #[test]
    pub fn test_latest_find_1() {
        test_with_context(|context| {
            setup_installed_versions(context);
            let args = FenvLatestArgs {
                from_remote: false,
                known: false,
                quiet: false,
                prefix: "1".to_string(),
            };
            let service = FenvLatestService::new(args);

            // execution
            let mut stdout: Vec<u8> = Vec::new();
            service.execute(context, &mut stdout).unwrap();

            // validation
            assert_eq!("1.22.6\n", String::from_utf8(stdout).unwrap())
        });
    }

    #[test]
    pub fn test_latest_find_1_1() {
        test_with_context(|context| {
            setup_installed_versions(context);
            let args = FenvLatestArgs {
                from_remote: false,
                known: false,
                quiet: false,
                prefix: "1.1".to_string(),
            };
            let service = FenvLatestService::new(args);

            // execution
            let mut stdout: Vec<u8> = Vec::new();
            service.execute(context, &mut stdout).unwrap();

            // validation
            assert_eq!("v1.1.0\n", String::from_utf8(stdout).unwrap())
        });
    }

    #[test]
    pub fn test_latest_find_v1_4() {
        test_with_context(|context| {
            setup_installed_versions(context);
            let args = FenvLatestArgs {
                from_remote: false,
                known: false,
                quiet: false,
                prefix: "v1.4".to_string(),
            };
            let service = FenvLatestService::new(args);

            // execution
            let mut stdout: Vec<u8> = Vec::new();

            service.execute(context, &mut stdout).unwrap();

            // validation
            assert_eq!("v1.4.9-hotfix.1\n", String::from_utf8(stdout).unwrap())
        });
    }

    #[test]
    pub fn test_latest_find_1_4() {
        test_with_context(|context| {
            setup_installed_versions(context);
            let args = FenvLatestArgs {
                from_remote: false,
                known: false,
                quiet: false,
                prefix: "1.4".to_string(),
            };
            let service = FenvLatestService::new(args);

            // execution
            let mut stdout: Vec<u8> = Vec::new();
            service.execute(context, &mut stdout).unwrap();

            // validation
            assert_eq!("v1.4.9-hotfix.1\n", String::from_utf8(stdout).unwrap())
        });
    }

    #[test]
    pub fn test_latest_find_1_4_5() {
        test_with_context(|context| {
            setup_installed_versions(context);
            let args = FenvLatestArgs {
                from_remote: false,
                known: false,
                quiet: false,
                prefix: "1.4.5".to_string(),
            };
            let service = FenvLatestService::new(args);

            // execution
            let mut stdout: Vec<u8> = Vec::new();
            service.execute(context, &mut stdout).unwrap();

            // validation
            assert_eq!("v1.4.5-hotfix.2\n", String::from_utf8(stdout).unwrap())
        });
    }

    #[test]
    pub fn test_latest_find_3() {
        test_with_context(|context| {
            setup_installed_versions(context);
            let args = FenvLatestArgs {
                from_remote: false,
                known: false,
                quiet: false,
                prefix: "3".to_string(),
            };
            let service = FenvLatestService::new(args);

            // execution
            let mut stdout: Vec<u8> = Vec::new();
            service.execute(context, &mut stdout).unwrap();

            // validation
            assert_eq!("3.10.10\n", String::from_utf8(stdout).unwrap())
        });
    }

    #[test]
    pub fn test_latest_find_3_1() {
        test_with_context(|context| {
            setup_installed_versions(context);
            let args = FenvLatestArgs {
                from_remote: false,
                known: false,
                quiet: false,
                prefix: "3.1".to_string(),
            };
            let service = FenvLatestService::new(args);

            // execution
            let mut stdout: Vec<u8> = Vec::new();
            service.execute(context, &mut stdout).unwrap();

            // validation
            assert_eq!("3.1.10\n", String::from_utf8(stdout).unwrap())
        });
    }

    #[test]
    pub fn test_latest_find_3_10() {
        test_with_context(|context| {
            setup_installed_versions(context);
            let args = FenvLatestArgs {
                from_remote: false,
                known: false,
                quiet: false,
                prefix: "3.10".to_string(),
            };
            let service = FenvLatestService::new(args);

            // execution
            let mut stdout: Vec<u8> = Vec::new();
            service.execute(context, &mut stdout).unwrap();

            // validation
            assert_eq!("3.10.10\n", String::from_utf8(stdout).unwrap())
        });
    }

    #[test]
    pub fn test_latest_find_3_10_9() {
        test_with_context(|context| {
            setup_installed_versions(context);
            let args = FenvLatestArgs {
                from_remote: false,
                known: false,
                quiet: false,
                prefix: "3.10.9".to_string(),
            };
            let service = FenvLatestService::new(args);

            // execution
            let mut stdout: Vec<u8> = Vec::new();
            service.execute(context, &mut stdout).unwrap();

            // validation
            assert_eq!("3.10.9\n", String::from_utf8(stdout).unwrap())
        });
    }

    #[test]
    pub fn test_latest_find_stable() {
        test_with_context(|context| {
            setup_installed_versions(context);
            let args = FenvLatestArgs {
                from_remote: false,
                known: false,
                quiet: false,
                prefix: "stable".to_string(),
            };
            let service = FenvLatestService::new(args);

            // execution
            let mut stdout: Vec<u8> = Vec::new();
            service.execute(context, &mut stdout).unwrap();

            // validation
            assert_eq!("stable\n", String::from_utf8(stdout).unwrap())
        });
    }

    #[test]
    pub fn test_latest_find_m() {
        test_with_context(|context| {
            setup_installed_versions(context);
            let args = FenvLatestArgs {
                from_remote: false,
                known: false,
                quiet: false,
                prefix: "m".to_string(),
            };
            let service = FenvLatestService::new(args);

            // execution
            let mut stdout: Vec<u8> = Vec::new();
            service.execute(context, &mut stdout).unwrap();

            // validation
            assert_eq!("master\n", String::from_utf8(stdout).unwrap())
        });
    }

    #[test]
    pub fn test_latest_find_unknown_when_quiet_is_disabled() {
        test_with_context(|context| {
            setup_installed_versions(context);
            let args = FenvLatestArgs {
                from_remote: false,
                known: false,
                quiet: false,
                prefix: "unknown".to_string(),
            };
            let service = FenvLatestService::new(args);

            // execution
            let mut stdout: Vec<u8> = Vec::new();
            let error = service.execute(context, &mut stdout).unwrap_err();

            // validation
            assert_eq!(
                "Not found any matched flutter sdk version: `unknown`",
                error.to_string()
            );
        });
    }

    #[test]
    pub fn test_latest_find_unknown_when_quiet_is_enabled() {
        test_with_context(|context| {
            setup_installed_versions(context);
            let args = FenvLatestArgs {
                from_remote: false,
                known: false,
                quiet: true,
                prefix: "1.2.3.4".to_string(),
            };
            let service = FenvLatestService::new(args);

            // execution
            let mut stdout: Vec<u8> = Vec::new();
            service.execute(context, &mut stdout).unwrap();

            // validation
            assert_eq!("", String::from_utf8(stdout).unwrap())
        });
    }

    #[test]
    pub fn test_latest_remote_find_v1() {
        test_with_context(|context| {
            let args = FenvLatestArgs {
                from_remote: true,
                known: false,
                quiet: false,
                prefix: "v1".to_string(),
            };
            let service = FenvLatestService::new(args);

            // execution
            let mut stdout: Vec<u8> = Vec::new();
            service.execute(context, &mut stdout).unwrap();

            // validation
            assert_eq!("1.22.6\n", String::from_utf8(stdout).unwrap())
        });
    }

    #[test]
    pub fn test_latest_remote_find_1() {
        test_with_context(|context| {
            let args = FenvLatestArgs {
                from_remote: true,
                known: false,
                quiet: false,
                prefix: "1".to_string(),
            };
            let service = FenvLatestService::new(args);

            // execution
            let mut stdout: Vec<u8> = Vec::new();
            service.execute(context, &mut stdout).unwrap();

            // validation
            assert_eq!("1.22.6\n", String::from_utf8(stdout).unwrap())
        });
    }

    #[test]
    pub fn test_latest_remote_find_1_1() {
        test_with_context(|context| {
            let args = FenvLatestArgs {
                from_remote: true,
                known: false,
                quiet: false,
                prefix: "1.1".to_string(),
            };
            let service = FenvLatestService::new(args);

            // execution
            let mut stdout: Vec<u8> = Vec::new();
            service.execute(context, &mut stdout).unwrap();

            // validation
            assert_eq!("v1.1.9\n", String::from_utf8(stdout).unwrap())
        });
    }

    #[test]
    pub fn test_latest_remote_find_v1_4() {
        test_with_context(|context| {
            setup_installed_versions(context);
            let args = FenvLatestArgs {
                from_remote: true,
                known: false,
                quiet: false,
                prefix: "v1.4".to_string(),
            };
            let service = FenvLatestService::new(args);

            // execution
            let mut stdout: Vec<u8> = Vec::new();

            service.execute(context, &mut stdout).unwrap();

            // validation
            assert_eq!("v1.4.19\n", String::from_utf8(stdout).unwrap())
        });
    }

    #[test]
    pub fn test_latest_remote_find_1_4() {
        test_with_context(|context| {
            setup_installed_versions(context);
            let args = FenvLatestArgs {
                from_remote: true,
                known: false,
                quiet: false,
                prefix: "1.4".to_string(),
            };
            let service = FenvLatestService::new(args);

            // execution
            let mut stdout: Vec<u8> = Vec::new();
            service.execute(context, &mut stdout).unwrap();

            // validation
            assert_eq!("v1.4.19\n", String::from_utf8(stdout).unwrap())
        });
    }

    #[test]
    pub fn test_latest_remote_find_1_4_5() {
        test_with_context(|context| {
            setup_installed_versions(context);
            let args = FenvLatestArgs {
                from_remote: true,
                known: false,
                quiet: false,
                prefix: "1.4.5".to_string(),
            };
            let service = FenvLatestService::new(args);

            // execution
            let mut stdout: Vec<u8> = Vec::new();
            service.execute(context, &mut stdout).unwrap();

            // validation
            assert_eq!("v1.4.5-hotfix.2\n", String::from_utf8(stdout).unwrap())
        });
    }

    #[test]
    pub fn test_latest_remote_find_stable() {
        test_with_context(|context| {
            let args = FenvLatestArgs {
                from_remote: true,
                known: false,
                quiet: false,
                prefix: "stable".to_string(),
            };
            let service = FenvLatestService::new(args);

            // execution
            let mut stdout: Vec<u8> = Vec::new();
            service.execute(context, &mut stdout).unwrap();

            // validation
            assert_eq!("stable\n", String::from_utf8(stdout).unwrap())
        });
    }

    #[test]
    pub fn test_latest_remote_find_m() {
        test_with_context(|context| {
            let args = FenvLatestArgs {
                from_remote: true,
                known: false,
                quiet: false,
                prefix: "m".to_string(),
            };
            let service = FenvLatestService::new(args);

            // execution
            let mut stdout: Vec<u8> = Vec::new();
            service.execute(context, &mut stdout).unwrap();

            // validation
            assert_eq!("master\n", String::from_utf8(stdout).unwrap())
        });
    }

    #[test]
    pub fn test_latest_remote_find_unknown_when_quiet_is_disabled() {
        test_with_context(|context| {
            let args = FenvLatestArgs {
                from_remote: true,
                known: false,
                quiet: false,
                prefix: "unknown".to_string(),
            };
            let service = FenvLatestService::new(args);

            // execution
            let mut stdout: Vec<u8> = Vec::new();
            let error = service.execute(context, &mut stdout).unwrap_err();

            // validation
            assert_eq!(
                "Not found any matched flutter sdk version: `unknown`",
                error.to_string()
            );
        });
    }

    #[test]
    pub fn test_latest_remote_find_unknown_when_quiet_is_enabled() {
        test_with_context(|context| {
            let args = FenvLatestArgs {
                from_remote: true,
                known: false,
                quiet: true,
                prefix: "1.2.3.4".to_string(),
            };
            let service = FenvLatestService::new(args);

            // execution
            let mut stdout: Vec<u8> = Vec::new();
            service.execute(context, &mut stdout).unwrap();

            // validation
            assert_eq!("", String::from_utf8(stdout).unwrap())
        });
    }
}
