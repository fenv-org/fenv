use crate::{
    context::FenvContext, sdk_service::sdk_service::SdkService, service::service::Service,
    util::io::ConsoleOutput,
};

pub struct FenvVersionService;

impl FenvVersionService {
    pub fn new() -> Self {
        Self
    }
}

impl<OUT, ERR> Service<OUT, ERR> for FenvVersionService
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
