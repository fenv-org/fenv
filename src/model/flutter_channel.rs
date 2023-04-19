#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FlutterChannel {
    Dev,
    Beta,
    Master,
    Stable,
}

impl FlutterChannel {
    pub fn channel_name(&self) -> &'static str {
        match self {
            FlutterChannel::Dev => "dev",
            FlutterChannel::Beta => "beta",
            FlutterChannel::Master => "master",
            FlutterChannel::Stable => "stable",
        }
    }

    pub fn parse(channel_name: &str) -> Option<FlutterChannel> {
        match channel_name {
            "dev" => Some(FlutterChannel::Dev),
            "beta" => Some(FlutterChannel::Beta),
            "master" => Some(FlutterChannel::Master),
            "stable" => Some(FlutterChannel::Stable),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::seq::SliceRandom;

    use super::FlutterChannel;

    #[test]
    fn ensure_the_correct_order() {
        let mut all_values = vec![
            FlutterChannel::Dev,
            FlutterChannel::Beta,
            FlutterChannel::Master,
            FlutterChannel::Stable,
        ];

        let mut rng = rand::thread_rng();
        all_values.shuffle(&mut rng);
        println!("shuffled: {:?}", all_values);

        all_values.sort();
        println!("sorted: {:?}", all_values);
        assert_eq!(
            vec![
                FlutterChannel::Dev,
                FlutterChannel::Beta,
                FlutterChannel::Master,
                FlutterChannel::Stable,
            ],
            all_values
        );
    }

    #[test]
    fn test_channel_names() {
        assert_eq!(FlutterChannel::Dev.channel_name(), "dev");
        assert_eq!(FlutterChannel::Beta.channel_name(), "beta");
        assert_eq!(FlutterChannel::Master.channel_name(), "master");
        assert_eq!(FlutterChannel::Stable.channel_name(), "stable");
    }
}
