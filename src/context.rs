use crate::util::path_like::PathLike;
use anyhow::{bail, Context, Ok, Result};
use log::{debug, info};
use std::{collections::HashMap, path::Path};

pub trait FenvContext: Clone {
    /// The home directory.
    ///
    /// Equivalent to `$HOME`.
    fn home(&self) -> PathLike;

    /// The shell executable that `$SHELL` holds.
    fn default_shell(&self) -> String;

    /// The location where `fenv` is installed.
    ///
    /// `$FENV_ROOT` if the environment variable is set,
    /// otherwise, `$HOME/.fenv`.
    fn fenv_root(&self) -> PathLike;

    /// The working directory of the current `fenv` process.
    ///
    /// `$FENV_DIR` if the environment variable is set,
    /// otherwise, `$PWD`.
    fn fenv_dir(&self) -> PathLike;

    /// The directory where `fenv` executable is located.
    ///
    /// `{fenv_root}/bin`
    fn fenv_bin(&self) -> PathLike {
        self.fenv_root().join("bin")
    }

    /// The directory where `flutter` and `dart` shell scripts are located.
    ///
    /// `{fenv_root}/shims`.
    fn fenv_shims(&self) -> PathLike {
        self.fenv_root().join("shims")
    }

    /// The directory where the downloaded Flutter SDKs are located.
    ///
    /// `{fenv_root}/versions`.
    fn fenv_versions(&self) -> PathLike {
        self.fenv_root().join("versions")
    }

    /// The directory where any miscellaneous cache files are located.
    ///
    /// `{fenv_root}/cache`.
    fn fenv_cache(&self) -> PathLike {
        self.fenv_root().join("cache")
    }

    /// The file where the global flutter version is recorded.
    ///
    /// `{fenv_root}/version`.
    fn fenv_global_version_file(&self) -> PathLike {
        self.fenv_root().join("version")
    }

    /// The directory where the given `version_or_channel` is installed.
    ///
    /// `{fenv_root}/versions/{version_or_channel}`.
    fn fenv_sdk_root(&self, version_or_channel: &str) -> PathLike {
        self.fenv_versions().join(version_or_channel)
    }
}

/// The real implementation of [`FenvContext`].
#[derive(Debug, Clone)]
pub struct RealFenvContext {
    home: PathLike,
    default_shell: String,
    fenv_root: PathLike,
    fenv_dir: PathLike,
}

impl RealFenvContext {
    pub fn new(fenv_root: &str, fenv_dir: &str, home: &str, default_shell: &str) -> Self {
        Self {
            fenv_root: PathLike::from(fenv_root),
            fenv_dir: PathLike::from(fenv_dir),
            home: PathLike::from(home),
            default_shell: String::from(default_shell),
        }
    }

    /// Creates a new [`Config`] from the given command line arguments `args` and
    /// the captured environment variables `env_vars`.
    pub fn from(env_map: &HashMap<String, String>) -> Result<Self> {
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
        Ok(Self::new(
            &fenv_root,
            &fenv_dir,
            &home,
            &find_in_env_vars(&env_map, "SHELL")?,
        ))
    }
}

impl FenvContext for RealFenvContext {
    fn home(&self) -> PathLike {
        self.home.clone()
    }

    fn default_shell(&self) -> String {
        self.default_shell.clone()
    }

    fn fenv_root(&self) -> PathLike {
        self.fenv_root.clone()
    }

    fn fenv_dir(&self) -> PathLike {
        self.fenv_dir.clone()
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
