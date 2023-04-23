use crate::config::Config;

use crate::service::service::Service;

use crate::service::versions::versions_service::FenvVersionsService;
use crate::util::chrono_wrapper::{Clock, SystemClock};
use crate::{args, model::remote_flutter_sdk::RemoteFlutterSdk};
use anyhow::{bail, Result};

use super::flutter_command::{FlutterCommand, FlutterCommandImpl};
use super::git_command::{GitCommand, GitCommandImpl};
use super::install_sdk::{self, install_sdk};
use super::list_remote_sdk::{list_remote_sdks, show_remote_sdks, ShowRemoteSdksArguments};

pub struct FenvInstallService {
    pub args: args::FenvInstallArgs,
    git_command: Box<dyn GitCommand>,
    flutter_command: Box<dyn FlutterCommand>,
}

impl FenvInstallService {
    pub fn from(args: args::FenvInstallArgs) -> FenvInstallService {
        FenvInstallService {
            args,
            git_command: Box::from(GitCommandImpl::new()),
            flutter_command: Box::from(FlutterCommandImpl::new()),
        }
    }

    pub fn list_remote_sdks() -> Result<Vec<RemoteFlutterSdk>> {
        let git_command: Box<dyn GitCommand> = Box::new(GitCommandImpl::new());
        list_remote_sdks(&git_command)
    }

    pub fn exists_installing_marker(
        versions_directory: &str,
        target_version_or_channel: &str,
    ) -> bool {
        install_sdk::exists_installing_marker(versions_directory, target_version_or_channel)
    }
}

impl Service for FenvInstallService {
    fn execute(&self, config: &Config, stdout: &mut impl std::io::Write) -> Result<()> {
        if self.args.list {
            // let sdks = cached_or_fetch_remote_sdks(&config.fenv_cache(), &self.git_command)?;
            let clock: Box<dyn Clock> = Box::new(SystemClock::new());
            let installed_sdks = FenvVersionsService::list_installed_sdks(config)?;
            let args = ShowRemoteSdksArguments {
                cache_directory: &config.fenv_cache(),
                bare: self.args.bare,
                installed_sdks: &installed_sdks,
                git_command: &self.git_command,
                clock: &clock,
            };
            show_remote_sdks(&args, stdout)
        } else if let Some(version) = &self.args.version {
            install_sdk(
                &config.fenv_versions(),
                &version,
                self.args.should_precache,
                &self.git_command,
                &self.flutter_command,
            )
        } else {
            bail!("Cannot handle arguments: {}", self.args)
        }
    }
}

#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use anyhow::{anyhow, Ok};

    use super::*;

    struct MockValidGitCommand;

    impl GitCommand for MockValidGitCommand {
        fn clone_flutter_sdk_by_channel(&self, _channel: &str, destination: &str) -> Result<()> {
            std::fs::create_dir(destination).map_err(|e| anyhow!(e))
        }

        fn clone_flutter_sdk_by_version(&self, _version: &str, destination: &str) -> Result<()> {
            std::fs::create_dir(destination).map_err(|e| anyhow!(e))
        }

        fn list_remote_sdks_by_tags(&self) -> Result<String> {
            read_resource_file("resources/test/install_service/git_lf-remote_tags.txt")
                .map_err(|e| anyhow!(e))
        }

        fn list_remote_sdks_by_branches(&self) -> Result<String> {
            read_resource_file("resources/test/install_service/git_lf-remote_heads.txt")
                .map_err(|e| anyhow!(e))
        }

        fn hard_reset_to_refs(&self, _working_dir: &str, _refs: &str) -> Result<()> {
            // do nothing
            Ok(())
        }
    }

    struct MockDummyGitCommand;

    impl GitCommand for MockDummyGitCommand {
        fn clone_flutter_sdk_by_channel(&self, _channel: &str, _destination: &str) -> Result<()> {
            panic!()
        }

        fn clone_flutter_sdk_by_version(&self, _version: &str, _destination: &str) -> Result<()> {
            panic!()
        }

        fn list_remote_sdks_by_tags(&self) -> Result<String> {
            panic!()
        }

        fn list_remote_sdks_by_branches(&self) -> Result<String> {
            panic!()
        }

        fn hard_reset_to_refs(&self, _working_dir: &str, _refs: &str) -> Result<()> {
            panic!()
        }
    }

    fn read_resource_file(relative_path: &str) -> std::io::Result<String> {
        let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        root.push(relative_path);
        std::fs::read_to_string(root)
    }

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
    fn text_list_remote_sdks_without_bare_option() {
        // setup
        let clock: Box<dyn Clock> = Box::new(SystemClock::new());
        let temp_fenv_root = tempfile::tempdir().unwrap();
        let temp_fenv_dir = tempfile::tempdir().unwrap();
        let temp_home = tempfile::tempdir().unwrap();
        let config = generate_config(&temp_fenv_root, &temp_fenv_dir, &temp_home);
        let installed_sdks = vec![];
        let git_command: Box<dyn GitCommand> = Box::new(MockValidGitCommand);
        let mut args = ShowRemoteSdksArguments {
            cache_directory: &config.fenv_cache(),
            bare: false,
            installed_sdks: &installed_sdks,
            git_command: &git_command,
            clock: &clock,
        };
        let mut stdout: Vec<u8> = Vec::new();

        // execution
        show_remote_sdks(&args, &mut stdout).unwrap();

        // validation of the `git ls-remote` behavior
        let output = String::from_utf8(stdout.clone()).unwrap();
        let expected = read_resource_file(
            "resources/test/install_service/install-list-result-without-bare.txt",
        )
        .unwrap();
        assert_eq!(output, expected);

        // setup with dummy git_command
        let git_command: Box<dyn GitCommand> = Box::new(MockDummyGitCommand);
        args.git_command = &git_command;
        stdout.clear();

        // execution
        show_remote_sdks(&args, &mut stdout).unwrap();

        // validation of the cache behavior
        let output = String::from_utf8(stdout.clone()).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn text_list_remote_sdks_with_bare_option() {
        // setup
        let clock: Box<dyn Clock> = Box::new(SystemClock::new());
        let temp_fenv_root = tempfile::tempdir().unwrap();
        let temp_fenv_dir = tempfile::tempdir().unwrap();
        let temp_home = tempfile::tempdir().unwrap();
        let config = generate_config(&temp_fenv_root, &temp_fenv_dir, &temp_home);
        let installed_sdks = vec![];
        let git_command: Box<dyn GitCommand> = Box::new(MockValidGitCommand);
        let mut args = ShowRemoteSdksArguments {
            cache_directory: &config.fenv_cache(),
            bare: true,
            installed_sdks: &installed_sdks,
            git_command: &git_command,
            clock: &clock,
        };
        let mut stdout: Vec<u8> = Vec::new();

        // execution
        show_remote_sdks(&args, &mut stdout).unwrap();

        // validation of the `git ls-remote` behavior
        let output = String::from_utf8(stdout.clone()).unwrap();
        let expected =
            read_resource_file("resources/test/install_service/install-list-result-with-bare.txt")
                .unwrap();
        assert_eq!(output, expected);

        // setup with dummy git_command
        let git_command: Box<dyn GitCommand> = Box::new(MockDummyGitCommand);
        args.git_command = &git_command;
        stdout.clear();

        // execution
        show_remote_sdks(&args, &mut stdout).unwrap();

        // validation of the cache behavior
        let output = String::from_utf8(stdout.clone()).unwrap();
        assert_eq!(output, expected);
    }
}
