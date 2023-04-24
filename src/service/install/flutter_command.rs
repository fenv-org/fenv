use anyhow::{Context as _, Ok, Result};

use std::{env, path::PathBuf, process::Command};

use crate::spawn_and_wait;

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
        let mut command = Command::new("bash");
        spawn_and_wait!(
            command
                .current_dir(flutter_sdk_root)
                .env("PATH", flutter_sdk_root_merged_env_path(flutter_sdk_root)?)
                .arg("-c")
                .arg("bin/flutter doctor --suppress-analytics"),
            "doctor",
            "Failed to execute `flutter doctor` on `{flutter_sdk_root}/bin`",
        );
        Ok(())
    }

    fn precache(&self, flutter_sdk_root: &str) -> Result<()> {
        let mut command = Command::new("bash");
        spawn_and_wait!(
            command
                .current_dir(flutter_sdk_root)
                .env("PATH", flutter_sdk_root_merged_env_path(flutter_sdk_root)?)
                .arg("-c")
                .arg("bin/flutter precache"),
            "doctor",
            "Failed to execute `flutter precache` on `{flutter_sdk_root}/bin`",
        );
        Ok(())
    }
}

/// Generates a new PATH environment value by merging the given `flutter_sdk_root` with the `PATH` environment.
fn flutter_sdk_root_merged_env_path(flutter_sdk_root: &str) -> Result<String> {
    let env_path = &env::var("PATH").unwrap_or_default();
    let mut current_env_path = env::split_paths(env_path).collect::<Vec<_>>();
    current_env_path.insert(0, PathBuf::from(flutter_sdk_root).join("bin"));
    env::join_paths(&current_env_path)
        .map(|s| s.to_string_lossy().to_string())
        .map_err(|e| anyhow::anyhow!(e))
}
