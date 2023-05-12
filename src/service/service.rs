use crate::{context::FenvContext, sdk_service::sdk_service::SdkService, util::io::ConsoleOutput};

pub trait Service<OUT, ERR>
where
    OUT: std::io::Write,
    ERR: std::io::Write,
{
    fn execute(
        &self,
        context: &impl FenvContext,
        sdk_service: &impl SdkService,
        output: &mut dyn ConsoleOutput<OUT, ERR>,
    ) -> anyhow::Result<()>;
}
