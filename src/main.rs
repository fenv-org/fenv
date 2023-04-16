use std::{collections::HashMap, env};

use anyhow::Error;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut env_vars: HashMap<String, String> = HashMap::new();
    for (key, value) in env::vars() {
        env_vars.insert(key, value);
    }

    let debug = args.contains(&String::from("--debug"));
    if debug {
        env::set_var("RUST_BACKTRACE", "1");
        env::set_var("RUST_LOG", "debug");
    } else {
        env::remove_var("RUST_BACKTRACE");
        env::set_var("RUST_LOG", "off");
    }

    env_logger::init();

    if let Err(err) = fenv::try_run(&args, &env_vars) {
        print_error(err, debug);
        std::process::exit(1);
    }
}

fn print_error(err: Error, debug: bool) {
    if debug {
        eprintln!("{:?}", err);
        return;
    }

    eprintln!("fenv: {}", err);
    let error_chain = err.chain().skip(1);
    if error_chain.len() > 0 {
        eprintln!();
        eprintln!("caused by:");
        error_chain.for_each(|cause| eprintln!("    {}", cause));
    }
}
