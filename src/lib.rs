pub mod game;
pub mod helper;
pub mod ui;
use std::fs;
use std::fs::OpenOptions;
use std::panic;

use anyhow::{anyhow, Context, Result};
use backtrace::Backtrace;
use clap::ArgMatches;
use tracing::error;
use tracing_appender;
use tracing_subscriber;

use game::state::GameState;
use ui::interface::{Interface, TerminalInterface};

pub fn run(args: ArgMatches) -> Result<()> {
    let log_file = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open("main.log")
        .context("Could not prepare log file")?;

    let (writer, _guard) = tracing_appender::non_blocking(log_file);
    tracing_subscriber::fmt()
        .with_writer(writer)
        .with_ansi(false)
        .with_max_level(if args.is_present("debug") {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .init();

    panic::set_hook(Box::new(|panic_info| {
        let backtrace = Backtrace::new();
        error!("{}\n{:?}", panic_info, backtrace);
    }));

    let game_file =
        fs::read(args.value_of("INPUT").unwrap()).context("Couldn't open story file.")?;

    let interface_name = args.value_of("interface").unwrap();
    let mut interface: Box<dyn Interface> = match interface_name {
        "terminal" => Box::new(TerminalInterface::new().context("Couldn't start UI")?),
        _ => return Err(anyhow!("Invalid interface")), // Should be unreachable; CLAP enforces valid parameters.
    };

    let mut game_state =
        GameState::new(game_file, interface.as_mut()).context("Error loading story file")?;

    let result = game_state.run();

    match result {
        Ok(_) => {
            interface.done()?;
        }
        Err(_) => {
            interface.quit();
        }
    };
    result
}
