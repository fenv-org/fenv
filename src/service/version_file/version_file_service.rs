use crate::{
    args::FenvVersionFileArgs, context::FenvContext, service::service::Service,
    util::path_like::PathLike,
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

    pub fn look_up_version_file<'a>(
        context: &impl FenvContext,
        dir: &PathLike,
    ) -> anyhow::Result<PathLike> {
        debug!("Looking up version file in `{dir}`");
        fn version_file_of(dir: &PathLike) -> PathLike {
            dir.join(".flutter-version")
        }

        fn has_version_file(dir: &PathLike) -> bool {
            version_file_of(dir).is_file()
        }

        if has_version_file(dir) {
            debug!("Found version file in `{dir}`");
            return Ok(version_file_of(dir));
        }

        let mut current = dir.parent();
        while let Some(dir) = &current {
            debug!("Looking up version file in `{dir}`");
            if has_version_file(&dir) {
                debug!("Found version file in `{dir}`");
                return Ok(version_file_of(dir));
            }
            current = dir.parent();
        }

        debug!("Looking up the global version file");
        let global_version_file = context.fenv_global_version_file();
        if global_version_file.exists() {
            debug!("Found global version file");
            return Ok(global_version_file);
        }
        bail!("No version file found");
    }
}

impl Service for FenvVersionFileService {
    fn execute(
        &self,
        context: &impl FenvContext,
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
        let version_file = FenvVersionFileService::look_up_version_file(context, &start_dir)?;
        debug!("Found version file `{version_file}`");
        writeln!(stdout, "{version_file}")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::macros::test_with_context;
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
            service.execute(context, &mut stdout).unwrap();

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
            service.execute(context, &mut stdout).unwrap();

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
            let result = service.execute(context, &mut stdout);

            // validation
            assert!(result.is_err());
            assert_eq!("No version file found", result.unwrap_err().to_string())
        })
    }
}
