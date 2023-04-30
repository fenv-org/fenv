use crate::spawn_and_wait;
use anyhow::{Context as _, Ok, Result};
use std::{env, path::PathBuf, process::Command};

pub trait FlutterCommand {
    fn doctor(&self, flutter_sdk_root: &str) -> Result<()>;
    fn precache(&self, flutter_sdk_root: &str) -> Result<()>;
}

pub struct FlutterCommandImpl {}

impl FlutterCommandImpl {
    pub fn new() -> FlutterCommandImpl {
        FlutterCommandImpl {}
    }
}

impl FlutterCommand for FlutterCommandImpl {
    fn doctor(&self, flutter_sdk_root: &str) -> Result<()> {
        let flutter_bin_directory = [flutter_sdk_root, "bin"].join(std::path::MAIN_SEPARATOR_STR);
        let mut command = Command::new("flutter");
        spawn_and_wait!(
            command
                .current_dir(&flutter_bin_directory)
                .env(
                    "PATH",
                    flutter_sdk_root_merged_env_path(&flutter_bin_directory)?
                )
                .args(["doctor", "--suppress-analytics", "--verbose"]),
            "doctor",
            "Failed to execute `flutter doctor` on `{flutter_bin_directory}`",
        );
        Ok(())
    }

    fn precache(&self, flutter_sdk_root: &str) -> Result<()> {
        let flutter_bin_directory = [flutter_sdk_root, "bin"].join(std::path::MAIN_SEPARATOR_STR);
        let mut command = Command::new("flutter");
        spawn_and_wait!(
            command
                .current_dir(&flutter_bin_directory)
                .env(
                    "PATH",
                    flutter_sdk_root_merged_env_path(&flutter_bin_directory)?
                )
                .arg("precache"),
            "doctor",
            "Failed to execute `flutter precache` on `{flutter_bin_directory}`",
        );
        Ok(())
    }
}

/// Generates a new PATH environment value by merging the given `flutter_sdk_root` with the `PATH` environment.
fn flutter_sdk_root_merged_env_path(flutter_bin_directory: &str) -> Result<String> {
    let env_path = &env::var("PATH").unwrap_or_default();
    let mut current_env_path = env::split_paths(env_path).collect::<Vec<_>>();
    current_env_path.insert(0, PathBuf::from(flutter_bin_directory));
    env::join_paths(&current_env_path)
        .map(|s| s.to_string_lossy().to_string())
        .map_err(|e| anyhow::anyhow!(e))
}
