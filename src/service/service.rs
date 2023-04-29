use crate::context::FenvContext;

pub trait Service {
    fn execute<'a>(
        &self,
        context: &impl FenvContext<'a>,
        stdout: &mut impl std::io::Write,
    ) -> anyhow::Result<()>;
}
