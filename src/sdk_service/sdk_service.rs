use super::{
    local_repository::{LocalSdkRepository, LOCAL_SDK_REPOSITORY},
    model::{local_flutter_sdk::LocalFlutterSdk, remote_flutter_sdk::RemoteFlutterSdk},
    remote_repository::{RemoteSdkRepository, REMOTE_SDK_REPOSITORY},
    remote_sdk_list_cache::{RemoteSdkListCache, REMOTE_SDK_LIST_CACHE},
    version_prefix_match::matches_prefix,
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

struct SdkServiceInner<G: GitCommand, C: Clock> {
    git_command: G,
    clock: C,
    local_sdk_repository: LocalSdkRepository,
    remote_sdk_repository: RemoteSdkRepository,
    remote_sdk_list_cache: RemoteSdkListCache,
}

pub struct RealSdkService<G: GitCommand, C: Clock> {
    inner: SdkServiceInner<G, C>,
}

impl RealSdkService<GitCommandImpl, SystemClock> {
    pub fn new() -> Self {
        Self {
            inner: SdkServiceInner {
                git_command: GitCommandImpl::new(),
                clock: SystemClock,
                local_sdk_repository: LOCAL_SDK_REPOSITORY,
                remote_sdk_repository: REMOTE_SDK_REPOSITORY,
                remote_sdk_list_cache: REMOTE_SDK_LIST_CACHE,
            },
        }
    }
}

impl<G, C> RealSdkService<G, C>
where
    G: GitCommand,
    C: Clock,
{
    pub fn from(git_command: G, clock: C) -> Self {
        Self {
            inner: SdkServiceInner {
                git_command,
                clock,
                local_sdk_repository: LOCAL_SDK_REPOSITORY,
                remote_sdk_repository: REMOTE_SDK_REPOSITORY,
                remote_sdk_list_cache: REMOTE_SDK_LIST_CACHE,
            },
        }
    }
}

impl<'a, G, C> RealSdkService<G, C>
where
    G: GitCommand,
    C: Clock,
{
    fn local(&'a self) -> &'a LocalSdkRepository {
        &self.inner.local_sdk_repository
    }

    fn remote(&'a self) -> &'a RemoteSdkRepository {
        &self.inner.remote_sdk_repository
    }

    fn remote_list_cache(&'a self) -> &'a RemoteSdkListCache {
        &self.inner.remote_sdk_list_cache
    }

    fn git_command(&'a self) -> &'a G {
        &self.inner.git_command
    }

    fn clock(&'a self) -> &'a C {
        &self.inner.clock
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
        self.local().ensure_versions_exists(context)?;
        self.local().get_installed_sdk_list(context)
    }

    fn get_available_remote_sdk_list(
        &self,
        context: &impl FenvContext,
    ) -> anyhow::Result<Vec<RemoteFlutterSdk>> {
        if let Some(sdks) = self.remote_list_cache().load_list(context, self.clock()) {
            debug!("sdk list from cache");
            return anyhow::Ok(sdks);
        }

        let result = self.remote().fetch_available_sdk_list(self.git_command());
        if let Ok(sdks) = &result {
            debug!("sdk list from remote");
            if let Err(e) = self
                .remote_list_cache()
                .store_list(context, self.clock(), sdks)
            {
                warn!("{e}");
            }
        }
        result
    }

    fn is_installed(&self, context: &impl FenvContext, version_or_channel: &str) -> bool {
        self.local().is_installed(context, version_or_channel)
    }

    fn find_nearest_version_file(
        &self,
        context: &impl FenvContext,
        start_dir: &PathLike,
    ) -> anyhow::Result<PathLike> {
        self.local()
            .find_nearest_local_version_file(start_dir)
            .or_else(|| self.local().find_global_version_file(context))
            .context("Could not find any version file")
    }

    fn read_version_file(
        &self,
        context: &impl FenvContext,
        version_file: &PathLike,
    ) -> anyhow::Result<ReadVersionFileResult> {
        self.local()
            .read_version_file(context, version_file)
            .map(|(sdk, installed)| ReadVersionFileResult { sdk, installed })
    }

    fn find_global_version_file(&self, context: &impl FenvContext) -> anyhow::Result<PathLike> {
        self.local()
            .find_global_version_file(context)
            .context("Could not find the global version file")
    }

    fn find_latest_local(
        &self,
        context: &impl FenvContext,
        prefix: &str,
    ) -> anyhow::Result<LocalFlutterSdk> {
        let sdks: Vec<LocalFlutterSdk> = self.get_installed_sdk_list(context)?;
        let filtered_sdks = matches_prefix(&sdks, prefix);
        filtered_sdks
            .last()
            .map(|sdk| sdk.to_owned())
            .ok_or_else(|| anyhow::anyhow!("Not found any matched flutter sdk version: `{prefix}`"))
    }

    fn find_latest_remote(
        &self,
        context: &impl FenvContext,
        prefix: &str,
    ) -> anyhow::Result<RemoteFlutterSdk> {
        let sdks: Vec<RemoteFlutterSdk> = self.get_available_remote_sdk_list(context)?;
        let filtered_sdks = matches_prefix(&sdks, prefix);
        filtered_sdks
            .last()
            .map(|sdk| sdk.to_owned())
            .ok_or_else(|| anyhow::anyhow!("Not found any matched flutter sdk version: `{prefix}`"))
    }
}
