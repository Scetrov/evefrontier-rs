//! EVE Frontier CLI library.
//!
//! This crate provides command-line interface utilities for the EVE Frontier
//! pathfinder, including terminal styling and output formatting.

pub mod output;
pub mod output_helpers;
pub mod terminal;

#[cfg(test)]
pub(crate) mod test_helpers;
