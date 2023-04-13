pub mod args;

use clap::Parser;

fn main() {
    let args = args::FenvArgs::parse();
    print!("{:?}", args);
}
