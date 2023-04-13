use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(name = "fenv", author, about, version)]
struct FenvArgs {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    debug: bool,

    #[command(subcommand)]
    command: FenvSubcommands,
}

#[derive(Debug, Subcommand)]
enum FenvSubcommands {
    /// A command that helps registering `fenv` to your `PATH` env. variable
    Init(FenvInitArgs),
}

#[derive(Debug, clap::Args)]
struct FenvInitArgs {
    /// Detects the current running shell
    #[arg(long = "detect-shell", action = clap::ArgAction::SetTrue)]
    detect_shell: bool,

    /// `-` shows shell instructions to add `fenv` to the `PATH`
    path_mode: Option<String>,
}

fn main() {
    let args = FenvArgs::parse();
    print!("{:?}", args);
}
