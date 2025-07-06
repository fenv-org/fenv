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

#[cfg(test)]
mod tests {
    use crate::{
        context::RealFenvContext, sdk_service::sdk_service::RealSdkService, try_run,
        util::io::BufferedOutput,
    };

    #[test]
    fn test_root() {
        // setup
        let context = RealFenvContext::new(
            "/home/user/.fenv/",
            "don't care",
            "don't care",
            "don't care",
            "don't care",
            crate::context::OperatingSystem::Linux,
            crate::context::Architecture::X86_64,
            env!("CARGO_PKG_VERSION"),
        );
        let mut output = BufferedOutput::new();
        let sdk_service = RealSdkService::new();

        // execution
        try_run(&["fenv", "root"], &context, &sdk_service, &mut output).unwrap();

        // validation
        assert_eq!(output.stdout_to_string(), "/home/user/.fenv\n");
        assert!(output.stderr_to_string().is_empty());
    }
}
