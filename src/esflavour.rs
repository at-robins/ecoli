//! This module provides different execution strategies.

use crate::{compiler::EsCompiler, esparser::EsParser};

#[derive(clap::ValueEnum, Debug, Clone, Copy)]
pub enum EsFlavour {
    SF01,
}

impl EsFlavour {
    /// Returns the respective parser.
    pub fn es_parser(&self) -> EsParser {
        match self {
            EsFlavour::SF01 => EsParser::SF01Parser,
        }
    }

    /// Returns the respective compiler.
    pub fn es_conpiler(&self) -> EsCompiler {
        match self {
            EsFlavour::SF01 => EsCompiler::SF01Compiler,
        }
    }
}

impl std::fmt::Display for EsFlavour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            EsFlavour::SF01 => "sf01",
        };
        write!(f, "{}", name)
    }
}