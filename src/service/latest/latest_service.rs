use crate::{
    args::FenvLatestArgs,
    context::FenvContext,
    model::{
        flutter_sdk::FlutterSdk, local_flutter_sdk::LocalFlutterSdk,
        remote_flutter_sdk::RemoteFlutterSdk,
    },
    service::{
        list_remote::list_remote_service::FenvListRemoteService, service::Service,
        versions::versions_service::FenvVersionsService,
    },
};
use anyhow::bail;
use std::result::Result::Ok;

pub struct FenvLatestService {
    pub args: FenvLatestArgs,
}

impl FenvLatestService {
    pub fn new(args: FenvLatestArgs) -> Self {
        Self { args }
    }

    pub fn latest(context: &FenvContext, prefix: &str) -> anyhow::Result<LocalFlutterSdk> {
        latest(context, prefix)
    }

    pub fn latest_remote(context: &FenvContext, prefix: &str) -> anyhow::Result<RemoteFlutterSdk> {
        latest_remote(context, prefix)
    }
}

impl Service for FenvLatestService {
    fn execute(
        &self,
        context: &FenvContext,
        stdout: &mut impl std::io::Write,
    ) -> anyhow::Result<()> {
        #[allow(deprecated)]
        let version_or_channel: anyhow::Result<String> = if self.args.from_remote || self.args.known
        {
            let latest = latest_remote(context, &self.args.prefix);
            latest
                .map(|sdk| sdk.display_name())
                .map_err(|e| anyhow::anyhow!(e))
        } else {
            let latest = latest(context, &self.args.prefix);
            latest
                .map(|sdk| sdk.display_name())
                .map_err(|e| anyhow::anyhow!(e))
        };

        if version_or_channel.is_err() && self.args.quiet {
            Ok(())
        } else if let Ok(version_or_channel) = version_or_channel {
            writeln!(stdout, "{}", version_or_channel)?;
            Ok(())
        } else {
            version_or_channel.map(|_| ())
        }
    }
}

fn latest(context: &FenvContext, prefix: &str) -> anyhow::Result<LocalFlutterSdk> {
    let sdks = FenvVersionsService::list_installed_sdks(context)?;
    let filtered_sdks = matches_prefix(&sdks, &prefix);
    match filtered_sdks.last() {
        Some(sdk) => anyhow::Ok(sdk.to_owned()),
        None => bail!("Not found any matched flutter sdk version: `{prefix}`"),
    }
}

fn latest_remote(context: &FenvContext, prefix: &str) -> anyhow::Result<RemoteFlutterSdk> {
    let sdks = FenvListRemoteService::list_remote_sdks(context)?;
    let filtered_sdks = matches_prefix(&sdks, &prefix);
    match filtered_sdks.last() {
        Some(sdk) => anyhow::Ok(sdk.to_owned()),
        None => bail!("Not found any matched flutter sdk version: `{prefix}`"),
    }
}

fn matches_prefix<T: FlutterSdk>(list: &[T], prefix: &str) -> Vec<T> {
    list.to_vec()
        .into_iter()
        .filter(|x| x.display_name().starts_with(prefix))
        .collect()
}
