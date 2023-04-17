pub mod args;
pub mod config;
pub mod model;
pub mod service;

use log::debug;
use std::collections::HashMap;

use crate::{
    config::Config,
    service::{init::init_service::FenvInitService, install::install_service::FenvInstallService},
};
use anyhow::{Context as _, Ok, Result};
use clap::Parser;

pub fn try_run(args: &Vec<String>, env_vars: &HashMap<String, String>) -> Result<()> {
    let args = args::FenvArgs::parse_from(args);
    let config = Config::from(&args, &env_vars)?;

    debug!("config = {config:?}");
    debug!("arguments = {args:?}");

    match &args.command {
        args::FenvSubcommands::Init(sub_args) => {
            let service = FenvInitService::from(sub_args.clone());
            service
                .execute(&config)
                .context("Failed to execute `fenv init`")?;
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
