use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, Context, Ok, Result};
use once_cell::sync::OnceCell;

use crate::args::FenvArgs;

/// The global configuration.
#[derive(Debug)]
pub struct Config {
    /// `true` if `--debug` command line argument is provided.
    pub debug: bool,

    /// The location where `fenv` is installed.
    ///
    /// `$FENV_ROOT` if the environment variable is set,
    /// otherwise, `$HOME/.fenv`.
    pub fenv_root: String,

    /// The working directory of the current `fenv` process.
    ///
    /// `$FENV_DIR` if the environment variable is set,
    /// otherwise, `$PWD`.
    pub fenv_dir: String,

    /// The shell executable that `$SHELL` holds.
    pub default_shell: String,
}

static _CONFIG: OnceCell<Config> = OnceCell::new();

impl Config {
    /// Returns the singleton instance.
    pub fn instance() -> &'static Config {
        _CONFIG.get().expect("Config is not initialized")
    }

    /// Sets the singleton instance.
    pub fn set_instance(config: Config) -> Result<()> {
        _CONFIG
            .set(config)
            .map_err(|_| anyhow!("Already initialized"))?;
        Ok(())
    }

    /// Creates a new [`Config`] from the given command line arguments `args` and
    /// the captured environment variables `env_vars`.
    pub fn from(args: &FenvArgs, mut env_vars: env::Vars) -> Result<Config> {
        let fenv_root = match requires_directory(&mut env_vars, "FENV_ROOT") {
            Result::Ok(fenv_root) => fenv_root,
            Err(_) => {
                let home = find_in_env_vars(&mut env_vars, "HOME")?;
                let mut fenv_root_path = PathBuf::new();
                fenv_root_path.push(home);
                fenv_root_path.push(".fenv");
                String::from(fenv_root_path.to_str().unwrap())
            }
        };
        let fenv_dir = match requires_directory(&mut env_vars, "FENV_DIR") {
            Result::Ok(fenv_dir) => fenv_dir,
            Err(_) => find_in_env_vars(&mut env_vars, "PWD")?,
        };
        Ok(Config {
            debug: args.debug,
            fenv_root,
            fenv_dir,
            default_shell: find_in_env_vars(&mut env_vars, "SHELL")?,
        })
    }
}

fn find_in_env_vars(env_vars: &mut env::Vars, lookup_target: &str) -> Result<String> {
    match env_vars.find(|(key, _)| key == lookup_target) {
        Some((_, value)) => Ok(value),
        None => bail!(format!("env.{} is not defined", lookup_target)),
    }
}

fn requires_directory(env_args: &mut env::Vars, env_key: &str) -> Result<String> {
    let env_value = find_in_env_vars(env_args, env_key)?;
    let path = Path::new(&env_value);
    if !path.is_dir() {
        bail!(
            "env.{} is set but no directory exists: `{}`",
            env_key,
            env_value
        )
    }
    Ok(String::from(
        path.canonicalize()
            .with_context(|| format!("Failed to canonicalize `{}`", env_value))?
            .to_str()
            .unwrap(),
    ))
}
