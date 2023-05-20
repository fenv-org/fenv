use log::debug;

use crate::{
    args::FenvUninstallArgs,
    context::FenvContext,
    sdk_service::{results::LookupResult, sdk_service::SdkService},
    service::service::Service,
    util::io::ConsoleOutput,
};

pub struct FenvUninstallService {
    pub args: FenvUninstallArgs,
}

impl FenvUninstallService {
    pub fn new(args: FenvUninstallArgs) -> Self {
        Self { args }
    }
}

impl<OUT, ERR> Service<OUT, ERR> for FenvUninstallService
where
    OUT: std::io::Write,
    ERR: std::io::Write,
{
    fn execute(
        &self,
        context: &impl FenvContext,
        sdk_service: &impl SdkService,
        output: &mut dyn ConsoleOutput<OUT, ERR>,
    ) -> anyhow::Result<()> {
        for prefix in &self.args.prefixes {
            uninstall_version(context, sdk_service, output, prefix)?
        }
        Ok(())
    }
}

fn uninstall_version<OUT, ERR>(
    context: &impl FenvContext,
    sdk_service: &impl SdkService,
    output: &mut dyn ConsoleOutput<OUT, ERR>,
    prefix: &str,
) -> anyhow::Result<()>
where
    OUT: std::io::Write,
    ERR: std::io::Write,
{
    debug!("Attempting to uninstall `{}`", prefix);
    let mut lookup_result = sdk_service.find_latest_local(context, prefix);
    if let LookupResult::None = lookup_result {
        writeln!(
            output.stderr(),
            "Could not find any installed sdk: `{}`",
            prefix
        )?;
        return anyhow::Ok(());
    }

    loop {
        match lookup_result {
            LookupResult::Found(sdk) => {
                debug!("Found sdk: `{}`", sdk);
                let result = sdk_service.uninstall(context, &sdk);
                if result.is_err() {
                    break result;
                }
                writeln!(output.stdout(), "{}", sdk)?;
                lookup_result = sdk_service.find_latest_local(context, prefix)
            }
            LookupResult::Err(err) => break Result::Err(anyhow::anyhow!(err)),
            LookupResult::None => break anyhow::Ok(()),
        }
    }
}
