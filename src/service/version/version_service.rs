use crate::{
    args::FenvStartDirArgs,
    context::FenvContext,
    sdk_service::sdk_service::SdkService,
    service::service::Service,
    try_run,
    util::io::{BufferedOutput, ConsoleOutput},
};

pub struct FenvVersionService {
    pub args: FenvStartDirArgs,
}

impl FenvVersionService {
    pub fn new(args: FenvStartDirArgs) -> Self {
        Self { args }
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
        let dir: String = if let Some(dir) = &self.args.dir {
            dir.clone()
        } else {
            context.fenv_dir().path().to_str().unwrap().to_string()
        };
        let version_name = retrieve_version_name(context, sdk_service, output, &dir)?;
        let version_file = retrieve_version_file(context, sdk_service, output, &dir)?;
        writeln!(output.stdout(), "{version_name} (set by `{version_file}`)")?;
        anyhow::Ok(())
    }
}

fn retrieve_version_name<OUT, ERR>(
    context: &impl FenvContext,
    sdk_service: &impl SdkService,
    output: &mut dyn ConsoleOutput<OUT, ERR>,
    dir: &str,
) -> anyhow::Result<String>
where
    OUT: std::io::Write,
    ERR: std::io::Write,
{
    let mut buffered_output = BufferedOutput::new();
    try_run(
        &["fenv", "version-name", dir],
        context,
        sdk_service,
        &mut buffered_output,
    )?;
    write!(output.stderr(), "{}", buffered_output.stderr_to_string())?;
    let version_name = buffered_output.stdout_to_string().trim_end().to_string();
    return anyhow::Ok(version_name);
}

fn retrieve_version_file<OUT, ERR>(
    context: &impl FenvContext,
    sdk_service: &impl SdkService,
    output: &mut dyn ConsoleOutput<OUT, ERR>,
    dir: &str,
) -> anyhow::Result<String>
where
    OUT: std::io::Write,
    ERR: std::io::Write,
{
    let mut buffered_output = BufferedOutput::new();
    try_run(
        &["fenv", "version-file", dir],
        context,
        sdk_service,
        &mut buffered_output,
    )?;
    write!(output.stderr(), "{}", buffered_output.stderr_to_string())?;
    let version_file = buffered_output.stdout_to_string().trim_end().to_string();
    return anyhow::Ok(version_file);
}
