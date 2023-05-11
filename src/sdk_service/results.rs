use super::model::local_flutter_sdk::LocalFlutterSdk;

pub enum LookupResult<T> {
    Found(T),
    Err(anyhow::Error),
    None,
}

impl<T> From<Result<T, anyhow::Error>> for LookupResult<T> {
    fn from(value: Result<T, anyhow::Error>) -> Self {
        match value {
            Ok(t) => Self::Found(t),
            Err(e) => Self::Err(e),
        }
    }
}

impl<T> From<Option<T>> for LookupResult<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(t) => Self::Found(t),
            None => Self::None,
        }
    }
}

impl<T> From<Result<Option<T>, anyhow::Error>> for LookupResult<T> {
    fn from(value: Result<Option<T>, anyhow::Error>) -> Self {
        match value {
            Ok(t_or_none) => Self::from(t_or_none),
            Err(e) => Self::Err(e),
        }
    }
}

impl<T> From<Option<Result<T, anyhow::Error>>> for LookupResult<T> {
    fn from(value: Option<Result<T, anyhow::Error>>) -> Self {
        value.transpose().into()
    }
}

impl<T> LookupResult<T> {
    pub fn none_to_err<F: FnOnce() -> anyhow::Error>(self, op: F) -> anyhow::Result<T> {
        match self {
            LookupResult::Found(t) => anyhow::Result::Ok(t),
            LookupResult::Err(e) => anyhow::Result::Err(e),
            LookupResult::None => anyhow::Result::Err(op()),
        }
    }

    pub fn to_result(self) -> anyhow::Result<Option<T>> {
        match self {
            LookupResult::Found(t) => anyhow::Result::Ok(Some(t)),
            LookupResult::Err(e) => anyhow::Result::Err(e),
            LookupResult::None => anyhow::Result::Ok(None),
        }
    }

    pub fn is_found(&self) -> bool {
        if let LookupResult::Found(_) = self {
            true
        } else {
            false
        }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, op: F) -> LookupResult<U> {
        match self {
            LookupResult::Found(t) => LookupResult::Found(op(t)),
            LookupResult::Err(e) => LookupResult::Err(e),
            LookupResult::None => LookupResult::None,
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

pub struct VersionFileReadResult {
    pub sdk: LocalFlutterSdk,
    pub installed: bool,
}
