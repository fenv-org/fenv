use crate::{
    args::FenvGlobalArgs,
    context::FenvContext,
    sdk_service::{
        model::flutter_sdk::FlutterSdk, sdk_service::RealSdkService, sdk_service::SdkService as _,
    },
    service::service::Service,
};
use anyhow::{bail, Context};
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
    let sdk_service = RealSdkService::new();
    let local_sdk = match sdk_service.find_latest_local(context, version_prefix) {
        Result::Ok(sdk) => sdk,
        Err(err) => {
            if sdk_service
                .find_latest_remote(context, version_prefix)
                .is_ok()
            {
                bail!(
                    "The specified version is not installed: do `fenv install {}`",
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
    let sdk_service = RealSdkService::new();
    let version_file = match sdk_service.find_global_version_file(context) {
        Result::Ok(version_file) => version_file,
        Result::Err(err) => return Result::Err(err),
    };
    let (sdk, installed) = match sdk_service.read_version_file(context, &version_file) {
        Result::Ok(result) => (result.sdk, result.installed),
        Result::Err(err) => return Result::Err(err),
    };
    if installed {
        writeln!(stdout, "{}", sdk).map_err(|e| anyhow::anyhow!(e))
    } else {
        bail!("The specified version in `{version_file}` is not installed: do `fenv install {sdk}`")
    }
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
                "The specified version is not installed: do `fenv install stable`"
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
            assert_eq!(err.to_string(), "Could not find the global version file");
        });
    }

    #[test]
    fn test_show_global_version_fails_when_global_version_exists_but_not_installed() {
        test_with_context(|context| {
            // setup
            let args = FenvGlobalArgs {
                version_prefix: None,
            };
            let mut stdout: Vec<u8> = Vec::new();
            let service = FenvGlobalService::new(args);
            // generates global version file
            let version_file_path = context.fenv_root().join("version");
            version_file_path.write("1.0.0").unwrap();

            // execution
            let result = service.execute(context, &mut stdout);

            // validation
            let err = &result.err().unwrap();
            assert_eq!(
                err.to_string(),
                format!(
                    "The specified version in `{}` is not installed: do `fenv install 1.0.0`",
                    context.fenv_global_version_file()
                )
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
            assert_eq!(err.to_string(), "Invalid Flutter SDK: `invalid`");
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
