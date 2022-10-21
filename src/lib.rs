pub mod cli;
pub mod game;
pub mod helper;
pub mod ui;

use std::fs;

use crate::cli::{Cli, InterfaceMode};
use crate::game::Result;
use game::state::GameState;
use ui::interface::{Interface, TerminalInterface};

pub fn run(args: Cli) -> Result<()> {
    let game_file = fs::read(&args.game_file)?;

    let interface_type = args.interface.unwrap_or(InterfaceMode::Terminal);
    let mut interface: Box<dyn Interface> = match interface_type {
        InterfaceMode::Terminal => Box::new(TerminalInterface::new()?),
    };

    let mut game_state = GameState::new(game_file, interface.as_mut())?;

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
