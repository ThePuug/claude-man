//! Terminal output formatting for claude-man CLI
//!
//! Provides consistent formatting for CLI output including tables,
//! success/error messages, and status displays.

use chrono::{DateTime, Utc};
use crate::types::session::SessionMetadata;

/// Format a success message with a checkmark
pub fn success(message: &str) -> String {
    format!("✓ {}", message)
}

/// Format an error message with an X
pub fn error(message: &str) -> String {
    format!("✗ {}", message)
}

/// Format an info message
pub fn info(message: &str) -> String {
    format!("ℹ {}", message)
}

/// Format a warning message
pub fn warning(message: &str) -> String {
    format!("⚠ {}", message)
}

/// Format a timestamp for display
pub fn format_timestamp(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// Format a duration in human-readable form
pub fn format_duration(duration: &chrono::Duration) -> String {
    let total_seconds = duration.num_seconds();

    if total_seconds < 60 {
        format!("{}s", total_seconds)
    } else if total_seconds < 3600 {
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{}m {}s", minutes, seconds)
    } else {
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        format!("{}h {}m", hours, minutes)
    }
}

/// Print a table of sessions
pub fn print_sessions_table(sessions: &[SessionMetadata]) {
    if sessions.is_empty() {
        println!("{}", info("No active sessions"));
        return;
    }

    // Header
    println!("{:<15} {:<12} {:<12} {:<20}", "SESSION-ID", "ROLE", "STATUS", "STARTED");
    println!("{}", "-".repeat(60));

    // Rows
    for session in sessions {
        let started = session
            .started_at
            .as_ref()
            .map(format_timestamp)
            .unwrap_or_else(|| "Not started".to_string());

        println!(
            "{:<15} {:<12} {:<12} {:<20}",
            session.id,
            session.role,
            session.status,
            started
        );
    }
}

/// Print detailed session information
pub fn print_session_details(metadata: &SessionMetadata) {
    println!("Session: {}", metadata.id);
    println!("  Role:       {}", metadata.role);
    println!("  Status:     {}", metadata.status);
    println!("  Task:       {}", metadata.task);
    println!("  Created:    {}", format_timestamp(&metadata.created_at));

    if let Some(started) = &metadata.started_at {
        println!("  Started:    {}", format_timestamp(started));
    }

    if let Some(ended) = &metadata.ended_at {
        println!("  Ended:      {}", format_timestamp(ended));
    }

    if let Some(duration) = metadata.duration() {
        println!("  Duration:   {}", format_duration(&duration));
    }

    if let Some(pid) = metadata.pid {
        println!("  PID:        {}", pid);
    }

    println!("  Log dir:    {}", metadata.log_dir.display());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::role::Role;
    use crate::types::session::SessionId;
    use std::path::PathBuf;

    #[test]
    fn test_success_format() {
        assert!(success("Test").starts_with('✓'));
        assert!(success("Test").contains("Test"));
    }

    #[test]
    fn test_error_format() {
        assert!(error("Test").starts_with('✗'));
        assert!(error("Test").contains("Test"));
    }

    #[test]
    fn test_format_duration() {
        let duration = chrono::Duration::seconds(45);
        assert_eq!(format_duration(&duration), "45s");

        let duration = chrono::Duration::seconds(125);
        assert_eq!(format_duration(&duration), "2m 5s");

        let duration = chrono::Duration::seconds(3665);
        assert_eq!(format_duration(&duration), "1h 1m");
    }

    #[test]
    fn test_format_timestamp() {
        let dt = Utc::now();
        let formatted = format_timestamp(&dt);
        assert!(formatted.contains("UTC"));
    }

    #[test]
    fn test_print_sessions_table() {
        let session_id = SessionId::new(Role::Developer, 1);
        let metadata = SessionMetadata::new(
            session_id,
            Role::Developer,
            "test".to_string(),
            PathBuf::from("/tmp"),
        );

        // This just tests that it doesn't panic
        print_sessions_table(&[metadata]);
        print_sessions_table(&[]);
    }
}
