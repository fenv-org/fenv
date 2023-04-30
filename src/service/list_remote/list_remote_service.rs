use crate::{
    args,
    context::FenvContext,
    external::git_command::{GitCommand, GitCommandImpl},
    sdk_service::{
        model::{
            flutter_sdk::FlutterSdk, local_flutter_sdk::LocalFlutterSdk,
            remote_flutter_sdk::RemoteFlutterSdk,
        },
        sdk_service::{RealSdkService, SdkService},
    },
    service::service::Service,
};
use std::collections::HashSet;

pub struct FenvListRemoteService {
    pub args: args::FenvListRemoteArgs,
    git_command: Box<dyn GitCommand>,
}

impl FenvListRemoteService {
    pub fn new(args: args::FenvListRemoteArgs) -> Self {
        Self {
            args,
            git_command: Box::from(GitCommandImpl::new()),
        }
    }
}

impl Service for FenvListRemoteService {
    fn execute(
        &self,
        context: &impl FenvContext,
        stdout: &mut impl std::io::Write,
    ) -> anyhow::Result<()> {
        let sdk_service = RealSdkService::new();
        execute_list_remote_command(context, stdout, &sdk_service, self.args.bare)
    }
}

fn execute_list_remote_command(
    context: &impl FenvContext,
    stdout: &mut impl std::io::Write,
    sdk_service: &impl SdkService,
    bare: bool,
) -> anyhow::Result<()> {
    let remote_sdks = sdk_service.get_available_remote_sdk_list(context)?;
    let installed_sdks = sdk_service.get_installed_sdk_list(context)?;
    display_remote_sdks(stdout, &remote_sdks, &installed_sdks, bare)
}

fn display_remote_sdks(
    stdout: &mut impl std::io::Write,
    remote_sdks: &[RemoteFlutterSdk],
    installed_sdks: &[LocalFlutterSdk],
    bare: bool,
) -> anyhow::Result<()> {
    let installed_sdks_set: HashSet<String> =
        installed_sdks.iter().map(|sdk| sdk.refs_name()).collect();

    for sdk in remote_sdks {
        if bare {
            writeln!(stdout, "{}", sdk.display_name())?;
        } else {
            let is_installed = installed_sdks_set.contains(&sdk.long);
            if is_installed {
                writeln!(stdout, "* {:18} [{}]", sdk.display_name(), &sdk.sha[..7])?;
            } else {
                writeln!(stdout, "  {:18} [{}]", sdk.display_name(), &sdk.sha[..7])?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{define_mock_valid_git_command, service::macros::test_with_context};
    use anyhow::Result;

    define_mock_valid_git_command!();

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

    #[test]
    fn text_list_remote_sdks_without_bare_option() {
        test_with_context(|config| {
            // setup
            let clock: Box<dyn Clock> = Box::new(SystemClock::new());
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
        });
    }

    #[test]
    fn text_list_remote_sdks_with_bare_option() {
        test_with_context(|config| {
            // setup
            let clock: Box<dyn Clock> = Box::new(SystemClock::new());
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
            let expected = read_resource_file(
                "resources/test/install_service/install-list-result-with-bare.txt",
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
        });
    }
}
