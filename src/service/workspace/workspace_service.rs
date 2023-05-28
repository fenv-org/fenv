use crate::{
    args::FenvWorkspaceArgs,
    context::FenvContext,
    sdk_service::{results::LookupResult, sdk_service::SdkService},
    service::{
        service::Service,
        workspace::{
            dart_sdk_xml::{Classes, DartSdkXml, Library, LibraryEntry, Root},
            package_config_json::{Package, PackageConfigJson},
        },
    },
    spawn_and_wait,
    util::{io::ConsoleOutput, path_like::PathLike},
};
use anyhow::{bail, Context};
use log::{debug, info};
use std::process::Command;

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

        // Generates `.dart_tool/package_config.json` to activate the dedicated version of flutter sdk.
        if !self.args.should_pub_get {
            generate_package_config_json_manually(
                output,
                &workspace_path,
                &sdk_root_path,
                self.args.force,
            )?;
        } else {
            generate_package_config_json_by_pub_get(&workspace_path, &sdk_root_path)?;
        }

        support_intellij_dart_plugin(
            output,
            &workspace_path,
            &sdk_root_path,
            &context.home(),
            self.args.force,
        )
    }
}

/// Triggers a failure if the given `workspace_path` does not have a `pubspec.yaml` file.
fn ensure_pubspec_yaml_contains(workspace_path: &PathLike) -> anyhow::Result<()> {
    if !workspace_path.join("pubspec.yaml").is_file() {
        bail!("Specify a workspace path that contains `pubspec.yaml` file.");
    }
    anyhow::Ok(())
}

/// Generates `.dart_tool/package_config.json` manually to set `flutter`'s version to the given
/// `sdk_root_path`.
///
/// If the `.dart_tool/package_config.json` already exists and has the same `flutter` package, it will not be
/// regenerated.
fn generate_package_config_json_manually<OUT: std::io::Write, ERR: std::io::Write>(
    output: &mut dyn ConsoleOutput<OUT, ERR>,
    workspace_path: &PathLike,
    sdk_root_path: &PathLike,
    force: bool,
) -> anyhow::Result<()> {
    let dart_tool_dir = workspace_path.join(".dart_tool");
    let flutter_package_path = sdk_root_path.join("packages").join("flutter");
    let package_config_json_path = dart_tool_dir.join("package_config.json");

    // If an existing `package_config.json` has the same `flutter` package,
    // we don't need to re-generate it.
    if !force && package_config_json_path.is_file() {
        if let Ok(existing_package_config_json) = PackageConfigJson::read(&package_config_json_path)
        {
            let flutter_package = existing_package_config_json
                .packages
                .iter()
                .find(|p| p.name == "flutter");
            if let Some(flutter_package) = flutter_package {
                if &flutter_package.root_uri == &format!("file://{}", flutter_package_path) {
                    info!("`{}` is already generated", &package_config_json_path);
                    writeln!(
                        output.stdout(),
                        "No need to re-generate `{package_config_json_path}`",
                    )?;
                    return anyhow::Ok(());
                }
            }
        }
    }

    if dart_tool_dir.is_dir() {
        debug!("Removing the existing `{dart_tool_dir}`");
        dart_tool_dir.remove_dir_all()?;
    }
    debug!("Generating `{dart_tool_dir}/package_config.json` with `{flutter_package_path}`");
    package_config_json_path
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

/// Generates `.dart_tool/package_config.json` by running `dart pub get`.
fn generate_package_config_json_by_pub_get(
    workspace_path: &PathLike,
    sdk_root_path: &PathLike,
) -> anyhow::Result<()> {
    debug!("`dart pub get` is started on `{workspace_path}`");
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

fn support_intellij_dart_plugin<OUT: std::io::Write, ERR: std::io::Write>(
    output: &mut dyn ConsoleOutput<OUT, ERR>,
    workspace_path: &PathLike,
    sdk_root_path: &PathLike,
    home_path: &PathLike,
    force: bool,
) -> anyhow::Result<()> {
    let dart_sdk_xml_path = workspace_path
        .join(".idea")
        .join("libraries")
        .join("Dart_SDK.xml");
    let user_home_replaced_sdk_root_path = sdk_root_path
        .to_string()
        .replace(home_path.to_string().as_str(), "$USER_HOME$");
    let dart_core_package_uri =
        format!("file://{user_home_replaced_sdk_root_path}/bin/cache/dart-sdk/lib/core");

    debug!("dart_sdk_xml_path={dart_sdk_xml_path}");

    // If an existing `Dart_SDK.xml` has the same `lib/core` package,
    // we don't need to re-generate it.
    if !force && dart_sdk_xml_path.is_file() {
        debug!("Hello world");
        debug!(
            "{}:\n{}",
            dart_sdk_xml_path,
            dart_sdk_xml_path.read_to_string()?
        );
        match DartSdkXml::read(&dart_sdk_xml_path) {
            Ok(xml) => {
                if xml.has_library(&dart_core_package_uri) {
                    info!("`{}` is already generated", dart_sdk_xml_path);
                    writeln!(
                        output.stdout(),
                        "No need to re-generate `{dart_sdk_xml_path}`"
                    )?;
                    return anyhow::Ok(());
                }
                info!("Updating `{}`", dart_sdk_xml_path)
            }
            Err(err) => bail!("Failed to read `{dart_sdk_xml_path}`: {err}"),
        }
    }

    let dart_libs = list_dart_libs(sdk_root_path)?;
    let dart_sdk_xml = DartSdkXml {
        name: String::from("libraryTable"),
        library: Library {
            name: String::from("Dart SDK"),
            entries: vec![LibraryEntry::Classes(Classes {
                roots: dart_libs
                    .iter()
                    .map(|name|
                        Root {
                            url:format!("file://{user_home_replaced_sdk_root_path}/bin/cache/dart-sdk/lib/{name}"),
                        }
                    )
                    .collect(),
            })],
        },
    };

    println!("{}", dart_sdk_xml.stringify());
    todo!()
}

fn list_dart_libs(sdk_root_path: &PathLike) -> anyhow::Result<Vec<String>> {
    let dart_sdk_path = sdk_root_path.join("lib");
    let read_dir = dart_sdk_path.path().read_dir()?;
    let mut dart_libs: Vec<String> = read_dir
        .flatten()
        .filter_map(|entry| {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    return entry.file_name().into_string().ok();
                }
            }
            None
        })
        .filter(|name| !name.starts_with("_") && !name.starts_with("dev_"))
        .collect();
    dart_libs.sort();
    anyhow::Ok(dart_libs)
}
