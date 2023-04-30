use crate::{
    args::FenvLatestArgs,
    context::FenvContext,
    sdk_service::{
        model::flutter_sdk::FlutterSdk,
        sdk_service::{RealSdkService, SdkService},
    },
    service::service::Service,
};
use std::result::Result::Ok;

pub struct FenvLatestService {
    pub args: FenvLatestArgs,
}

impl FenvLatestService {
    pub fn new(args: FenvLatestArgs) -> Self {
        Self { args }
    }
}

impl Service for FenvLatestService {
    fn execute(
        &self,
        context: &impl FenvContext,
        stdout: &mut impl std::io::Write,
    ) -> anyhow::Result<()> {
        #[allow(deprecated)]
        let from_remote = self.args.from_remote || self.args.known;
        let sdk_service = RealSdkService::new();
        let version_or_channel: anyhow::Result<String> = if from_remote {
            let latest = sdk_service.find_latest_remote(context, &self.args.prefix);
            latest
                .map(|sdk| sdk.display_name())
                .map_err(|e| anyhow::anyhow!(e))
        } else {
            let latest = sdk_service.find_latest_local(context, &self.args.prefix);
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

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use crate::service::macros::test_with_context;

    fn setup_installed_versions<'a>(context: &impl FenvContext) {
        let versions = context.fenv_versions();
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
