use crate::{
    args::FenvPrefixArgs, context::FenvContext, sdk_service::sdk_service::SdkService,
    service::service::Service, util::io::ConsoleOutput,
};

pub struct FenvPrefixService {
    pub args: FenvPrefixArgs,
}

impl FenvPrefixService {
    pub fn new(args: FenvPrefixArgs) -> Self {
        Self { args }
    }
}

impl<OUT, ERR> Service<OUT, ERR> for FenvPrefixService
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
        Ok(())
    }
}
