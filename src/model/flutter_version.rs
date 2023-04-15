use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FlutterVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
    pub hotfix: u8,
}

impl FlutterVersion {
    pub fn parse(flutter_version_string: &str) -> Option<FlutterVersion> {
        lazy_static! {
          static ref PATTERN: Regex = Regex::new(
            r"^(?P<major>\d+)\.(?P<minor>\d+)\.(?P<patch>\d+)(?:(?:\+|-)hotfix\.(?P<hotfix>\d+))?$"
          )
          .unwrap();
        }

        return match PATTERN.captures(flutter_version_string) {
            Some(capture) => {
                let major = capture
                    .name("major")
                    .map(|s| s.as_str().parse::<u8>().unwrap())
                    .expect("Cannot find `major` part");
                let minor = capture
                    .name("minor")
                    .map(|s| s.as_str().parse::<u8>().unwrap())
                    .expect("Cannot find `minor` part");
                let patch = capture
                    .name("patch")
                    .map(|s| s.as_str().parse::<u8>().unwrap())
                    .expect("Cannot find `patch` part");
                let hotfix = capture
                    .name("hotfix")
                    .map(|s| s.as_str().parse::<u8>().unwrap())
                    .unwrap_or(0);
                Some(FlutterVersion {
                    major,
                    minor,
                    patch,
                    hotfix,
                })
            }
            None => None,
        };
    }
}
