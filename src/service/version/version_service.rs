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
            context.fenv_dir().to_string()
        };
        let version_name = retrieve_version_name(context, sdk_service, output, &dir)?;
        let version_file = retrieve_version_file(context, sdk_service, output, &dir).unwrap();
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

#[cfg(test)]
mod tests {
    use crate::{
        context::FenvContext, define_mock_valid_git_command,
        external::flutter_command::FlutterCommandImpl, sdk_service::sdk_service::RealSdkService,
        service::macros::test_with_context, try_run, util::chrono_wrapper::SystemClock,
    };

    define_mock_valid_git_command!();

    #[test]
    fn test_show_version_succeeds_if_global_version_is_set_and_installed() {
        test_with_context(|context, output| {
            // setup
            // make sure v1.0.0 sdk is installed
            context
                .fenv_versions()
                .join("v1.0.0")
                .create_dir_all()
                .unwrap();
            // prepare the global version file
            context.fenv_root().join("version").writeln("1").unwrap();
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            try_run(&["fenv", "version"], context, &sdk_service, output).unwrap();

            // validation
            assert_eq!(
                output.stdout_to_string(),
                format!("v1.0.0 (set by `{}/version`)\n", context.fenv_root())
            );
        })
    }
}
