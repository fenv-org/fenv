use crate::util::path_like::PathLike;
use anyhow::{bail, Ok, Result};
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

    /// `$PUB_CACHE` if the environment variable is set. Otherwise, `$HOME/.pub-cache`.
    fn pub_cache(&self) -> PathLike;

    /// The operating system that the current `fenv` process is running on.
    fn os(&self) -> OperatingSystem;

    /// The architecture that the current `fenv` process is running on.
    fn arch(&self) -> Architecture;

    /// The directory where temporary files should be created.
    fn temp_dir(&self) -> PathLike;
}

/// The operating system types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperatingSystem {
    MacOS,
    Linux,
}

/// The architecture types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Architecture {
    X86_64,
    Aarch64,
}

/// The real implementation of [`FenvContext`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealFenvContext {
    home: PathLike,
    default_shell: String,
    fenv_root: PathLike,
    fenv_dir: PathLike,
    pub_cache: PathLike,
    os: OperatingSystem,
    arch: Architecture,
}

impl RealFenvContext {
    pub fn new(
        fenv_root: &str,
        fenv_dir: &str,
        home: &str,
        default_shell: &str,
        pub_cache: &str,
        running_os: OperatingSystem,
        running_arch: Architecture,
    ) -> Self {
        Self {
            fenv_root: PathLike::from(fenv_root),
            fenv_dir: PathLike::from(fenv_dir),
            home: PathLike::from(home),
            default_shell: String::from(default_shell),
            pub_cache: PathLike::from(pub_cache),
            os: running_os,
            arch: running_arch,
        }
    }

    /// Creates a new [`Config`] from the given command line arguments `args` and
    /// the captured environment variables `env_vars`.
    pub fn from(env_map: &HashMap<String, String>, os: &str, arch: &str) -> Result<Self> {
        let home = find_in_env_vars(&env_map, "HOME")?;
        let fenv_root = match requires_directory(&env_map, "FENV_ROOT") {
            Result::Ok(fenv_root) => {
                info!("Config::from(): Found `$FENV_ROOT`: {}", fenv_root);
                fenv_root
            }
            Err(_) => {
                info!("Config::from(): Could not find `$FENV_ROOT`. Fallback to `$HOME/.fenv");
                PathLike::from(home.as_str()).join(".fenv").to_string()
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
        let pub_cache = if let Some(pub_cache) = env_map.get("PUB_CACHE") {
            info!("Config::from(): Found `$PUB_CACHE`: {}", fenv_dir);
            pub_cache.to_owned()
        } else {
            info!("Config::from(): Could not find `$PUB_CACHE`. Fallback to `$HOME/.pub-cache`");
            PathLike::from(home.as_str()).join(".pub-cache").to_string()
        };
        let os = match os {
            "macos" => OperatingSystem::MacOS,
            "linux" => OperatingSystem::Linux,
            _ => bail!("Unsupported OS: {}", os),
        };
        let arch = match arch {
            "x86_64" => Architecture::X86_64,
            "aarch64" => Architecture::Aarch64,
            _ => bail!("Unsupported architecture: {}", arch),
        };

        Ok(Self::new(
            &fenv_root,
            &fenv_dir,
            &home,
            &find_in_env_vars(&env_map, "SHELL")?,
            &pub_cache,
            os,
            arch,
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

    fn pub_cache(&self) -> PathLike {
        self.pub_cache.clone()
    }

    fn os(&self) -> OperatingSystem {
        self.os.clone()
    }

    fn arch(&self) -> Architecture {
        self.arch.clone()
    }

    fn temp_dir(&self) -> PathLike {
        self.fenv_root.join("temp")
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
    Ok(env_value)
}

#[cfg(test)]
mod tests {
    use super::{FenvContext, RealFenvContext};
    use crate::util::path_like::PathLike;
    use std::{collections::HashMap, env::consts};

    fn generate_env_map(vars: &[(&str, &str)]) -> HashMap<String, String> {
        vars.iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn test_ensure_essential_variables_are_set() {
        assert!(RealFenvContext::from(&generate_env_map(&[]), "linux", "x86_64").is_err());
        assert!(RealFenvContext::from(
            &generate_env_map(&[("HOME", "/home/user")]),
            "linux",
            "x86_64"
        )
        .is_err());
        assert!(RealFenvContext::from(
            &generate_env_map(&[("SHELL", "/bin/bash")]),
            "linux",
            "x86_64"
        )
        .is_err());
        assert!(RealFenvContext::from(
            &generate_env_map(&[("PWD", "/home/user")]),
            "linux",
            "x86_64"
        )
        .is_err());
        assert!(RealFenvContext::from(
            &generate_env_map(&[
                ("HOME", "/home/user"),
                ("SHELL", "/bin/bash"),
                ("PWD", "/home/user"),
            ]),
            "linux",
            "x86_64"
        )
        .is_ok());
    }

    #[test]
    fn test_from_if_all_variables_are_set_and_directories_exist() {
        // setup
        let temp_root = tempfile::tempdir().unwrap();
        let home = PathLike::from(temp_root.path());
        let fenv_root = home.join("temp/.fenv");
        let fenv_dir = home.join("pwd/here");
        let pub_cache = home.join(".temp/pub-cache");
        let pwd = home.join("pwd");
        let env_map = generate_env_map(&[
            ("HOME", home.to_string().as_str()),
            ("FENV_ROOT", fenv_root.to_string().as_str()),
            ("FENV_DIR", fenv_dir.to_string().as_str()),
            ("PUB_CACHE", pub_cache.to_string().as_str()),
            ("PWD", pwd.to_string().as_str()),
            ("SHELL", "/bin/bash"),
        ]);

        // create directories
        fenv_root.create_dir_all().unwrap();
        fenv_dir.create_dir_all().unwrap();

        // execution
        let context = RealFenvContext::from(&env_map, "linux", "x86_64").unwrap();

        // validation
        assert_eq!(
            context,
            RealFenvContext {
                home,
                default_shell: "/bin/bash".to_string(),
                fenv_root,
                fenv_dir,
                pub_cache,
                os: super::OperatingSystem::Linux,
                arch: super::Architecture::X86_64,
            }
        )
    }

    #[test]
    fn test_from_fails_with_all_variables_are_set_but_directories_do_not_exist() {
        // setup
        let env_map = generate_env_map(&[
            ("HOME", "/fake_home/user"),
            ("FENV_ROOT", "/fake_fenv_root"),
            ("FENV_DIR", "/fake_fenv_dir"),
            ("PUB_CACHE", "/fake_pub_cache"),
            ("PWD", "/fake_pwd"),
            ("SHELL", "/bin/bash"),
        ]);

        // execution
        let context = RealFenvContext::from(&env_map, "macos", "x86_64").unwrap();

        // validation
        assert_eq!(
            context,
            RealFenvContext {
                home: PathLike::from("/fake_home/user"),
                default_shell: "/bin/bash".to_string(),
                fenv_root: PathLike::from("/fake_home/user/.fenv"),
                fenv_dir: PathLike::from("/fake_pwd"),
                pub_cache: PathLike::from("/fake_pub_cache"),
                os: super::OperatingSystem::MacOS,
                arch: super::Architecture::X86_64,
            }
        )
    }

    #[test]
    fn test_from_running_os_is_intel_macos() {
        // setup
        let env_map = generate_env_map(&[
            ("HOME", "/fake_home/user"),
            ("FENV_ROOT", "/fake_fenv_root"),
            ("FENV_DIR", "/fake_fenv_dir"),
            ("PUB_CACHE", "/fake_pub_cache"),
            ("PWD", "/fake_pwd"),
            ("SHELL", "/bin/bash"),
        ]);

        // execution
        let context = RealFenvContext::from(&env_map, "macos", "x86_64").unwrap();

        // validation
        assert_eq!(
            context,
            RealFenvContext {
                home: PathLike::from("/fake_home/user"),
                default_shell: "/bin/bash".to_string(),
                fenv_root: PathLike::from("/fake_home/user/.fenv"),
                fenv_dir: PathLike::from("/fake_pwd"),
                pub_cache: PathLike::from("/fake_pub_cache"),
                os: super::OperatingSystem::MacOS,
                arch: super::Architecture::X86_64,
            }
        )
    }

    #[test]
    fn test_from_running_os_is_arm_macos() {
        // setup
        let env_map = generate_env_map(&[
            ("HOME", "/fake_home/user"),
            ("FENV_ROOT", "/fake_fenv_root"),
            ("FENV_DIR", "/fake_fenv_dir"),
            ("PUB_CACHE", "/fake_pub_cache"),
            ("PWD", "/fake_pwd"),
            ("SHELL", "/bin/bash"),
        ]);

        // execution
        let context = RealFenvContext::from(&env_map, "macos", "aarch64").unwrap();

        // validation
        assert_eq!(
            context,
            RealFenvContext {
                home: PathLike::from("/fake_home/user"),
                default_shell: "/bin/bash".to_string(),
                fenv_root: PathLike::from("/fake_home/user/.fenv"),
                fenv_dir: PathLike::from("/fake_pwd"),
                pub_cache: PathLike::from("/fake_pub_cache"),
                os: super::OperatingSystem::MacOS,
                arch: super::Architecture::Aarch64,
            }
        )
    }

    #[test]
    fn test_from_fails_if_macos_but_unsupported_architecture() {
        // setup
        let env_map = generate_env_map(&[
            ("HOME", "/fake_home/user"),
            ("FENV_ROOT", "/fake_fenv_root"),
            ("FENV_DIR", "/fake_fenv_dir"),
            ("PUB_CACHE", "/fake_pub_cache"),
            ("PWD", "/fake_pwd"),
            ("SHELL", "/bin/bash"),
        ]);

        // execution
        let result = RealFenvContext::from(&env_map, "macos", "arm64");

        // validation
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            "Unsupported architecture: arm64"
        )
    }

    #[test]
    fn test_from_fails_if_running_os_is_ios() {
        // setup
        let env_map = generate_env_map(&[
            ("HOME", "/fake_home/user"),
            ("FENV_ROOT", "/fake_fenv_root"),
            ("FENV_DIR", "/fake_fenv_dir"),
            ("PUB_CACHE", "/fake_pub_cache"),
            ("PWD", "/fake_pwd"),
            ("SHELL", "/bin/bash"),
        ]);

        // execution
        let result = RealFenvContext::from(&env_map, "ios", "aarch64");

        // validation
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().to_string(), "Unsupported OS: ios")
    }

    #[test]
    fn test_from_fails_if_running_os_is_android() {
        // setup
        let env_map = generate_env_map(&[
            ("HOME", "/fake_home/user"),
            ("FENV_ROOT", "/fake_fenv_root"),
            ("FENV_DIR", "/fake_fenv_dir"),
            ("PUB_CACHE", "/fake_pub_cache"),
            ("PWD", "/fake_pwd"),
            ("SHELL", "/bin/bash"),
        ]);

        // execution
        let result = RealFenvContext::from(&env_map, "android", "aarch64");

        // validation
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().to_string(), "Unsupported OS: android")
    }

    #[test]
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    fn test_linux_x86_64_environment() {
        // setup
        let env_map = generate_env_map(&[
            ("HOME", "/home/user"),
            ("SHELL", "/bin/bash"),
            ("PWD", "/home/user"),
        ]);

        // execution
        let context = RealFenvContext::from(&env_map, consts::OS, consts::ARCH).unwrap();

        // validation
        assert_eq!(context.os(), super::OperatingSystem::Linux);
        assert_eq!(context.arch(), super::Architecture::X86_64);
    }

    #[test]
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    fn test_linux_aarch64_environment() {
        // setup
        let env_map = generate_env_map(&[
            ("HOME", "/home/user"),
            ("SHELL", "/bin/bash"),
            ("PWD", "/home/user"),
        ]);

        // execution
        let context = RealFenvContext::from(&env_map, consts::OS, consts::ARCH).unwrap();

        // validation
        assert_eq!(context.os(), super::OperatingSystem::Linux);
        assert_eq!(context.arch(), super::Architecture::Aarch64);
    }

    #[test]
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    fn test_macos_x86_64_environment() {
        // setup
        let env_map = generate_env_map(&[
            ("HOME", "/Users/user"),
            ("SHELL", "/bin/zsh"),
            ("PWD", "/Users/user"),
        ]);

        // execution
        let context = RealFenvContext::from(&env_map, consts::OS, consts::ARCH).unwrap();

        // validation
        assert_eq!(context.os(), super::OperatingSystem::MacOS);
        assert_eq!(context.arch(), super::Architecture::X86_64);
    }

    #[test]
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    fn test_macos_aarch64_environment() {
        // setup
        let env_map = generate_env_map(&[
            ("HOME", "/Users/user"),
            ("SHELL", "/bin/zsh"),
            ("PWD", "/Users/user"),
        ]);

        // execution
        let context = RealFenvContext::from(&env_map, consts::OS, consts::ARCH).unwrap();

        // validation
        assert_eq!(context.os(), super::OperatingSystem::MacOS);
        assert_eq!(context.arch(), super::Architecture::Aarch64);
    }
}
