use anyhow::{bail, Ok};
use log::debug;

use crate::{
    args::FenvVersionFileArgs, context::FenvContext, service::service::Service,
    util::path_like::PathLike,
};

pub struct FenvVersionFileService {
    args: FenvVersionFileArgs,
}

impl FenvVersionFileService {
    pub fn new(args: FenvVersionFileArgs) -> Self {
        Self { args }
    }

    pub fn look_up_version_file(context: &FenvContext, dir: &PathLike) -> anyhow::Result<PathLike> {
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
        context: &crate::context::FenvContext,
        stdout: &mut impl std::io::Write,
    ) -> anyhow::Result<()> {
        let start_dir = match &self.args.dir {
            Some(dir) => {
                debug!("Start looking for version file from `{dir}`");
                PathLike::from(dir.as_str())
            }
            None => {
                debug!("Start looking for version file from the current directory");
                context.fenv_dir.to_owned()
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
        writeln!(stdout, "{}", version_file)?;
        Ok(())
    }
}
