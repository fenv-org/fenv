use anyhow::{bail, Ok, Result};

use super::{
    flutter_channel::FlutterChannel, flutter_sdk::FlutterSdk, flutter_version::FlutterVersion,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LocalFlutterSdk {
    Version {
        version: FlutterVersion,
        display_name: String,
    },
    Channel(FlutterChannel),
}

impl LocalFlutterSdk {
    pub fn parse(channel_or_version: &str) -> Result<LocalFlutterSdk> {
        if let Some(channel) = FlutterChannel::parse(channel_or_version) {
            return Ok(LocalFlutterSdk::Channel(channel));
        }
        if let Some(version) = FlutterVersion::parse(channel_or_version) {
            return Ok(LocalFlutterSdk::Version {
                version,
                display_name: channel_or_version.to_owned(),
            });
        }
        bail!("Invalid Flutter SDK: `{channel_or_version}`")
    }

    pub fn refs_name(&self) -> String {
        match self {
            LocalFlutterSdk::Version {
                version: _,
                display_name,
            } => format!("refs/tags/{display_name}"),
            LocalFlutterSdk::Channel(channel) => {
                format!("refs/heads/{channel}", channel = channel.channel_name())
            }
        }
    }
}

impl FlutterSdk for LocalFlutterSdk {
    fn display_name(&self) -> String {
        match self {
            LocalFlutterSdk::Version {
                version: _,
                display_name,
            } => display_name.clone(),
            LocalFlutterSdk::Channel(channel) => channel.channel_name().to_string(),
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
            LocalFlutterSdk::parse("1.17.0").unwrap(),
            LocalFlutterSdk::Version {
                version: FlutterVersion::new(1, 17, 0, 0),
                display_name: "1.17.0".to_owned()
            }
        );
        assert_eq!(
            LocalFlutterSdk::parse("stable").unwrap(),
            LocalFlutterSdk::Channel(FlutterChannel::Stable)
        );
    }

    #[test]
    fn test_parse_invalid() {
        let result = LocalFlutterSdk::parse("invalid");
        assert!(result.is_err());
        match result {
            Result::Ok(_) => panic!("Should have failed"),
            Err(err) => assert_eq!(err.to_string(), "Invalid Flutter SDK: `invalid`"),
        }
    }

    #[test]
    fn test_order() {
        let mut sdks = vec![
            LocalFlutterSdk::parse("10.231.5+hotfix.2").unwrap(),
            LocalFlutterSdk::parse("1.0.0").unwrap(),
            LocalFlutterSdk::parse("v2.23.40-hotfix.10").unwrap(),
            LocalFlutterSdk::parse("v10.231.5").unwrap(),
            LocalFlutterSdk::parse("stable").unwrap(),
            LocalFlutterSdk::parse("beta").unwrap(),
            LocalFlutterSdk::parse("dev").unwrap(),
            LocalFlutterSdk::parse("master").unwrap(),
        ];

        let mut rng = rand::thread_rng();
        sdks.shuffle(&mut rng);
        println!("shuffled: {:?}", sdks);

        sdks.sort();
        println!("sorted: {:?}", sdks);
        assert_eq!(
            vec![
                LocalFlutterSdk::parse("1.0.0").unwrap(),
                LocalFlutterSdk::parse("v2.23.40-hotfix.10").unwrap(),
                LocalFlutterSdk::parse("v10.231.5").unwrap(),
                LocalFlutterSdk::parse("10.231.5+hotfix.2").unwrap(),
                LocalFlutterSdk::parse("dev").unwrap(),
                LocalFlutterSdk::parse("beta").unwrap(),
                LocalFlutterSdk::parse("master").unwrap(),
                LocalFlutterSdk::parse("stable").unwrap(),
            ],
            sdks
        );
    }
}
