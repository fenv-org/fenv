use clap::{Parser, Subcommand};

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

    /// Set the global Flutter version.
    /// The global version can be overridden by setting a directory-specific version
    /// with `fenv local`.
    Global(FenvGlobalArgs),

    /// Help registering `fenv` to your `PATH` env. variable.
    Init(FenvInitArgs),

    /// Install an uninstalled Flutter SDK, and show the list of available Flutter SDK versions.
    Install(FenvInstallArgs),

    /// Print the latest installed or known version with the given prefix.
    Latest(FenvLatestArgs),

    /// List all installed Flutter SDKs. Alias of `versions` command.
    List,

    /// Show the list of the available Flutter SDK versions.
    /// Alias of `install --list` command.
    ListRemote(FenvListRemoteArgs),

    /// Show the file path of the nearest local version file or the global version file.
    VersionFile(FenvVersionFileArgs),

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

    /// A prefix of a version or a channel to install, such as `3`, `3.7`, `3.7.0`, `stable`, `beta`.
    /// If omitted, attempts to install the version which is specified in the nearest `.flutter-version` file.
    pub version_prefix: Option<String>,
}

#[derive(Debug, clap::Args, Clone, PartialEq, Eq)]
pub struct FenvListRemoteArgs {
    /// If set, do not mark installed Flutter SDK versions on the version list.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub bare: bool,
}

#[derive(Debug, clap::Args, Clone, PartialEq, Eq)]
pub struct FenvCompletionsArgs {
    /// Shell with auto-generated completion script available.
    #[arg(value_parser = ["bash", "zsh", "fish"])]
    pub shell: String,
}

#[derive(Debug, clap::Args, Clone, PartialEq, Eq)]
pub struct FenvGlobalArgs {
    /// A specific version of a channel. For example, e.g. `3.0.0`, `stable`
    /// If omitted, shows the current global version.
    pub version_or_channel: Option<String>,
}

#[derive(Debug, clap::Args, Clone, PartialEq, Eq)]
pub struct FenvVersionFileArgs {
    /// If given, find the nearest version file in the given directory.
    /// Otherwise, find the nearest version file in the current directory.
    pub dir: Option<String>,
}

#[derive(Debug, clap::Args, Clone, PartialEq, Eq)]
pub struct FenvLatestArgs {
    /// Select from all available versions regardless of whether they are installed.
    #[arg(short = 'r', long = "remote", action = clap::ArgAction::SetTrue)]
    pub from_remote: bool,

    /// Select from all available versions regardless of whether they are installed.
    /// `--known` is deprecated. Use `--remote` instead.
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    #[deprecated(note = "Use --remote instead")]
    pub known: bool,

    /// Do not print an error message on resolution failure.
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    pub quiet: bool,

    /// Prefix of any version or any channel.
    pub prefix: String,
}
