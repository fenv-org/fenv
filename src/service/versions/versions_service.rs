use crate::{
    context::FenvContext,
    sdk_service::{model::flutter_sdk::FlutterSdk, sdk_service::SdkService},
    service::service::Service,
    util::io::ConsoleOutput,
};

pub struct FenvVersionsService {}

impl FenvVersionsService {
    pub fn new() -> FenvVersionsService {
        FenvVersionsService {}
    }
}

impl<OUT, ERR> Service<OUT, ERR> for FenvVersionsService
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
        let sdks = sdk_service.get_installed_sdk_list(context)?;
        for sdk in sdks {
            writeln!(output.stdout(), "{}", &sdk.display_name())?;
        }
        anyhow::Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        context::FenvContext, sdk_service::sdk_service::RealSdkService,
        service::macros::test_with_context, try_run,
    };
    use indoc::formatdoc;
    use std::fs;

    #[test]
    fn test_sorted_order_of_list_installed_sdks() {
        test_with_context(|context, output| {
            // setup
            let fenv_versions = context.fenv_versions();
            fs::create_dir_all(&fenv_versions).unwrap();
            fs::create_dir(fenv_versions.join("10.231.5+hotfix.2")).unwrap();
            fs::create_dir(fenv_versions.join("1.0.0")).unwrap();
            fs::create_dir(fenv_versions.join("v2.23.40-hotfix.10")).unwrap();
            fs::create_dir(fenv_versions.join("v10.231.5")).unwrap();
            fs::create_dir(fenv_versions.join("stable")).unwrap();
            fs::create_dir(fenv_versions.join("beta")).unwrap();
            fs::create_dir(fenv_versions.join("dev")).unwrap();
            fs::create_dir(fenv_versions.join("master")).unwrap();

            // execution
            try_run(
                &["fenv", "versions"],
                context,
                &RealSdkService::new(),
                output,
            )
            .unwrap();

            // validation
            assert_eq!(
                formatdoc! {
                    "
                    1.0.0
                    v2.23.40-hotfix.10
                    v10.231.5
                    10.231.5+hotfix.2
                    dev
                    beta
                    master
                    stable
                    "
                },
                output.stdout_to_string()
            );
        });
    }

    #[test]
    fn test_filter_out_installing_markers() {
        test_with_context(|context, output| {
            // setup
            let fenv_versions = context.fenv_versions();
            fs::create_dir_all(&fenv_versions).unwrap();
            fs::create_dir(fenv_versions.join("1.0.0")).unwrap();
            fs::create_dir(fenv_versions.join("v2.23.40-hotfix.10")).unwrap();
            fs::create_dir(fenv_versions.join("v10.231.5")).unwrap();
            fs::create_dir(fenv_versions.join("10.231.5+hotfix.2")).unwrap();
            fs::create_dir(fenv_versions.join("dev")).unwrap();
            fs::create_dir(fenv_versions.join("beta")).unwrap();
            fs::create_dir(fenv_versions.join("master")).unwrap();
            fs::create_dir(fenv_versions.join("stable")).unwrap();

            fs::File::create(fenv_versions.join(".install_v10.231.5")).unwrap();
            fs::File::create(fenv_versions.join(".install_master")).unwrap();

            // execution
            try_run(
                &["fenv", "versions"],
                context,
                &RealSdkService::new(),
                output,
            )
            .unwrap();

            // validation
            assert_eq!(
                formatdoc! {
                    "
                    1.0.0
                    v2.23.40-hotfix.10
                    10.231.5+hotfix.2
                    dev
                    beta
                    stable
                    "
                },
                output.stdout_to_string()
            );
        })
    }
}
