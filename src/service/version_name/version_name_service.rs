use crate::{
    args::FenvStartDirArgs, context::FenvContext, sdk_service::sdk_service::SdkService,
    service::service::Service, util::io::ConsoleOutput,
};

pub struct FenvVersionNameService {
    pub args: FenvStartDirArgs,
}

impl FenvVersionNameService {
    pub fn new(args: FenvStartDirArgs) -> Self {
        Self { args }
    }
}

impl<OUT, ERR> Service<OUT, ERR> for FenvVersionNameService
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
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        context::FenvContext, sdk_service::sdk_service::RealSdkService,
        service::macros::test_with_context, try_run,
    };

    #[test]
    pub fn test_show_version_name_succeeds_if_global_version_name_is_found() {
        test_with_context(|context, output| {
            // setup
            context
                .fenv_versions()
                .join("1.0.0")
                .create_dir_all()
                .unwrap();
            context.fenv_global_version_file().writeln("1").unwrap();

            // execution
            try_run(
                &["fenv", "version-name"],
                context,
                &RealSdkService::new(),
                output,
            )
            .unwrap();

            // verification
            assert_eq!(output.stdout_to_string(), "1.0.0\n");
            assert_eq!(output.stderr_to_string(), "");
        })
    }

    #[test]
    pub fn test_show_version_name_succeeds_if_local_version_name_is_found() {
        test_with_context(|context, output| {
            // setup
            context
                .fenv_versions()
                .join("master")
                .create_dir_all()
                .unwrap();
            context
                .fenv_dir()
                .join(".flutter-version")
                .writeln("m")
                .unwrap();

            // execution
            try_run(
                &["fenv", "version-name"],
                context,
                &RealSdkService::new(),
                output,
            )
            .unwrap();

            // verification
            assert_eq!(output.stdout_to_string(), "master\n");
            assert_eq!(output.stderr_to_string(), "");
        })
    }

    #[test]
    pub fn test_show_version_name_fails_if_no_version_name_is_found() {
        test_with_context(|context, output| {
            // execution
            let result = try_run(
                &["fenv", "version-name"],
                context,
                &RealSdkService::new(),
                output,
            );

            // verification
            assert!(result.is_err());
            assert_eq!(
                result.err().unwrap().to_string(),
                "Could not find any version file"
            );
        })
    }

    #[test]
    pub fn test_show_version_name_fails_if_global_version_is_not_installed() {
        test_with_context(|context, output| {
            // preparation
            context.fenv_global_version_file().writeln("1").unwrap();

            // execution
            let result = try_run(
                &["fenv", "version-name"],
                context,
                &RealSdkService::new(),
                output,
            );

            // verification
            assert!(result.is_err());
            assert_eq!(
                result.err().unwrap().to_string(),
                format!(
                    "The specified version `1` is not installed (set by `{path}`): do `fenv install`",
                    path = context.fenv_root().join("version")
                )
            );
        })
    }
}
