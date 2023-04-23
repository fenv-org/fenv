use crate::context::FenvContext;
use anyhow::Result;

pub trait Service {
    fn execute(&self, context: &FenvContext, stdout: &mut impl std::io::Write) -> Result<()>;
}
