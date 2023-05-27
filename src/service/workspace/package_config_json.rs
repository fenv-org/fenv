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
    pub fn read<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content)
    }

    pub fn parse(raw_json: &str) -> anyhow::Result<Self> {
        serde_json::from_str(raw_json)
            .with_context(|| "Failed to parse the given `package_config.json`")
    }

    pub fn stringify(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    pub name: String,
    pub root_uri: String,
    pub package_uri: String,
}

impl Package {
    pub fn new(name: &str, root_uri: &str, package_uri: &str) -> Self {
        Self {
            name: name.to_string(),
            root_uri: root_uri.to_string(),
            package_uri: package_uri.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        service::workspace::package_config_json::{Package, PackageConfigJson},
        util::path_like::PathLike,
    };

    #[test]
    fn test_parsing() {
        let actual = PackageConfigJson::read(
            PathLike::from(std::env!("CARGO_MANIFEST_DIR"))
                .join("resources/test/package_config/sample.json"),
        )
        .unwrap();
        assert_eq!(
            actual,
            PackageConfigJson {
                config_version: 2,
                packages: vec![
                    Package {
                        name: "_fe_analyzer_shared".to_string(),
                        root_uri: "file:///home/user/.pub-cache/hosted/pub.dartlang.org/_fe_analyzer_shared-50.0.0".to_string(),
                        package_uri: "lib/".to_string(),
                    },
                    Package {
                        name: "flutter".to_string(),
                        root_uri: "file:///home/user/.fenv/versions/3.3.10/packages/flutter".to_string(),
                        package_uri: "lib/".to_string(),
                    },
                ]
            }
        )
    }
}
