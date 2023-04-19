use anyhow::{Context as _, Ok, Result};

use std::process::Command;

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
                .arg("-c")
                .arg("bin/flutter precache"),
            "doctor",
            "Failed to execute `flutter precache` on `{flutter_sdk_root}/bin`",
        );
        Ok(())
    }
}
