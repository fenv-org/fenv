use crate::{
    args::FenvUpdateArgs,
    context::FenvContext,
    external::github_client::GitHubClient,
    release_service::model::fenv_release::FenvRelease,
    sdk_service::sdk_service::SdkService,
    service::service::Service,
    util::io::{ConsoleInput, ConsoleOutput},
};
use anyhow::{Context, Result};
use std::io::Write;

pub struct FenvUpdateService {
    args: FenvUpdateArgs,
}

impl FenvUpdateService {
    pub fn new(args: FenvUpdateArgs) -> Self {
        Self { args }
    }

    fn check_for_updates(
        &self,
        github_client: &dyn GitHubClient,
        current_version: &str,
    ) -> Result<Option<FenvRelease>> {
        let release = github_client
            .fetch_latest_release()
            .context("Failed to check for updates")?;

        match release {
            Some(release_info) => {
                let latest_version = FenvRelease::from_version_or_tag(&release_info.tag_name)
                    .context("Invalid version format from GitHub release")?;
                let current = FenvRelease::new(current_version)
                    .context("Invalid current version format")?;

                if latest_version.is_newer_than(&current) {
                    Ok(Some(latest_version))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    fn prompt_confirmation<OUT: Write, ERR: Write>(
        &self,
        input: &mut dyn ConsoleInput,
        output: &mut dyn ConsoleOutput<OUT, ERR>,
    ) -> Result<bool> {
        write!(output.stdout(), "Do you want to update? [Y/n] ")?;
        output.stdout().flush()?;

        let response = input.read_char()?;
        // None (Enter) = default Yes, 'y'/'Y' = Yes, anything else = No
        Ok(response.is_none() || matches!(response, Some('y') | Some('Y')))
    }

    fn download_and_execute_install_script(&self) -> Result<()> {
        // Download the install script
        let client = reqwest::blocking::Client::new();
        let script_content = client
            .get("https://fenv-install.jerry.company")
            .send()
            .context("Failed to download install script")?
            .text()
            .context("Failed to read install script content")?;

        // Create a temporary file
        let temp_file = tempfile::NamedTempFile::new()
            .context("Failed to create temporary file")?;
        let temp_path = temp_file.path();

        // Write the script to the temp file
        std::fs::write(temp_path, script_content)
            .context("Failed to write install script to temporary file")?;

        // Execute the script with bash
        let status = std::process::Command::new("bash")
            .arg(temp_path)
            .status()
            .context("Failed to execute install script")?;

        if !status.success() {
            anyhow::bail!("Install script failed with exit code: {:?}", status.code());
        }

        Ok(())
    }
}

impl<OUT: Write, ERR: Write> Service<OUT, ERR> for FenvUpdateService {
    fn execute(
        &self,
        context: &impl FenvContext,
        sdk_service: &impl SdkService,
        input: &mut dyn ConsoleInput,
        output: &mut dyn ConsoleOutput<OUT, ERR>,
    ) -> Result<()> {
        let current_version = context.current_version();

        // Check for updates
        let latest_version =
            self.check_for_updates(sdk_service.github_client(), &current_version)?;

        match latest_version {
            Some(latest) if self.args.force || latest.is_newer_than(&FenvRelease::new(&current_version).unwrap()) => {
                writeln!(
                    output.stdout(),
                    "A new version is available: {} → {}",
                    current_version,
                    latest.version
                )?;

                // Prompt for confirmation unless --yes is set
                if !self.args.yes && !self.prompt_confirmation(input, output)? {
                    writeln!(output.stdout(), "Update cancelled.")?;
                    return Ok(());
                }

                // Download and execute the install script
                writeln!(output.stdout(), "Downloading and installing update...")?;
                self.download_and_execute_install_script()?;
                writeln!(output.stdout(), "Update completed successfully!")?;
            }
            _ if self.args.force => {
                writeln!(
                    output.stdout(),
                    "Force updating fenv (current version: {})",
                    current_version
                )?;

                // Prompt for confirmation unless --yes is set
                if !self.args.yes && !self.prompt_confirmation(input, output)? {
                    writeln!(output.stdout(), "Update cancelled.")?;
                    return Ok(());
                }

                // Download and execute the install script
                writeln!(output.stdout(), "Downloading and installing update...")?;
                self.download_and_execute_install_script()?;
                writeln!(output.stdout(), "Update completed successfully!")?;
            }
            _ => {
                writeln!(
                    output.stdout(),
                    "fenv is up to date (version {})",
                    current_version
                )?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        external::{
            flutter_command::FlutterCommandImpl,
            git_command::GitCommandImpl,
            github_client::{GitHubReleaseResponse, MockGitHubClient},
        },
        sdk_service::sdk_service::RealSdkService,
        service::macros::test_with_context,
        try_run,
        util::{
            chrono_wrapper::SystemClock,
            io::BufferedInput,
        },
    };

    fn create_sdk_service_with_mock_github(
        mock_client: MockGitHubClient,
    ) -> RealSdkService<GitCommandImpl, SystemClock, FlutterCommandImpl, MockGitHubClient> {
        RealSdkService::from(
            GitCommandImpl::new(),
            SystemClock,
            FlutterCommandImpl::new(),
            mock_client,
        )
    }

    #[test]
    fn test_update_no_new_version() {
        test_with_context(|context, output| {
            // Setup
            let mut mock_client = MockGitHubClient::new();
            mock_client
                .expect_fetch_latest_release()
                .times(1)
                .returning(|| {
                    Ok(Some(GitHubReleaseResponse {
                        tag_name: "v0.3.0".to_string(),
                        name: "Release 0.3.0".to_string(),
                        html_url: "https://github.com/fenv-org/fenv/releases/tag/v0.3.0"
                            .to_string(),
                        published_at: "2024-01-01T00:00:00Z".to_string(),
                    }))
                });

            let sdk_service = create_sdk_service_with_mock_github(mock_client);
            let mut input = BufferedInput::new();

            // Execute
            let result = try_run(
                &["fenv", "update"],
                context,
                &sdk_service,
                &mut input,
                output,
            );

            // Verify
            assert!(result.is_ok());
            assert!(output.stdout_to_string().contains("fenv is up to date"));
        });
    }

    #[test]
    fn test_update_new_version_available_with_yes_flag() {
        test_with_context(|context, output| {
            // Setup
            let mut mock_client = MockGitHubClient::new();
            mock_client
                .expect_fetch_latest_release()
                .times(1)
                .returning(|| {
                    Ok(Some(GitHubReleaseResponse {
                        tag_name: "v99.0.0".to_string(),
                        name: "Release 99.0.0".to_string(),
                        html_url: "https://github.com/fenv-org/fenv/releases/tag/v99.0.0"
                            .to_string(),
                        published_at: "2024-01-02T00:00:00Z".to_string(),
                    }))
                });

            let sdk_service = create_sdk_service_with_mock_github(mock_client);
            let mut input = BufferedInput::new();

            // Execute
            let result = try_run(
                &["fenv", "update", "--yes"],
                context,
                &sdk_service,
                &mut input,
                output,
            );

            // Verify - should show the new version message
            let stdout = output.stdout_to_string();
            assert!(stdout.contains("A new version is available"));
            assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
            assert!(stdout.contains("99.0.0"));
            // The execute call should fail at the download step (expected behavior)
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_update_user_confirms_with_empty_input_default_yes() {
        test_with_context(|context, output| {
            // Setup
            let mut mock_client = MockGitHubClient::new();
            mock_client
                .expect_fetch_latest_release()
                .times(1)
                .returning(|| {
                    Ok(Some(GitHubReleaseResponse {
                        tag_name: "v99.0.0".to_string(),
                        name: "Release 0.4.0".to_string(),
                        html_url: "https://github.com/fenv-org/fenv/releases/tag/v99.0.0"
                            .to_string(),
                        published_at: "2024-01-02T00:00:00Z".to_string(),
                    }))
                });

            let sdk_service = create_sdk_service_with_mock_github(mock_client);
            // BufferedInput with None (Enter key = default Yes)
            let mut input = BufferedInput::with_responses(vec![None]);

            // Execute
            let result = try_run(
                &["fenv", "update"],
                context,
                &sdk_service,
                &mut input,
                output,
            );

            // Verify - should proceed with update (not cancelled)
            let stdout = output.stdout_to_string();
            assert!(stdout.contains("A new version is available"));
            assert!(stdout.contains("Downloading and installing update"));
            assert!(!stdout.contains("Update cancelled"));
            // The execute call should fail at the download step (expected behavior)
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_update_user_confirms_with_y() {
        test_with_context(|context, output| {
            // Setup
            let mut mock_client = MockGitHubClient::new();
            mock_client
                .expect_fetch_latest_release()
                .times(1)
                .returning(|| {
                    Ok(Some(GitHubReleaseResponse {
                        tag_name: "v99.0.0".to_string(),
                        name: "Release 0.4.0".to_string(),
                        html_url: "https://github.com/fenv-org/fenv/releases/tag/v99.0.0"
                            .to_string(),
                        published_at: "2024-01-02T00:00:00Z".to_string(),
                    }))
                });

            let sdk_service = create_sdk_service_with_mock_github(mock_client);
            // BufferedInput with 'y'
            let mut input = BufferedInput::with_responses(vec![Some('y')]);

            // Execute
            let result = try_run(
                &["fenv", "update"],
                context,
                &sdk_service,
                &mut input,
                output,
            );

            // Verify - should proceed with update (not cancelled)
            let stdout = output.stdout_to_string();
            assert!(stdout.contains("A new version is available"));
            assert!(stdout.contains("Downloading and installing update"));
            assert!(!stdout.contains("Update cancelled"));
            // The execute call should fail at the download step (expected behavior)
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_update_user_declines_with_n() {
        test_with_context(|context, output| {
            // Setup
            let mut mock_client = MockGitHubClient::new();
            mock_client
                .expect_fetch_latest_release()
                .times(1)
                .returning(|| {
                    Ok(Some(GitHubReleaseResponse {
                        tag_name: "v99.0.0".to_string(),
                        name: "Release 0.4.0".to_string(),
                        html_url: "https://github.com/fenv-org/fenv/releases/tag/v99.0.0"
                            .to_string(),
                        published_at: "2024-01-02T00:00:00Z".to_string(),
                    }))
                });

            let sdk_service = create_sdk_service_with_mock_github(mock_client);
            // BufferedInput with 'n'
            let mut input = BufferedInput::with_responses(vec![Some('n')]);

            // Execute
            let result = try_run(
                &["fenv", "update"],
                context,
                &sdk_service,
                &mut input,
                output,
            );

            // Verify - should be cancelled
            assert!(result.is_ok());
            let stdout = output.stdout_to_string();
            assert!(stdout.contains("Update cancelled"));
            assert!(!stdout.contains("Downloading"));
        });
    }

    #[test]
    fn test_update_user_declines_with_other_key() {
        test_with_context(|context, output| {
            // Setup
            let mut mock_client = MockGitHubClient::new();
            mock_client
                .expect_fetch_latest_release()
                .times(1)
                .returning(|| {
                    Ok(Some(GitHubReleaseResponse {
                        tag_name: "v99.0.0".to_string(),
                        name: "Release 0.4.0".to_string(),
                        html_url: "https://github.com/fenv-org/fenv/releases/tag/v99.0.0"
                            .to_string(),
                        published_at: "2024-01-02T00:00:00Z".to_string(),
                    }))
                });

            let sdk_service = create_sdk_service_with_mock_github(mock_client);
            // BufferedInput with 'x' (any key other than y/Y/Enter should decline)
            let mut input = BufferedInput::with_responses(vec![Some('x')]);

            // Execute
            let result = try_run(
                &["fenv", "update"],
                context,
                &sdk_service,
                &mut input,
                output,
            );

            // Verify - should be cancelled
            assert!(result.is_ok());
            let stdout = output.stdout_to_string();
            assert!(stdout.contains("Update cancelled"));
            assert!(!stdout.contains("Downloading"));
        });
    }
}
