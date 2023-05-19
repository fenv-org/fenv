use crate::{
    args::FenvUninstallArgs, context::FenvContext, sdk_service::sdk_service::SdkService,
    service::service::Service, util::io::ConsoleOutput,
};

pub struct FenvUninstallService {
    pub args: FenvUninstallArgs,
}

impl FenvUninstallService {
    pub fn new(args: FenvUninstallArgs) -> Self {
        Self { args }
    }
}

impl<OUT, ERR> Service<OUT, ERR> for FenvUninstallService
where
    OUT: std::io::Write,
    ERR: std::io::Write,
{
    fn execute(
        &self,
        context: &impl FenvContext,
        sdk_service: &impl SdkService,
        output: &mut dyn ConsoleOutput<OUT, ERR>,
    ) -> anyhow::Result<()> {
        println!("{:?}", self.args);
        Ok(())
    }
}
