// Module exports for CLI subcommands
//
// Each module handles a specific CLI subcommand, following the Single Responsibility Principle.
// The main.rs dispatches to these handlers, keeping the entry point focused on parsing and coordination.

pub mod mcp;
pub mod scout;
