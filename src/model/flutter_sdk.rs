use super::{flutter_channel::FlutterChannel, flutter_version::FlutterVersion};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FlutterSdk {
    Version(FlutterVersion),
    Channel(FlutterChannel),
}
