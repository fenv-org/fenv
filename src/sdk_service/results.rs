use super::model::local_flutter_sdk::LocalFlutterSdk;

pub enum LookupResult<T> {
    Found(T),
    Err(anyhow::Error),
    None,
}

pub struct VersionFileReadResult {
    pub sdk: LocalFlutterSdk,
    pub installed: bool,
}
