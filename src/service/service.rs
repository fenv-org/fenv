use crate::{
    args::FenvSubcommands,
    config::Config,
    service::{init::init_service::FenvInitService, install::install_service::FenvInstallService},
};
use anyhow::Result;

pub trait Service {
    fn execute(&self, config: &Config) -> Result<()>;
}

impl FenvSubcommands {
    pub fn create_service(&self) -> Box<dyn Service> {
        match &self {
            FenvSubcommands::Init(sub_args) => {
                let service = FenvInitService::from(sub_args.clone());
                Box::new(service)
            }
            FenvSubcommands::Install(sub_args) => {
                let service = FenvInstallService::from(sub_args.clone());
                Box::new(service)
            }
        }
    }
}
