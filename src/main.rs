use std::fs::OpenOptions;
use std::panic;

use backtrace::Backtrace;
use clap::Parser;

use tracing::{error, info};
use tracing_appender;
use tracing_subscriber;

use zanthe::cli::Cli;
use zanthe::run;

fn main() {
    let args = Cli::parse();

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
        .with_max_level(if args.debug {tracing::Level::DEBUG} else {tracing::Level::WARN})
        .init();

    panic::set_hook(Box::new(|panic_info| {
        let backtrace = Backtrace::new();
        error!("{}\n{:?}", panic_info, backtrace);
    }));


    if let Err(e) = run(args) {
        eprintln!("{}", e);
        error!("Exited with error: {}", e);
        std::process::exit(1);
    }
    info!("Exited normally");
}
