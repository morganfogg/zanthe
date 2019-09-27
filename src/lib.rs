mod game;

use std::fs;
use std::fs::OpenOptions;

use clap::ArgMatches;
use simplelog::*;

use game::state::GameState;

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

    let mut game_state = match GameState::new(game_file) {
        Ok(state) => state,
        Err(error) => {
            return Err(format!("Error loading story file: {}", error));
        }
    };

    if let Err(e) = game_state.run() {
        return Err(format!("{}", e));
    };

    Ok(())
}
