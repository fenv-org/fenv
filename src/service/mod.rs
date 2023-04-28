pub mod completions;
pub mod global;
pub mod init;
pub mod install;
pub mod latest;
pub mod list_remote;
pub mod service;
pub mod version_file;
pub mod versions;

pub mod macros {
    use crate::{context::FenvContext, util::path_like::PathLike};

    #[macro_export(local_inner_macros)]
    macro_rules! spawn_and_wait {
        ($expr: expr, $fn_name: expr, $($arg:tt)+) => {{
            // TODO: Infer the execution function name in the macro.
            let command = $expr;
            log::info!(
                "{}(): command: program={:?}: args={:?}",
                $fn_name,
                command.get_program(),
                command.get_args()
            );
            let child = &mut command.spawn().with_context(|| std::format!($($arg)+))?;
            let exit_status = &mut child.wait().with_context(|| std::format!($($arg)+))?;
            if !exit_status.success() {
                let message = std::format!($($arg)+);
                anyhow::bail!(
                    "{message}: OS state code - {code}",
                    code = exit_status.code().unwrap()
                )
            }
        }};
    }

    #[macro_export(local_inner_macros)]
    macro_rules! spawn_and_capture {
        ($expr: expr, $fn_name: expr, $($arg:tt)+) => {{
            // TODO: Infer the execution function name in the macro.
            let command = $expr;
            log::info!(
                "{}(): command: program={:?}: args={:?}",
                $fn_name,
                command.get_program(),
                command.get_args()
            );
            let output = command.output().with_context(|| std::format!($($arg)+))?;
            if !output.status.success() {
                log::debug!(
                    "{}(): stderr:\n{}",
                    $fn_name,
                    String::from_utf8(output.stderr)?
                );
                let message = std::format!($($arg)+);
                anyhow::bail!(
                    "{message}: OS state code - {code}",
                    code = output.status.code().unwrap()
                )
            }
            let stdout_output = String::from_utf8(output.stdout)?;
            stdout_output
        }};
    }

    #[macro_export(local_inner_macros)]
    macro_rules! define_mock_valid_git_command {
        () => {
            struct MockValidGitCommand;

            impl crate::external::git_command::GitCommand for MockValidGitCommand {
                fn clone_flutter_sdk_by_channel(
                    &self,
                    _channel: &str,
                    destination: &str,
                ) -> anyhow::Result<()> {
                    std::fs::create_dir(destination).map_err(|e| anyhow::anyhow!(e))
                }

                fn clone_flutter_sdk_by_version(
                    &self,
                    _version: &str,
                    destination: &str,
                ) -> anyhow::Result<()> {
                    std::fs::create_dir(destination).map_err(|e| anyhow::anyhow!(e))
                }

                fn list_remote_sdks_by_tags(&self) -> anyhow::Result<String> {
                    read_resource_file("resources/test/install_service/git_lf-remote_tags.txt")
                        .map_err(|e| anyhow::anyhow!(e))
                }

                fn list_remote_sdks_by_branches(&self) -> anyhow::Result<String> {
                    read_resource_file("resources/test/install_service/git_lf-remote_heads.txt")
                        .map_err(|e| anyhow::anyhow!(e))
                }

                fn hard_reset_to_refs(
                    &self,
                    _working_dir: &str,
                    _refs: &str,
                ) -> anyhow::Result<()> {
                    // do nothing
                    anyhow::Ok(())
                }
            }

            fn read_resource_file(relative_path: &str) -> std::io::Result<String> {
                crate::util::path_like::PathLike::from(std::env!("CARGO_MANIFEST_DIR"))
                    .join(relative_path)
                    .read_to_string()
            }
        };
    }

    #[macro_export(local_inner_macros)]
    macro_rules! generate_test_config {
        ($root: ident, $dir: ident, $home: ident) => {
            FenvContext {
                debug: false,
                fenv_root: PathLike::from(&$root),
                fenv_dir: PathLike::from(&$dir),
                home: PathLike::from(&$home),
                default_shell: "bash".to_string(),
            }
        };

        ($root: ident, $dir: ident, $home: ident, $shell: literal) => {
            FenvContext {
                debug: false,
                fenv_root: PathLike::from(&$root),
                fenv_dir: PathLike::from(&$dir),
                home: PathLike::from(&$home),
                default_shell: $shell.to_string(),
            }
        };
    }

    pub fn test_with_config<F>(lambda: F)
    where
        F: Fn(&FenvContext),
    {
        let temp_fenv_root = tempfile::tempdir().unwrap();
        let temp_fenv_dir = tempfile::tempdir().unwrap();
        let temp_home = tempfile::tempdir().unwrap();
        let config = generate_test_config!(temp_fenv_root, temp_fenv_dir, temp_home);
        lambda(&config);
    }
}
