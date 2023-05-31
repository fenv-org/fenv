use crate::{
    context::FenvContext, sdk_service::sdk_service::SdkService, service::service::Service,
    util::io::ConsoleOutput,
};

pub struct FenvRootService;

impl FenvRootService {
    pub fn new() -> Self {
        Self
    }
}

impl<OUT, ERR> Service<OUT, ERR> for FenvRootService
where
    OUT: std::io::Write,
    ERR: std::io::Write,
{
    fn execute(
        &self,
        context: &impl FenvContext,
        _: &impl SdkService,
        output: &mut dyn ConsoleOutput<OUT, ERR>,
    ) -> anyhow::Result<()> {
        writeln!(output.stdout(), "{}", context.fenv_root())?;
        anyhow::Ok(())
    }
}
