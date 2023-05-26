use std::{process::Command, result};

use anyhow::{bail, Context};
use log::info;

use crate::{
    args::FenvWorkspaceArgs,
    context::FenvContext,
    invoke_command,
    sdk_service::{
        results::{LookupResult, VersionFileReadResult},
        sdk_service::SdkService,
    },
    service::service::Service,
    spawn_and_wait,
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
        ensure_pubspec_yaml_contains(&workspace_path)?;
        let prefix = self.args.prefix.as_ref().map(|s| &s[..]);
        let sdk_root_path = find_sdk_root_path(context, sdk_service, &workspace_path, prefix)?;
        if !self.args.should_pub_get {
            generate_package_config_json_manually(output, &workspace_path, &sdk_root_path)?;
        } else {
            generate_package_config_json_by_pub_get(&workspace_path, &sdk_root_path)?;
        }

        anyhow::Ok(())
    }
}

fn ensure_pubspec_yaml_contains(workspace_path: &PathLike) -> anyhow::Result<()> {
    if !workspace_path.join("pubspec.yaml").is_file() {
        bail!("Specify a workspace path that contains `pubspec.yaml` file.");
    }
    anyhow::Ok(())
}

fn generate_package_config_json_manually<OUT: std::io::Write, ERR: std::io::Write>(
    output: &mut dyn ConsoleOutput<OUT, ERR>,
    workspace_path: &PathLike,
    sdk_root_path: &PathLike,
) -> anyhow::Result<()> {
    let dart_tool_dir = workspace_path.join(".dart_tool");
    if dart_tool_dir.is_dir() {
        info!("Removing the existing `{dart_tool_dir}`");
        dart_tool_dir.remove_dir_all()?;
    }
    let flutter_package_path = sdk_root_path.join("packages").join("flutter");
    info!("Generating `{dart_tool_dir}/package_config.json` with `{flutter_package_path}`");
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
        .with_context(|| anyhow::anyhow!("Failed to write package_config.json"))?;

    writeln!(
        output.stdout(),
        "`{}/.dart_tool/package_config.json` is generated",
        workspace_path
    )?;
    anyhow::Ok(())
}

fn generate_package_config_json_by_pub_get(
    workspace_path: &PathLike,
    sdk_root_path: &PathLike,
) -> anyhow::Result<()> {
    info!("`dart pub get` is started on `{workspace_path}`");
    let dart_cli_path = sdk_root_path.join("bin").join("dart");
    let mut command = Command::new(dart_cli_path.path());
    spawn_and_wait!(
        command.current_dir(workspace_path).args(["pub", "get"]),
        "generate_package_config_json_by_pub_get",
        "Failed to execute `dart pub get`"
    );
    anyhow::Ok(())
}

fn find_sdk_root_path(
    context: &impl FenvContext,
    sdk_service: &impl SdkService,
    workspace_path: &PathLike,
    prefix: Option<&str>,
) -> anyhow::Result<PathLike> {
    match prefix {
        Some(prefix) => match sdk_service.find_latest_local(context, prefix) {
            LookupResult::Found(sdk) => anyhow::Ok(context.fenv_sdk_root(&sdk.to_string())),
            LookupResult::Err(err) => anyhow::Result::Err(err),
            LookupResult::None => {
                bail!("Not found any matched flutter sdk version: `{prefix}`")
            }
        },
        None => {
            let read_result = sdk_service.read_nearest_version_file(context, &workspace_path);
            let installed_sdk_summary = sdk_service.ensure_sdk_is_available(&read_result)?;
            anyhow::Ok(installed_sdk_summary.path_to_sdk_root)
        }
    }
}
