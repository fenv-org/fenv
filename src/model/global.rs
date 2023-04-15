pub struct Global {
    pub debug: bool,
    pub fenv_root: String,
    pub fenv_dir: String,
}

macro_rules! verbose {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {{
        $crate::io::_print($crate::format_args_nl!($($arg)*));
    }};
}
