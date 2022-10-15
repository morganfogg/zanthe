pub mod game;
pub mod helper;
pub mod ui;
pub mod cli;
use std::fs;

use anyhow::{anyhow, Context, Result};
use clap::ArgMatches;
use tracing::error;

use crate::cli::{Cli, InterfaceMode};
use game::state::GameState;
use ui::interface::{Interface, TerminalInterface};

pub fn run(args: Cli) -> Result<()> {
    let game_file =
        fs::read(&args.game_file).context("Couldn't open story file.")?;

    let interface_type = args.interface.unwrap_or(InterfaceMode::Terminal);
    let mut interface: Box<dyn Interface> = match interface_type {
        InterfaceMode::Terminal => Box::new(TerminalInterface::new().context("Couldn't start UI")?),
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
