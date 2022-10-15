use clap::{ValueEnum, Parser};
use clap::{Command, Arg, ArgAction};
use clap::builder::PossibleValuesParser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    pub game_file: String,
    #[arg(short, long)]
    pub debug: bool,
    #[arg(short, long, value_enum)]
    pub interface: Option<InterfaceMode>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum InterfaceMode {
    Terminal,
}
