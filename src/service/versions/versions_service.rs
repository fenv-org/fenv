use std::path::PathBuf;

use anyhow::{anyhow, bail, Context, Ok, Result};

use crate::{
    model::flutter_sdk::FlutterSdk,
    service::{install::install_service::FenvInstallService, service::Service},
};

pub struct FenvVersionsService {}

impl FenvVersionsService {
    pub fn new() -> FenvVersionsService {
        FenvVersionsService {}
    }
}

impl Service for FenvVersionsService {
    fn execute(
        &self,
        config: &crate::config::Config,
        stdout: &mut impl std::io::Write,
    ) -> Result<()> {
        let path = PathBuf::from(&config.fenv_versions());
        if !path.is_dir() {
            if path.exists() {
                bail!("`{}` exists but not a directory.", path.to_str().unwrap())
            }
            std::fs::create_dir_all(&path).ok();
            if !path.is_dir() {
                panic!("`{}` must exist now", path.to_str().unwrap())
            }
        }

        let sdks = list_installed_sdks(&path.to_str().unwrap())?;
        for sdk in sdks {
            writeln!(stdout, "{}", &sdk.display_name())?;
        }
        Ok(())
    }
}

fn list_installed_sdks(versions_directory: &str) -> Result<Vec<FlutterSdk>> {
    let versions_path = PathBuf::from(versions_directory);
    let entries = versions_path
        .read_dir()
        .with_context(|| anyhow!("Could not read `{versions_directory}`"))?;
    let mut sdks: Vec<FlutterSdk> = entries
        .flatten()
        .filter_map(|dir_entry| {
            let file_name_in_os_string = dir_entry.file_name();
            let file_name = file_name_in_os_string.to_str().unwrap();
            if let Result::Ok(file_type) = &dir_entry.file_type() {
                if file_type.is_dir()
                    && !FenvInstallService::exists_installing_marker(versions_directory, file_name)
                {
                    return FlutterSdk::parse(file_name).ok();
                }
            }
            None
        })
        .collect();
    sdks.sort();
    return Ok(sdks);
}
