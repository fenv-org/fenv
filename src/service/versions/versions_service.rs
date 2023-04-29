use crate::{
    context::FenvContext,
    model::{flutter_sdk::FlutterSdk, local_flutter_sdk::LocalFlutterSdk},
    service::{install::install_service::FenvInstallService, service::Service},
    util::path_like::PathLike,
};
use anyhow::{anyhow, bail, Context, Ok, Result};

pub struct FenvVersionsService {}

impl FenvVersionsService {
    pub fn new() -> FenvVersionsService {
        FenvVersionsService {}
    }

    pub fn list_installed_sdks<'a>(context: &impl FenvContext) -> Result<Vec<LocalFlutterSdk>> {
        list_installed_sdks(&context.fenv_versions())
    }

    pub fn is_installed_versions_or_channel<'a>(
        context: &impl FenvContext,
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
    fn execute(
        &self,
        context: &impl FenvContext,
        stdout: &mut impl std::io::Write,
    ) -> anyhow::Result<()> {
        let versions_directory = context.fenv_versions();
        if !versions_directory.is_dir() {
            if versions_directory.exists() {
                bail!("`{versions_directory}` exists but not a directory.")
            }
            versions_directory.create_dir_all().ok();
            if !versions_directory.is_dir() {
                panic!("`{versions_directory}` must exist now")
            }
        }

        let sdks = list_installed_sdks(&versions_directory)?;
        for sdk in sdks {
            writeln!(stdout, "{}", &sdk.display_name())?;
        }
        Ok(())
    }
}

fn list_installed_sdks(versions_directory: &PathLike) -> Result<Vec<LocalFlutterSdk>> {
    if !versions_directory.is_dir() {
        return Ok(Vec::new());
    }

    let entries = versions_directory
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
    use super::FenvVersionsService;
    use crate::{
        context::FenvContext,
        service::{macros::test_with_context, service::Service},
    };
    use indoc::formatdoc;
    use std::fs;

    #[test]
    fn test_sorted_order_of_list_installed_sdks() {
        test_with_context(|context| {
            // setup
            let fenv_versions = context.fenv_versions();
            fs::create_dir_all(&fenv_versions).unwrap();
            fs::create_dir(fenv_versions.join("10.231.5+hotfix.2")).unwrap();
            fs::create_dir(fenv_versions.join("1.0.0")).unwrap();
            fs::create_dir(fenv_versions.join("v2.23.40-hotfix.10")).unwrap();
            fs::create_dir(fenv_versions.join("v10.231.5")).unwrap();
            fs::create_dir(fenv_versions.join("stable")).unwrap();
            fs::create_dir(fenv_versions.join("beta")).unwrap();
            fs::create_dir(fenv_versions.join("dev")).unwrap();
            fs::create_dir(fenv_versions.join("master")).unwrap();

            // execution
            let mut stdout: Vec<u8> = Vec::new();
            FenvVersionsService::new()
                .execute(context, &mut stdout)
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
        });
    }

    #[test]
    fn test_filter_out_installing_markers() {
        test_with_context(|context| {
            // setup
            let fenv_versions = context.fenv_versions();
            fs::create_dir_all(&fenv_versions).unwrap();
            fs::create_dir(fenv_versions.join("1.0.0")).unwrap();
            fs::create_dir(fenv_versions.join("v2.23.40-hotfix.10")).unwrap();
            fs::create_dir(fenv_versions.join("v10.231.5")).unwrap();
            fs::create_dir(fenv_versions.join("10.231.5+hotfix.2")).unwrap();
            fs::create_dir(fenv_versions.join("dev")).unwrap();
            fs::create_dir(fenv_versions.join("beta")).unwrap();
            fs::create_dir(fenv_versions.join("master")).unwrap();
            fs::create_dir(fenv_versions.join("stable")).unwrap();

            fs::File::create(fenv_versions.join(".install_v10.231.5")).unwrap();
            fs::File::create(fenv_versions.join(".install_master")).unwrap();

            // execution
            let mut stdout: Vec<u8> = Vec::new();
            FenvVersionsService::new()
                .execute(context, &mut stdout)
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
        })
    }
}
