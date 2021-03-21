use clap::{App, Arg};
use tracing::{error, info};

use zanthe::run;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args = App::new("Zanthe")
        .version(APP_VERSION)
        .about("A Z-Machine interpreter")
        .arg(
            Arg::with_name("INPUT")
                .help("Input file")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("interface")
                .short("i")
                .help("The interface to use")
                .takes_value(true)
                .default_value("terminal")
                .possible_values(&["terminal"]),
        )
        .arg(
            Arg::with_name("debug")
                .short("d")
                .help("Enable debug logging"),
        )
        .get_matches();

    if let Err(e) = run(args) {
        eprintln!("{}", e);
        error!("Exited with error: {}", e);
        std::process::exit(1);
    }
    info!("Exited normally");
}
