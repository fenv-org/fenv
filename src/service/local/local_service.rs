use crate::{
    args::FenvLocalArgs,
    context::FenvContext,
    sdk_service::{
        model::flutter_sdk::FlutterSdk,
        results::{LookupResult, VersionFileReadResult},
        sdk_service::SdkService,
    },
    service::service::Service,
};
use anyhow::{bail, Context};
use log::debug;
use std::{io::Write, os::unix::fs};

pub struct FenvLocalService {
    args: FenvLocalArgs,
}

impl FenvLocalService {
    pub fn new(args: FenvLocalArgs) -> Self {
        Self { args }
    }
}

impl Service for FenvLocalService {
    fn execute(
        &self,
        context: &impl FenvContext,
        sdk_service: &impl SdkService,
        stdout: &mut impl Write,
    ) -> anyhow::Result<()> {
        match &self.args.prefix {
            Some(prefix) => {
                set_local_version(context, sdk_service, prefix)?;
                install_symlink(context, sdk_service)?;
                Ok(())
            }
            None => {
                if self.args.symlink {
                    install_symlink_and_show_local_version(context, sdk_service, stdout)
                } else {
                    show_local_version(context, sdk_service, stdout)
                }
            }
        }
    }
}

fn read_nearest_local_version_inner(
    context: &impl FenvContext,
    sdk_service: &impl SdkService,
) -> anyhow::Result<VersionFileReadResult> {
    match sdk_service.read_nearest_local_version(context, &context.fenv_dir()) {
        LookupResult::Found(result) => Ok(result),
        LookupResult::Err(err) => {
            let file = sdk_service
                .find_nearest_local_version_file(&context.fenv_dir())
                .unwrap();
            return Result::Err(anyhow::anyhow!(err))
                .context(format!("Failed to read the local version: `{file}`"));
        }
        LookupResult::None => bail!("Could not find any local version file"),
    }
}

fn create_symlink_inner(context: &impl FenvContext, sdk: &impl FlutterSdk) -> anyhow::Result<()> {
    let original_path = context.fenv_sdk_root(&sdk.display_name());
    let symlink_path = context.fenv_dir().join(".flutter");
    debug!("original_path: {original_path}",);
    debug!("symlink_path: {symlink_path}",);
    symlink_path
        .remove_file()
        .with_context(|| format!("Failed to remove the existing symlink: `{symlink_path}`"))?;
    fs::symlink(&original_path, &symlink_path).with_context(|| {
        format!("Failed to create a symlink to the installed version: `{symlink_path}`")
    })
}

fn show_local_version(
    context: &impl FenvContext,
    sdk_service: &impl SdkService,
    stdout: &mut impl Write,
) -> anyhow::Result<()> {
    let read_result = read_nearest_local_version_inner(context, sdk_service)?;
    if !read_result.installed {
        let local_version_file = sdk_service
            .find_nearest_local_version_file(&context.fenv_dir())
            .unwrap();
        eprintln!( "warn: The specified version in `{local_version_file}` is not installed: do `fenv install && fenv local --symlink`")
    }
    writeln!(stdout, "{}", read_result.sdk).map_err(|e| anyhow::anyhow!(e))
}

fn install_symlink_and_show_local_version(
    context: &impl FenvContext,
    sdk_service: &impl SdkService,
    stdout: &mut impl Write,
) -> anyhow::Result<()> {
    let read_result = read_nearest_local_version_inner(context, sdk_service)?;
    if !read_result.installed {
        let local_version_file = sdk_service
            .find_nearest_local_version_file(&context.fenv_dir())
            .unwrap();
        bail!(
            "The specified version in `{local_version_file}` is not installed: do `fenv install {sdk}`",
            sdk = read_result.sdk
        )
    }

    create_symlink_inner(context, &read_result.sdk)?;
    writeln!(stdout, "{}", read_result.sdk).map_err(|e| anyhow::anyhow!(e))
}

fn install_symlink(
    context: &impl FenvContext,
    sdk_service: &impl SdkService,
) -> anyhow::Result<()> {
    let read_result = match sdk_service.read_nearest_local_version(context, &context.fenv_dir()) {
        LookupResult::Found(result) => result,
        LookupResult::Err(err) => return Result::Err(anyhow::anyhow!(err)),
        LookupResult::None => bail!("Could not find any local version file"),
    };

    if read_result.installed {
        let original_path = context.fenv_sdk_root(&read_result.sdk.display_name());
        let symlink_path = context.fenv_dir().join(".flutter");
        debug!("original_path: {original_path}",);
        debug!("symlink_path: {symlink_path}",);
        symlink_path
            .remove_file()
            .with_context(|| format!("Failed to remove the existing symlink: `{symlink_path}`"))?;
        fs::symlink(&original_path, &symlink_path).with_context(|| {
            format!("Failed to create a symlink to the installed version: `{symlink_path}`")
        })
    } else {
        let local_version_file = sdk_service
            .find_nearest_local_version_file(&context.fenv_dir())
            .unwrap();
        bail!(
            "The specified version in `{local_version_file}` is not installed: do `fenv install {sdk}`",
            sdk = read_result.sdk
        )
    }
}

fn set_local_version(
    context: &impl FenvContext,
    sdk_service: &impl SdkService,
    prefix: &str,
) -> anyhow::Result<()> {
    let sdk = match sdk_service.find_latest_local(context, prefix) {
        LookupResult::Found(sdk) => sdk,
        LookupResult::Err(err) => return Err(anyhow::anyhow!(err)),
        LookupResult::None => {
            if sdk_service.find_latest_remote(context, prefix).is_found() {
                bail!("The specified version is not installed: do `fenv install {prefix} && fenv local {prefix}`")
            } else {
                bail!("Not found any matched flutter sdk version: `{prefix}`")
            }
        }
    };

    sdk_service.write_local_version(&context.fenv_dir(), &sdk)
}

#[cfg(test)]
mod tests {
    use crate::{
        context::FenvContext, define_mock_valid_git_command,
        external::flutter_command::FlutterCommandImpl, sdk_service::sdk_service::RealSdkService,
        service::macros::test_with_context, stdout_to_string, try_run,
        util::chrono_wrapper::SystemClock,
    };
    use std::{io::Write, path::PathBuf};

    define_mock_valid_git_command!();

    #[test]
    pub fn test_show_local_version_fails_if_no_local_version_file_exists() {
        test_with_context(|context| {
            // setup
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            let mut stdout: Vec<u8> = vec![];
            let result = try_run(&["fenv", "local"], context, &sdk_service, &mut stdout);

            // validation
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                "Could not find any local version file"
            )
        })
    }

    #[test]
    pub fn test_show_local_version_succeeds_even_if_specified_version_is_not_installed() {
        test_with_context(|context| {
            // setup
            // prepare the local version file to contain sdk version 1.0.0
            context
                .fenv_dir()
                .join(".flutter-version")
                .writeln("1.0.0")
                .unwrap();
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            let mut stdout: Vec<u8> = vec![];
            try_run(&["fenv", "local"], context, &sdk_service, &mut stdout).unwrap();

            // validation
            assert_eq!(stdout_to_string!(stdout), "1.0.0\n")
        })
    }

    #[test]
    pub fn test_show_local_version_succeeds_if_specified_version_is_installed() {
        test_with_context(|context| {
            // setup
            // prepare the local version file to contain sdk version 1.0.0
            context
                .fenv_dir()
                .join(".flutter-version")
                .writeln("1.0.0")
                .unwrap();
            context
                .fenv_versions()
                .join("1.0.0")
                .create_dir_all()
                .unwrap();
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            let mut stdout: Vec<u8> = vec![];
            try_run(&["fenv", "local"], context, &sdk_service, &mut stdout).unwrap();

            // validation
            assert_eq!(stdout_to_string!(stdout), "1.0.0\n")
        })
    }

    #[test]
    pub fn test_show_local_version_fails_if_error_happens_while_reading_local_version_file() {
        test_with_context(|context| {
            // setup
            // prepare the local version file to contain invalid UTF-8 sequence
            let mut version_file = context
                .fenv_dir()
                .join(".flutter-version")
                .create_file()
                .unwrap();
            version_file.write(&[0xDE, 0xED, 0xBE, 0xEF]).unwrap();
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            let mut stdout: Vec<u8> = vec![];
            let result = try_run(&["fenv", "local"], context, &sdk_service, &mut stdout);

            // validation
            assert!(result.is_err());
            assert_eq!(
                result.err().unwrap().to_string(),
                format!(
                    "Failed to read the local version: `{}/.flutter-version`",
                    context.fenv_dir()
                )
            )
        })
    }

    #[test]
    pub fn test_install_symlink_succeeds_if_specified_version_is_installed() {
        test_with_context(|context| {
            // setup
            context
                .fenv_dir()
                .join(".flutter-version")
                .writeln("1.0.0")
                .unwrap();
            context
                .fenv_versions()
                .join("1.0.0")
                .create_dir_all()
                .unwrap();
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            let mut stdout: Vec<u8> = vec![];
            try_run(
                &["fenv", "local", "--symlink"],
                context,
                &sdk_service,
                &mut stdout,
            )
            .unwrap();

            // validation
            assert_eq!(stdout_to_string!(stdout), "1.0.0\n");
            let symlink: PathBuf = context.fenv_dir().join(".flutter").path().to_owned();
            assert!(symlink.is_symlink());
            let symlink_destination = symlink.read_link().unwrap();
            let sdk_path = context.fenv_versions().join("1.0.0").path().to_owned();
            assert_eq!(symlink_destination.to_str(), sdk_path.to_str());
        })
    }

    #[test]
    pub fn test_install_symlink_fails_if_no_local_version_file_exists() {
        test_with_context(|context| {
            // setup
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            let mut stdout: Vec<u8> = vec![];
            let result = try_run(
                &["fenv", "local", "--symlink"],
                context,
                &sdk_service,
                &mut stdout,
            );

            // validation
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                "Could not find any local version file"
            )
        })
    }

    #[test]
    pub fn test_install_symlink_fails_if_specified_version_is_not_installed() {
        test_with_context(|context| {
            // setup
            // prepare the local version file to contain sdk version 1.0.0
            context
                .fenv_dir()
                .join(".flutter-version")
                .writeln("1.0.0")
                .unwrap();
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            let mut stdout: Vec<u8> = vec![];
            let result = try_run(
                &["fenv", "local", "--symlink"],
                context,
                &sdk_service,
                &mut stdout,
            );

            // validation
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                format!(
                    "The specified version in `{}/.flutter-version` is not installed: do `fenv install 1.0.0`",
                    context.fenv_dir()
                )
            )
        })
    }

    #[test]
    pub fn test_set_local_version_succeeds_if_specified_version_is_installed() {
        test_with_context(|context| {
            // setup
            context
                .fenv_versions()
                .join("1.0.0")
                .create_dir_all()
                .unwrap();
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            let mut stdout: Vec<u8> = vec![];
            try_run(
                &["fenv", "local", "1.0.0"],
                context,
                &sdk_service,
                &mut stdout,
            )
            .unwrap();

            // validation
            assert_eq!(stdout_to_string!(stdout), "");
            assert_eq!(
                context
                    .fenv_dir()
                    .join(".flutter-version")
                    .read_to_string()
                    .unwrap(),
                "1.0.0\n"
            );
            let symlink: PathBuf = context.fenv_dir().join(".flutter").path().to_owned();
            assert!(symlink.is_symlink());
            let symlink_destination = symlink.read_link().unwrap();
            let sdk_path = context.fenv_versions().join("1.0.0").path().to_owned();
            assert_eq!(symlink_destination.to_str(), sdk_path.to_str());
        })
    }

    #[test]
    pub fn test_set_local_version_fails_if_specified_version_is_not_installed() {
        test_with_context(|context| {
            // setup
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            let mut stdout: Vec<u8> = vec![];
            let result = try_run(&["fenv", "local", "1"], context, &sdk_service, &mut stdout);

            // validation
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                "The specified version is not installed: do `fenv install 1 && fenv local 1`"
            )
        })
    }

    #[test]
    pub fn test_set_local_version_fails_if_specified_version_does_not_exist() {
        test_with_context(|context| {
            // setup
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            let mut stdout: Vec<u8> = vec![];
            let result = try_run(
                &["fenv", "local", "invalid"],
                context,
                &sdk_service,
                &mut stdout,
            );

            // validation
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                "Not found any matched flutter sdk version: `invalid`"
            )
        })
    }
}