use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(name = "fenv", author, about, version)]
pub struct FenvArgs {
    /// Turn debugging information on.
    #[arg(short, long, global = true, action = clap::ArgAction::SetTrue)]
    debug: bool,

    #[command(subcommand)]
    command: FenvSubcommands,
}

#[derive(Debug, Subcommand)]
pub enum FenvSubcommands {
    /// Help registering `fenv` to your `PATH` env. variable.
    Init(FenvInitArgs),

    /// Install an uninstalled Flutter SDK and show the list of available Flutter SDK versions.
    Install(FenvInstallArgs),
}

#[derive(Debug, clap::Args)]
pub struct FenvInitArgs {
    /// Detects the current running shell.
    #[arg(long = "detect-shell", action = clap::ArgAction::SetTrue)]
    detect_shell: bool,

    /// `-` shows shell instructions to add `fenv` to the `PATH`.
    #[arg(value_parser = ["-"])]
    path_mode: Option<String>,
}

#[derive(Debug, clap::Args)]
pub struct FenvInstallArgs {
    /// Show the all available Flutter SDK versions.
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    list: bool,

    /// If enabled, do not mark installed Flutter SDK versions on the version list.
    /// If `--list` is not given, will be ignored.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    bare: bool,

    /// If enabled, do not execute `flutter precache` command after downloading Flutter SDK.
    /// If `--list` is given, will be ignored.
    /// By default, disabled.
    #[arg(name = "no-precache", long, action = clap::ArgAction::SetFalse)]
    should_precache: bool,

    /// Flutter SDK's version to be installed. If omitted, attempts to install the version
    /// which is specified in `.flutter-version`.
    version: Option<String>,
}
