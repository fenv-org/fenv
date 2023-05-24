use crate::{
    args::FenvWhichArgs,
    context::FenvContext,
    invoke_command,
    sdk_service::sdk_service::SdkService,
    service::service::Service,
    util::{io::ConsoleOutput, path_like::PathLike},
};
use anyhow::bail;
use is_executable::is_executable;

pub struct FenvWhichService {
    pub args: FenvWhichArgs,
}

impl FenvWhichService {
    pub fn new(args: FenvWhichArgs) -> Self {
        Self { args }
    }
}

impl<OUT, ERR> Service<OUT, ERR> for FenvWhichService
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
        let executable = &self.args.executable;
        let version_or_channel = invoke_command!(context, sdk_service, output, "version-name")?;
        let prefix = invoke_command!(context, sdk_service, output, "prefix", &version_or_channel)?;
        let command_path = PathLike::from(prefix.as_str()).join("bin").join(executable);
        if is_executable(&command_path) {
            writeln!(output.stdout(), "{}", command_path)?;
            return anyhow::Ok(());
        }

        let command_path = context.pub_cache().join("bin").join(executable);
        if is_executable(&command_path) {
            writeln!(output.stdout(), "{}", command_path)?;
            return anyhow::Ok(());
        }

        bail!("Could not find the specified executable: `{executable}`")
    }
}
