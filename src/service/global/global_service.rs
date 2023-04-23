use std::path::PathBuf;

use anyhow::{bail, Context};

use crate::{
    args::FenvGlobalArgs,
    config::Config,
    service::{service::Service, versions::versions_service::FenvVersionsService},
};

pub struct FenvGlobalService {
    args: FenvGlobalArgs,
}

impl FenvGlobalService {
    pub fn from(args: FenvGlobalArgs) -> Self {
        Self { args }
    }
}

impl Service for FenvGlobalService {
    fn execute(&self, config: &Config, stdout: &mut impl std::io::Write) -> anyhow::Result<()> {
        match self.args.version_or_channel {
            Some(_) => todo!(),
            None => show_global_version(&config, stdout),
        }
    }
}

fn show_global_version(config: &Config, stdout: &mut impl std::io::Write) -> anyhow::Result<()> {
    let version_file = PathBuf::from(&config.fenv_root).join("version");
    if !version_file.is_file() {
        if version_file.exists() {
            panic!("unexpected file: {:?}", version_file);
        }
        bail!("no specified global version.")
    }
    let version_or_channel = std::fs::read_to_string(&version_file)
        .context("failed to read the global version file")
        .map(|s| s.trim().to_string())?;
    let installed_versions = FenvVersionsService::list_installed_sdks(config)?;
    if let None = installed_versions
        .iter()
        .find(|sdk| sdk.display_name() == version_or_channel)
    {
        bail!(
            "the specified global version is not installed: {}",
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
    ) -> Config {
        Config {
            debug: false,
            fenv_root: temp_fenv_root.path().to_str().unwrap().to_string(),
            fenv_dir: temp_fenv_dir.path().to_str().unwrap().to_string(),
            default_shell: "bash".to_string(),
            home: temp_home.path().to_str().unwrap().to_string(),
        }
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
        let service = FenvGlobalService::from(args);

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
        let service = FenvGlobalService::from(args);
        // generates global version file
        let version_file_path = temp_fenv_root.path().join("version");
        std::fs::write(&version_file_path, "1.0.0".as_bytes()).unwrap();

        // execution
        let result = service.execute(&config, &mut stdout);

        // validation
        let err = &result.err().unwrap();
        assert_eq!(
            err.to_string(),
            "the specified global version is not installed: 1.0.0"
        );
    }

    #[test]
    fn test_show_global_version_works() {
        // setup
        let args = FenvGlobalArgs {
            version_or_channel: None,
        };
        let temp_fenv_root = tempfile::tempdir().unwrap();
        let temp_fenv_dir = tempfile::tempdir().unwrap();
        let temp_home = tempfile::tempdir().unwrap();
        let config = generate_config(&temp_fenv_root, &temp_fenv_dir, &temp_home);
        let mut stdout: Vec<u8> = Vec::new();
        let service = FenvGlobalService::from(args);
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
