use super::model::{local_flutter_sdk::LocalFlutterSdk, remote_flutter_sdk::RemoteFlutterSdk};
use crate::util::path_like::PathLike;

pub enum LookupResult<T> {
    Found(T),
    Err(anyhow::Error),
    None,
}

impl<T> From<Option<T>> for LookupResult<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(t) => Self::Found(t),
            None => Self::None,
        }
    }
}

impl<T> LookupResult<T> {
    pub fn is_found(&self) -> bool {
        if let LookupResult::Found(_) = self {
            true
        } else {
            false
        }
    }

    pub fn unwrap(self) -> T {
        match self {
            LookupResult::Found(t) => t,
            LookupResult::Err(e) => panic!("{}", e),
            LookupResult::None => panic!("No exists"),
        }
    }
}

#[macro_export]
macro_rules! unwrap_or_return {
    ($result: expr) => {
        match $result {
            Result::Err(e) => {
                return LookupResult::Err(e);
            }
            Result::Ok(t) => t,
        }
    };
}

pub enum VersionFileReadResult {
    NotFoundVersionFile,
    FoundButNotInstalled {
        stored_version_prefix: String,
        path_to_version_file: PathLike,
        is_global: bool,
        latest_remote_sdk: Option<RemoteFlutterSdk>,
    },
    FoundAndInstalled {
        store_version_prefix: String,
        path_to_version_file: PathLike,
        is_global: bool,
        latest_local_sdk: LocalFlutterSdk,
        path_to_sdk_root: PathLike,
    },
    Err(anyhow::Error),
}
