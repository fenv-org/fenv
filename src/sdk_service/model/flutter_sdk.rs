pub trait FlutterSdk
where
    Self: Clone,
    Self: Sized,
{
    fn display_name(&self) -> String;
}
