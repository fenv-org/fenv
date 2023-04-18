pub mod service;

pub mod init {
    pub mod init_service;
}

pub mod install {
    mod git_command;
    mod install_sdk;
    pub mod install_service;
    mod list_remote_sdk;
}
