use std::{collections::HashMap, path::Path};

use anyhow::{bail, Context, Ok, Result};
use log::{debug, info};

use crate::{args::FenvArgs, util::path_like::PathLike};

/// The global configuration.
#[derive(Debug)]
pub struct FenvContext {
    /// `true` if `--debug` command line argument is provided.
    pub debug: bool,

    /// The location where `fenv` is installed.
    ///
    /// `$FENV_ROOT` if the environment variable is set,
    /// otherwise, `$HOME/.fenv`.
    pub fenv_root: PathLike,

    /// The working directory of the current `fenv` process.
    ///
    /// `$FENV_DIR` if the environment variable is set,
    /// otherwise, `$PWD`.
    pub fenv_dir: PathLike,

    /// The home directory.
    ///
    /// Equivalent to `$HOME`.
    pub home: PathLike,

    /// The shell executable that `$SHELL` holds.
    pub default_shell: String,
}

impl FenvContext {
    pub fn new(
        debug: bool,
        fenv_root: &str,
        fenv_dir: &str,
        home: &str,
        default_shell: &str,
    ) -> Self {
        Self {
            debug,
            fenv_root: PathLike::from(fenv_root),
            fenv_dir: PathLike::from(fenv_dir),
            home: PathLike::from(home),
            default_shell: String::from(default_shell),
        }
    }
}

impl FenvContext {
    /// Creates a new [`Config`] from the given command line arguments `args` and
    /// the captured environment variables `env_vars`.
    pub fn from(args: &FenvArgs, env_map: &HashMap<String, String>) -> Result<FenvContext> {
        let home = find_in_env_vars(&env_map, "HOME")?;
        let fenv_root = match requires_directory(&env_map, "FENV_ROOT") {
            Result::Ok(fenv_root) => {
                info!("Config::from(): Found `$FENV_ROOT`: {}", fenv_root);
                fenv_root
            }
            Err(_) => {
                info!("Config::from(): Could not find `$FENV_ROOT`. Fallback to `$HOME/.fenv");
                String::from(format!("{home}/.fenv"))
            }
        };
        let fenv_dir = match requires_directory(&env_map, "FENV_DIR") {
            Result::Ok(fenv_dir) => {
                info!("Config::from(): Found `$FENV_DIR`: {}", fenv_dir);
                fenv_dir
            }
            Err(_) => {
                info!("Config::from(): Could not find `$FENV_DIR`. Fallback to `$PWD`");
                find_in_env_vars(&env_map, "PWD")?
            }
        };
        Ok(FenvContext::new(
            args.debug,
            &fenv_root,
            &fenv_dir,
            &home,
            &find_in_env_vars(&env_map, "SHELL")?,
        ))
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
        debug!(
            "requires_directory(): Found `${}` but the directory `{}` does not exists",
            env_key, env_value
        );
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
