use clap_complete::generate;

use crate::{args::FenvCompletionsArgs, build_command, service::service::Service};

pub struct FenvCompletionsService {
    pub args: FenvCompletionsArgs,
}

impl FenvCompletionsService {
    pub fn from(args: FenvCompletionsArgs) -> FenvCompletionsService {
        FenvCompletionsService { args }
    }
}

impl Service for FenvCompletionsService {
    fn execute(
        &self,
        _: &crate::config::Config,
        stdout: &mut impl std::io::Write,
    ) -> anyhow::Result<()> {
        generate(self.args.shell, &mut build_command(), "fenv", stdout);
        Ok(())
    }
}
