pub mod args;
pub mod service;

use crate::service::install_service::FenvInstallService;
use anyhow::{Ok, Result};
use clap::Parser;
use std::env;

fn main() -> Result<()> {
  let args = args::FenvArgs::parse();
  if args.debug {
    println!("{:?}", args);
  }

  if args.debug {
    env::set_var("RUST_BACKTRACE", "1")
  } else {
    env::remove_var("RUST_BACKTRACE")
  }
  match args.command {
    args::FenvSubcommands::Init(_) => (),
    args::FenvSubcommands::Install(sub_args) => {
      let service = FenvInstallService::from(sub_args);
      service.execute()?;
    }
  };
  Ok(())
}
