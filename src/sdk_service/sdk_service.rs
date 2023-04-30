use super::{
    list_remote_sdk_cache::{RealRemoteSdkListCache, RemoteSdkListCache},
    local_repository::LocalSdkRepository,
    model::{local_flutter_sdk::LocalFlutterSdk, remote_flutter_sdk::RemoteFlutterSdk},
    remote_repository::RemoteSdkRepository,
};
use crate::{
    context::FenvContext, external::git_command::GitCommandImpl, util::path_like::PathLike,
};
use anyhow::Context;
use log::{debug, warn};

pub trait SdkService {
    fn get_installed_sdk_list(
        &self,
        context: &impl FenvContext,
    ) -> anyhow::Result<Vec<LocalFlutterSdk>>;

    fn get_available_remote_sdk_list(
        &self,
        context: &impl FenvContext,
    ) -> anyhow::Result<Vec<RemoteFlutterSdk>>;

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

    fn find_latest_local(
        &self,
        context: &impl FenvContext,
        prefix: &str,
    ) -> anyhow::Result<LocalFlutterSdk>;

    fn find_latest_remote(
        &self,
        context: &impl FenvContext,
        prefix: &str,
    ) -> anyhow::Result<RemoteFlutterSdk>;
}

pub struct ReadVersionFileResult {
    pub sdk: LocalFlutterSdk,
    pub installed: bool,
}

pub struct RealSdkService {
    git_command: GitCommandImpl,
    remote_sdk_list_cache: RealRemoteSdkListCache,
    local_repository: LocalSdkRepository,
    remote_repository: RemoteSdkRepository,
}

impl RealSdkService {
    pub fn new() -> Self {
        Self {
            git_command: GitCommandImpl::new(),
            remote_sdk_list_cache: RealRemoteSdkListCache::new(),
            local_repository: LocalSdkRepository::new(),
            remote_repository: RemoteSdkRepository::new(),
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

    fn get_available_remote_sdk_list(
        &self,
        context: &impl FenvContext,
    ) -> anyhow::Result<Vec<RemoteFlutterSdk>> {
        if let Some(sdks) = self.remote_sdk_list_cache.load_list(context) {
            debug!("sdk list from cache");
            return anyhow::Ok(sdks);
        }

        let result = self
            .remote_repository
            .fetch_available_sdk_list(&self.git_command);

        if let Ok(sdks) = &result {
            debug!("sdk list from remote");
            if let Err(e) = self.remote_sdk_list_cache.store_list(context, sdks) {
                warn!("{e}");
            }
        }

        result
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

    fn find_latest_local(
        &self,
        context: &impl FenvContext,
        prefix: &str,
    ) -> anyhow::Result<LocalFlutterSdk> {
        let sdk_or_none = self.local_repository.find_latest_local(context, prefix)?;
        match sdk_or_none {
            Some(sdk) => anyhow::Ok(sdk),
            None => Result::Err(anyhow::anyhow!(
                "Not found any matched flutter sdk version: `{prefix}`"
            )),
        }
    }

    fn find_latest_remote(
        &self,
        _context: &impl FenvContext,
        _prefix: &str,
    ) -> anyhow::Result<RemoteFlutterSdk> {
        todo!()
    }
}
