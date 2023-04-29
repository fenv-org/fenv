use std::fmt::Display;

pub trait FlutterSdk
where
    Self: Clone,
    Self: Sized,
    Self: Display,
{
    fn display_name(&self) -> String;
}
