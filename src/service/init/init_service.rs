use crate::{
    args::FenvInitArgs,
    context::FenvContext,
    debug,
    sdk_service::sdk_service::SdkService,
    service::{completions::completions_service::FenvCompletionsService, service::Service},
    spawn_and_capture,
    util::io::ConsoleOutput,
};
use anyhow::{anyhow, Context as _, Ok, Result};
use clap::ValueEnum;
use clap_complete::Shell;
use indoc::writedoc;
use lazy_static::lazy_static;
use nix::unistd::getppid;
use regex::Regex;
use std::{io::Write, process::Command};

pub struct FenvInitService {
    pub args: FenvInitArgs,
}

impl FenvInitService {
    pub fn new(args: FenvInitArgs) -> FenvInitService {
        FenvInitService { args }
    }

    fn execute_detect_shell<'a>(
        &self,
        context: &impl FenvContext,
        stdout: &mut impl Write,
    ) -> Result<()> {
        let shell = detect_shell(context).context("Failed to detect the interactive shell")?;
        let mut profile = String::new();
        let mut profile_explain = String::new();
        let mut rc = String::new();
        detect_profile(context, &shell, &mut profile, &mut profile_explain, &mut rc);
        writeln!(stdout, "FENV_SHELL_DETECT={}", shell)?;
        writeln!(stdout, "FENV_PROFILE_DETECT={}", profile)?;
        writeln!(stdout, "FENV_RC_DETECT={}", rc)?;
        Ok(())
    }

    fn show_help<'a>(&self, context: &impl FenvContext, stdout: &mut impl Write) -> Result<()> {
        let shell = match &self.args.shell {
            Some(shell) => String::from(shell),
            None => detect_shell(context).context("Failed to detect the current shell")?,
        };

        match &shell[..] {
            "fish" => {
                writedoc!(
                    stdout,
                    "
                    # Add fenv executable to PATH by running
                    # the following interactively:

                    set -Ux FENV_ROOT $HOME/.fenv
                    fish_add_path $FENV_ROOT/bin

                    # Load fenv automatically by appending
                    # the following to ~/.config/fish/conf.d/fenv.fish:

                    fenv init - | source
                    "
                )?;
            }
            _ => {
                writedoc!(
                    stdout,
                    "
                    # Load fenv automatically by appending
                    # the following to "
                )?;

                let mut profile = String::new();
                let mut profile_explain = String::new();
                let mut rc = String::new();
                detect_profile(context, &shell, &mut profile, &mut profile_explain, &mut rc);

                if profile == rc {
                    writeln!(stdout, "{profile} :")?;
                    writeln!(stdout)?;
                } else if profile_explain.is_empty() {
                    writedoc!(
                        stdout,
                        "

                        # {profile} (for login shells)
                        # and {rc} (for interactive shells) :

                        "
                    )?;
                } else {
                    writedoc!(
                        stdout,
                        "

                        # {profile_explain} (for login shells)
                        # and {rc} (for interactive shells) :

                        "
                    )?;
                }
                writedoc!(
                    stdout,
                    "
                    export FENV_ROOT=\"$HOME/.fenv\"
                    command -v fenv >/dev/null || export PATH=\"$FENV_ROOT/bin:$PATH\"
                    eval \"$(fenv init -)\"

                    "
                )?;
            }
        }
        writedoc!(
            stdout,
            "
            # Restart your shell for the changes to take effect.

            exec $SHELL -l

            "
        )?;
        Ok(())
    }

    fn print_path<'a>(
        &self,
        context: &impl FenvContext,
        detected_shell: &str,
        stdout: &mut impl Write,
    ) -> Result<()> {
        match &detected_shell[..] {
            "fish" => writedoc!(
                stdout,
                r#"
                    while set fenv_index (contains -i -- "{fenv_root}/shims" $PATH)
                    set -eg PATH[$fenv_index]; end; set -e fenv_index
                    set -gx PATH '{fenv_root}/shims' $PATH
                "#,
                fenv_root = context.fenv_root().to_string(),
            ),
            _ => writedoc!(
                stdout,
                r#"
                    PATH="$(bash --norc -ec 'IFS=:; paths=($PATH);
                    for i in ${{!paths[@]}}; do
                    if [[ ${{paths[i]}} == "''{fenv_root}/shims''" ]]; then unset '\''paths[i]'\'';
                    fi; done;
                    echo "${{paths[*]}}"')"
                    export PATH="{fenv_root}/shims:${{PATH}}"
                "#,
                fenv_root = context.fenv_root().to_string(),
            ),
        }
        .map_err(|e| anyhow!(e))
    }

    fn print_completions(&self, detected_shell: &str, stdout: &mut impl Write) -> Result<()> {
        let shell = Shell::from_str(detected_shell, true).map_err(|e| anyhow::anyhow!(e))?;
        let completions_commands = FenvCompletionsService::completions_commands(&shell);
        write!(stdout, "{}", completions_commands).map_err(|e| anyhow::anyhow!(e))
    }
}

impl<OUT, ERR> Service<OUT, ERR> for FenvInitService
where
    OUT: std::io::Write,
    ERR: std::io::Write,
{
    fn execute(
        &self,
        context: &impl FenvContext,
        _: &impl SdkService,
        output: &mut dyn ConsoleOutput<OUT, ERR>,
    ) -> anyhow::Result<()> {
        if self.args.detect_shell {
            return self.execute_detect_shell(context, output.stdout());
        }

        match self.args.path_mode {
            Some(_) => {
                let shell = match &self.args.shell {
                    Some(shell) => String::from(shell),
                    None => detect_shell(context).context("Failed to detect the current shell")?,
                };
                self.print_path(context, &shell, output.stdout())?;
                match &shell[..] {
                    "fish" | "zsh" | "bash" => self.print_completions(&shell, output.stdout()),
                    _ => Ok(()),
                }
            }
            None => self.show_help(context, output.stdout()),
        }
    }
}

fn detect_shell<'a>(context: &impl FenvContext) -> Result<String> {
    // With `ps -o 'args='`,
    // captures the command line arguments which launched the shell.
    let ppid = getppid().as_raw();
    let mut command = Command::new("bash");
    let ps_output = spawn_and_capture!(
        command
            .arg("-c")
            .arg(format!("ps -p {ppid} -o 'args=' 2>/dev/null || true")),
        "detect_shell",
        "Failed to acquire the interactive shell name",
    );
    debug!("detect_shell(): stdout:\n`{ps_output}`");
    let executable_path = extract_shell_executable(context, &ps_output.trim_end());
    debug!("detect_shell(): executable_path=`{executable_path}`");
    Ok(extract_shell_name_from_executable_path(&executable_path).unwrap_or(executable_path))
}

/// Tries to extract a shell executable from the given `ps_output`.
///
/// If failed, fallback the `$SHELL` environment variable.
fn extract_shell_executable<'a>(context: &impl FenvContext, ps_output: &str) -> String {
    lazy_static! {
        static ref EXECUTABLE_PATTERN: Regex = Regex::new(r"^\s*\-*(\S+)(?:\s.*)?\s*$").unwrap();
    }
    match EXECUTABLE_PATTERN.captures(ps_output) {
        Some(capture) => String::from(capture.get(1).map(|s| s.as_str()).unwrap()),
        None => context.default_shell().to_owned(),
    }
}

fn extract_shell_name_from_executable_path(executable: &str) -> Option<String> {
    lazy_static! {
        static ref SHELL_NAME_PATTERN: Regex = Regex::new(r"^(?:.*/)([^/-]+)(?:-.*)?$").unwrap();
    }
    SHELL_NAME_PATTERN
        .captures(executable)
        .map(|capture| String::from(capture.get(1).map(|s| s.as_str()).unwrap()))
}

fn detect_profile<'a>(
    context: &impl FenvContext,
    shell: &str,
    profile: &mut String,
    profile_explain: &mut String,
    rc: &mut String,
) {
    match shell {
        "bash" => {
            let bash_profile_path = context.home().join(".bash_profile");
            if bash_profile_path.exists() {
                debug!("detect_profile(): bash: `~/.bash_profile` exists");
                profile.push_str("~/.bash_profile")
            } else {
                debug!("detect_profile(): bash: `~/.bash_profile` does not exist");
                profile.push_str("~/.profile");
            }
            profile_explain.push_str("~/.bash_profile if it exists, otherwise ~/.profile");
            rc.push_str("~/.bashrc");
        }
        "zsh" => {
            profile.push_str("~/.zprofile");
            rc.push_str("~/.zshrc");
        }
        "ksh" => {
            profile.push_str("~/.profile");
            rc.push_str("~/.profile");
        }
        _ => {}
    }
}
