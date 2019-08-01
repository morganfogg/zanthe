mod game;

use clap::ArgMatches;
use game::state::GameState;
use simplelog::*;
use std::fs;
use std::fs::OpenOptions;

pub fn run(args: ArgMatches) -> Result<(), String> {
    let log_file = match OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open("main.log")
    {
        Ok(data) => data,
        Err(e) => {
        return Err(format!("Couldn't prepare log file: {}", e));
        }
    };

    if let Err(e) = WriteLogger::init(LevelFilter::Trace, Config::default(), log_file) {
        return Err(format!("Couldn't start logger: {}", e));
    };

    let game_file = match fs::read(args.value_of("INPUT").unwrap()) {
        Ok(file) => file,
        Err(e) => {
            return Err(format!("Couldn't open story file: {}", e));
        }
    };

    let _game_state = match GameState::new(game_file) {
        Ok(state) => state,
        Err(error) => {
            return Err(format!("Error loading story file: {}", error));
        }
    };

    Ok(())
}
