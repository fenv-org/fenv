use super::{
    list_remote_sdk_cache::{RemoteSdkListCache, REMOTE_SDK_LIST_CACHE},
    local_repository::{LocalSdkRepository, LOCAL_SDK_REPOSITORY},
    model::{local_flutter_sdk::LocalFlutterSdk, remote_flutter_sdk::RemoteFlutterSdk},
    remote_repository::{RemoteSdkRepository, REMOTE_SDK_REPOSITORY},
};
use crate::{
    context::FenvContext,
    external::git_command::{GitCommand, GitCommandImpl},
    util::{
        chrono_wrapper::{Clock, SystemClock},
        path_like::PathLike,
    },
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

// #[derive(Clone)]
struct SdkServiceDependencies<G: GitCommand, C: Clock> {
    git_command: G,
    clock: C,
    local_sdk_repository: LocalSdkRepository,
    remote_sdk_repository: RemoteSdkRepository,
    remote_sdk_list_cache: RemoteSdkListCache,
}

impl<G, C> SdkServiceDependencies<G, C>
where
    G: GitCommand,
    C: Clock,
{
}

impl<G, C> SdkService for SdkServiceDependencies<G, C>
where
    G: GitCommand,
    C: Clock,
{
    fn get_installed_sdk_list(
        &self,
        context: &impl FenvContext,
    ) -> anyhow::Result<Vec<LocalFlutterSdk>> {
        self.local_sdk_repository.ensure_versions_exists(context)?;
        self.local_sdk_repository.get_installed_sdk_list(context)
    }

    fn get_available_remote_sdk_list(
        &self,
        context: &impl FenvContext,
    ) -> anyhow::Result<Vec<RemoteFlutterSdk>> {
        if let Some(sdks) = self.remote_sdk_list_cache.load_list(context, &self.clock) {
            debug!("sdk list from cache");
            return anyhow::Ok(sdks);
        }

        let result = self
            .remote_sdk_repository
            .fetch_available_sdk_list(&self.git_command);
        if let Ok(sdks) = &result {
            debug!("sdk list from remote");
            if let Err(e) = self
                .remote_sdk_list_cache
                .store_list(context, &self.clock, sdks)
            {
                warn!("{e}");
            }
        }
        result
    }

    fn is_installed(&self, context: &impl FenvContext, version_or_channel: &str) -> bool {
        self.local_sdk_repository
            .is_installed(context, version_or_channel)
    }

    fn find_nearest_version_file(
        &self,
        context: &impl FenvContext,
        start_dir: &PathLike,
    ) -> anyhow::Result<PathLike> {
        self.local_sdk_repository
            .find_nearest_local_version_file(start_dir)
            .or_else(|| self.local_sdk_repository.find_global_version_file(context))
            .context("Could not find any version file")
    }

    fn read_version_file(
        &self,
        context: &impl FenvContext,
        version_file: &PathLike,
    ) -> anyhow::Result<ReadVersionFileResult> {
        self.local_sdk_repository
            .read_version_file(context, version_file)
            .map(|(sdk, installed)| ReadVersionFileResult { sdk, installed })
    }

    fn find_global_version_file(&self, context: &impl FenvContext) -> anyhow::Result<PathLike> {
        self.local_sdk_repository
            .find_global_version_file(context)
            .context("Could not find the global version file")
    }

    fn find_latest_local(
        &self,
        context: &impl FenvContext,
        prefix: &str,
    ) -> anyhow::Result<LocalFlutterSdk> {
        let sdk_or_none = self
            .local_sdk_repository
            .find_latest_local(context, prefix)?;
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

pub struct RealSdkService<G: GitCommand, C: Clock> {
    dependencies: SdkServiceDependencies<G, C>,
}

impl RealSdkService<GitCommandImpl, SystemClock> {
    pub fn new() -> Self {
        Self {
            dependencies: SdkServiceDependencies {
                git_command: GitCommandImpl::new(),
                clock: SystemClock,
                local_sdk_repository: LOCAL_SDK_REPOSITORY,
                remote_sdk_repository: REMOTE_SDK_REPOSITORY,
                remote_sdk_list_cache: REMOTE_SDK_LIST_CACHE,
            },
        }
    }
}

impl<G, C> SdkService for RealSdkService<G, C>
where
    G: GitCommand,
    C: Clock,
{
    fn get_installed_sdk_list(
        &self,
        context: &impl FenvContext,
    ) -> anyhow::Result<Vec<LocalFlutterSdk>> {
        self.dependencies.get_installed_sdk_list(context)
    }

    fn get_available_remote_sdk_list(
        &self,
        context: &impl FenvContext,
    ) -> anyhow::Result<Vec<RemoteFlutterSdk>> {
        self.dependencies.get_available_remote_sdk_list(context)
    }

    fn is_installed(&self, context: &impl FenvContext, version_or_channel: &str) -> bool {
        self.dependencies.is_installed(context, version_or_channel)
    }

    fn find_nearest_version_file(
        &self,
        context: &impl FenvContext,
        start_dir: &PathLike,
    ) -> anyhow::Result<PathLike> {
        self.dependencies
            .find_nearest_version_file(context, start_dir)
    }

    fn read_version_file(
        &self,
        context: &impl FenvContext,
        version_file: &PathLike,
    ) -> anyhow::Result<ReadVersionFileResult> {
        self.dependencies.read_version_file(context, version_file)
    }

    fn find_global_version_file(&self, context: &impl FenvContext) -> anyhow::Result<PathLike> {
        self.dependencies.find_global_version_file(context)
    }

    fn find_latest_local(
        &self,
        context: &impl FenvContext,
        prefix: &str,
    ) -> anyhow::Result<LocalFlutterSdk> {
        self.dependencies.find_latest_local(context, prefix)
    }

    fn find_latest_remote(
        &self,
        context: &impl FenvContext,
        prefix: &str,
    ) -> anyhow::Result<RemoteFlutterSdk> {
        self.dependencies.find_latest_remote(context, prefix)
    }
}
