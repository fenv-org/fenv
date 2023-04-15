#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        if crate::config::Config::instance().debug {
            eprintln!($($arg)*);
        }
    }
}
