use std::fmt::Display;

use clap::{Parser, Subcommand};
use clap_complete::Shell;

#[derive(Debug, Parser, PartialEq, Eq)]
#[clap(name = "fenv", author("<fenv@jerry.company>"), about, version)]
pub struct FenvArgs {
    /// Turn on all debug logging.
    #[arg(long, global = true, action = clap::ArgAction::SetTrue)]
    pub debug: bool,

    /// Turn on all info logging.
    /// If `--info` is set, we regard `--debug` also enabled.
    #[arg(long, global = true, action = clap::ArgAction::SetTrue)]
    pub info: bool,

    #[command(subcommand)]
    pub command: FenvSubcommands,
}

#[derive(Debug, Subcommand, PartialEq, Eq)]
pub enum FenvSubcommands {
    /// Generate shell completion.
    Completions(FenvCompletionsArgs),

    /// Help registering `fenv` to your `PATH` env. variable.
    Init(FenvInitArgs),

    /// Install an uninstalled Flutter SDK, and show the list of available Flutter SDK versions.
    Install(FenvInstallArgs),

    /// List all installed Flutter SDKs.
    Versions,
}

#[derive(Debug, clap::Args, Clone, PartialEq, Eq)]
pub struct FenvInitArgs {
    /// Detects the current running shell.
    #[arg(long = "detect-shell", action = clap::ArgAction::SetTrue)]
    pub detect_shell: bool,

    /// Specifies the shell type instead of detecting the running interactive shell.
    #[arg(short, long, value_parser = ["bash", "zsh", "fish", "ksh"])]
    pub shell: Option<String>,

    /// `-` shows shell instructions to add `fenv` to the `PATH`.
    #[arg(value_parser = ["-"])]
    pub path_mode: Option<String>,
}

impl Display for FenvInitArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer = String::from("init");
        if self.detect_shell {
            buffer.push_str(" --detect-shell");
        }
        if let Some(shell) = &self.shell {
            buffer.push_str(" --shell ");
            buffer.push_str(shell);
        }
        if let Some(_) = &self.path_mode {
            buffer.push_str(" -");
        }
        write!(f, "{}", buffer)
    }
}

#[derive(Debug, clap::Args, Clone, PartialEq, Eq)]
pub struct FenvInstallArgs {
    /// Show the all available Flutter SDK versions.
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    pub list: bool,

    /// If enabled, do not mark installed Flutter SDK versions on the version list.
    /// If `--list` is not given, will be ignored.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub bare: bool,

    /// If enabled, do not execute `flutter precache` command after downloading Flutter SDK.
    /// If `--list` is given, will be ignored.
    /// By default, disabled.
    #[arg(name = "no-precache", long, action = clap::ArgAction::SetFalse)]
    pub should_precache: bool,

    /// Flutter SDK's version to be installed. If omitted, attempts to install the version
    /// which is specified in `.flutter-version`.
    pub version: Option<String>,
}

impl Display for FenvInstallArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer = String::from("install");
        if self.list {
            buffer.push_str(" --list");
        }
        if self.bare {
            buffer.push_str(" --bare");
        }
        if !self.should_precache {
            buffer.push_str(" --no-precache");
        }
        if let Some(version) = &self.version {
            buffer.push_str(&format!(" {}", version));
        }
        write!(f, "{}", buffer)
    }
}

#[derive(Debug, clap::Args, Clone, PartialEq, Eq)]
pub struct FenvCompletionsArgs {
    #[arg(value_enum)]
    pub shell: Shell,
}
