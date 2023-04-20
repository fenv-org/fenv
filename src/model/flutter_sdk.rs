use anyhow::{bail, Ok, Result};

use super::{flutter_channel::FlutterChannel, flutter_version::FlutterVersion};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FlutterSdk {
    Version {
        version: FlutterVersion,
        display_name: String,
    },
    Channel(FlutterChannel),
}

impl FlutterSdk {
    pub fn parse(channel_or_version: &str) -> Result<FlutterSdk> {
        if let Some(channel) = FlutterChannel::parse(channel_or_version) {
            return Ok(FlutterSdk::Channel(channel));
        }
        if let Some(version) = FlutterVersion::parse(channel_or_version) {
            return Ok(FlutterSdk::Version {
                version,
                display_name: channel_or_version.to_owned(),
            });
        }
        bail!("Invalid Flutter SDK: `{channel_or_version}`")
    }

    pub fn display_name(&self) -> &str {
        match self {
            FlutterSdk::Version {
                version: _,
                display_name,
            } => &display_name,
            FlutterSdk::Channel(channel) => &channel.channel_name(),
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::seq::SliceRandom;

    use super::*;

    #[test]
    fn test_parse() {
        assert_eq!(
            FlutterSdk::parse("1.17.0").unwrap(),
            FlutterSdk::Version {
                version: FlutterVersion::new(1, 17, 0, 0),
                display_name: "1.17.0".to_owned()
            }
        );
        assert_eq!(
            FlutterSdk::parse("stable").unwrap(),
            FlutterSdk::Channel(FlutterChannel::Stable)
        );
    }

    #[test]
    fn test_parse_invalid() {
        let result = FlutterSdk::parse("invalid");
        assert!(result.is_err());
        match result {
            Result::Ok(_) => panic!("Should have failed"),
            Err(err) => assert_eq!(err.to_string(), "Invalid Flutter SDK: `invalid`"),
        }
    }

    #[test]
    fn test_order() {
        let mut sdks = vec![
            FlutterSdk::parse("10.231.5+hotfix.2").unwrap(),
            FlutterSdk::parse("1.0.0").unwrap(),
            FlutterSdk::parse("v2.23.40-hotfix.10").unwrap(),
            FlutterSdk::parse("v10.231.5").unwrap(),
            FlutterSdk::parse("stable").unwrap(),
            FlutterSdk::parse("beta").unwrap(),
            FlutterSdk::parse("dev").unwrap(),
            FlutterSdk::parse("master").unwrap(),
        ];

        let mut rng = rand::thread_rng();
        sdks.shuffle(&mut rng);
        println!("shuffled: {:?}", sdks);

        sdks.sort();
        println!("sorted: {:?}", sdks);
        assert_eq!(
            vec![
                FlutterSdk::parse("1.0.0").unwrap(),
                FlutterSdk::parse("v2.23.40-hotfix.10").unwrap(),
                FlutterSdk::parse("v10.231.5").unwrap(),
                FlutterSdk::parse("10.231.5+hotfix.2").unwrap(),
                FlutterSdk::parse("dev").unwrap(),
                FlutterSdk::parse("beta").unwrap(),
                FlutterSdk::parse("master").unwrap(),
                FlutterSdk::parse("stable").unwrap(),
            ],
            sdks
        );
    }
}
