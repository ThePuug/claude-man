//! CLI command implementations
//!
//! Implements the core CLI commands: spawn, list, stop, etc.

use crate::cli::output;
use crate::core::session::SessionRegistry;
use crate::types::error::Result;
use crate::types::role::Role;
use crate::types::session::SessionId;
use std::sync::Arc;
use tracing::info;

/// Spawn a new Claude session
///
/// # Arguments
///
/// * `registry` - The session registry
/// * `role` - The role to assign to the session
/// * `task` - The task description
pub async fn spawn_session(
    registry: Arc<SessionRegistry>,
    role: Role,
    task: String,
) -> Result<()> {
    info!("Executing spawn command: role={}, task={}", role, task);

    let session_id = registry.spawn_session(role, task).await?;

    // Get the PID from the session
    let pid = if let Some(metadata) = registry.get_session(&session_id).await {
        metadata.pid.map(|p| format!(" (PID: {})", p)).unwrap_or_default()
    } else {
        String::new()
    };

    println!("{}", output::success(&format!("Session {} started{}", session_id, pid)));
    println!();

    // Wait for the session to complete
    info!("Waiting for session {} to complete...", session_id);

    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        if let Some(metadata) = registry.get_session(&session_id).await {
            if !metadata.is_active() {
                println!();
                match metadata.status {
                    crate::types::session::SessionStatus::Completed => {
                        println!("{}", output::success(&format!("Session {} completed successfully", session_id)));
                    }
                    crate::types::session::SessionStatus::Failed => {
                        println!("{}", output::info(&format!("Session {} failed", session_id)));
                    }
                    crate::types::session::SessionStatus::Stopped => {
                        println!("{}", output::info(&format!("Session {} was stopped", session_id)));
                    }
                    _ => {}
                }
                break;
            }
        } else {
            // Session no longer in registry
            break;
        }
    }

    println!();
    println!("View logs:  claude-man logs {}", session_id);

    Ok(())
}

/// List all active sessions
///
/// # Arguments
///
/// * `registry` - The session registry
pub async fn list_sessions(registry: Arc<SessionRegistry>) -> Result<()> {
    info!("Executing list command");

    let sessions = registry.list_sessions().await;

    output::print_sessions_table(&sessions);

    Ok(())
}

/// Stop a specific session
///
/// # Arguments
///
/// * `registry` - The session registry
/// * `session_id` - The ID of the session to stop
pub async fn stop_session(
    registry: Arc<SessionRegistry>,
    session_id: SessionId,
) -> Result<()> {
    info!("Executing stop command for session {}", session_id);

    registry.stop_session(&session_id).await?;

    println!("{}", output::success(&format!("Session {} stopped", session_id)));

    Ok(())
}

/// Stop all active sessions
///
/// # Arguments
///
/// * `registry` - The session registry
pub async fn stop_all_sessions(registry: Arc<SessionRegistry>) -> Result<()> {
    info!("Executing stop-all command");

    let sessions = registry.list_sessions().await;
    let count = sessions.len();

    if count == 0 {
        println!("{}", output::info("No active sessions to stop"));
        return Ok(());
    }

    registry.stop_all_sessions().await?;

    println!("{}", output::success(&format!("Stopped {} session(s)", count)));

    Ok(())
}

/// Get detailed information about a session
///
/// # Arguments
///
/// * `registry` - The session registry
/// * `session_id` - The ID of the session
pub async fn get_session_info(
    registry: Arc<SessionRegistry>,
    session_id: SessionId,
) -> Result<()> {
    info!("Executing info command for session {}", session_id);

    let metadata = registry
        .get_session(&session_id)
        .await
        .ok_or_else(|| crate::types::error::ClaudeManError::SessionNotFound(session_id.to_string()))?;

    output::print_session_details(&metadata);

    Ok(())
}

/// View session logs
///
/// # Arguments
///
/// * `registry` - The session registry
/// * `session_id` - The ID of the session
/// * `follow` - Whether to follow the log (like tail -f)
/// * `lines` - Number of lines to show (0 for all)
pub async fn view_logs(
    registry: Arc<SessionRegistry>,
    session_id: SessionId,
    follow: bool,
    lines: usize,
) -> Result<()> {
    use crate::core::logger::{session_log_dir, IoEvent};
    use std::fs::File;
    use std::io::{BufRead, BufReader, Seek, SeekFrom};
    use tokio::time::{sleep, Duration};

    info!("Viewing logs for session {}", session_id);

    // Get the log file path
    let log_dir = session_log_dir(&session_id);
    let log_path = log_dir.join("io.log");

    if !log_path.exists() {
        return Err(crate::types::error::ClaudeManError::SessionNotFound(
            format!("Log file not found for session {}", session_id),
        ));
    }

    // Open the log file
    let mut file = File::open(&log_path)?;
    let mut reader = BufReader::new(&mut file);

    // Read all lines first
    let mut all_lines = Vec::new();
    let mut line = String::new();
    while reader.read_line(&mut line)? > 0 {
        all_lines.push(line.clone());
        line.clear();
    }

    // Determine which lines to show
    let start_idx = if lines == 0 || lines >= all_lines.len() {
        0
    } else {
        all_lines.len() - lines
    };

    // Display the selected lines
    for log_line in &all_lines[start_idx..] {
        if let Ok(event) = serde_json::from_str::<IoEvent>(log_line.trim()) {
            print_log_event(&event, &session_id);
        }
    }

    // If follow mode, keep reading new lines
    if follow {
        println!();
        println!("{}", output::info("Following log output (Ctrl+C to stop)..."));
        println!();

        // Get current position
        let mut pos = file.seek(SeekFrom::End(0))?;

        loop {
            // Check if session is still running
            if let Some(metadata) = registry.get_session(&session_id).await {
                if !metadata.is_active() {
                    println!();
                    println!("{}", output::info("Session ended, stopping log follow"));
                    break;
                }
            } else {
                break;
            }

            // Try to read new lines
            file.seek(SeekFrom::Start(pos))?;
            let mut new_reader = BufReader::new(&file);
            let mut new_line = String::new();

            while new_reader.read_line(&mut new_line)? > 0 {
                if let Ok(event) = serde_json::from_str::<IoEvent>(new_line.trim()) {
                    print_log_event(&event, &session_id);
                }
                pos += new_line.len() as u64;
                new_line.clear();
            }

            // Sleep briefly before checking again
            sleep(Duration::from_millis(200)).await;
        }
    }

    Ok(())
}

/// Print a log event to stdout
fn print_log_event(event: &crate::core::logger::IoEvent, session_id: &SessionId) {
    use crate::core::logger::IoEventType;

    match event.event_type {
        IoEventType::Output => {
            println!("[{}] {}", session_id, event.content);
        }
        IoEventType::Error => {
            eprintln!("[{} ERROR] {}", session_id, event.content);
        }
        IoEventType::Lifecycle => {
            println!("{}", output::info(&format!("[{}] {}", session_id, event.content)));
        }
        IoEventType::Input => {
            println!("{}", output::info(&format!("[{} INPUT] {}", session_id, event.content)));
        }
    }
}

/// Print a list of sessions (wrapper for daemon mode)
///
/// # Arguments
///
/// * `sessions` - A slice of session metadata
pub fn print_sessions_list(sessions: &[crate::types::SessionMetadata]) {
    output::print_sessions_table(sessions);
}

/// Print detailed session info (wrapper for daemon mode)
///
/// # Arguments
///
/// * `metadata` - The session metadata to print
pub fn print_session_info(metadata: &crate::types::SessionMetadata) {
    output::print_session_details(metadata);
}

/// Attach to a running session (view live output from beginning)
///
/// # Arguments
///
/// * `registry` - The session registry
/// * `session_id` - The ID of the session to attach to
pub async fn attach_session(
    registry: Arc<SessionRegistry>,
    session_id: SessionId,
) -> Result<()> {
    use crate::core::logger::{session_log_dir, IoEvent};
    use std::fs::File;
    use std::io::{BufRead, BufReader, Seek, SeekFrom};
    use tokio::time::{sleep, Duration};

    info!("Attaching to session {}", session_id);

    // Verify session exists
    let metadata = registry
        .get_session(&session_id)
        .await
        .ok_or_else(|| crate::types::error::ClaudeManError::SessionNotFound(session_id.to_string()))?;

    println!("{}", output::info(&format!("Attaching to session {} ({})", session_id, metadata.role)));
    println!("{}", output::info("Press Ctrl+C to detach"));
    println!();

    // Get the log file path
    let log_dir = session_log_dir(&session_id);
    let log_path = log_dir.join("io.log");

    if !log_path.exists() {
        return Err(crate::types::error::ClaudeManError::SessionNotFound(
            format!("Log file not found for session {}", session_id),
        ));
    }

    // Open the log file
    let mut file = File::open(&log_path)?;
    let mut reader = BufReader::new(&mut file);

    // Read all existing lines first
    let mut line = String::new();
    while reader.read_line(&mut line)? > 0 {
        if let Ok(event) = serde_json::from_str::<IoEvent>(line.trim()) {
            print_log_event(&event, &session_id);
        }
        line.clear();
    }

    // Get current position and start following
    let mut pos = file.seek(SeekFrom::End(0))?;

    loop {
        // Check if session is still running
        if let Some(metadata) = registry.get_session(&session_id).await {
            if !metadata.is_active() {
                println!();
                println!("{}", output::info(&format!("Session ended with status: {}", metadata.status)));
                break;
            }
        } else {
            println!();
            println!("{}", output::info("Session not found in registry"));
            break;
        }

        // Try to read new lines
        file.seek(SeekFrom::Start(pos))?;
        let mut new_reader = BufReader::new(&file);
        let mut new_line = String::new();

        while new_reader.read_line(&mut new_line)? > 0 {
            if let Ok(event) = serde_json::from_str::<IoEvent>(new_line.trim()) {
                print_log_event(&event, &session_id);
            }
            pos += new_line.len() as u64;
            new_line.clear();
        }

        // Sleep briefly before checking again
        sleep(Duration::from_millis(200)).await;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_sessions_empty() {
        let registry = Arc::new(SessionRegistry::new());
        let result = list_sessions(registry).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stop_nonexistent_session() {
        let registry = Arc::new(SessionRegistry::new());
        let session_id = SessionId::from_string("INVALID-999".to_string());

        let result = stop_session(registry, session_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_stop_all_sessions_empty() {
        let registry = Arc::new(SessionRegistry::new());
        let result = stop_all_sessions(registry).await;
        assert!(result.is_ok());
    }
}
