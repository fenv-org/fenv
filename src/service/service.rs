use crate::{
    context::FenvContext,
    sdk_service::sdk_service::SdkService,
    util::io::{ConsoleInput, ConsoleOutput},
};

pub trait Service<OUT, ERR>
where
    OUT: std::io::Write,
    ERR: std::io::Write,
{
    fn execute(
        &self,
        context: &impl FenvContext,
        sdk_service: &impl SdkService,
        input: &mut dyn ConsoleInput,
        output: &mut dyn ConsoleOutput<OUT, ERR>,
    ) -> anyhow::Result<()>;
}
