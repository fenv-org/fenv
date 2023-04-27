use crate::{
    args::FenvLatestArgs,
    model::flutter_sdk::FlutterSdk,
    service::{
        install::install_service::FenvInstallService, service::Service,
        versions::versions_service::FenvVersionsService,
    },
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
        if self.args.known {
            let sdks = match FenvInstallService::list_remote_sdks(context) {
                Ok(sdks) => sdks,
                Err(err) => return if self.args.quiet { Err(err) } else { Ok(()) },
            };
            matches_prefix(&sdks, &self.args.prefix);
        } else {
            let sdks = match FenvVersionsService::list_installed_sdks(context) {
                Ok(sdks) => sdks,
                Err(err) => return if self.args.quiet { Err(err) } else { Ok(()) },
            };
            matches_prefix(&sdks, &self.args.prefix);
        }
        todo!()
    }
}

fn matches_prefix<T: FlutterSdk>(list: &[T], prefix: &str) -> Vec<T> {
    list.to_vec()
        .into_iter()
        .filter(|x| x.display_name().starts_with(prefix))
        .collect()
}
