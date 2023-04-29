use super::list_remote_sdk::{
    cached_or_fetch_remote_sdks, show_remote_sdks, ShowRemoteSdksArguments,
};
use crate::{
    args,
    context::FenvContext,
    external::git_command::{GitCommand, GitCommandImpl},
    sdk_service::model::remote_flutter_sdk::RemoteFlutterSdk,
    service::{service::Service, versions::versions_service::FenvVersionsService},
    util::chrono_wrapper::{Clock, SystemClock},
};

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

    pub fn list_remote_sdks<'a>(
        context: &impl FenvContext,
        git_command: &Box<dyn GitCommand>,
    ) -> anyhow::Result<Vec<RemoteFlutterSdk>> {
        let clock: Box<dyn Clock> = Box::new(SystemClock::new());
        cached_or_fetch_remote_sdks(&context.fenv_cache(), git_command, &clock)
    }
}

impl Service for FenvListRemoteService {
    fn execute(
        &self,
        context: &impl FenvContext,
        stdout: &mut impl std::io::Write,
    ) -> anyhow::Result<()> {
        let clock: Box<dyn Clock> = Box::new(SystemClock::new());
        let installed_sdks = FenvVersionsService::list_installed_sdks(context)?;
        let args = ShowRemoteSdksArguments {
            cache_directory: &context.fenv_cache(),
            bare: self.args.bare,
            installed_sdks: &installed_sdks,
            git_command: &self.git_command,
            clock: &clock,
        };
        show_remote_sdks(&args, stdout)
    }
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
