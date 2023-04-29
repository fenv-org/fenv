use crate::{
    args::FenvGlobalArgs,
    context::FenvContext,
    sdk_service::model::{flutter_sdk::FlutterSdk, local_flutter_sdk::LocalFlutterSdk},
    service::{
        latest::latest_service::FenvLatestService, service::Service,
        versions::versions_service::FenvVersionsService,
    },
};
use anyhow::{bail, Context, Ok};
use std::io::Write;

pub struct FenvGlobalService {
    args: FenvGlobalArgs,
}

impl FenvGlobalService {
    pub fn new(args: FenvGlobalArgs) -> Self {
        Self { args }
    }
}

impl Service for FenvGlobalService {
    fn execute(
        &self,
        context: &impl FenvContext,
        stdout: &mut impl std::io::Write,
    ) -> anyhow::Result<()> {
        match &self.args.version_prefix {
            Some(version_prefix) => set_global_version(context, version_prefix),
            None => show_global_version(context, stdout),
        }
    }
}

fn set_global_version<'a>(context: &impl FenvContext, version_prefix: &str) -> anyhow::Result<()> {
    let local_sdk = match FenvLatestService::latest(context, version_prefix) {
        Result::Ok(sdk) => sdk,
        Err(err) => {
            if FenvLatestService::latest_remote(context, version_prefix).is_ok() {
                bail!(
                    "the specified version is not installed: please do `fenv install {}` first",
                    version_prefix
                )
            } else {
                return Err(anyhow::anyhow!(err));
            }
        }
    };

    let version_file = context.fenv_global_version_file();
    version_file.parent().unwrap().create_dir_all()?;
    let mut file = std::fs::File::create(&version_file)?;
    writeln!(file, "{}", local_sdk.display_name())
        .with_context(|| format!("failed to write to file: {:?}", version_file))
}

fn show_global_version<'a>(
    context: &impl FenvContext,
    stdout: &mut impl std::io::Write,
) -> anyhow::Result<()> {
    let version_file = context.fenv_global_version_file();
    if !version_file.is_file() {
        if version_file.exists() {
            panic!("unexpected file: {:?}", version_file);
        }
        bail!("no specified global version.")
    }
    let version_or_channel = version_file
        .read_to_string()
        .context("failed to read the global version file")
        .map(|s| s.trim().to_string())?;
    if let Err(_) = LocalFlutterSdk::parse(&version_or_channel) {
        bail!(
            "the specified global version is neither a valid flutter version nor a channel: {}",
            version_or_channel
        )
    }

    if !FenvVersionsService::is_installed_versions_or_channel(context, &version_or_channel)? {
        bail!(
            "the specified global version is not installed: please do `fenv install {}` first",
            version_or_channel
        )
    }

    writeln!(stdout, "{version_or_channel}")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::macros::test_with_context;

    #[test]
    fn test_set_global_version_succeeds() {
        test_with_context(|config| {
            // setup
            let args = FenvGlobalArgs {
                version_prefix: Some("stable".to_string()),
            };
            let service = FenvGlobalService::new(args);
            // emulates installation of stable
            config
                .fenv_root()
                .join("versions/stable")
                .create_dir_all()
                .unwrap();

            // execution
            service.execute(config, &mut std::io::stdout()).unwrap();

            // validation
            let version_file_path = config.fenv_root().join("version");
            assert_eq!(
                std::fs::read_to_string(&version_file_path).unwrap(),
                "stable\n"
            );
        });
    }

    #[test]
    fn test_set_global_version_fails_when_not_a_valid_flutter_version() {
        test_with_context(|config| {
            // setup
            let args = FenvGlobalArgs {
                version_prefix: Some("invalid".to_string()),
            };
            let service = FenvGlobalService::new(args);

            // execution
            let result = service.execute(config, &mut std::io::stdout());

            // validation
            let err = &result.err().unwrap();
            assert_eq!(
                err.to_string(),
                "Not found any matched flutter sdk version: `invalid`"
            );
        });
    }

    #[test]
    fn test_set_global_version_fails_when_no_version_exists() {
        test_with_context(|config| {
            // setup
            let args = FenvGlobalArgs {
                version_prefix: Some("stable".to_string()),
            };
            let service = FenvGlobalService::new(args);

            // execution
            let result = service.execute(config, &mut std::io::stdout());

            // validation
            let err = &result.err().unwrap();
            assert_eq!(
                err.to_string(),
                "the specified version is not installed: please do `fenv install stable` first"
            );
        });
    }

    #[test]
    fn test_show_global_version_fails_when_no_global_version_file_exists() {
        test_with_context(|config| {
            // setup
            let args = FenvGlobalArgs {
                version_prefix: None,
            };
            let mut stdout: Vec<u8> = Vec::new();
            let service = FenvGlobalService::new(args);

            // execution
            let result = service.execute(config, &mut stdout);

            // validation
            let err = &result.err().unwrap();
            assert_eq!(err.to_string(), "no specified global version.");
        });
    }

    #[test]
    fn test_show_global_version_fails_when_global_version_exists_but_not_installed() {
        test_with_context(|config| {
            // setup
            let args = FenvGlobalArgs {
                version_prefix: None,
            };
            let mut stdout: Vec<u8> = Vec::new();
            let service = FenvGlobalService::new(args);
            // generates global version file
            let version_file_path = config.fenv_root().join("version");
            version_file_path.write("1.0.0").unwrap();

            // execution
            let result = service.execute(config, &mut stdout);

            // validation
            let err = &result.err().unwrap();
            assert_eq!(
            err.to_string(),
            "the specified global version is not installed: please do `fenv install 1.0.0` first"
        );
        });
    }

    #[test]
    fn test_show_global_version_fails_when_global_version_exists_but_not_valid() {
        test_with_context(|config| {
            // setup
            let args = FenvGlobalArgs {
                version_prefix: None,
            };
            let mut stdout: Vec<u8> = Vec::new();
            let service = FenvGlobalService::new(args);
            // generates global version file
            let version_file_path = config.fenv_root().join("version");
            version_file_path.write("invalid").unwrap();

            // execution
            let result = service.execute(config, &mut stdout);

            // validation
            let err = &result.err().unwrap();
            assert_eq!(
            err.to_string(),
            "the specified global version is neither a valid flutter version nor a channel: invalid"
        );
        });
    }

    #[test]
    fn test_show_global_version_succeeds() {
        test_with_context(|config| {
            // setup
            let args = FenvGlobalArgs {
                version_prefix: None,
            };
            let mut stdout: Vec<u8> = Vec::new();
            let service = FenvGlobalService::new(args);
            // generates global version file
            let version_file_path = config.fenv_root().join("version");
            version_file_path.write("1.0.0").unwrap();
            // emulates installation of 1.0.0
            config
                .fenv_root()
                .join("versions/1.0.0")
                .create_dir_all()
                .unwrap();

            // execution
            service.execute(config, &mut stdout).unwrap();

            // validation: check if stdout and "1.0.0" are equal
            assert_eq!(String::from_utf8(stdout.clone()).unwrap(), "1.0.0\n")
        });
    }
}
