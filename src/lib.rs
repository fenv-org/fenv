pub mod args;
pub mod config;
pub mod model;
pub mod service;

use log::debug;
use std::collections::HashMap;

use crate::config::Config;
use anyhow::{Ok, Result};
use clap::Parser;

pub fn try_run(args: &Vec<String>, env_vars: &HashMap<String, String>) -> Result<()> {
    let args = args::FenvArgs::parse_from(args);
    let config = Config::from(&args, &env_vars)?;

    debug!("config = {config:?}");
    debug!("arguments = {args:?}");

    let _ = &args.command.create_service().execute(&config)?;
    Ok(())
}
