use crate::{
    args::FenvPrefixArgs, context::FenvContext, invoke_command,
    sdk_service::sdk_service::SdkService, service::service::Service, util::io::ConsoleOutput,
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
        let version_prefix = match &self.args.prefix {
            Some(prefix) => prefix.to_owned(),
            None => invoke_command!(context, sdk_service, output, "version-name")?,
        };
        let version_or_channel =
            invoke_command!(context, sdk_service, output, "latest", &version_prefix)?;
        writeln!(
            output.stdout(),
            "{}",
            context.fenv_sdk_root(&version_or_channel).to_string()
        )?;
        Ok(())
    }
}
