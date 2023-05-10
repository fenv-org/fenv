use crate::{
    args::FenvVersionFileArgs, context::FenvContext, sdk_service::sdk_service::SdkService,
    service::service::Service, util::path_like::PathLike,
};
use anyhow::{bail, Ok};
use log::debug;

pub struct FenvVersionFileService {
    args: FenvVersionFileArgs,
}

impl FenvVersionFileService {
    pub fn new(args: FenvVersionFileArgs) -> Self {
        Self { args }
    }
}

impl Service for FenvVersionFileService {
    fn execute(
        &self,
        context: &impl FenvContext,
        sdk_service: &impl SdkService,
        stdout: &mut impl std::io::Write,
    ) -> anyhow::Result<()> {
        let start_dir = match &self.args.dir {
            Some(dir) => {
                debug!("Start looking for version file from `{dir}`");
                PathLike::from(dir.as_str())
            }
            None => {
                debug!("Start looking for version file from the current directory");
                context.fenv_dir().to_owned()
            }
        };
        if !start_dir.exists() {
            bail!("`{start_dir}` does not exist");
        }
        if !start_dir.is_dir() {
            bail!("`{start_dir}` is not a directory");
        }
        match sdk_service.find_nearest_version_file(context, &start_dir) {
            crate::sdk_service::results::LookupResult::Found(version_file) => {
                debug!("Found version file `{version_file}`");
                writeln!(stdout, "{version_file}")?;
                Ok(())
            }
            crate::sdk_service::results::LookupResult::Err(e) => {
                anyhow::Result::Err(anyhow::anyhow!(e))
            }
            crate::sdk_service::results::LookupResult::None => {
                bail!("Could not find any version file")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{sdk_service::sdk_service::RealSdkService, service::macros::test_with_context};
    use std::io::Write;

    #[test]
    fn test_look_up_version_file_outputs_global_version_file_path_when_no_local_version_file_exists(
    ) {
        test_with_context(|context| {
            // setup
            let args = FenvVersionFileArgs { dir: None };
            let service = FenvVersionFileService::new(args);

            // prepare the global version file
            let global_version_filepath = context.fenv_root().join("version");
            global_version_filepath.writeln("1.2.3").unwrap();

            // execution
            let mut stdout: Vec<u8> = Vec::new();
            service
                .execute(context, &RealSdkService::new(), &mut stdout)
                .unwrap();

            // validation
            assert_eq!(
                String::from_utf8(stdout).unwrap(),
                format!(
                    "{root}{separator}version\n",
                    root = context.fenv_root(),
                    separator = std::path::MAIN_SEPARATOR
                ),
            );
        });
    }

    #[test]
    fn test_look_up_version_file_outputs_local_version_file_path_when_local_version_file_exists() {
        test_with_context(|context| {
            // setup
            // prepare the lookup directory: `$HOME/a/b/c`
            let lookup_dir = context.home().join("a").join("b").join("c");
            lookup_dir.create_dir_all().unwrap();
            let args = FenvVersionFileArgs {
                dir: Some(lookup_dir.path().to_str().unwrap().to_string()),
            };
            let service = FenvVersionFileService::new(args);

            // prepare the local version file: `$HOME/a/.flutter-version`
            let local_version_filepath = context.home().join("a").join(".flutter-version");
            let mut local_version_filepath = &local_version_filepath.create_file().unwrap();
            writeln!(local_version_filepath, "1.2.3").unwrap();

            // execution
            let mut stdout: Vec<u8> = Vec::new();
            service
                .execute(context, &RealSdkService::new(), &mut stdout)
                .unwrap();

            // validation
            assert_eq!(
                String::from_utf8(stdout).unwrap(),
                format!(
                    "{root}{separator}a{separator}.flutter-version\n",
                    root = context.home(),
                    separator = std::path::MAIN_SEPARATOR
                ),
            );
        });
    }

    #[test]
    fn test_look_up_version_file_fails_when_no_version_file_exists() {
        test_with_context(|context| {
            // setup
            // prepare the lookup directory: `$HOME/a/b/c`
            let lookup_dir = context.home().join("a").join("b").join("c");
            lookup_dir.create_dir_all().unwrap();
            let args = FenvVersionFileArgs {
                dir: Some(lookup_dir.path().to_str().unwrap().to_string()),
            };
            let service = FenvVersionFileService::new(args);

            // execution
            let mut stdout: Vec<u8> = Vec::new();
            let result = service.execute(context, &RealSdkService::new(), &mut stdout);

            // validation
            assert!(result.is_err());
            assert_eq!(
                "Could not find any version file",
                result.unwrap_err().to_string()
            )
        })
    }
}
