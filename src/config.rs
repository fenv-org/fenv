use std::{
    env,
    path::{Path, PathBuf},
};

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
            Err(_) => {
                let home = env::var("HOME").context("env.HOME is not set")?;
                let mut fenv_root_path = PathBuf::new();
                fenv_root_path.push(home);
                fenv_root_path.push(".fenv");
                String::from(fenv_root_path.to_str().unwrap())
            }
        };
        let fenv_dir = match Config::ensure_directory(&mut env_args, "FENV_DIR") {
            Result::Ok(fenv_dir) => fenv_dir,
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
