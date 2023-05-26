use std::result;

use anyhow::{bail, Context};

use crate::{
    args::FenvWorkspaceArgs,
    context::FenvContext,
    invoke_command,
    sdk_service::{
        results::{LookupResult, VersionFileReadResult},
        sdk_service::SdkService,
    },
    service::service::Service,
    util::{
        io::ConsoleOutput,
        package_config_json::{Package, PackageConfigJson},
        path_like::PathLike,
    },
};

pub struct FenvWorkspaceService {
    pub args: FenvWorkspaceArgs,
}

impl FenvWorkspaceService {
    pub fn new(args: FenvWorkspaceArgs) -> Self {
        Self { args }
    }
}

impl<OUT, ERR> Service<OUT, ERR> for FenvWorkspaceService
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
        let workspace = &self.args.workspace[..];
        let workspace_path = PathLike::from(workspace);
        if !workspace_path.join("pubspec.yaml").is_file() {
            bail!("Specify a workspace path that contains `pubspec.yaml` file.");
        }

        let sdk_root_path = match &self.args.prefix {
            Some(prefix) => match sdk_service.find_latest_local(context, prefix) {
                LookupResult::Found(sdk) => context.fenv_sdk_root(&sdk.to_string()),
                LookupResult::Err(err) => return Result::Err(err),
                LookupResult::None => {
                    bail!("Not found any matched flutter sdk version: `{prefix}`")
                }
            },
            None => {
                let read_result = sdk_service.read_nearest_version_file(context, &workspace_path);
                let installed_sdk_summary = sdk_service.ensure_sdk_is_available(&read_result)?;
                installed_sdk_summary.path_to_sdk_root
            }
        };
        let flutter_package_path = sdk_root_path.join("packages/flutter");

        workspace_path
            .join(".dart_tool")
            .join("package_config.json")
            .writeln(
                PackageConfigJson {
                    config_version: 2,
                    packages: vec![Package::new(
                        "flutter",
                        &format!("file://{}", flutter_package_path),
                        "lib/",
                    )],
                }
                .stringify(),
            )
            .with_context(|| anyhow::anyhow!("Failed to write package_config.json"))
    }
}
