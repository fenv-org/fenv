use super::{local_repository::LocalSdkRepository, model::local_flutter_sdk::LocalFlutterSdk};
use crate::{
    context::FenvContext,
    util::{chrono_wrapper::SystemClock, path_like::PathLike},
};
use anyhow::Context;

pub trait SdkService {
    fn get_installed_sdk_list(
        &self,
        context: &impl FenvContext,
    ) -> anyhow::Result<Vec<LocalFlutterSdk>>;

    fn is_installed(&self, context: &impl FenvContext, version_or_channel: &str) -> bool;

    fn find_nearest_version_file(
        &self,
        context: &impl FenvContext,
        start_dir: &PathLike,
    ) -> anyhow::Result<PathLike>;

    fn find_global_version_file(&self, context: &impl FenvContext) -> anyhow::Result<PathLike>;

    fn read_version_file(
        &self,
        context: &impl FenvContext,
        version_file: &PathLike,
    ) -> anyhow::Result<ReadVersionFileResult>;
}

pub struct ReadVersionFileResult {
    pub sdk: LocalFlutterSdk,
    pub installed: bool,
}

pub struct RealSdkService {
    _clock: SystemClock,
    local_repository: LocalSdkRepository,
}

impl RealSdkService {
    pub fn new() -> Self {
        Self {
            _clock: SystemClock::new(),
            local_repository: LocalSdkRepository::new(),
        }
    }
}

impl SdkService for RealSdkService {
    fn get_installed_sdk_list(
        &self,
        context: &impl FenvContext,
    ) -> anyhow::Result<Vec<LocalFlutterSdk>> {
        self.local_repository.ensure_versions_exists(context)?;
        self.local_repository.get_installed_sdk_list(context)
    }

    fn is_installed(&self, context: &impl FenvContext, version_or_channel: &str) -> bool {
        self.local_repository
            .is_installed(context, version_or_channel)
    }

    fn find_nearest_version_file(
        &self,
        context: &impl FenvContext,
        start_dir: &PathLike,
    ) -> anyhow::Result<PathLike> {
        self.local_repository
            .find_nearest_local_version_file(start_dir)
            .or_else(|| self.local_repository.find_global_version_file(context))
            .context("Could not find any version file")
    }

    fn read_version_file(
        &self,
        context: &impl FenvContext,
        version_file: &PathLike,
    ) -> anyhow::Result<ReadVersionFileResult> {
        self.local_repository
            .read_version_file(context, version_file)
            .map(|(sdk, installed)| ReadVersionFileResult { sdk, installed })
    }

    fn find_global_version_file(&self, context: &impl FenvContext) -> anyhow::Result<PathLike> {
        self.local_repository
            .find_global_version_file(context)
            .context("Could not find the global version file")
    }
}
