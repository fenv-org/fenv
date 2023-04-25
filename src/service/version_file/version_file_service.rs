use anyhow::{bail, Ok};
use log::debug;

use crate::{args::FenvVersionFileArgs, service::service::Service, util::path_like::PathLike};

pub struct FenvVersionFileService {
    args: FenvVersionFileArgs,
}

impl FenvVersionFileService {
    pub fn new(args: FenvVersionFileArgs) -> Self {
        Self { args }
    }

    pub fn look_up_version_file(dir: &PathLike) -> anyhow::Result<PathLike> {
        fn has_version_file(dir: &PathLike) -> bool {
            dir.join(".flutter-version").is_file()
        }
        todo!()
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
        let version_file = FenvVersionFileService::look_up_version_file(&start_dir)?;
        debug!("Found version file `{version_file}`");
        writeln!(stdout, "{}", version_file)?;
        Ok(())
    }
}
