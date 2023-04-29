use std::fs::DirEntry;

use crate::{
    context::FenvContext, sdk_service::model::local_flutter_sdk::LocalFlutterSdk,
    util::path_like::PathLike,
};
use anyhow::Context as _;

pub struct LocalSdkRepository;

impl LocalSdkRepository {
    pub fn new() -> Self {
        Self
    }

    pub fn ensure_versions_exists(&self, context: &impl FenvContext) -> anyhow::Result<()> {
        let versions_directory = context.fenv_versions();
        versions_directory
            .create_dir_all()
            .with_context(|| format!("Could not create `{versions_directory}`"))
    }

    pub fn get_installed_sdk_list(
        &self,
        context: &impl FenvContext,
    ) -> anyhow::Result<Vec<LocalFlutterSdk>> {
        let versions_directory = context.fenv_versions();
        if !versions_directory.is_dir() {
            return anyhow::Ok(vec![]);
        }
        let mut sdks: Vec<LocalFlutterSdk> = list_all_sdks_in_directory(&versions_directory)?;
        sdks.sort();
        return anyhow::Ok(sdks);
    }

    pub fn is_installed(&self, context: &impl FenvContext, version_or_channel: &str) -> bool {
        let versions_directory = context.fenv_versions();
        let sdk_root = versions_directory.join(version_or_channel);
        if !sdk_root.is_dir() {
            return false;
        }
        let installing_marker = installing_marker_of(version_or_channel);
        return !sdk_root.join(&installing_marker).exists();
    }
}

fn list_all_sdks_in_directory(
    versions_directory: &PathLike,
) -> anyhow::Result<Vec<LocalFlutterSdk>> {
    let children = versions_directory
        .read_dir()
        .with_context(|| anyhow::anyhow!("Could not read `{versions_directory}`"))?;

    let sdks: Vec<LocalFlutterSdk> = children
        .flatten()
        .filter(|child| is_directory(child))
        .filter_map(|child| child.file_name().to_str().map(|s| s.to_owned()))
        .filter_map(|child_name| {
            let is_installation_incomplete = versions_directory
                .join(installing_marker_of(&child_name))
                .exists();
            if is_installation_incomplete {
                None
            } else {
                LocalFlutterSdk::parse(&child_name).ok()
            }
        })
        .collect();
    Ok(sdks)
}

fn is_directory(dir_entry: &DirEntry) -> bool {
    match &dir_entry.file_type() {
        Ok(file_type) => file_type.is_dir(),
        Err(_) => return false,
    }
}

fn installing_marker_of(version_or_channel: &str) -> String {
    format!(".install_{version_or_channel}")
}
