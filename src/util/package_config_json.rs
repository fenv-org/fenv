use anyhow::Context;
use serde::{Deserialize, Serialize};

/// A definition of format of `.dart_tool/package_config.json` file.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PackageConfigJson {
    pub config_version: u64,
    pub packages: Vec<Package>,
}

impl PackageConfigJson {
    pub fn parse(raw_json: &str) -> anyhow::Result<Self> {
        serde_json::from_str(raw_json)
            .with_context(|| "Failed to parse the given `package_config.json`")
    }

    pub fn stringify(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    pub name: String,
    pub root_uri: String,
    pub package_uri: String,
    pub language_version: String,
}

#[cfg(test)]
mod tests {
    use crate::util::{
        package_config_json::{Package, PackageConfigJson},
        path_like::PathLike,
    };

    fn read_resource_file(relative_path: &str) -> std::io::Result<String> {
        PathLike::from(std::env!("CARGO_MANIFEST_DIR"))
            .join(relative_path)
            .read_to_string()
    }

    #[test]
    fn test_parsing() {
        let raw_json: String =
            read_resource_file("resources/test/package_config/sample.json").unwrap();
        let actual = PackageConfigJson::parse(&raw_json).unwrap();
        assert_eq!(
            actual,
            PackageConfigJson {
                config_version: 2,
                packages: vec![
                    Package {
                        name: "_fe_analyzer_shared".to_string(),
                        root_uri: "file:///home/user/.pub-cache/hosted/pub.dartlang.org/_fe_analyzer_shared-50.0.0".to_string(),
                        package_uri: "lib/".to_string(),
                        language_version: "2.17".to_string(),
                    },
                    Package {
                        name: "flutter".to_string(),
                        root_uri: "file:///home/user/.fenv/versions/3.3.10/packages/flutter".to_string(),
                        package_uri: "lib/".to_string(),
                        language_version: "2.17".to_string(),
                    },
                ]
            }
        )
    }
}
