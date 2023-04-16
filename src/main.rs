pub mod args;
pub mod config;
pub mod logger;
pub mod model;
pub mod service;

use crate::{
    config::Config,
    service::{init_service::FenvInitService, install_service::FenvInstallService},
};
use anyhow::{Context as _, Ok, Result};
use clap::Parser;
// use config::Config;
use std::env;

fn main() {
    let args = args::FenvArgs::parse();

    if let Err(err) = try_main(&args) {
        if args.debug {
            eprintln!("{:?}", err);
        } else {
            eprintln!("fenv: {}", err);
            let error_chain = err.chain().skip(1);
            if error_chain.len() > 0 {
                eprintln!();
                eprintln!("caused by:");
                error_chain.for_each(|cause| eprintln!("    {}", cause));
            }
        }
        std::process::exit(1);
    }
}

fn try_main(args: &args::FenvArgs) -> Result<()> {
    let config = Config::from(&args, env::vars())?;
    Config::set_instance(config)?;

    debug!("config = {:?}", Config::instance());
    debug!("arguments = {:?}", args);

    if Config::instance().debug {
        env::set_var("RUST_BACKTRACE", "1")
    } else {
        env::remove_var("RUST_BACKTRACE")
    }
    match &args.command {
        args::FenvSubcommands::Init(sub_args) => {
            let service = FenvInitService::from(sub_args.clone());
            service.execute().context("Failed to execute `fenv init`")?;
        }
        args::FenvSubcommands::Install(sub_args) => {
            let service = FenvInstallService::from(sub_args.clone());
            service
                .execute()
                .context("Failed to execute `fenv install`")?;
        }
    };
    Ok(())
}
