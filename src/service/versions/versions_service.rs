use std::{io, path::PathBuf};

use anyhow::{bail, Ok, Result};

use crate::{model::flutter_version::FlutterVersion, service::service::Service};

pub struct FenvVersionsService {}

impl FenvVersionsService {
    pub fn new() -> FenvVersionsService {
        FenvVersionsService {}
    }
}

impl Service for FenvVersionsService {
    fn execute(&self, config: &crate::config::Config) -> Result<()> {
        let path = PathBuf::from(&config.fenv_versions());
        if !path.is_dir() {
            if path.exists() {
                bail!("`{}` exists but not a directory.", path.to_str().unwrap())
            }
            std::fs::create_dir_all(&path).ok();
            if !path.is_dir() {
                panic!("`{}` must exist now", path.to_str().unwrap())
            }
        }

        if let io::Result::Ok(entries) = path.read_dir() {
            let mut children: Vec<String> = entries
                .flatten()
                .map(|entry| String::from(entry.file_name().to_str().unwrap()))
                .collect();
            children.sort_by_key(|version_string| FlutterVersion::parse(&version_string));
            for child in children {
                println!("{}", child)
            }
            // TODO: define a function to return Vec<VersionOrChannel>
            // TODO: Sort `Vec<VersionOrChannel>` correctly.
            // TODO: Exclude any directories where the installing marker files still alive.
        }
        Ok(())
    }
}
