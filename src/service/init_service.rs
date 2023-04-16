use anyhow::{bail, Context as _, Ok, Result};
use lazy_static::lazy_static;
use nix::unistd::getppid;
use regex::Regex;
use std::{path::PathBuf, process::Command};

use crate::{args::FenvInitArgs, config::Config, debug};

pub struct FenvInitService {
    pub args: FenvInitArgs,
}

impl FenvInitService {
    pub fn from(args: FenvInitArgs) -> FenvInitService {
        FenvInitService { args }
    }

    pub fn execute(&self) -> Result<()> {
        if self.args.detect_shell {
            self.execute_detect_shell()
        } else {
            bail!("Cannot handle arguments: {}", self.args)
        }
    }

    fn execute_detect_shell(&self) -> Result<()> {
        let shell = detect_shell().context("Failed to detect the interactive shell")?;
        let mut profile = String::new();
        let mut profile_explain = String::new();
        let mut rc = String::new();
        detect_profile(&shell, &mut profile, &mut profile_explain, &mut rc);
        println!("FENV_SHELL_DETECT={}", shell);
        println!("FENV_PROFILE_DETECT={}", profile);
        println!("FENV_RC_DETECT={}", rc);
        Ok(())
    }
}

fn detect_shell() -> Result<String> {
    const ERROR_MESSAGE: &str = "Failed to acquire the interactive shell name";

    // With `ps -o 'args='`,
    // captures the command line arguments which launched the shell.
    let ppid = getppid().as_raw();
    let command_output = Command::new("bash")
        .arg("-c")
        .arg(format!("ps -p {} -o 'args=' 2>/dev/null || true", ppid))
        .output()
        .context(ERROR_MESSAGE)?;
    if !command_output.status.success() {
        bail!("{}: {}", ERROR_MESSAGE, command_output.status);
    }
    let ps_output = String::from_utf8(command_output.stdout)?;
    let executable_path = extract_shell_executable(&ps_output.trim_end());
    Ok(extract_shell_name_from_executable_path(&executable_path).unwrap_or(executable_path))
}

/// Tries to extract a shell executable from the given `ps_output`.
///
/// If failed, fallback the `$SHELL` environment variable.
fn extract_shell_executable(ps_output: &str) -> String {
    lazy_static! {
        static ref EXECUTABLE_PATTERN: Regex = Regex::new(r"^\s*\-*(\S+)(?:\s.*)?\s*$").unwrap();
    }
    match EXECUTABLE_PATTERN.captures(ps_output) {
        Some(capture) => String::from(capture.get(1).map(|s| s.as_str()).unwrap()),
        None => String::from(&Config::instance().default_shell),
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
    shell: &str,
    profile: &mut String,
    profile_explain: &mut String,
    rc: &mut String,
) {
    match shell {
        "bash" => {
            let mut bash_profile_path = PathBuf::new();
            bash_profile_path.push(&Config::instance().home);
            bash_profile_path.push(".bash_profile");
            if bash_profile_path.exists() {
                debug!("detect_profile(): bash: `~/.bash_profile` exists");
                profile.push_str(bash_profile_path.to_str().unwrap())
            } else {
                debug!("detect_profile(): bash: `~/.bash_profile` does not exist");
                profile.push_str(&Config::instance().home);
                profile.push_str("/.profile");
            }
            profile_explain.push_str("~/.bash_profile if it exists, otherwise ~/.profile");
            rc.push_str(&Config::instance().home);
            rc.push_str("/.bashrc");
        }
        "zsh" => {
            profile.push_str(&Config::instance().home);
            profile.push_str("/.zprofile");
            rc.push_str(&Config::instance().home);
            rc.push_str("/.zshrc");
        }
        "ksh" => {
            profile.push_str(&Config::instance().home);
            profile.push_str("/.profile");
            rc.push_str(&Config::instance().home);
            rc.push_str("/.profile");
        }
        _ => {}
    }
}
