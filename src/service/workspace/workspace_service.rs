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
        info!("Need to re-write the existing file `{package_config_json_path}`")
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
    // The `lib/core` package is very important for `intellij-dart`.
    // The `intellij-dart` plugin uses this package to find the location of Dart SDK.
    let dart_core_package_uri =
        format!("file://{user_home_replaced_sdk_root_path}/bin/cache/dart-sdk/lib/core");

    // If an existing `Dart_SDK.xml` has the same `lib/core` package,
    // we don't need to re-generate it.
    if !force && dart_sdk_xml_path.is_file() {
        debug!("Loading `{dart_sdk_xml_path}`...");
        if let Ok(xml) = DartSdkXml::read(&dart_sdk_xml_path) {
            if xml.has_library(&dart_core_package_uri) {
                info!("`{dart_core_package_uri}` is found in `{dart_sdk_xml_path}`");
                writeln!(
                    output.stdout(),
                    "No need to re-generate `{dart_sdk_xml_path}`"
                )?;
                return anyhow::Ok(());
            }
        }
        info!("Need to re-write the existing file `{dart_sdk_xml_path}`")
    }

    // Generates the new `Dart_SDK.xml`, which may contain the `lib/core` package.
    let dart_libs = list_dart_libs(sdk_root_path)?;
    let roots: Vec<Root> = dart_libs
        .iter()
        .map(|name| Root {
            url: format!("file://{user_home_replaced_sdk_root_path}/bin/cache/dart-sdk/lib/{name}"),
        })
        .collect();
    let dart_sdk_xml = DartSdkXml {
        name: String::from("libraryTable"),
        library: Library {
            name: String::from("Dart SDK"),
            entries: vec![LibraryEntry::Classes(Classes { roots })],
        },
    };

    debug!("Writing `{dart_sdk_xml_path}`...");
    dart_sdk_xml_path
        .write(dart_sdk_xml.stringify())
        .map_err(|err| anyhow::anyhow!("Failed to write `{dart_sdk_xml_path}`: {err}"))?;
    writeln!(output.stdout(), "`{dart_sdk_xml_path}` is generated",)?;
    anyhow::Ok(())
}

fn list_dart_libs(sdk_root_path: &PathLike) -> anyhow::Result<Vec<String>> {
    let dart_sdk_path = sdk_root_path
        .join("bin")
        .join("cache")
        .join("dart-sdk")
        .join("lib");
    if !dart_sdk_path.is_dir() {
        debug!("`{}` is not a directory", dart_sdk_path);
        return anyhow::Ok(vec![]);
    }

    let read_dir = dart_sdk_path
        .path()
        .read_dir()
        .map_err(|err| anyhow::anyhow!("Could not retrieve the list of Dart libraries: {err}"))?;
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

#[cfg(test)]
mod tests {
    use crate::{
        context::FenvContext, sdk_service::sdk_service::RealSdkService,
        service::macros::test_with_context, try_run, util::path_like::PathLike,
    };

    fn prepare_valid_workspace(context: &impl FenvContext) {
        context
            .fenv_dir()
            .join("workspace")
            .create_dir_all()
            .unwrap();
        context
            .fenv_dir()
            .join("workspace")
            .join("pubspec.yaml")
            .write("")
            .unwrap();
    }

    fn prepare_flutter_sdk(context: &impl FenvContext, version_or_channel: &str) {
        let dart_sdk_lib = context
            .fenv_root()
            .join("versions")
            .join(version_or_channel)
            .join("bin")
            .join("cache")
            .join("dart-sdk")
            .join("lib");
        dart_sdk_lib.join("_http").create_dir_all().unwrap();
        dart_sdk_lib.join("_internal").create_dir_all().unwrap();
        dart_sdk_lib.join("core").create_dir_all().unwrap();
        dart_sdk_lib.join("async").create_dir_all().unwrap();
        dart_sdk_lib.join("svg").create_dir_all().unwrap();
        dart_sdk_lib.join("dev_compiler").create_dir_all().unwrap();
        dart_sdk_lib.join("libraries.json").writeln("").unwrap();
    }

    fn generate_package_config_json_content(root: &PathLike, version_or_channel: &str) -> String {
        indoc::formatdoc! {
            "{{
              \"configVersion\": 2,
              \"packages\": [
                {{
                  \"name\": \"flutter\",
                  \"rootUri\": \"file://{root}/versions/{version_or_channel}/packages/flutter\",
                  \"packageUri\": \"lib/\"
                }}
              ]
            }}
            "
        }
    }

    fn generate_dart_xml_content(version_or_channel: &str) -> String {
        indoc::formatdoc! {r#"
            <component name="libraryTable">
              <library name="Dart SDK">
                <CLASSES>
                  <root url="file://$USER_HOME$/.fenv/versions/{version_or_channel}/bin/cache/dart-sdk/lib/async" />
                  <root url="file://$USER_HOME$/.fenv/versions/{version_or_channel}/bin/cache/dart-sdk/lib/core" />
                  <root url="file://$USER_HOME$/.fenv/versions/{version_or_channel}/bin/cache/dart-sdk/lib/svg" />
                </CLASSES>
              </library>
            </component>
            "#,
        }
    }

    fn read_package_config_json(context: &impl FenvContext) -> std::io::Result<String> {
        context
            .fenv_dir()
            .join("workspace/.dart_tool/package_config.json")
            .read_to_string()
    }

    fn write_package_config_json(context: &impl FenvContext, content: &str) -> std::io::Result<()> {
        context
            .fenv_dir()
            .join("workspace/.dart_tool/package_config.json")
            .write(content)
    }

    fn read_dart_sdk_xml(context: &impl FenvContext) -> std::io::Result<String> {
        context
            .fenv_dir()
            .join("workspace/.idea/libraries/Dart_SDK.xml")
            .read_to_string()
    }

    fn write_dart_sdk_xml(context: &impl FenvContext, content: &str) -> std::io::Result<()> {
        context
            .fenv_dir()
            .join("workspace/.idea/libraries/Dart_SDK.xml")
            .write(content)
    }

    #[test]
    fn test_fails_if_non_workspace_directory_is_given() {
        test_with_context(|context, output| {
            // setup
            // prepare a directory that does not have `pubspec.yaml`.
            context
                .fenv_dir()
                .join("workspace")
                .create_dir_all()
                .unwrap();
            let sdk_service = RealSdkService::new();

            // execution
            let result = try_run(
                &[
                    "fenv",
                    "workspace",
                    &format!("{}/workspace", context.fenv_dir()),
                ],
                context,
                &sdk_service,
                output,
            );

            // validation
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                "Specify a workspace path that contains `pubspec.yaml` file."
            );
        })
    }

    #[test]
    fn test_fails_if_specified_sdk_is_not_installed() {
        test_with_context(|context, output| {
            // setup
            prepare_valid_workspace(context);
            let sdk_service = RealSdkService::new();

            // execution
            let result = try_run(
                &[
                    "fenv",
                    "workspace",
                    &format!("{}/workspace", context.fenv_dir()),
                    "3",
                ],
                context,
                &sdk_service,
                output,
            );

            // validation
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                "Not found any matched flutter sdk version: `3`"
            );
        })
    }

    #[test]
    fn test_successfully_generate_required_files_if_workspace_directory_is_given() {
        test_with_context(|context, output| {
            // setup
            prepare_valid_workspace(context);
            prepare_flutter_sdk(context, "stable");
            let sdk_service = RealSdkService::new();

            // execution
            try_run(
                &[
                    "fenv",
                    "workspace",
                    &format!("{}/workspace", context.fenv_dir()),
                    "s",
                ],
                context,
                &sdk_service,
                output,
            )
            .unwrap();

            // validation
            let expected_package_config_json_content =
                generate_package_config_json_content(&context.fenv_root(), "stable");
            assert_eq!(
                expected_package_config_json_content,
                read_package_config_json(context).unwrap()
            );

            let expected_dart_sdk_xml_content = generate_dart_xml_content("stable");
            assert_eq!(
                expected_dart_sdk_xml_content,
                read_dart_sdk_xml(context).unwrap()
            );

            assert_eq!(
                output.stdout_to_string(),
                format!(
                    "`{workspace}/.dart_tool/package_config.json` is generated\n`{workspace}/.idea/libraries/Dart_SDK.xml` is generated\n",
                    workspace = context.fenv_dir().join("workspace")
                ),
            );
            assert!(output.stderr_to_string().is_empty());
        })
    }

    #[test]
    fn test_skip_regenerating_files_if_not_needed() {
        test_with_context(|context, output| {
            // setup
            prepare_valid_workspace(context);
            prepare_flutter_sdk(context, "3.7.12");
            // prepare `.flutter-version`, which is set to `3`.
            context
                .fenv_dir()
                .join("workspace/.flutter-version")
                .write("3")
                .unwrap();
            // prepare `package_config.json`, which is set to `3.7.12`.
            write_package_config_json(
                context,
                &generate_package_config_json_content(&context.fenv_root(), "3.7.12"),
            )
            .unwrap();
            // prepare `Dart_SDK.xml`, which is set to `3.7.12`.
            write_dart_sdk_xml(context, &generate_dart_xml_content("3.7.12")).unwrap();

            let sdk_service = RealSdkService::new();

            // execution
            try_run(
                &[
                    "fenv",
                    "workspace",
                    &format!("{}/workspace", context.fenv_dir()),
                ],
                context,
                &sdk_service,
                output,
            )
            .unwrap();

            // validation
            let expected_package_config_json_content =
                generate_package_config_json_content(&context.fenv_root(), "3.7.12");
            assert_eq!(
                expected_package_config_json_content,
                read_package_config_json(context).unwrap()
            );

            let expected_dart_sdk_xml_content = generate_dart_xml_content("3.7.12");
            assert_eq!(
                expected_dart_sdk_xml_content,
                read_dart_sdk_xml(context).unwrap()
            );

            assert_eq!(
                output.stdout_to_string(),
                format!(
                    "No need to re-generate `{workspace}/.dart_tool/package_config.json`\nNo need to re-generate `{workspace}/.idea/libraries/Dart_SDK.xml`\n",
                    workspace = context.fenv_dir().join("workspace")
                ),
            );
            assert!(output.stderr_to_string().is_empty());
        })
    }

    #[test]
    fn test_regenerating_files_if_needed() {
        test_with_context(|context, output| {
            // setup
            prepare_valid_workspace(context);
            prepare_flutter_sdk(context, "3.7.12");
            // prepare the global `version`, which is set to `3`.
            context.fenv_root().join("version").write("3").unwrap();
            // prepare `package_config.json`, which is set to `stable`.
            write_package_config_json(
                context,
                &generate_package_config_json_content(&context.fenv_root(), "stable"),
            )
            .unwrap();
            // prepare `Dart_SDK.xml`, which is set to `stable`.
            write_dart_sdk_xml(context, &generate_dart_xml_content("stable")).unwrap();

            let sdk_service = RealSdkService::new();

            // execution
            try_run(
                &[
                    "fenv",
                    "workspace",
                    &format!("{}/workspace", context.fenv_dir()),
                ],
                context,
                &sdk_service,
                output,
            )
            .unwrap();

            // validation
            let expected_package_config_json_content =
                generate_package_config_json_content(&context.fenv_root(), "3.7.12");
            assert_eq!(
                expected_package_config_json_content,
                read_package_config_json(context).unwrap()
            );

            let expected_dart_sdk_xml_content = generate_dart_xml_content("3.7.12");
            assert_eq!(
                expected_dart_sdk_xml_content,
                read_dart_sdk_xml(context).unwrap()
            );

            assert_eq!(
                output.stdout_to_string(),
                format!(
                    "`{workspace}/.dart_tool/package_config.json` is generated\n`{workspace}/.idea/libraries/Dart_SDK.xml` is generated\n",
                    workspace = context.fenv_dir().join("workspace")
                ),
            );
            assert!(output.stderr_to_string().is_empty());
        })
    }
}
