use crate::{
    args::FenvPrefixArgs,
    context::FenvContext,
    sdk_service::sdk_service::SdkService,
    service::service::Service,
    try_run,
    util::io::{BufferedOutput, ConsoleOutput},
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
            None => retrieve_version_name(context, sdk_service, output)?,
        };
        // let version_or_channel =

        Ok(())
    }
}

fn retrieve_version_name<OUT, ERR>(
    context: &impl FenvContext,
    sdk_service: &impl SdkService,
    output: &mut dyn ConsoleOutput<OUT, ERR>,
) -> anyhow::Result<String>
where
    OUT: std::io::Write,
    ERR: std::io::Write,
{
    let mut buffered_output = BufferedOutput::new();
    try_run(
        &["fenv", "version-name"],
        context,
        sdk_service,
        &mut buffered_output,
    )?;
    write!(output.stderr(), "{}", buffered_output.stderr_to_string())?;
    let version_name = buffered_output.stdout_to_string().trim_end().to_string();
    return anyhow::Ok(version_name);
}

// fn resolve_prefix
