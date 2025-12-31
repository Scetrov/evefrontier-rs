//! MCP (Model Context Protocol) server for EVE Frontier
//!
//! This module provides a stdio-based MCP server that exposes EVE Frontier
//! routing and system query functionality to AI assistants via the Model
//! Context Protocol.
//!
//! # Architecture
//!
//! The MCP server is organized into the following submodules:
//! - `server`: Main server initialization and lifecycle management
//! - `tools`: Tool implementations (route_plan, system_info, systems_nearby, gates_from)
//! - `resources`: Resource implementations (dataset metadata, algorithms, spatial index status)
//! - `error`: Error types and RFC 9457 problem details
//!
//! # Transport
//!
//! The server communicates via stdio using JSON-RPC 2.0 message format
//! as specified by the MCP specification. All logging is redirected to
//! stderr to prevent stdout protocol corruption.

#![allow(dead_code)] // Many items are placeholders for Phase 2+

pub mod error;
pub mod resources;
pub mod server;
pub mod tools;

pub use error::{Error, Result};
pub use server::McpServerState;
