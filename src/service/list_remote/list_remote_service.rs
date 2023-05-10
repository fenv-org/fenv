use crate::{
    args,
    context::FenvContext,
    sdk_service::{
        model::{
            flutter_sdk::FlutterSdk, local_flutter_sdk::LocalFlutterSdk,
            remote_flutter_sdk::RemoteFlutterSdk,
        },
        sdk_service::SdkService,
    },
    service::service::Service,
};
use std::collections::HashSet;

pub struct FenvListRemoteService {
    pub args: args::FenvListRemoteArgs,
}

impl FenvListRemoteService {
    pub fn new(args: args::FenvListRemoteArgs) -> Self {
        Self { args }
    }
}

impl Service for FenvListRemoteService {
    fn execute(
        &self,
        context: &impl FenvContext,
        sdk_service: &impl SdkService,
        stdout: &mut impl std::io::Write,
    ) -> anyhow::Result<()> {
        execute_list_remote_command(context, stdout, sdk_service, self.args.bare)
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
    use crate::{
        define_mock_dummy_git_command, define_mock_valid_git_command,
        external::flutter_command::FlutterCommandImpl, sdk_service::sdk_service::RealSdkService,
        service::macros::test_with_context, util::chrono_wrapper::SystemClock,
    };

    define_mock_valid_git_command!();
    define_mock_dummy_git_command!();

    #[test]
    fn text_list_remote_sdks_without_bare_option() {
        test_with_context(|context| {
            // setup
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );
            let mut stdout: Vec<u8> = Vec::new();

            // execution
            execute_list_remote_command(context, &mut stdout, &sdk_service, false).unwrap();

            // validation of the `git ls-remote` behavior
            let output = String::from_utf8(stdout.clone()).unwrap();
            let expected = read_resource_file(
                "resources/test/install_service/install-list-result-without-bare.txt",
            )
            .unwrap();
            assert_eq!(output, expected);

            // setup with dummy git_command
            let sdk_service = RealSdkService::from(
                MockDummyGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );
            stdout.clear();

            // execution
            execute_list_remote_command(context, &mut stdout, &sdk_service, false).unwrap();

            // validation of the cache behavior
            let output = String::from_utf8(stdout.clone()).unwrap();
            assert_eq!(output, expected);
        });
    }

    #[test]
    fn text_list_remote_sdks_with_bare_option() {
        test_with_context(|context| {
            // setup
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );
            let mut stdout: Vec<u8> = Vec::new();

            // execution
            execute_list_remote_command(context, &mut stdout, &sdk_service, true).unwrap();

            // validation of the `git ls-remote` behavior
            let output = String::from_utf8(stdout.clone()).unwrap();
            let expected = read_resource_file(
                "resources/test/install_service/install-list-result-with-bare.txt",
            )
            .unwrap();
            assert_eq!(output, expected);

            // setup with dummy git_command
            let sdk_service = RealSdkService::from(
                MockDummyGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );
            stdout.clear();

            // execution
            execute_list_remote_command(context, &mut stdout, &sdk_service, true).unwrap();

            // validation of the cache behavior
            let output = String::from_utf8(stdout.clone()).unwrap();
            assert_eq!(output, expected);
        });
    }
}
