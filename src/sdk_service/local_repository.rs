use super::model::flutter_sdk::FlutterSdk;
use crate::{
    context::FenvContext, sdk_service::model::local_flutter_sdk::LocalFlutterSdk,
    util::path_like::PathLike,
};
use anyhow::Context as _;
use indoc::formatdoc;
use log::{debug, info};
use std::fs::DirEntry;

pub struct LocalSdkRepository;

pub const LOCAL_SDK_REPOSITORY: LocalSdkRepository = LocalSdkRepository;

impl LocalSdkRepository {
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

    fn is_installed(&self, context: &impl FenvContext, version_or_channel: &str) -> bool {
        let sdk_root = context.fenv_sdk_root(version_or_channel);
        if !sdk_root.is_dir() {
            return false;
        }
        let installing_marker = installing_marker_of(version_or_channel);
        return !sdk_root.join(&installing_marker).exists();
    }

    pub fn version_file_of(&self, dir: &PathLike) -> PathLike {
        dir.join(".flutter-version")
    }

    pub fn find_nearest_local_version_file(&self, start_dir: &PathLike) -> Option<PathLike> {
        debug!("Looking up version file in `{start_dir}`");
        if self.version_file_of(start_dir).is_file() {
            debug!("Found version file in `{start_dir}`");
            return Some(self.version_file_of(start_dir));
        }

        let mut current = start_dir.parent();
        while let Some(dir) = &current {
            debug!("Looking up version file in `{dir}`");
            if self.version_file_of(&dir).is_file() {
                debug!("Found version file in `{dir}`");
                return Some(self.version_file_of(dir));
            }
            current = dir.parent();
        }
        None
    }

    pub fn find_global_version_file(&self, context: &impl FenvContext) -> Option<PathLike> {
        debug!("Looking up the global version file");
        let global_version_file = context.fenv_global_version_file();
        if global_version_file.is_file() {
            debug!("Found global version file");
            Some(global_version_file)
        } else {
            None
        }
    }

    /**
     * Reads the version file that the given `path` points to and return a tuple of
     * the `sdk` and `is_global_version_file`, and the path of the `sdk_root`.
     */
    pub fn read_version_file(
        &self,
        context: &impl FenvContext,
        path: &PathLike,
    ) -> anyhow::Result<(LocalFlutterSdk, bool, Option<PathLike>)> {
        let content = path.read_to_string().map(|s| s.trim().to_owned())?;
        let sdk = LocalFlutterSdk::parse(&content)?;
        let version_or_channel = sdk.display_name();
        let installed = self.is_installed(context, &version_or_channel);
        let is_global_version_file = path.path() == context.fenv_global_version_file().path();
        Ok((
            sdk,
            is_global_version_file,
            if installed {
                Some(context.fenv_sdk_root(&version_or_channel))
            } else {
                None
            },
        ))
    }

    pub fn write_version_file(&self, path: &PathLike, sdk: &impl FlutterSdk) -> anyhow::Result<()> {
        path.writeln(sdk.display_name()).with_context(|| {
            format!(
                "Failed to write `{}` to the version file: `{path}`",
                sdk.display_name()
            )
        })
    }

    pub fn remove_installation_garbages(
        &self,
        context: &impl FenvContext,
        version_or_channel: &str,
    ) -> anyhow::Result<()> {
        let versions_directory = context.fenv_versions();
        let install_destination = versions_directory.join(version_or_channel);
        let marker = versions_directory.join(installing_marker_of(version_or_channel));
        if marker.exists() {
            info!(
                "{}",
                formatdoc! {"
                install_sdk(): a previous trial to install `{versions_directory}` \
                ended unsuccessfully: remove the `{install_destination}`
                "},
            );
            install_destination.remove_dir_all()?;
            marker.remove_file()?;
        }
        anyhow::Ok(())
    }

    pub fn create_installing_marker(
        &self,
        context: &impl FenvContext,
        version_or_channel: &str,
    ) -> anyhow::Result<()> {
        let versions_directory = context.fenv_versions();
        let marker = versions_directory.join(installing_marker_of(version_or_channel));
        if !marker.exists() {
            marker
                .create_file()
                .map(|_| ())
                .with_context(|| format!("Failed to create an installing marker: `{marker}`"))
        } else {
            anyhow::Ok(())
        }
    }

    pub fn remove_installing_marker(
        &self,
        context: &impl FenvContext,
        version_or_channel: &str,
    ) -> anyhow::Result<()> {
        let versions_directory = context.fenv_versions();
        let marker = versions_directory.join(installing_marker_of(version_or_channel));
        marker
            .remove_file()
            .map(|_| ())
            .with_context(|| format!("Failed to create an installing marker: `{marker}`"))
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
