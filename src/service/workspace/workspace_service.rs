use crate::{
    args::FenvWorkspaceArgs, context::FenvContext, sdk_service::sdk_service::SdkService,
    service::service::Service, util::io::ConsoleOutput,
};

pub struct FenvWorkspaceService {
    pub args: FenvWorkspaceArgs,
}

impl FenvWorkspaceService {
    pub fn new(args: FenvWorkspaceArgs) -> Self {
        Self { args }
    }
}

impl<OUT, ERR> Service<OUT, ERR> for FenvWorkspaceService
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
        anyhow::Ok(())
    }
}
