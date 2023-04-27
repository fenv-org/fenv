use std::path::PathBuf;

use anyhow::{anyhow, bail, Context, Ok, Result};

use crate::{
    context::FenvContext,
    model::{flutter_sdk::FlutterSdk, local_flutter_sdk::LocalFlutterSdk},
    service::{install::install_service::FenvInstallService, service::Service},
};

pub struct FenvVersionsService {}

impl FenvVersionsService {
    pub fn new() -> FenvVersionsService {
        FenvVersionsService {}
    }

    pub fn list_installed_sdks(context: &FenvContext) -> Result<Vec<LocalFlutterSdk>> {
        list_installed_sdks(&context.fenv_versions())
    }

    pub fn is_installed_versions_or_channel(
        context: &FenvContext,
        version_or_channel: &str,
    ) -> Result<bool> {
        let installed_sdks = FenvVersionsService::list_installed_sdks(context)?;
        let is_installed = installed_sdks
            .iter()
            .find(|sdk| &sdk.display_name() == version_or_channel)
            .is_some();
        Ok(is_installed)
    }
}

impl Service for FenvVersionsService {
    fn execute(&self, config: &FenvContext, stdout: &mut impl std::io::Write) -> Result<()> {
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

fn list_installed_sdks(versions_directory: &str) -> Result<Vec<LocalFlutterSdk>> {
    let versions_path = PathBuf::from(versions_directory);
    if !&versions_path.is_dir() {
        return Ok(Vec::new());
    }

    let entries = versions_path
        .read_dir()
        .with_context(|| anyhow!("Could not read `{versions_directory}`"))?;
    let mut sdks: Vec<LocalFlutterSdk> = entries
        .flatten()
        .filter_map(|dir_entry| {
            let file_name_in_os_string = dir_entry.file_name();
            let file_name = file_name_in_os_string.to_str().unwrap();
            if let Result::Ok(file_type) = &dir_entry.file_type() {
                if file_type.is_dir()
                    && !FenvInstallService::exists_installing_marker(versions_directory, file_name)
                {
                    return LocalFlutterSdk::parse(file_name).ok();
                }
            }
            None
        })
        .collect();
    sdks.sort();
    return Ok(sdks);
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use indoc::formatdoc;

    use crate::{context::FenvContext, service::service::Service, util::path_like::PathLike};

    use super::FenvVersionsService;

    #[test]
    fn test_sorted_order_of_list_installed_sdks() {
        // setup
        let temp_fenv_root = tempfile::tempdir().unwrap();
        let temp_fenv_dir = tempfile::tempdir().unwrap();
        let config = FenvContext {
            debug: false,
            fenv_root: PathLike::from(&temp_fenv_root),
            fenv_dir: PathLike::from(&temp_fenv_dir),
            home: PathLike::from("/home/user"),
            default_shell: "bash".to_string(),
        };

        let fenv_versions_path = PathBuf::from(&config.fenv_versions());
        fs::create_dir_all(&fenv_versions_path).unwrap();
        fs::create_dir(fenv_versions_path.join("10.231.5+hotfix.2")).unwrap();
        fs::create_dir(fenv_versions_path.join("1.0.0")).unwrap();
        fs::create_dir(fenv_versions_path.join("v2.23.40-hotfix.10")).unwrap();
        fs::create_dir(fenv_versions_path.join("v10.231.5")).unwrap();
        fs::create_dir(fenv_versions_path.join("stable")).unwrap();
        fs::create_dir(fenv_versions_path.join("beta")).unwrap();
        fs::create_dir(fenv_versions_path.join("dev")).unwrap();
        fs::create_dir(fenv_versions_path.join("master")).unwrap();

        // execution
        let mut stdout: Vec<u8> = Vec::new();
        FenvVersionsService::new()
            .execute(&config, &mut stdout)
            .unwrap();

        // validation
        assert_eq!(
            formatdoc! {
                "
                1.0.0
                v2.23.40-hotfix.10
                v10.231.5
                10.231.5+hotfix.2
                dev
                beta
                master
                stable
                "
            },
            String::from_utf8(stdout).unwrap()
        );
    }

    #[test]
    fn test_filter_out_installing_markers() {
        // setup
        let temp_fenv_root = tempfile::tempdir().unwrap();
        let temp_fenv_dir = tempfile::tempdir().unwrap();
        let config = FenvContext {
            debug: false,
            fenv_root: PathLike::from(&temp_fenv_root),
            fenv_dir: PathLike::from(&temp_fenv_dir),
            home: PathLike::from("/home/user"),
            default_shell: "bash".to_string(),
        };

        let fenv_versions_path = PathBuf::from(&config.fenv_versions());
        fs::create_dir_all(&fenv_versions_path).unwrap();
        fs::create_dir(fenv_versions_path.join("1.0.0")).unwrap();
        fs::create_dir(fenv_versions_path.join("v2.23.40-hotfix.10")).unwrap();
        fs::create_dir(fenv_versions_path.join("v10.231.5")).unwrap();
        fs::create_dir(fenv_versions_path.join("10.231.5+hotfix.2")).unwrap();
        fs::create_dir(fenv_versions_path.join("dev")).unwrap();
        fs::create_dir(fenv_versions_path.join("beta")).unwrap();
        fs::create_dir(fenv_versions_path.join("master")).unwrap();
        fs::create_dir(fenv_versions_path.join("stable")).unwrap();

        fs::File::create(fenv_versions_path.join(".install_v10.231.5")).unwrap();
        fs::File::create(fenv_versions_path.join(".install_master")).unwrap();

        // execution
        let mut stdout: Vec<u8> = Vec::new();
        FenvVersionsService::new()
            .execute(&config, &mut stdout)
            .unwrap();

        // validation
        assert_eq!(
            formatdoc! {
                "
                1.0.0
                v2.23.40-hotfix.10
                10.231.5+hotfix.2
                dev
                beta
                stable
                "
            },
            String::from_utf8(stdout).unwrap()
        );
    }
}
