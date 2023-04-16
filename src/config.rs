use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Ok, Result};
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

    /// The home directory.
    ///
    /// Equivalent to `$HOME`.
    pub home: String,
}

static _CONFIG: OnceCell<Config> = OnceCell::new();

impl Config {
    // /// Returns the singleton instance.
    // pub fn instance() -> &'static Config {
    //     _CONFIG.get().expect("Config is not initialized")
    // }

    // /// Sets the singleton instance.
    // pub fn set_instance(config: Config) -> Result<()> {
    //     _CONFIG
    //         .set(config)
    //         .map_err(|_| anyhow!("Already initialized"))?;
    //     Ok(())
    // }

    /// Creates a new [`Config`] from the given command line arguments `args` and
    /// the captured environment variables `env_vars`.
    pub fn from(args: &FenvArgs, env_map: &HashMap<String, String>) -> Result<Config> {
        let home = find_in_env_vars(&env_map, "HOME")?;
        let fenv_root = match requires_directory(&env_map, "FENV_ROOT") {
            Result::Ok(fenv_root) => fenv_root,
            Err(_) => {
                let mut fenv_root_path = PathBuf::new();
                fenv_root_path.push(&home);
                fenv_root_path.push(".fenv");
                String::from(fenv_root_path.to_str().unwrap())
            }
        };
        let fenv_dir = match requires_directory(&env_map, "FENV_DIR") {
            Result::Ok(fenv_dir) => fenv_dir,
            Err(_) => find_in_env_vars(&env_map, "PWD")?,
        };
        Ok(Config {
            debug: args.debug,
            fenv_root,
            fenv_dir,
            default_shell: find_in_env_vars(&env_map, "SHELL")?,
            home,
        })
    }

    /// The directory where `fenv` executable is located.
    ///
    /// `{fenv_root}/bin`
    pub fn fenv_bin(&self) -> String {
        format!("{}/bin", &self.fenv_root)
    }

    /// The directory where `flutter` and `dart` shell scripts are located.
    ///
    /// `{fenv_root}/shims`.
    pub fn fenv_shims(&self) -> String {
        format!("{}/shims", &self.fenv_root)
    }

    /// The directory where the downloaded Flutter SDKs are located.
    ///
    /// `{fenv_root}/versions`.
    pub fn fenv_versions(&self) -> String {
        format!("{}/versions", &self.fenv_root)
    }

    /// The directory where any miscellaneous cache files are located.
    ///
    /// `{fenv_root}/cache`.
    pub fn fenv_cache(&self) -> String {
        format!("{}/cache", &self.fenv_root)
    }
}

fn find_in_env_vars(env_map: &HashMap<String, String>, lookup_target: &str) -> Result<String> {
    match env_map.get(lookup_target) {
        Some(value) => Ok(String::from(value)),
        None => bail!(format!("env.{} is not defined", lookup_target)),
    }
}

fn requires_directory(env_map: &HashMap<String, String>, env_key: &str) -> Result<String> {
    let env_value = find_in_env_vars(env_map, env_key)?;
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
