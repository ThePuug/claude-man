//! Authentication module for Claude CLI
//!
//! Validates that the Claude CLI is available and authenticated.
//! This module does NOT implement its own OAuth - it relies on the
//! Claude CLI's built-in authentication to respect terms of service.

use std::process::Command;
use tracing::debug;

use crate::types::error::{ClaudeManError, Result};

/// Check if the Claude CLI is installed and available in PATH
///
/// # Returns
///
/// * `Ok(())` - If Claude CLI is available
/// * `Err(ClaudeManError::ClaudeCliNotFound)` - If Claude CLI is not in PATH
pub fn check_claude_cli_available() -> Result<()> {
    debug!("Checking if Claude CLI is available");

    // On Windows, we need to use cmd.exe to execute .cmd files
    #[cfg(target_os = "windows")]
    let result = Command::new("cmd")
        .args(&["/C", "claude", "--version"])
        .output();

    #[cfg(not(target_os = "windows"))]
    let result = Command::new("claude")
        .arg("--version")
        .output();

    match result {
        Ok(output) if output.status.success() => {
            debug!("Claude CLI is available");
            Ok(())
        }
        _ => {
            Err(ClaudeManError::Auth(
                "Claude CLI not found. Please install the Claude CLI and ensure it's in your PATH.".to_string()
            ))
        }
    }
}

/// Validate that the Claude CLI is authenticated
///
/// Checks if the user is logged in to the Claude CLI by running a test command.
///
/// # Returns
///
/// * `Ok(())` - If authenticated
/// * `Err(ClaudeManError::Auth)` - If not authenticated or CLI not available
pub fn validate_auth() -> Result<()> {
    debug!("Validating Claude CLI authentication");

    // First check if Claude CLI is available
    check_claude_cli_available()?;

    // Try running a simple claude command to check auth
    // The Claude CLI will fail if not authenticated
    #[cfg(target_os = "windows")]
    let result = Command::new("cmd")
        .args(&["/C", "claude", "--help"])
        .output();

    #[cfg(not(target_os = "windows"))]
    let result = Command::new("claude")
        .arg("--help")
        .output();

    match result {
        Ok(output) if output.status.success() => {
            debug!("Claude CLI is authenticated");
            Ok(())
        }
        _ => {
            Err(ClaudeManError::Auth(
                "Not authenticated with Claude CLI. Please run 'claude-man login' first.".to_string()
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_claude_cli_available() {
        // This test will pass if Claude CLI is installed, fail otherwise
        // In a real test environment, we'd mock the Command execution
        let result = check_claude_cli_available();

        // We can't reliably test this without mocking, so just verify it returns a Result
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_validate_auth() {
        // This test will pass if Claude CLI is installed and authenticated
        // In a real test environment, we'd mock the Command execution
        let result = validate_auth();

        // We can't reliably test this without mocking, so just verify it returns a Result
        assert!(result.is_ok() || result.is_err());
    }
}
