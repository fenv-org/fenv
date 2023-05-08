use super::{
    local_repository::{LocalSdkRepository, LOCAL_SDK_REPOSITORY},
    model::{local_flutter_sdk::LocalFlutterSdk, remote_flutter_sdk::RemoteFlutterSdk},
    remote_repository::{RemoteSdkRepository, REMOTE_SDK_REPOSITORY},
    remote_sdk_list_cache::{RemoteSdkListCache, REMOTE_SDK_LIST_CACHE},
    results::{LookupResult, VersionFileReadResult},
    version_prefix_match::matches_prefix,
};
use crate::{
    context::FenvContext,
    external::{
        flutter_command::{FlutterCommand, FlutterCommandImpl},
        git_command::{GitCommand, GitCommandImpl},
    },
    sdk_service::model::flutter_sdk::FlutterSdk,
    unwrap_or_return,
    util::{
        chrono_wrapper::{Clock, SystemClock},
        path_like::PathLike,
    },
};
use log::{debug, info, warn};

pub trait SdkService {
    fn install_sdk(
        &self,
        context: &impl FenvContext,
        prefix: &str,
        should_doctor: bool,
        should_precache: bool,
        fails_on_installed: bool,
    ) -> anyhow::Result<()>;

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
    ) -> LookupResult<PathLike>;

    fn find_nearest_local_version_file(
        &self,
        context: &impl FenvContext,
        start_dir: &PathLike,
    ) -> LookupResult<PathLike>;

    fn find_global_version_file(&self, context: &impl FenvContext) -> LookupResult<PathLike>;

    fn read_version_file(
        &self,
        context: &impl FenvContext,
        version_file: &PathLike,
    ) -> anyhow::Result<VersionFileReadResult>;

    fn find_latest_local(
        &self,
        context: &impl FenvContext,
        prefix: &str,
    ) -> LookupResult<LocalFlutterSdk>;

    fn find_latest_remote(
        &self,
        context: &impl FenvContext,
        prefix: &str,
    ) -> LookupResult<RemoteFlutterSdk>;

    fn read_nearest_local_version(
        &self,
        start_dir: &PathLike,
    ) -> LookupResult<VersionFileReadResult>;

    fn write_local_version(
        &self,
        start_dir: &PathLike,
        sdk: &impl FlutterSdk,
    ) -> anyhow::Result<()>;

    fn read_global_version(
        &self,
        context: &impl FenvContext,
    ) -> LookupResult<VersionFileReadResult>;

    fn write_global_version(
        &self,
        context: &impl FenvContext,
        sdk: &impl FlutterSdk,
    ) -> anyhow::Result<()>;
}

struct SdkServiceInner<G: GitCommand, C: Clock, F: FlutterCommand> {
    git_command: G,
    flutter_command: F,
    clock: C,
    local_sdk_repository: LocalSdkRepository,
    remote_sdk_repository: RemoteSdkRepository,
    remote_sdk_list_cache: RemoteSdkListCache,
}

pub struct RealSdkService<G: GitCommand, C: Clock, F: FlutterCommand> {
    inner: SdkServiceInner<G, C, F>,
}

impl RealSdkService<GitCommandImpl, SystemClock, FlutterCommandImpl> {
    pub fn new() -> Self {
        Self {
            inner: SdkServiceInner {
                git_command: GitCommandImpl::new(),
                flutter_command: FlutterCommandImpl::new(),
                clock: SystemClock,
                local_sdk_repository: LOCAL_SDK_REPOSITORY,
                remote_sdk_repository: REMOTE_SDK_REPOSITORY,
                remote_sdk_list_cache: REMOTE_SDK_LIST_CACHE,
            },
        }
    }
}

impl<G, C, F> RealSdkService<G, C, F>
where
    G: GitCommand,
    C: Clock,
    F: FlutterCommand,
{
    pub fn from(git_command: G, clock: C, flutter_command: F) -> Self {
        Self {
            inner: SdkServiceInner {
                git_command,
                flutter_command,
                clock,
                local_sdk_repository: LOCAL_SDK_REPOSITORY,
                remote_sdk_repository: REMOTE_SDK_REPOSITORY,
                remote_sdk_list_cache: REMOTE_SDK_LIST_CACHE,
            },
        }
    }
}

impl<'a, G, C, F> RealSdkService<G, C, F>
where
    G: GitCommand,
    C: Clock,
    F: FlutterCommand,
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

    fn flutter_command(&'a self) -> &'a F {
        &self.inner.flutter_command
    }

    fn clock(&'a self) -> &'a C {
        &self.inner.clock
    }
}

impl<G, C, F> SdkService for RealSdkService<G, C, F>
where
    G: GitCommand,
    C: Clock,
    F: FlutterCommand,
{
    fn install_sdk(
        &self,
        context: &impl FenvContext,
        prefix: &str,
        should_doctor: bool,
        should_precache: bool,
        fails_on_installed: bool,
    ) -> anyhow::Result<()> {
        self.local().ensure_versions_exists(context)?;

        let local_latest_sdk_result = self.find_latest_local(context, prefix);
        match local_latest_sdk_result {
            LookupResult::Found(sdk) => {
                if fails_on_installed {
                    anyhow::bail!("`{}` is already installed", sdk.display_name())
                } else {
                    info!("`{}` is already installed", sdk.display_name());
                    return anyhow::Ok(());
                }
            }
            LookupResult::Err(e) => return Err(e),
            LookupResult::None => {}
        }

        let remote_latest_sdk: RemoteFlutterSdk = match self.find_latest_remote(context, prefix) {
            LookupResult::Found(remote_latest_sdk) => remote_latest_sdk,
            LookupResult::Err(e) => return Result::Err(e),
            LookupResult::None => {
                return Result::Err(anyhow::anyhow!(
                    "Not found any matched flutter sdk version: `{prefix}`"
                ))
            }
        };
        let version_or_channel = &remote_latest_sdk.display_name()[..];

        self.local()
            .remove_installation_garbages(context, version_or_channel)?;
        self.local()
            .create_installing_marker(context, version_or_channel)?;

        macro_rules! early_returns_on_err {
            ($result: expr) => {
                match $result {
                    Err(e) => {
                        self.local()
                            .remove_installation_garbages(context, version_or_channel)?;
                        return Err(e);
                    }
                    Ok(v) => v,
                }
            };
        }

        let sdk_dir = early_returns_on_err!(self.remote().install_sdk(
            context,
            self.git_command(),
            &remote_latest_sdk
        ));

        if should_doctor {
            early_returns_on_err!(self.flutter_command().doctor(&sdk_dir.to_string(),));
        }
        if should_precache {
            early_returns_on_err!(self.flutter_command().precache(&sdk_dir.to_string(),));
        }

        if let Err(e) = self
            .local()
            .remove_installing_marker(context, version_or_channel)
        {
            info!("install_sdk(): Failed to remove the installing marker: `{e}`");
        }
        anyhow::Ok(())
    }

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
    ) -> LookupResult<PathLike> {
        self.local()
            .find_nearest_local_version_file(start_dir)
            .or_else(|| self.local().find_global_version_file(context))
            .into()
    }

    fn read_version_file(
        &self,
        context: &impl FenvContext,
        version_file: &PathLike,
    ) -> anyhow::Result<VersionFileReadResult> {
        self.local()
            .read_version_file(context, version_file)
            .map(|(sdk, installed)| VersionFileReadResult { sdk, installed })
    }

    fn find_global_version_file(&self, context: &impl FenvContext) -> LookupResult<PathLike> {
        self.local().find_global_version_file(context).into()
    }

    fn find_latest_local(
        &self,
        context: &impl FenvContext,
        prefix: &str,
    ) -> LookupResult<LocalFlutterSdk> {
        let sdks: Vec<LocalFlutterSdk> = unwrap_or_return!(self.get_installed_sdk_list(context));
        let filtered_sdks = matches_prefix(&sdks, prefix);
        filtered_sdks.last().map(|sdk| sdk.to_owned()).into()
    }

    fn find_latest_remote(
        &self,
        context: &impl FenvContext,
        prefix: &str,
    ) -> LookupResult<RemoteFlutterSdk> {
        let sdks: Vec<RemoteFlutterSdk> =
            unwrap_or_return!(self.get_available_remote_sdk_list(context));
        let filtered_sdks = matches_prefix(&sdks, prefix);
        filtered_sdks.last().map(|sdk| sdk.to_owned()).into()
    }

    fn find_nearest_local_version_file(
        &self,
        context: &impl FenvContext,
        start_dir: &PathLike,
    ) -> LookupResult<PathLike> {
        todo!()
    }

    fn read_nearest_local_version(
        &self,
        start_dir: &PathLike,
    ) -> LookupResult<VersionFileReadResult> {
        todo!()
    }

    fn write_local_version(
        &self,
        start_dir: &PathLike,
        sdk: &impl FlutterSdk,
    ) -> anyhow::Result<()> {
        todo!()
    }

    fn read_global_version(
        &self,
        context: &impl FenvContext,
    ) -> LookupResult<VersionFileReadResult> {
        let global_version_file_or_none = self.local().find_global_version_file(context);
        let global_version_file = match global_version_file_or_none {
            None => return LookupResult::None,
            Some(global_version_file) => global_version_file,
        };
        match self
            .local()
            .read_version_file(context, &global_version_file)
        {
            Ok((sdk, installed)) => LookupResult::Found(VersionFileReadResult { sdk, installed }),
            Err(e) => LookupResult::Err(e),
        }
    }

    fn write_global_version(
        &self,
        context: &impl FenvContext,
        sdk: &impl FlutterSdk,
    ) -> anyhow::Result<()> {
        self.local()
            .write_version_file(&context.fenv_global_version_file(), sdk)
    }
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::{RealSdkService, SdkService};
    use crate::{context::FenvContext, service::macros::test_with_context};

    #[test]
    pub fn test_install_sdk_with_skipping_doctor_and_precache() {
        test_with_context(|context| {
            // setup
            context
                .fenv_versions()
                .join(".install_3.3.10")
                .create_file()
                .unwrap();
            let sdk_service = RealSdkService::new();

            // execution
            sdk_service
                .install_sdk(context, "3.3", false, false, true)
                .unwrap();

            // verification
            assert!(context.fenv_versions().join("3.3.10").exists());
            assert!(!context.fenv_versions().join(".install_3.3.10").exists());

            // validate the git commit hash
            let output = Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(context.fenv_versions().join("3.3.10"))
                .output()
                .unwrap();
            let output = String::from_utf8(output.stdout).unwrap();
            assert_eq!(output, "135454af32477f815a7525073027a3ff9eff1bfd\n");

            // validate the current branch is `stable`.
            let output = Command::new("git")
                .args(["rev-parse", "--abbrev-ref", "HEAD"])
                .current_dir(context.fenv_versions().join("3.3.10"))
                .output()
                .unwrap();
            let output = String::from_utf8(output.stdout).unwrap();
            assert_eq!(output, "stable\n");
        });
    }

    #[test]
    pub fn test_install_sdk_fails_if_already_installed() {
        test_with_context(|context| {
            // setup
            context
                .fenv_versions()
                .join("3.3.10")
                .create_dir_all()
                .unwrap();
            let sdk_service = RealSdkService::new();

            // execution
            let result = sdk_service.install_sdk(context, "3.3", false, false, true);

            // verification
            assert!(result.is_err());
            assert_eq!(
                "`3.3.10` is already installed",
                result.unwrap_err().to_string(),
            )
        });
    }

    #[test]
    pub fn test_install_sdk_succeeds_even_if_already_installed() {
        test_with_context(|context| {
            // setup
            context
                .fenv_versions()
                .join("3.3.10")
                .create_dir_all()
                .unwrap();
            let sdk_service = RealSdkService::new();

            // execution
            let result = sdk_service.install_sdk(context, "3.3", false, false, false);

            // verification
            assert!(result.is_ok());
        });
    }
}
