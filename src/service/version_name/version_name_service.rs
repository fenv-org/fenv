use crate::{
    args::FenvStartDirArgs, context::FenvContext, sdk_service::sdk_service::SdkService,
    service::service::Service, util::io::ConsoleOutput,
};

pub struct FenvVersionNameService {
    pub args: FenvStartDirArgs,
}

impl FenvVersionNameService {
    pub fn new(args: FenvStartDirArgs) -> Self {
        Self { args }
    }
}

impl<OUT, ERR> Service<OUT, ERR> for FenvVersionNameService
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
        todo!()
    }
}
