//! Core functionality for claude-man
//!
//! This module contains the core business logic including:
//! - Authentication (via Claude CLI)
//! - Process management
//! - Session management
//! - I/O logging

pub mod auth;
pub mod logger;
pub mod process;
pub mod session;

// Re-export commonly used items
pub use logger::SessionLogger;
pub use session::{SessionHandle, SessionRegistry};
