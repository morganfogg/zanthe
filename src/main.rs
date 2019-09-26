use clap::{App, Arg};
use log::{error, info};

use zanthe::run;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_NAME: &str = env!("CARGO_PKG_NAME");

fn main() {
    let args = App::new(APP_NAME)
        .version(APP_VERSION)
        .arg(
            Arg::with_name("INPUT")
                .help("Input file")
                .required(true)
                .index(1),
        )
        .get_matches();

    if let Err(e) = run(args) {
        eprintln!("{}", e);
        error!("Exited with error: {}", e);
        std::process::exit(1);
    }
    info!("Exited normally");
}
