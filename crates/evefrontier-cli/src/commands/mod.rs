// Module exports for CLI subcommands
//
// Each module handles a specific CLI subcommand, following the Single Responsibility Principle.
// The main.rs dispatches to these handlers, keeping the entry point focused on parsing and coordination.
//
// Note: Some modules are prepared for future wiring but not yet called from main.rs.
// The dead_code allow is temporary until the refactoring is complete.

#[allow(dead_code)]
pub mod download;
#[allow(dead_code)]
pub mod fmap;
#[allow(dead_code)]
pub mod index;
pub mod mcp;
#[allow(dead_code)]
pub mod route;
pub mod scout;
#[allow(dead_code)]
pub mod ships;
