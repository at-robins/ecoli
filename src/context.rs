//! This module provides execution contexts.

use std::path::PathBuf;


// A context for processing SF01 data.
pub enum EsContext {
    Sf01Context{target_directory: PathBuf},
}