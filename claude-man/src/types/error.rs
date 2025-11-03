//! Error types for claude-man CLI
//!
//! This module defines all error types used throughout the application,
//! providing clear, actionable error messages for users.

use thiserror::Error;

/// Main error type for claude-man operations
#[derive(Error, Debug)]
pub enum ClaudeManError {
    /// Authentication errors
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// Missing authentication token
    #[error("CLAUDE_AUTH_TOKEN environment variable not set. Please set it before running claude-man.")]
    MissingAuthToken,

    /// Session management errors
    #[error("Session error: {0}")]
    Session(String),

    /// Session not found
    #[error("Session '{0}' not found")]
    SessionNotFound(String),

    /// Process management errors
    #[error("Process error: {0}")]
    Process(String),

    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Process spawn failed
    #[error("Failed to spawn process: {0}")]
    SpawnFailed(String),

    /// Process termination failed
    #[error("Failed to terminate process: {0}")]
    TerminationFailed(String),

    /// Log file errors
    #[error("Log error: {0}")]
    Log(String),

    /// Generic error with context
    #[error("{0}")]
    Other(String),
}

/// Convenience type alias for Results with ClaudeManError
pub type Result<T> = std::result::Result<T, ClaudeManError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ClaudeManError::MissingAuthToken;
        assert!(err.to_string().contains("CLAUDE_AUTH_TOKEN"));
    }

    #[test]
    fn test_session_not_found_error() {
        let err = ClaudeManError::SessionNotFound("TEST-001".to_string());
        assert!(err.to_string().contains("TEST-001"));
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let err: ClaudeManError = io_err.into();
        assert!(matches!(err, ClaudeManError::Io(_)));
    }
}
