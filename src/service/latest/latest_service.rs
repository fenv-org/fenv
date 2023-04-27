use crate::{
    args::FenvLatestArgs,
    service::{service::Service, versions::versions_service::FenvVersionsService},
};

pub struct FenvLatestService {
    pub args: FenvLatestArgs,
}

impl FenvLatestService {
    pub fn new(args: FenvLatestArgs) -> Self {
        Self { args }
    }
}

impl Service for FenvLatestService {
    fn execute(
        &self,
        context: &crate::context::FenvContext,
        stdout: &mut impl std::io::Write,
    ) -> anyhow::Result<()> {
        FenvVersionsService::list_installed_sdks(context);
        todo!()
    }
}

// fn matches_prefix()
