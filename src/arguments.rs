//! This module defines command line arguments.

use std::{fmt::Debug, path::PathBuf};

use clap::Parser;
use getset::{CopyGetters, Getters};
use log::LevelFilter;

/// Es Compiler Light.
#[derive(Parser, CopyGetters, Getters, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct CommandLineArguments {
    /// The paths to the input files.
    #[arg(short = 'i', required = true)]
    #[getset(get = "pub")]
    input_files: Vec<PathBuf>,
    /// The logging level. Extensive logging might slow down software execution [possible values: TRACE, DEBUG, INFO, WARN, ERROR]
    #[arg(short, long, default_value_t = LevelFilter::Warn)]
    #[getset(get_copy = "pub")]
    log_level: LevelFilter,
}
