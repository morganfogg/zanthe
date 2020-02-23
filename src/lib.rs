pub mod game;
pub mod ui;

use std::error::Error;
use std::fs;
use std::fs::OpenOptions;

use clap::ArgMatches;
use simplelog::*;

use game::state::GameState;
use ui::{Interface, NullInterface, TerminalInterface};

pub fn run(args: ArgMatches) -> Result<(), Box<dyn Error>> {
    let log_file = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open("main.log")
        .map_err(|e| format!("Couldn't prepare log file: {}", e))?;

    let log_level;
    if args.is_present("debug") {
        log_level = LevelFilter::Debug;
    } else {
        log_level = LevelFilter::Info;
    }

    if let Err(e) = WriteLogger::init(log_level, Config::default(), log_file) {
        return Err(format!("Couldn't start logger: {}", e).into()).into();
    };

    let game_file = fs::read(args.value_of("INPUT").unwrap())
        .map_err(|e| format!("Couldn't open story file: {}", e))?;

    let interface_name = args.value_of("interface").unwrap();
    let mut interface: Box<dyn Interface> = match interface_name {
        "terminal" => {
            Box::new(TerminalInterface::new().map_err(|e| format!("Couldn't start UI: {}", e))?)
        }
        "null" => Box::new(NullInterface::new()),
        _ => return Err("Invalid interface".into()), // Should be unreachable; CLAP enforces valid parameters.
    };

    let mut game_state = GameState::new(game_file, interface.as_mut())
        .map_err(|e| format!("Error loading story file: {}", e))?;

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
