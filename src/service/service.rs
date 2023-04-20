use crate::config::Config;
use anyhow::Result;

pub trait Service {
    fn execute(&self, config: &Config, stdout: &mut impl std::io::Write) -> Result<()>;
}
