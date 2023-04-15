use std::{env, path::Path};

use anyhow::{bail, Context, Ok, Result};
use once_cell::sync::OnceCell;

use crate::args::FenvArgs;

#[derive(Debug)]
pub struct Config {
    pub debug: bool,
    pub fenv_root: String,
    pub fenv_dir: String,
}

pub(crate) static CONFIG: OnceCell<Config> = OnceCell::new();

impl Config {
    pub fn instance() -> &'static Config {
        CONFIG.get().expect("Config is not initialized")
    }

    pub fn from(args: &FenvArgs, mut env_args: env::Vars) -> Result<Config> {
        let fenv_root = match Config::ensure_directory(&mut env_args, "FENV_ROOT") {
            Result::Ok(fenv_root) => fenv_root,
            Err(_) => env::var("HOME").context("env.HOME is not set")?,
        };
        let fenv_dir = match env::var("FENV_DIR") {
            Result::Ok(fenv_dir) => {
                let path = Path::new(&fenv_dir);
                if !path.is_dir() {
                    bail!("env.FENV_DIR is set but is not a directory: {}", fenv_dir)
                }
                String::from(
                    path.canonicalize()
                        .with_context(|| format!("Failed to canonicalize `{}`", fenv_dir))?
                        .to_str()
                        .unwrap(),
                )
            }
            Err(_) => env::var("PWD").context("env.PWD is not set")?,
        };
        Ok(Config {
            debug: args.debug,
            fenv_root,
            fenv_dir,
        })
    }

    fn ensure_directory(env_args: &mut env::Vars, env_key: &str) -> Result<String> {
        match env_args.find(|e| e.0 == env_key) {
            Some((key, value)) => {
                let path = Path::new(&value);
                if !path.is_dir() {
                    bail!("env.{} is set but is not a directory: {}", key, value)
                }
                Ok(String::from(
                    path.canonicalize()
                        .with_context(|| format!("Failed to canonicalize `{}`", value))?
                        .to_str()
                        .unwrap(),
                ))
            }
            None => bail!(format!("env.{} is not defined", env_key)),
        }
    }
}
