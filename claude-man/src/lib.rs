//! claude-man - AI Session Orchestration Tool
//!
//! A Rust-based CLI tool that orchestrates multiple Claude AI sessions
//! to enable parallel development workflows with context coherence.
//!
//! # Overview
//!
//! claude-man implements a MANAGER-based orchestration pattern where:
//! - A MANAGER session coordinates multiple child sessions
//! - Each session has a specific role (MANAGER, ARCHITECT, DEVELOPER, STAKEHOLDER)
//! - All I/O is logged to JSONL for persistence and debugging
//! - Sessions are properly managed with cleanup to prevent orphaned processes
//!
//! # Example
//!
//! ```no_run
//! use claude_man::core::SessionRegistry;
//! use claude_man::types::Role;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() {
//!     let registry = Arc::new(SessionRegistry::new());
//!
//!     // Spawn a developer session
//!     let session_id = registry.spawn_session(
//!         Role::Developer,
//!         "Implement a fibonacci function".to_string()
//!     ).await.unwrap();
//!
//!     println!("Started session: {}", session_id);
//! }
//! ```

pub mod cli;
pub mod core;
pub mod daemon;
pub mod types;

// Re-export commonly used items for convenience
pub use core::SessionRegistry;
pub use types::{ClaudeManError, Result, Role, SessionId, SessionMetadata, SessionStatus};
