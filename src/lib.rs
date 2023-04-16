pub mod args;
pub mod config;
pub mod logger;
pub mod model;
pub mod service;

use std::{collections::HashMap, env};

use crate::{
    config::Config,
    service::{init_service::FenvInitService, install_service::FenvInstallService},
};
use anyhow::{Context as _, Ok, Result};
use clap::Parser;

pub fn try_run(args: &Vec<String>, env_vars: &HashMap<String, String>) -> Result<()> {
    let args = args::FenvArgs::parse_from(args);
    let config = Config::from(&args, &env_vars)?;
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
