use anyhow::Error;
use std::{collections::HashMap, env};

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut env_vars: HashMap<String, String> = HashMap::new();
    for (key, value) in env::vars() {
        env_vars.insert(key, value);
    }

    // Just in case,
    // if `PWD` env variable is not set on a non-BSD OS,
    // put the current directory in it.
    if !env_vars.contains_key("PWD") {
        env_vars.insert(
            String::from("PWD"),
            env::current_dir().unwrap().to_str().unwrap().to_string(),
        );
    }

    let debug = args.contains(&String::from("--debug"));
    let info = args.contains(&String::from("--info"));
    if debug {
        env::set_var("RUST_BACKTRACE", "1");
        env::set_var("RUST_LOG", "debug");
    } else if info {
        env::set_var("RUST_BACKTRACE", "1");
        env::set_var("RUST_LOG", "info");
    } else {
        env::remove_var("RUST_BACKTRACE");
        #[cfg(debug_assertions)]
        env::set_var("RUST_LOG", "info");
        #[cfg(not(debug_assertions))]
        env::set_var("RUST_LOG", "off");
    }

    env_logger::init();

    if debug {
        log::debug!("Capture environment variables:");
        for (key, value) in &env_vars {
            log::debug!("  {key}: `{value}`")
        }
    }

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
