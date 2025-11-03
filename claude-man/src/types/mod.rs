//! Type definitions for claude-man
//!
//! This module contains all the core types used throughout the application,
//! including errors, roles, and session types.

pub mod error;
pub mod role;
pub mod session;

// Re-export commonly used types
pub use error::{ClaudeManError, Result};
pub use role::Role;
pub use session::{SessionId, SessionMetadata, SessionStatus};
