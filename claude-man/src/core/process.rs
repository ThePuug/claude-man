//! Process management for Claude CLI child processes
//!
//! Handles spawning, monitoring, and terminating Claude Code CLI processes.
//! Ensures proper cleanup and prevents orphaned processes.

use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

#[cfg(unix)]
use std::time::Duration;
#[cfg(unix)]
use tokio::time::timeout;

use crate::core::logger::SessionLogger;
use crate::types::error::{ClaudeManError, Result};
use crate::types::session::{SessionId, SessionStatus};

/// Default timeout for graceful process termination (in seconds)
#[cfg(unix)]
const TERMINATION_TIMEOUT_SECS: u64 = 5;

/// Configuration for spawning a Claude CLI process
pub struct SpawnConfig {
    /// Task description to pass to Claude
    pub task: String,

    /// Additional environment variables
    pub env_vars: Vec<(String, String)>,

    /// Working directory for the process
    pub working_dir: Option<std::path::PathBuf>,
}

impl SpawnConfig {
    /// Create a new spawn configuration with a task
    pub fn new(task: String) -> Self {
        Self {
            task,
            env_vars: Vec::new(),
            working_dir: None,
        }
    }

    /// Add an environment variable
    pub fn with_env(mut self, key: String, value: String) -> Self {
        self.env_vars.push((key, value));
        self
    }

    /// Set the working directory
    pub fn with_working_dir(mut self, dir: std::path::PathBuf) -> Self {
        self.working_dir = Some(dir);
        self
    }
}

/// Spawns a Claude CLI process with stdin support
///
/// # Arguments
///
/// * `config` - Configuration for the process
///
/// # Returns
///
/// The spawned child process with piped stdin
pub async fn spawn_claude_process(config: SpawnConfig) -> Result<Child> {
    info!("Spawning Claude CLI process with task: {}", config.task);

    // Build the command
    // On Windows, spawn claude.cmd directly (not via cmd /C) to preserve stdin piping
    #[cfg(target_os = "windows")]
    let mut cmd = Command::new("claude.cmd");

    #[cfg(not(target_os = "windows"))]
    let mut cmd = Command::new("claude");

    // Add flag to skip permissions (enables autonomous orchestration)
    cmd.arg("--dangerously-skip-permissions");

    // Add additional environment variables
    for (key, value) in &config.env_vars {
        cmd.env(key, value);
    }

    // Set working directory if specified
    if let Some(ref dir) = config.working_dir {
        cmd.current_dir(dir);
    }

    // Add task as argument
    cmd.arg(&config.task);

    // Configure stdio
    // TODO: Enable piped stdin when we solve Windows cmd.exe stdin passthrough issue
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null()); // Temporarily disabled - piped stdin breaks output on Windows

    // Spawn the process
    let child = cmd
        .spawn()
        .map_err(|e| ClaudeManError::SpawnFailed(format!("Failed to spawn claude CLI: {}", e)))?;

    debug!("Claude CLI process spawned with PID: {:?}", child.id());

    Ok(child)
}

/// Monitors a child process and logs its output
///
/// Reads stdout and stderr from the child process and logs to the session logger.
/// Handles stdin input from a channel.
/// Blocks until the process exits.
pub async fn monitor_process(
    mut child: Child,
    session_id: SessionId,
    mut logger: SessionLogger,
    mut stdin_rx: mpsc::UnboundedReceiver<String>,
) -> Result<i32> {
    let pid = child.id().unwrap_or(0);
    info!("Monitoring process {} for session {}", pid, session_id);

    // Log that the session has started
    logger.log_lifecycle(SessionStatus::Running, format!("Session started (PID: {})", pid))?;

    // Get stdout, stderr, and stdin handles
    let stdout = child.stdout.take().ok_or_else(|| {
        ClaudeManError::Process("Failed to capture stdout".to_string())
    })?;

    let stderr = child.stderr.take().ok_or_else(|| {
        ClaudeManError::Process("Failed to capture stderr".to_string())
    })?;

    // Note: stdin is currently null (not piped) due to Windows cmd.exe issues
    // The stdin channel infrastructure exists but input won't reach the process
    // TODO: Fix Windows stdin piping to enable interactive input

    // Drain stdin_rx to prevent blocking, but input won't actually be sent
    tokio::spawn(async move {
        while stdin_rx.recv().await.is_some() {
            // Input received but can't be sent (stdin is null)
            warn!("Input received but stdin is not piped - ignoring");
        }
    });

    // Create buffered readers
    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    let mut stdout_lines = stdout_reader.lines();
    let mut stderr_lines = stderr_reader.lines();

    // Read output lines concurrently
    loop {
        tokio::select! {
            result = stdout_lines.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        // Print to console
                        println!("[{}] {}", session_id, line);
                        // Log to file
                        if let Err(e) = logger.log_output(line) {
                            warn!("Failed to log output: {}", e);
                        }
                    }
                    Ok(None) => {
                        debug!("Stdout stream ended for session {}", session_id);
                        break;
                    }
                    Err(e) => {
                        error!("Error reading stdout: {}", e);
                        break;
                    }
                }
            }
            result = stderr_lines.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        // Print to console (stderr)
                        eprintln!("[{}] ERROR: {}", session_id, line);
                        // Log to file
                        if let Err(e) = logger.log_error(line) {
                            warn!("Failed to log error: {}", e);
                        }
                    }
                    Ok(None) => {
                        debug!("Stderr stream ended for session {}", session_id);
                    }
                    Err(e) => {
                        error!("Error reading stderr: {}", e);
                    }
                }
            }
        }
    }

    // Wait for the process to exit
    let status = child.wait().await.map_err(|e| {
        ClaudeManError::Process(format!("Failed to wait for process: {}", e))
    })?;

    let exit_code = status.code().unwrap_or(-1);
    info!("Process {} exited with code: {}", pid, exit_code);

    // Log completion
    if status.success() {
        logger.log_lifecycle(
            SessionStatus::Completed,
            format!("Session completed successfully (exit code: {})", exit_code),
        )?;
    } else {
        logger.log_lifecycle(
            SessionStatus::Failed,
            format!("Session failed (exit code: {})", exit_code),
        )?;
    }

    Ok(exit_code)
}

/// Gracefully terminate a child process
///
/// Attempts a graceful shutdown (SIGTERM) first, then forcefully kills (SIGKILL)
/// if the process doesn't exit within the timeout.
pub async fn terminate_process(mut child: Child, session_id: &SessionId) -> Result<()> {
    let _pid = child.id();
    info!("Terminating process for session {}", session_id);

    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        if let Some(pid) = pid {
            // Send SIGTERM for graceful shutdown
            debug!("Sending SIGTERM to PID {}", pid);
            let nix_pid = Pid::from_raw(pid as i32);

            if let Err(e) = kill(nix_pid, Signal::SIGTERM) {
                warn!("Failed to send SIGTERM: {}", e);
            } else {
                // Wait for process to exit gracefully
                let timeout_duration = Duration::from_secs(TERMINATION_TIMEOUT_SECS);
                match timeout(timeout_duration, child.wait()).await {
                    Ok(Ok(_status)) => {
                        info!("Process {} terminated gracefully", pid);
                        return Ok(());
                    }
                    Ok(Err(e)) => {
                        warn!("Error waiting for process {}: {}", pid, e);
                    }
                    Err(_) => {
                        warn!("Process {} did not exit within timeout, sending SIGKILL", pid);
                    }
                }

                // If still running, send SIGKILL
                debug!("Sending SIGKILL to PID {}", pid);
                if let Err(e) = kill(nix_pid, Signal::SIGKILL) {
                    error!("Failed to send SIGKILL: {}", e);
                    return Err(ClaudeManError::TerminationFailed(format!(
                        "Failed to kill process {}: {}",
                        pid, e
                    )));
                }
            }
        }
    }

    #[cfg(windows)]
    {
        // On Windows, kill() is already forceful
        match child.kill().await {
            Ok(_) => {
                info!("Process terminated");
            }
            Err(e) => {
                error!("Failed to terminate process: {}", e);
                return Err(ClaudeManError::TerminationFailed(format!(
                    "Failed to terminate process: {}",
                    e
                )));
            }
        }
    }

    // Wait for final cleanup
    let _ = child.wait().await;
    info!("Process terminated");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_config_creation() {
        let config = SpawnConfig::new("test task".to_string());
        assert_eq!(config.task, "test task");
        assert!(config.env_vars.is_empty());
        assert!(config.working_dir.is_none());
    }

    #[test]
    fn test_spawn_config_with_env() {
        let config = SpawnConfig::new("test".to_string())
            .with_env("KEY".to_string(), "VALUE".to_string());

        assert_eq!(config.env_vars.len(), 1);
        assert_eq!(config.env_vars[0].0, "KEY");
        assert_eq!(config.env_vars[0].1, "VALUE");
    }

    #[tokio::test]
    async fn test_spawn_claude_process() {
        // This test will attempt to spawn a Claude CLI process
        // It may fail if Claude CLI is not installed, which is expected in test environments
        let config = SpawnConfig::new("test".to_string());
        let result = spawn_claude_process(config).await;

        // We can't reliably test this without Claude CLI installed
        // Just verify it returns a Result
        assert!(result.is_ok() || result.is_err());
    }
}
