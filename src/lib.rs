pub mod game;
pub mod ui;

use std::fs;
use std::fs::OpenOptions;

use anyhow::{anyhow, Context, Result};
use clap::ArgMatches;
use simplelog::ConfigBuilder;
use simplelog::*;

use game::state::GameState;
use ui::interface::{EchoInterface, Interface, TerminalInterface};

pub fn run(args: ArgMatches) -> Result<()> {
    let log_file = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open("main.log")
        .context("Could not prepare log file")?;

    let log_level = if args.is_present("debug") {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    let config = ConfigBuilder::new()
        .set_target_level(LevelFilter::Trace)
        .set_thread_level(LevelFilter::Trace)
        .build();

    WriteLogger::init(log_level, config, log_file).context("Couldn't start logger")?;

    let game_file =
        fs::read(args.value_of("INPUT").unwrap()).context("Couldn't open story file.")?;

    let interface_name = args.value_of("interface").unwrap();
    let mut interface: Box<dyn Interface> = match interface_name {
        "terminal" => Box::new(TerminalInterface::new().context("Couldn't start UI")?),
        "echo" => Box::new(EchoInterface::new()),
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
