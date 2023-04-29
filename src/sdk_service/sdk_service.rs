use super::{local_repository::LocalSdkRepository, model::local_flutter_sdk::LocalFlutterSdk};
use crate::{context::FenvContext, util::chrono_wrapper::SystemClock};

pub trait SdkService {
    fn get_installed_sdk_list(
        &self,
        context: &impl FenvContext,
    ) -> anyhow::Result<Vec<LocalFlutterSdk>>;

    fn is_installed(&self, context: &impl FenvContext, version_or_channel: &str) -> bool;
}

pub struct RealSdkService {
    clock: SystemClock,
    local_repository: LocalSdkRepository,
}

impl RealSdkService {
    pub fn new() -> Self {
        Self {
            clock: SystemClock::new(),
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
}
