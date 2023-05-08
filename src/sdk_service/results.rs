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

impl<T> LookupResult<Option<T>> {
    fn flatten(self) -> LookupResult<T> {
        match self {
            LookupResult::Found(option) => match option {
                Some(t) => LookupResult::Found(t),
                None => LookupResult::None,
            },
            LookupResult::Err(e) => LookupResult::Err(e),
            LookupResult::None => LookupResult::None,
        }
    }
}

impl<T> LookupResult<Result<T, anyhow::Error>> {
    fn flatten(self) -> LookupResult<T> {
        match self {
            LookupResult::Found(result) => match result {
                Ok(t) => LookupResult::Found(t),
                Err(e) => LookupResult::Err(e),
            },
            LookupResult::Err(e) => LookupResult::Err(e),
            LookupResult::None => LookupResult::None,
        }
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
