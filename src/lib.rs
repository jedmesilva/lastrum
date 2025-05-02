//! Lastrum Certifield
//! 
//! A decentralized system for custody houses to issue and validate 
//! certificates for physical assets such as gold and silver.

pub mod config;
pub mod core;
pub mod certifield;
pub mod network;
pub mod utils;
pub mod errors;

#[cfg(test)]
mod tests;
