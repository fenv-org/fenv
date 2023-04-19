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
            r"^v?(?P<major>\d+)\.(?P<minor>\d+)\.(?P<patch>\d+)(?:(?:\+|-)hotfix\.(?P<hotfix>\d+))?$"
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

#[cfg(test)]
mod tests {
    use rand::seq::SliceRandom;

    use super::FlutterVersion;

    #[test]
    fn test_parse() {
        assert_eq!(
            FlutterVersion::parse("10.231.5+hotfix.2"),
            Some(FlutterVersion {
                major: 10,
                minor: 231,
                patch: 5,
                hotfix: 2,
            })
        );
        assert_eq!(
            FlutterVersion::parse("1.0.0"),
            Some(FlutterVersion {
                major: 1,
                minor: 0,
                patch: 0,
                hotfix: 0,
            })
        );
        assert_eq!(
            FlutterVersion::parse("v2.23.40-hotfix.10"),
            Some(FlutterVersion {
                major: 2,
                minor: 23,
                patch: 40,
                hotfix: 10,
            })
        );
        assert_eq!(
            FlutterVersion::parse("v10.231.5"),
            Some(FlutterVersion {
                major: 10,
                minor: 231,
                patch: 5,
                hotfix: 0,
            })
        );
        assert_eq!(FlutterVersion::parse("unknown"), None);
    }

    #[test]
    fn parse_and_order() {
        let mut versions = vec![
            FlutterVersion::parse("10.231.5+hotfix.2"),
            FlutterVersion::parse("1.0.0"),
            FlutterVersion::parse("v2.23.40-hotfix.10"),
            FlutterVersion::parse("v10.231.5"),
            FlutterVersion::parse("unknown"),
        ];

        let mut rng = rand::thread_rng();
        versions.shuffle(&mut rng);
        println!("shuffled: {:?}", versions);

        versions.sort();
        println!("sorted: {:?}", versions);
        assert_eq!(
            vec![
                FlutterVersion::parse("unknown"),
                FlutterVersion::parse("1.0.0"),
                FlutterVersion::parse("v2.23.40-hotfix.10"),
                FlutterVersion::parse("v10.231.5"),
                FlutterVersion::parse("10.231.5+hotfix.2"),
            ],
            versions
        );
    }
}
