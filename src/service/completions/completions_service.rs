use crate::{
    args::FenvCompletionsArgs, build_command, context::FenvContext,
    sdk_service::sdk_service::SdkService, service::service::Service, util::io::ConsoleOutput,
};
use anyhow::anyhow;
use clap::ValueEnum;
use clap_complete::{generate, Shell};

pub struct FenvCompletionsService {
    pub args: FenvCompletionsArgs,
}

impl FenvCompletionsService {
    pub fn new(args: FenvCompletionsArgs) -> FenvCompletionsService {
        FenvCompletionsService { args }
    }

    pub fn completions_commands(shell: &Shell) -> String {
        let mut buffer: Vec<u8> = Vec::new();
        generate(shell.to_owned(), &mut build_command(), "fenv", &mut buffer);
        return String::from_utf8_lossy(&buffer).to_string();
    }
}

impl<OUT, ERR> Service<OUT, ERR> for FenvCompletionsService
where
    OUT: std::io::Write,
    ERR: std::io::Write,
{
    fn execute(
        &self,
        _: &impl FenvContext,
        _: &impl SdkService,
        output: &mut dyn ConsoleOutput<OUT, ERR>,
    ) -> anyhow::Result<()> {
        let shell = Shell::from_str(&self.args.shell, true).map_err(|e| anyhow!(e))?;
        write!(
            output.stdout(),
            "{}",
            FenvCompletionsService::completions_commands(&shell)
        )
        .map_err(|e| anyhow!(e))
    }
}
