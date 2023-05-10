use crate::{context::FenvContext, sdk_service::sdk_service::SdkService};

pub trait Service {
    fn execute(
        &self,
        context: &impl FenvContext,
        sdk_service: &impl SdkService,
        stdout: &mut impl std::io::Write,
    ) -> anyhow::Result<()>;
}
