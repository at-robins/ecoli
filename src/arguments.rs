//! This module defines command line arguments.

use std::{fmt::Debug, path::PathBuf};

use clap::{Parser, Subcommand};
use getset::{CopyGetters, Getters};
use log::LevelFilter;

use crate::{error::ApplicationError, esflavour::EsFlavour};

/// Es Compiler Light.
#[derive(Parser, CopyGetters, Getters, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct CommandLineArguments {
    /// The logging level. Extensive logging might slow down software execution [possible values: TRACE, DEBUG, INFO, WARN, ERROR]
    #[arg(short, long, default_value_t = LevelFilter::Warn)]
    #[getset(get_copy = "pub")]
    log_level: LevelFilter,
    #[command(subcommand)]
    #[getset(get = "pub")]
    command: Command,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Compiles a project.
    Compile,
    /// Decompiles an es file.
    Decompile {
        /// The paths to the input file.
        #[arg(short = 'i', long, required = true)]
        input_file: PathBuf,
        /// The path to the project root directory.
        #[arg(short = 'o', long)]
        output_path: Option<PathBuf>,
        /// The path to the project root directory.
        #[arg(short = 'f', long, default_value_t=EsFlavour::SF01)]
        flavour: EsFlavour,
    },
}

impl Command {
    pub fn execute(&self) -> Result<(), ApplicationError> {
        match self {
            Command::Compile => todo!(),
            Command::Decompile {
                input_file,
                output_path,
                flavour,
            } => {
                flavour.es_parser().parse_es(input_file)?;
                Ok(())
            },
        }
    }
}
