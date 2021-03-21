use std::fs::OpenOptions;
use std::panic;

use anyhow::{anyhow, Context, Result};
use backtrace::Backtrace;
use clap::{App, Arg};
use tracing::{error, info};
use tracing_appender;
use tracing_subscriber;

use zanthe::run;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let log_file = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open("main.log")
        .expect("Could not prepare log file");

    let (writer, _guard) = tracing_appender::non_blocking(log_file);
    tracing_subscriber::fmt()
        .with_writer(writer)
        .with_ansi(false)
        .with_max_level(tracing::Level::INFO)
        .init();

    panic::set_hook(Box::new(|panic_info| {
        let backtrace = Backtrace::new();
        error!("{}\n{:?}", panic_info, backtrace);
    }));

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
