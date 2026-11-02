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
    Compile {
        /// The path to the project directory.
        #[arg(short = 'i', long, required = true)]
        input_path: PathBuf,
        /// The path to the build artifact directory.
        /// Defaults to the current working directory.
        #[arg(short = 'o', long)]
        output_path: Option<PathBuf>,
        /// The software execution flavour to use.
        #[arg(short = 'f', long, default_value_t=EsFlavour::SF01)]
        flavour: EsFlavour,
        /// The path to the directory containing the compiled dependencies.
        /// Defaults to the current working directory.
        #[arg(short = 'd', long)]
        dependency_path: Option<PathBuf>,
    },
    /// Decompiles an es file.
    Decompile {
        /// The path to the input file.
        #[arg(short = 'i', long, required = true)]
        input_file: PathBuf,
        /// The path to the project root directory.
        /// Defaults to the current working directory.
        #[arg(short = 'o', long)]
        output_path: Option<PathBuf>,
        /// The software execution flavour to use.
        #[arg(short = 'f', long, default_value_t=EsFlavour::SF01)]
        flavour: EsFlavour,
        /// The path to the directory containing the compiled dependencies.
        /// Defaults to the current working directory.
        #[arg(short = 'd', long)]
        dependency_path: Option<PathBuf>,
    },
}

impl Command {
    pub fn execute(&self) -> Result<(), ApplicationError> {
        match self {
            Command::Compile {
                input_path,
                output_path,
                flavour,
                dependency_path,
            } => {
                let mut container =
                    flavour
                        .es_compiler()
                        .compile_es(input_path)
                        .map_err(|err| {
                            ApplicationError::from(err)
                                .chain(format!("Failed to compile {}.", input_path.display()))
                        })?;
                let default_path = PathBuf::from(".");
                let artifact_directory = output_path.as_ref().unwrap_or(&default_path);
                container.compile(artifact_directory).map_err(|err| {
                    ApplicationError::from(err).chain(format!(
                        "Failed to compile {} to {}.",
                        input_path.display(),
                        artifact_directory.display()
                    ))
                })?;
                Ok(())
            },
            Command::Decompile {
                input_file,
                output_path,
                flavour,
                dependency_path,
            } => {
                let container = flavour.es_parser().parse_es(input_file).map_err(|err| {
                    ApplicationError::from(err)
                        .chain(format!("Failed to parse {}.", input_file.display()))
                })?;
                let default_path = PathBuf::from(".");
                let target_path = output_path.as_ref().unwrap_or(&default_path);
                container.decompile(&target_path).map_err(|err| {
                    ApplicationError::from(err).chain(format!(
                        "Failed to decompile {} to {}.",
                        input_file.display(),
                        target_path.display()
                    ))
                })?;
                Ok(())
            },
        }
    }
}
