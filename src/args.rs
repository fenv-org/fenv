use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
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

#[derive(Debug, Subcommand)]
pub enum FenvSubcommands {
    /// Generate shell completion.
    Completions(FenvCompletionsArgs),

    /// Set the global Flutter version.
    /// The global version can be overridden by executing `fenv local`.
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

    /// Set the local Flutter version.
    Local(FenvLocalArgs),

    /// Uninstall an installed Flutter SDK.
    Uninstall(FenvUninstallArgs),

    /// Show the directory where the given flutter version is installed.
    Prefix(FenvPrefixArgs),

    /// Show the name and the version file of the currently selected Flutter SDK version.
    Version(FenvStartDirArgs),

    /// Show the file path of the nearest local version file or the global version file.
    VersionFile(FenvStartDirArgs),

    /// Show the name of the currently selected Flutter SDK version.
    VersionName(FenvStartDirArgs),

    /// List all installed Flutter SDKs.
    Versions,

    /// Show the absolute path of the given command that is available is the current directory.
    Which(FenvWhichArgs),

    /// Generates `.dart_tool/package_config.json` file and `.idea/libraries/Dart_SDK.xml` file
    /// with the current Flutter version for VS Code and IntelliJ workspace.
    Workspace(FenvWorkspaceArgs),
}

#[derive(Debug, clap::Args, Clone)]
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

#[derive(Debug, clap::Args, Clone)]
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

    /// If enabled, do not fail even if the specified sdk is already installed.
    /// If `--list` is given, will be ignored.
    /// By default, disabled.
    #[arg(name = "ignore-installed", long, action = clap::ArgAction::SetFalse)]
    pub fails_on_installed: bool,

    /// A prefix of a version or a channel to install, such as `3`, `3.7`, `3.7.0`, `stable`, `beta`.
    /// If omitted, attempts to install the version which is specified in the nearest `.flutter-version` file.
    /// Can be repeated.
    #[arg(action = clap::ArgAction::Append)]
    pub prefixes: Vec<String>,
}

#[derive(Debug, clap::Args, Clone)]
pub struct FenvListRemoteArgs {
    /// If set, do not mark installed Flutter SDK versions on the version list.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub bare: bool,
}

#[derive(Debug, clap::Args, Clone)]
pub struct FenvCompletionsArgs {
    /// Shell with auto-generated completion script available.
    #[arg(value_parser = ["bash", "zsh", "fish"])]
    pub shell: String,
}

#[derive(Debug, clap::Args, Clone)]
pub struct FenvGlobalArgs {
    /// A prefix of a specific version or a channel. For example, `3.7`, `3.0.0`, `stable`, `s` are valid.
    /// If omitted, shows the current global version.
    pub prefix: Option<String>,
}

#[derive(Debug, clap::Args, Clone)]
pub struct FenvStartDirArgs {
    /// If given, find the nearest version file in the given directory.
    /// Otherwise, find the nearest version file in the current directory.
    pub dir: Option<String>,
}

#[derive(Debug, clap::Args, Clone)]
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

    /// A prefix of a specific version or a channel. For example, `3.7`, `3.0.0`, `stable`, `s` are valid.
    pub prefix: String,
}

#[derive(Debug, clap::Args, Clone)]
pub struct FenvLocalArgs {
    /// A prefix of a specific version or a channel. For example, `3.7`, `3.0.0`, `stable`, `s` are valid.
    /// If omitted, shows the current global version.
    /// If given, `--symlink` is ignored.
    pub prefix: Option<String>,

    /// Re-create a symbolic link to the local Flutter SDK.
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    pub symlink: bool,
}

#[derive(Debug, clap::Args, Clone)]
pub struct FenvUninstallArgs {
    /// A prefix of a version or a channel to uninstall, such as `3`, `3.7`, `3.7.0`, `stable`, `beta`.
    /// Must be specified once or more.
    #[arg(action = clap::ArgAction::Append)]
    pub prefixes: Vec<String>,
}

#[derive(Debug, clap::Args, Clone)]
pub struct FenvPrefixArgs {
    /// A prefix of a specific version or a channel. For example, `3.7`, `3.0.0`, `stable`, `s` are valid.
    /// If omitted, uses the current version.
    pub prefix: Option<String>,
}

#[derive(Debug, clap::Args, Clone)]
pub struct FenvWhichArgs {
    /// The executable name to find where. For example, `flutter`, `dart`, `melos` etc.
    pub executable: String,
}

#[derive(Debug, clap::Args, Clone, PartialEq, Eq)]
pub struct FenvWorkspaceArgs {
    /// The path to the workspace directory, which is the root of the Flutter project.
    /// Must contains `pubspec.yaml` files.
    pub workspace: String,

    /// A prefix of a specific version or a channel. For example, `3.7`, `3.0.0`, `stable`, `s` are valid.
    /// If omitted, uses the nearest local version from the workspace directory or the global version.
    pub prefix: Option<String>,

    /// Executes `flutter pub get` to generate `.dart_tool/package_config.json` file.
    /// If set, the minimum `.dart_tool/package_config.json` file is generated. By default, disabled.
    #[arg(short = 'g', long = "pub-get", action = clap::ArgAction::SetTrue)]
    pub should_pub_get: bool,

    /// Re-generate `.dart_tool/package_config.json` and `.idea/libraries/Dart_SDK.xml`
    /// if they are not needed to re-generate. By default, disabled.
    #[arg(short = 'f', long = "force", action = clap::ArgAction::SetTrue)]
    pub force: bool,
}
