use crate::{args::FenvLocalArgs, service::service::Service};

pub struct FenvLocalService {
    args: FenvLocalArgs,
}

impl FenvLocalService {
    pub fn new(args: FenvLocalArgs) -> Self {
        Self { args }
    }
}

impl Service for FenvLocalService {
    fn execute(
        &self,
        context: &impl crate::context::FenvContext,
        stdout: &mut impl std::io::Write,
    ) -> anyhow::Result<()> {
        todo!()
    }
}
