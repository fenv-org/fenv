use crate::{
    args::FenvGlobalArgs,
    context::FenvContext,
    model::flutter_sdk::FlutterSdk,
    service::{service::Service, versions::versions_service::FenvVersionsService},
    util::path_like::PathLike,
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
        context: &FenvContext,
        stdout: &mut impl std::io::Write,
    ) -> anyhow::Result<()> {
        match &self.args.version_or_channel {
            Some(version_or_channel) => set_global_version(version_or_channel, context),
            None => show_global_version(&context, stdout),
        }
    }
}

fn set_global_version(target_version_or_channel: &str, config: &FenvContext) -> anyhow::Result<()> {
    if let Err(_) = FlutterSdk::parse(&target_version_or_channel) {
        bail!(
            "the specified version is neither a valid flutter version nor a channel: {}",
            target_version_or_channel
        )
    }

    if !FenvVersionsService::is_installed_versions_or_channel(config, &target_version_or_channel)? {
        bail!(
            "the specified version is not installed: please do `fenv install {}` first",
            target_version_or_channel
        )
    }

    let version_file = &config.fenv_root.join("version");
    if !&version_file.parent().unwrap().exists() {
        std::fs::create_dir_all(&version_file.parent().unwrap())?;
    }

    let mut file = std::fs::File::create(&version_file)?;
    writeln!(file, "{}", target_version_or_channel)?;
    Ok(())
}

fn show_global_version(
    config: &FenvContext,
    stdout: &mut impl std::io::Write,
) -> anyhow::Result<()> {
    let version_file = PathLike::from(&config.fenv_root).join("version");
    if !version_file.is_file() {
        if version_file.exists() {
            panic!("unexpected file: {:?}", version_file);
        }
        bail!("no specified global version.")
    }
    let version_or_channel = std::fs::read_to_string(&version_file)
        .context("failed to read the global version file")
        .map(|s| s.trim().to_string())?;
    if let Err(_) = FlutterSdk::parse(&version_or_channel) {
        bail!(
            "the specified global version is neither a valid flutter version nor a channel: {}",
            version_or_channel
        )
    }

    if !FenvVersionsService::is_installed_versions_or_channel(config, &version_or_channel)? {
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

    fn generate_config(
        temp_fenv_root: &tempfile::TempDir,
        temp_fenv_dir: &tempfile::TempDir,
        temp_home: &tempfile::TempDir,
    ) -> FenvContext {
        FenvContext {
            debug: false,
            fenv_root: PathLike::from(temp_fenv_root),
            fenv_dir: PathLike::from(temp_fenv_dir),
            home: PathLike::from(temp_home),
            default_shell: "bash".to_string(),
        }
    }

    #[test]
    fn test_set_global_version_succeeds() {
        // setup
        let args = FenvGlobalArgs {
            version_or_channel: Some("stable".to_string()),
        };
        let temp_fenv_root = tempfile::tempdir().unwrap();
        let temp_fenv_dir = tempfile::tempdir().unwrap();
        let temp_home = tempfile::tempdir().unwrap();
        let config = generate_config(&temp_fenv_root, &temp_fenv_dir, &temp_home);
        let service = FenvGlobalService::new(args);
        // emulates installation of stable
        std::fs::create_dir_all(&temp_fenv_root.path().join("versions/stable")).unwrap();

        // execution
        service.execute(&config, &mut std::io::stdout()).unwrap();

        // validation
        let version_file_path = temp_fenv_root.path().join("version");
        assert_eq!(
            std::fs::read_to_string(&version_file_path).unwrap(),
            "stable\n"
        );
    }

    #[test]
    fn test_set_global_version_fails_when_not_a_valid_flutter_version() {
        // setup
        let args = FenvGlobalArgs {
            version_or_channel: Some("invalid".to_string()),
        };
        let temp_fenv_root = tempfile::tempdir().unwrap();
        let temp_fenv_dir = tempfile::tempdir().unwrap();
        let temp_home = tempfile::tempdir().unwrap();
        let config = generate_config(&temp_fenv_root, &temp_fenv_dir, &temp_home);
        let service = FenvGlobalService::new(args);

        // execution
        let result = service.execute(&config, &mut std::io::stdout());

        // validation
        let err = &result.err().unwrap();
        assert_eq!(
            err.to_string(),
            "the specified version is neither a valid flutter version nor a channel: invalid"
        );
    }

    #[test]
    fn test_set_global_version_fails_when_no_version_exists() {
        // setup
        let args = FenvGlobalArgs {
            version_or_channel: Some("stable".to_string()),
        };
        let temp_fenv_root = tempfile::tempdir().unwrap();
        let temp_fenv_dir = tempfile::tempdir().unwrap();
        let temp_home = tempfile::tempdir().unwrap();
        let config = generate_config(&temp_fenv_root, &temp_fenv_dir, &temp_home);
        let service = FenvGlobalService::new(args);

        // execution
        let result = service.execute(&config, &mut std::io::stdout());

        // validation
        let err = &result.err().unwrap();
        assert_eq!(
            err.to_string(),
            "the specified version is not installed: please do `fenv install stable` first"
        );
    }

    #[test]
    fn test_show_global_version_fails_when_no_global_version_file_exists() {
        // setup
        let args = FenvGlobalArgs {
            version_or_channel: None,
        };
        let temp_fenv_root = tempfile::tempdir().unwrap();
        let temp_fenv_dir = tempfile::tempdir().unwrap();
        let temp_home = tempfile::tempdir().unwrap();
        let config = generate_config(&temp_fenv_root, &temp_fenv_dir, &temp_home);
        let mut stdout: Vec<u8> = Vec::new();
        let service = FenvGlobalService::new(args);

        // execution
        let result = service.execute(&config, &mut stdout);

        // validation
        let err = &result.err().unwrap();
        assert_eq!(err.to_string(), "no specified global version.");
    }

    #[test]
    fn test_show_global_version_fails_when_global_version_exists_but_not_installed() {
        // setup
        let args = FenvGlobalArgs {
            version_or_channel: None,
        };
        let temp_fenv_root = tempfile::tempdir().unwrap();
        let temp_fenv_dir = tempfile::tempdir().unwrap();
        let temp_home = tempfile::tempdir().unwrap();
        let config = generate_config(&temp_fenv_root, &temp_fenv_dir, &temp_home);
        let mut stdout: Vec<u8> = Vec::new();
        let service = FenvGlobalService::new(args);
        // generates global version file
        let version_file_path = temp_fenv_root.path().join("version");
        std::fs::write(&version_file_path, "1.0.0".as_bytes()).unwrap();

        // execution
        let result = service.execute(&config, &mut stdout);

        // validation
        let err = &result.err().unwrap();
        assert_eq!(
            err.to_string(),
            "the specified global version is not installed: please do `fenv install 1.0.0` first"
        );
    }

    #[test]
    fn test_show_global_version_fails_when_global_version_exists_but_not_valid() {
        // setup
        let args = FenvGlobalArgs {
            version_or_channel: None,
        };
        let temp_fenv_root = tempfile::tempdir().unwrap();
        let temp_fenv_dir = tempfile::tempdir().unwrap();
        let temp_home = tempfile::tempdir().unwrap();
        let config = generate_config(&temp_fenv_root, &temp_fenv_dir, &temp_home);
        let mut stdout: Vec<u8> = Vec::new();
        let service = FenvGlobalService::new(args);
        // generates global version file
        let version_file_path = temp_fenv_root.path().join("version");
        std::fs::write(&version_file_path, "invalid".as_bytes()).unwrap();

        // execution
        let result = service.execute(&config, &mut stdout);

        // validation
        let err = &result.err().unwrap();
        assert_eq!(
            err.to_string(),
            "the specified global version is neither a valid flutter version nor a channel: invalid"
        );
    }

    #[test]
    fn test_show_global_version_succeeds() {
        // setup
        let args = FenvGlobalArgs {
            version_or_channel: None,
        };
        let temp_fenv_root = tempfile::tempdir().unwrap();
        let temp_fenv_dir = tempfile::tempdir().unwrap();
        let temp_home = tempfile::tempdir().unwrap();
        let config = generate_config(&temp_fenv_root, &temp_fenv_dir, &temp_home);
        let mut stdout: Vec<u8> = Vec::new();
        let service = FenvGlobalService::new(args);
        // generates global version file
        let version_file_path = &temp_fenv_root.path().join("version");
        std::fs::write(&version_file_path, "1.0.0".as_bytes()).unwrap();
        // emulates installation of 1.0.0
        std::fs::create_dir_all(&temp_fenv_root.path().join("versions/1.0.0")).unwrap();

        // execution
        service.execute(&config, &mut stdout).unwrap();

        // validation: check if stdout and "1.0.0" are equal
        assert_eq!(String::from_utf8(stdout.clone()).unwrap(), "1.0.0\n")
    }
}
