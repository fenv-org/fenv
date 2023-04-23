use anyhow::{bail, Context as _, Ok, Result};
use indoc::writedoc;
use lazy_static::lazy_static;
use nix::unistd::getppid;
use regex::Regex;
use std::{io::Write, path::PathBuf, process::Command};

use crate::{
    args::FenvInitArgs, context::FenvContext, debug, service::service::Service, spawn_and_capture,
};

pub struct FenvInitService {
    pub args: FenvInitArgs,
}

impl FenvInitService {
    pub fn from(args: FenvInitArgs) -> FenvInitService {
        FenvInitService { args }
    }

    fn execute_detect_shell(&self, context: &FenvContext, stdout: &mut impl Write) -> Result<()> {
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

    fn show_help(&self, config: &FenvContext, stdout: &mut impl Write) -> Result<()> {
        let shell = match &self.args.shell {
            Some(shell) => String::from(shell),
            None => detect_shell(config).context("Failed to detect the current shell")?,
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
                detect_profile(config, &shell, &mut profile, &mut profile_explain, &mut rc);

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

    fn print_path(&self, config: &FenvContext, stdout: &mut impl Write) -> Result<()> {
        let shell = match &self.args.shell {
            Some(shell) => String::from(shell),
            None => detect_shell(config).context("Failed to detect the current shell")?,
        };
        match &shell[..] {
            "fish" => writedoc!(
                stdout,
                r#"
                    while set fenv_index (contains -i -- "{fenv_root}/shims" $PATH)
                    set -eg PATH[$fenv_index]; end; set -e fenv_index
                    set -gx PATH '{fenv_root}/shims' $PATH
                "#,
                fenv_root = &config.fenv_root
            )?,
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
                fenv_root = &config.fenv_root
            )?,
        };
        Ok(())
    }
}

impl Service for FenvInitService {
    fn execute(&self, config: &FenvContext, stdout: &mut impl Write) -> Result<()> {
        if self.args.detect_shell {
            self.execute_detect_shell(config, stdout)
        } else if let None = self.args.path_mode {
            self.show_help(config, stdout)
        } else if let Some(_) = self.args.path_mode {
            self.print_path(config, stdout)
        } else {
            bail!("Cannot handle arguments: {}", self.args)
        }
    }
}

fn detect_shell(config: &FenvContext) -> Result<String> {
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
    let executable_path = extract_shell_executable(config, &ps_output.trim_end());
    debug!("detect_shell(): executable_path=`{executable_path}`");
    Ok(extract_shell_name_from_executable_path(&executable_path).unwrap_or(executable_path))
}

/// Tries to extract a shell executable from the given `ps_output`.
///
/// If failed, fallback the `$SHELL` environment variable.
fn extract_shell_executable(config: &FenvContext, ps_output: &str) -> String {
    lazy_static! {
        static ref EXECUTABLE_PATTERN: Regex = Regex::new(r"^\s*\-*(\S+)(?:\s.*)?\s*$").unwrap();
    }
    match EXECUTABLE_PATTERN.captures(ps_output) {
        Some(capture) => String::from(capture.get(1).map(|s| s.as_str()).unwrap()),
        None => String::from(&config.default_shell),
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

fn detect_profile(
    config: &FenvContext,
    shell: &str,
    profile: &mut String,
    profile_explain: &mut String,
    rc: &mut String,
) {
    match shell {
        "bash" => {
            let mut bash_profile_path = PathBuf::new();
            bash_profile_path.push(&config.home);
            bash_profile_path.push(".bash_profile");
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
