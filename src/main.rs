use clap::Parser;

use crate::{arguments::CommandLineArguments, error::ApplicationError};

/// Runs the application.
fn main() -> Result<(), ApplicationError> {
    // Logs any uncatched errors.
    main_internal(CommandLineArguments::try_parse(), false).map_err(|err| {
        err.log_default();
        err
    })
}

/// An internal helper function to allow easier testing and error logging.
///
/// # Parameters
///
/// * `command_line_arguments_result` - the results of parsing the command line arguments. This parameter mainly exists to allow testing.
/// * `disable_logging` - disables starting of the logger. This parameter mainly exists to allow testing.
fn main_internal(
    command_line_arguments_result: Result<CommandLineArguments, clap::Error>,
    disable_logging: bool,
) -> Result<(), ApplicationError> {
    // Tries to parse the command line arguments.
    let cl_args_result = match command_line_arguments_result {
        Ok(cl_args) => Ok(cl_args),
        Err(err) => {
            match err.kind() {
                // Returns successful after the help message has been printed
                // or an error if the printing failed.
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion => {
                    if let Err(err) = err.print() {
                        Err(ApplicationError::from(err)
                            .chain("The command line arguments could not be parsed."))
                    } else {
                        return Ok(());
                    }
                },
                // On an actual error, returns the error.
                _ => Err(ApplicationError::from(err)
                    .chain("The command line arguments could not be parsed.")),
            }
        },
    };
    // In case of an error sets a default log level to allow logging of the error.
    let log_level = cl_args_result
        .as_ref()
        .map(|cl_args| cl_args.log_level())
        .unwrap_or(log::LevelFilter::Warn);

    // Initialises the logger.
    if !disable_logging {
        env_logger::builder()
            .filter_level(log_level)
            .try_init()
            .map_err(|err| {
                ApplicationError::from(err).chain("The logger could not be initialised.")
            })?;
    }

    let command_line_arguments = cl_args_result?;
    log::debug!("Running with arguments: {:?}", command_line_arguments);

    command_line_arguments.command().execute().map_err(|err| {
        err.chain(format!("Failed to execute command {:?}", command_line_arguments.command()))
    })?;

    log::info!("Finished successfully.");
    Ok(())
}

mod arguments;
mod context;
mod error;
mod esdata;
mod esflavour;
mod esparser;
mod essource;
mod utils;
