use std::fs::OpenOptions;
use std::panic;

use anyhow::{anyhow, Context, Result};
use backtrace::Backtrace;
use clap::{ValueEnum, Parser};
use clap::{Command, Arg, ArgAction};
use clap::builder::PossibleValuesParser;
use tracing::{error, info};
use tracing_appender;
use tracing_subscriber;

use zanthe::run;
use zanthe::cli::Cli;


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

    let args = Cli::parse();

    if let Err(e) = run(args) {
        eprintln!("{}", e);
        error!("Exited with error: {}", e);
        std::process::exit(1);
    }
    info!("Exited normally");
}
