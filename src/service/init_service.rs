use anyhow::{bail, Context as _, Ok, Result};
use lazy_static::lazy_static;
use nix::unistd::getppid;
use regex::Regex;
use std::process::Command;

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
            println!("{:?}", detect_shell());
        }
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
    debug!("result: `{}`", &ps_output.trim_end());
    extract_shell_executable(&ps_output.trim_end())
}

/// Tries to extract a shell executable from the given `ps_output`.
///
/// If failed, fallback the `$SHELL` environment variable.
fn extract_shell_executable(ps_output: &str) -> Result<String> {
    lazy_static! {
        static ref EXECUTABLE_PATTERN: Regex = Regex::new(r"^\s*\-*(\S+)(?:\s.*)?\s*$").unwrap();
    }
    let executable = match EXECUTABLE_PATTERN.captures(ps_output) {
        Some(capture) => String::from(capture.get(1).map(|s| s.as_str()).unwrap()),
        None => String::from(&Config::instance().default_shell),
    };
    return Ok(executable);
}
