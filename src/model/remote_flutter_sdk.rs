use super::flutter_version::FlutterVersion;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct RemoteFlutterSdk {
    pub kind: GitRefsKind,
    pub sha: String,
    pub short: String,
    pub long: String,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Clone)]
pub enum GitRefsKind {
    Tag(FlutterVersion),
    Head(String),
}

impl RemoteFlutterSdk {
    pub fn parse(line: &str) -> Option<RemoteFlutterSdk> {
        lazy_static! {
            static ref PATTERN: Regex =
                Regex::new(r"^([a-z0-9]+)\s+(refs/((?:tags)|(?:heads))/(\S+))").unwrap();
        }
        match PATTERN.captures(line) {
            Some(capture) => {
                let sha = capture.get(1).map_or("", |m| m.as_str());
                let long = capture.get(2).map_or("", |m| m.as_str());
                let tags_or_heads = capture.get(3).map_or("", |m| m.as_str());
                let short = capture.get(4).map_or("", |m| m.as_str());
                let kind = match tags_or_heads {
                    "tags" => match RemoteFlutterSdk::tag_to_version(&short) {
                        Some(flutter_version) => GitRefsKind::Tag(flutter_version),
                        None => return None,
                    },
                    "heads" => GitRefsKind::Head(String::from(short)),
                    _ => return None,
                };
                Some(RemoteFlutterSdk {
                    kind,
                    sha: String::from(sha),
                    short: String::from(short),
                    long: String::from(long),
                })
            }
            None => return None,
        }
    }

    fn tag_to_version(tag: &str) -> Option<FlutterVersion> {
        FlutterVersion::parse(&tag)
    }
}
