use anyhow::Ok;

use crate::{args::FenvVersionFileArgs, service::service::Service, util::path_like::PathLike};

struct FenvVersionFileService {
    args: FenvVersionFileArgs,
}

impl FenvVersionFileService {
    fn new(args: FenvVersionFileArgs) -> Self {
        Self { args }
    }
}

impl Service for FenvVersionFileService {
    fn execute(
        &self,
        context: &crate::context::FenvContext,
        stdout: &mut impl std::io::Write,
    ) -> anyhow::Result<()> {
        let start_dir = match &self.args.dir {
            Some(dir) => PathLike::from(dir.as_str()),
            None => context.fenv_dir.to_owned(),
        };
        Ok(())
    }
}
