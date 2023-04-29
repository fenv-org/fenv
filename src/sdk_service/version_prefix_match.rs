use super::model::flutter_sdk::FlutterSdk;
use lazy_static::lazy_static;
use regex::Regex;

pub fn matches_prefix<T: FlutterSdk>(list: &[T], prefix: &str) -> Vec<T> {
    let fragments = VersionFragments::parse(prefix);
    list.to_vec()
        .into_iter()
        .filter(|sdk| fragments.matches(sdk))
        .collect()
}

enum VersionFragments<'a> {
    Version(Vec<&'a str>),
    Channel(&'a str),
}

impl<'a> VersionFragments<'a> {
    fn parse(prefix: &'a str) -> Self {
        lazy_static! {
            static ref VERSION_PATTERN: Regex = Regex::new(r"^v?(\d.*)$").unwrap();
            static ref SPLITTER: Regex = Regex::new(r"((-|\+)hotfix)?\.").unwrap();
        }
        match VERSION_PATTERN.captures(prefix) {
            Some(captures) => {
                let version = captures.get(1).unwrap().as_str();
                let fragments: Vec<&str> = SPLITTER.split(version).collect();
                Self::Version(fragments)
            }
            None => Self::Channel(prefix),
        }
    }

    fn matches(&self, sdk: &impl FlutterSdk) -> bool {
        let version_or_channel = sdk.display_name();
        let sdk_fragments = VersionFragments::parse(&version_or_channel);
        match self {
            VersionFragments::Version(version_me) => match sdk_fragments {
                VersionFragments::Version(version_you) => {
                    if version_me.len() > version_you.len() {
                        false
                    } else {
                        version_you[..version_me.len()] == *version_me
                    }
                }
                VersionFragments::Channel(_) => false,
            },
            VersionFragments::Channel(channel_me) => match sdk_fragments {
                VersionFragments::Version(_) => false,
                VersionFragments::Channel(channel_you) => channel_you.starts_with(*channel_me),
            },
        }
    }
}
