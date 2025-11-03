//! Session management and registry
//!
//! Manages the lifecycle of Claude sessions including creation, tracking,
//! and cleanup. Maintains an in-memory registry of active sessions.

use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

use crate::core::logger::{session_log_dir, SessionLogger};
use crate::core::process::{monitor_process, spawn_claude_process, SpawnConfig};
use crate::types::error::{ClaudeManError, Result};
use crate::types::role::Role;
use crate::types::session::{SessionId, SessionMetadata};

/// Session handle containing the running process and metadata
pub struct SessionHandle {
    /// Session metadata
    pub metadata: SessionMetadata,

    /// Handle to the monitoring task
    pub task_handle: Option<JoinHandle<Result<i32>>>,

    /// Channel for sending input to the session's stdin
    pub stdin_tx: Option<mpsc::UnboundedSender<String>>,
}

impl SessionHandle {
    /// Check if the session is still running
    pub fn is_running(&self) -> bool {
        self.task_handle
            .as_ref()
            .map(|h| !h.is_finished())
            .unwrap_or(false)
    }
}

/// Session registry managing all active sessions
pub struct SessionRegistry {
    /// Map of session ID to session handle
    sessions: Arc<RwLock<HashMap<SessionId, SessionHandle>>>,

    /// Counter for generating unique session IDs per role
    role_counters: Arc<RwLock<HashMap<Role, u32>>>,
}

impl SessionRegistry {
    /// Create a new empty session registry
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            role_counters: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get role-specific context for a session
    fn get_role_context(role: Role) -> Option<String> {
        match role {
            Role::Manager => Some(r#"# MANAGER Role Context

You are a MANAGER session in claude-man. Your job is to orchestrate child sessions to accomplish complex goals.

## Setup

If claude-man commands require approval, run this once:
```bash
claude-man init  # Sets up auto-approval for orchestration
```

## Available Commands

Spawn child sessions (returns immediately, runs in background):
```bash
claude-man spawn --role DEVELOPER "<task>"
claude-man spawn --role ARCHITECT "<task>"
claude-man spawn --role STAKEHOLDER "<task>"
```

Resume sessions with additional input (use this for interactive workflows):
```bash
claude-man resume <session-id> "<message or input>"
```

Monitor sessions:
```bash
claude-man list                    # List all sessions with status
claude-man info <session-id>       # Get detailed session info
claude-man logs <session-id> -n 50 # View last 50 lines of output
claude-man attach <session-id>     # Stream live output
```

Stop sessions:
```bash
claude-man stop <session-id>
claude-man stop --all
```

## Orchestration Pattern

1. Analyze the goal and break it into tasks
2. Spawn child sessions for parallel work
3. Monitor with `claude-man list`
4. Read results with `claude-man logs <id>`
5. Spawn next wave based on results
6. Report completion to user

## Example Workflow

```bash
# Spawn architecture session
claude-man spawn --role ARCHITECT "Design auth system"

# Wait and check
claude-man list
claude-man logs ARCH-001

# Spawn parallel implementation
claude-man spawn --role DEVELOPER "Implement backend auth"
claude-man spawn --role DEVELOPER "Implement frontend auth"

# Monitor until complete
while true; do
  claude-man list
  sleep 5
done
```
"#.to_string()),
            _ => None,
        }
    }

    /// Write role context to a markdown file in the session directory
    fn write_role_context(log_dir: &std::path::Path, context: &str) -> Result<()> {
        let context_path = log_dir.join("role-context.md");
        fs::write(&context_path, context)?;
        Ok(())
    }

    /// Create .claude directory with hooks for auto-approval
    fn setup_session_claude_config(log_dir: &std::path::Path) -> Result<()> {
        let claude_dir = log_dir.join(".claude");
        let hooks_dir = claude_dir.join("hooks");
        fs::create_dir_all(&hooks_dir)?;

        // Create pre-tool-use hook that auto-approves claude-man commands
        let hook_script = r#"#!/usr/bin/env bash
# Auto-approve claude-man commands for orchestration
if echo "$TOOL_USE_JSON" | grep -q "claude-man"; then
  exit 0  # Approve
fi
exit 1  # Require approval for other commands
"#;

        let hook_path = hooks_dir.join("pre-tool-use.sh");
        fs::write(&hook_path, hook_script)?;

        // Make hook executable (Unix only, no-op on Windows)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&hook_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&hook_path, perms)?;
        }

        Ok(())
    }

    /// Load sessions from disk
    ///
    /// Scans the .claude-man/sessions directory and loads all session metadata.
    /// Only includes sessions that are marked as running and have valid PIDs.
    pub async fn load_from_disk(&self) -> Result<()> {
        use crate::core::logger::default_log_dir;
        use std::fs;

        let sessions_dir = default_log_dir();
        if !sessions_dir.exists() {
            return Ok(());
        }

        info!("Loading sessions from disk...");

        for entry in fs::read_dir(sessions_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }

            let session_dir = entry.path();
            let metadata_path = session_dir.join("metadata.json");

            if !metadata_path.exists() {
                continue;
            }

            // Load metadata
            if let Ok(metadata) = Self::load_metadata_from_path(&metadata_path) {
                // Only load if marked as running
                if metadata.status == crate::types::session::SessionStatus::Running {
                    // Check if process is still alive
                    if let Some(pid) = metadata.pid {
                        if Self::is_process_alive(pid) {
                            info!("Loaded session {} (PID: {})", metadata.id, pid);

                            // Create handle without monitoring task (process already running)
                            // Note: stdin_tx is None for recovered sessions (can't attach to existing process stdin)
                            let handle = SessionHandle {
                                metadata,
                                task_handle: None,
                                stdin_tx: None,
                            };

                            let mut sessions = self.sessions.write().await;
                            sessions.insert(handle.metadata.id.clone(), handle);
                        } else {
                            // Process is dead, update metadata
                            let mut dead_metadata = metadata;
                            dead_metadata.mark_failed();
                            let _ = self.save_metadata(&dead_metadata);
                            info!("Session {} process is dead, marked as failed", dead_metadata.id);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Load metadata from a specific path
    fn load_metadata_from_path(path: &std::path::Path) -> Result<crate::types::session::SessionMetadata> {
        let json = std::fs::read_to_string(path)?;
        let metadata: crate::types::session::SessionMetadata = serde_json::from_str(&json)?;
        Ok(metadata)
    }

    /// Check if a process is alive
    fn is_process_alive(pid: u32) -> bool {
        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;

            let nix_pid = Pid::from_raw(pid as i32);
            kill(nix_pid, Signal::SIGCONT).is_ok()
        }

        #[cfg(windows)]
        {
            // On Windows, use tasklist to check if process exists
            if let Ok(output) = std::process::Command::new("tasklist")
                .args(&["/FI", &format!("PID eq {}", pid), "/NH"])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.contains(&pid.to_string())
            } else {
                false
            }
        }
    }

    /// Generate the next session ID for a given role
    async fn next_session_id(&self, role: Role) -> SessionId {
        let mut counters = self.role_counters.write().await;
        let counter = counters.entry(role).or_insert(0);
        *counter += 1;
        SessionId::new(role, *counter)
    }

    /// Spawn a new session
    ///
    /// Creates a new session, spawns the Claude CLI process, and starts monitoring it.
    pub async fn spawn_session(&self, role: Role, task: String) -> Result<SessionId> {
        let session_id = self.next_session_id(role).await;
        let log_dir = session_log_dir(&session_id);

        info!("Spawning session {} with role {:?}", session_id, role);

        // Create session metadata
        let mut metadata = SessionMetadata::new(
            session_id.clone(),
            role,
            task.clone(),
            log_dir.clone(),
        );

        // Set up .claude directory with hooks for auto-approval
        Self::setup_session_claude_config(&log_dir)?;

        // Create logger
        let logger = SessionLogger::new(session_id.clone(), &log_dir)?;

        // Save metadata to file
        self.save_metadata(&metadata)?;

        // Write role-specific context file if applicable
        let task_with_context = if let Some(context) = Self::get_role_context(role) {
            Self::write_role_context(&log_dir, &context)?;
            format!("First, read role-context.md in your working directory for your role instructions. Then: {}", task)
        } else {
            task.clone()
        };

        // Create spawn configuration with working directory set to log dir
        let config = SpawnConfig::new(task_with_context).with_working_dir(log_dir.clone());

        // Spawn the Claude CLI process with stdin support
        let child = spawn_claude_process(config).await?;
        let pid = child.id().ok_or_else(|| {
            ClaudeManError::Process("Failed to get process ID".to_string())
        })?;

        // Update metadata with PID
        metadata.mark_started(pid);
        self.save_metadata(&metadata)?;

        // Create stdin channel for sending input to the session
        let (stdin_tx, stdin_rx) = mpsc::unbounded_channel::<String>();

        // Spawn monitoring task with registry access for metadata updates
        let session_id_clone = session_id.clone();
        let sessions_for_task = self.sessions.clone();

        let task_handle = tokio::spawn(async move {
            let exit_code = monitor_process(child, session_id_clone.clone(), logger, stdin_rx).await;

            // Update metadata in registry based on exit code
            let mut sessions = sessions_for_task.write().await;
            if let Some(handle) = sessions.get_mut(&session_id_clone) {
                match exit_code {
                    Ok(0) => handle.metadata.mark_completed(),
                    Ok(_) => handle.metadata.mark_failed(),
                    Err(_) => handle.metadata.mark_failed(),
                }
            }

            exit_code
        });

        // Create session handle with stdin sender
        let handle = SessionHandle {
            metadata,
            task_handle: Some(task_handle),
            stdin_tx: Some(stdin_tx),
        };

        // Add to registry
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), handle);

        info!("Session {} started successfully", session_id);

        Ok(session_id)
    }

    /// Spawn a child session with a parent
    ///
    /// Creates a new session as a child of an existing parent session.
    pub async fn spawn_child_session(
        &self,
        parent_id: SessionId,
        role: Role,
        task: String,
    ) -> Result<SessionId> {
        // Verify parent session exists
        if self.get_session(&parent_id).await.is_none() {
            return Err(ClaudeManError::SessionNotFound(format!(
                "Parent session not found: {}",
                parent_id
            )));
        }

        let session_id = self.next_session_id(role).await;
        let log_dir = session_log_dir(&session_id);

        info!(
            "Spawning child session {} with role {:?} (parent: {})",
            session_id, role, parent_id
        );

        // Create session metadata with parent
        let mut metadata = SessionMetadata::new_child(
            session_id.clone(),
            role,
            task.clone(),
            log_dir.clone(),
            parent_id,
        );

        // Create log directory
        fs::create_dir_all(&log_dir)?;

        // Set up .claude directory with hooks for auto-approval
        Self::setup_session_claude_config(&log_dir)?;

        // Create logger
        let logger = SessionLogger::new(session_id.clone(), &log_dir)?;

        // Save metadata to file
        self.save_metadata(&metadata)?;

        // Write role-specific context file if applicable
        let task_with_context = if let Some(context) = Self::get_role_context(role) {
            Self::write_role_context(&log_dir, &context)?;
            format!("First, read role-context.md in your working directory for your role instructions. Then: {}", task)
        } else {
            task.clone()
        };

        // Create spawn configuration with working directory set to log dir
        let config = SpawnConfig::new(task_with_context).with_working_dir(log_dir.clone());

        // Spawn the Claude CLI process with stdin support
        let child = spawn_claude_process(config).await?;
        let pid = child.id().ok_or_else(|| {
            ClaudeManError::Process("Failed to get process ID".to_string())
        })?;

        // Update metadata with PID
        metadata.mark_started(pid);
        self.save_metadata(&metadata)?;

        // Create stdin channel for sending input to the session
        let (stdin_tx, stdin_rx) = mpsc::unbounded_channel::<String>();

        // Spawn monitoring task with registry access for metadata updates
        let session_id_clone = session_id.clone();
        let sessions_for_task = self.sessions.clone();

        let task_handle = tokio::spawn(async move {
            let exit_code = monitor_process(child, session_id_clone.clone(), logger, stdin_rx).await;

            // Update metadata in registry based on exit code
            let mut sessions = sessions_for_task.write().await;
            if let Some(handle) = sessions.get_mut(&session_id_clone) {
                match exit_code {
                    Ok(0) => handle.metadata.mark_completed(),
                    Ok(_) => handle.metadata.mark_failed(),
                    Err(_) => handle.metadata.mark_failed(),
                }
            }

            exit_code
        });

        // Create session handle with stdin sender
        let handle = SessionHandle {
            metadata,
            task_handle: Some(task_handle),
            stdin_tx: Some(stdin_tx),
        };

        // Add to registry
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), handle);

        info!("Child session {} started successfully", session_id);

        Ok(session_id)
    }

    /// Resume an existing session with additional input
    ///
    /// Uses Claude's --resume flag to continue a session
    pub async fn resume_session(
        &self,
        session_id: SessionId,
        message: String,
    ) -> Result<()> {
        info!("Resuming session {} with message", session_id);

        // Get existing session metadata
        let metadata = self
            .get_session(&session_id)
            .await
            .ok_or_else(|| ClaudeManError::SessionNotFound(session_id.to_string()))?;

        let log_dir = &metadata.log_dir;

        // Create logger (will append to existing log)
        let mut logger = SessionLogger::new(session_id.clone(), log_dir)?;

        // Log that we're resuming
        logger.log_lifecycle(
            crate::types::SessionStatus::Running,
            format!("Resuming session with message: {}", message),
        )?;

        // Create spawn config for resume
        let config = SpawnConfig::new(format!("--resume {} {}", session_id, message));

        // Spawn the resume process
        let child = spawn_claude_process(config).await?;
        let pid = child.id().ok_or_else(|| {
            ClaudeManError::Process("Failed to get process ID".to_string())
        })?;

        info!("Resume process started with PID {}", pid);

        // Create stdin channel (unused but required for monitor_process signature)
        let (_stdin_tx, stdin_rx) = mpsc::unbounded_channel::<String>();

        // Monitor the resume process (this blocks until complete)
        let exit_code = monitor_process(child, session_id.clone(), logger, stdin_rx).await?;

        info!("Resume process completed with exit code: {}", exit_code);

        Ok(())
    }

    /// Get a list of all active sessions
    pub async fn list_sessions(&self) -> Vec<SessionMetadata> {
        let sessions = self.sessions.read().await;
        sessions
            .values()
            .map(|handle| handle.metadata.clone())
            .collect()
    }

    /// Get child sessions of a parent
    pub async fn get_children(&self, parent_id: &SessionId) -> Vec<SessionMetadata> {
        let sessions = self.sessions.read().await;
        sessions
            .values()
            .filter(|handle| {
                handle.metadata.parent_id.as_ref() == Some(parent_id)
            })
            .map(|handle| handle.metadata.clone())
            .collect()
    }

    /// Get metadata for a specific session
    pub async fn get_session(&self, session_id: &SessionId) -> Option<SessionMetadata> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).map(|handle| handle.metadata.clone())
    }

    /// Send input to a running session
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session
    /// * `input` - The input text to send
    pub async fn send_input(&self, session_id: &SessionId, input: String) -> Result<()> {
        info!("Sending input to session {}: {}", session_id, input);

        let sessions = self.sessions.read().await;

        let handle = sessions
            .get(session_id)
            .ok_or_else(|| ClaudeManError::SessionNotFound(session_id.to_string()))?;

        // Check if session is still active
        if !handle.metadata.is_active() {
            return Err(ClaudeManError::InvalidInput(format!(
                "Session {} is not active (status: {})",
                session_id, handle.metadata.status
            )));
        }

        // Send input through the channel
        if let Some(stdin_tx) = &handle.stdin_tx {
            stdin_tx
                .send(input)
                .map_err(|_| ClaudeManError::Process("Failed to send input: channel closed".to_string()))?;
        } else {
            return Err(ClaudeManError::Process(
                "Session stdin channel not available".to_string(),
            ));
        }

        Ok(())
    }

    /// Stop a specific session
    pub async fn stop_session(&self, session_id: &SessionId) -> Result<()> {
        info!("Stopping session {}", session_id);

        let mut sessions = self.sessions.write().await;

        let handle = sessions
            .get_mut(session_id)
            .ok_or_else(|| ClaudeManError::SessionNotFound(session_id.to_string()))?;

        // Kill the process if we have a PID
        if let Some(pid) = handle.metadata.pid {
            info!("Terminating process {} for session {}", pid, session_id);

            #[cfg(unix)]
            {
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;

                let nix_pid = Pid::from_raw(pid as i32);
                // Try SIGTERM first for graceful shutdown
                let _ = kill(nix_pid, Signal::SIGTERM);

                // Give it a moment, then SIGKILL if needed
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                let _ = kill(nix_pid, Signal::SIGKILL);
            }

            #[cfg(windows)]
            {
                // On Windows, use taskkill
                let _ = std::process::Command::new("taskkill")
                    .args(&["/F", "/PID", &pid.to_string()])
                    .output();
            }
        }

        // Abort the monitoring task if still running
        if let Some(task_handle) = handle.task_handle.take() {
            task_handle.abort();
        }

        // Update metadata
        handle.metadata.mark_stopped();
        self.save_metadata(&handle.metadata)?;

        info!("Session {} stopped", session_id);

        Ok(())
    }

    /// Stop all active sessions
    pub async fn stop_all_sessions(&self) -> Result<()> {
        info!("Stopping all sessions");

        let session_ids: Vec<SessionId> = {
            let sessions = self.sessions.read().await;
            sessions.keys().cloned().collect()
        };

        for session_id in session_ids {
            if let Err(e) = self.stop_session(&session_id).await {
                warn!("Failed to stop session {}: {}", session_id, e);
            }
        }

        Ok(())
    }

    /// Clean up completed sessions from the registry
    pub async fn cleanup_completed(&self) {
        let mut sessions = self.sessions.write().await;
        sessions.retain(|_id, handle| handle.is_running());
    }

    /// Save session metadata to disk
    fn save_metadata(&self, metadata: &SessionMetadata) -> Result<()> {
        let metadata_path = metadata.log_dir.join("metadata.json");

        // Ensure directory exists
        fs::create_dir_all(&metadata.log_dir)?;

        // Write metadata as JSON
        let json = serde_json::to_string_pretty(metadata)?;
        fs::write(metadata_path, json)?;

        debug!("Saved metadata for session {}", metadata.id);

        Ok(())
    }

    /// Load session metadata from disk
    pub fn load_metadata(session_id: &SessionId) -> Result<SessionMetadata> {
        let log_dir = session_log_dir(session_id);
        let metadata_path = log_dir.join("metadata.json");

        if !metadata_path.exists() {
            return Err(ClaudeManError::SessionNotFound(session_id.to_string()));
        }

        let json = fs::read_to_string(metadata_path)?;
        let metadata: SessionMetadata = serde_json::from_str(&json)?;

        Ok(metadata)
    }
}

impl Default for SessionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_registry_creation() {
        let registry = SessionRegistry::new();
        let sessions = registry.list_sessions().await;
        assert!(sessions.is_empty());
    }

    #[tokio::test]
    async fn test_next_session_id() {
        let registry = SessionRegistry::new();

        let id1 = registry.next_session_id(Role::Developer).await;
        assert_eq!(id1.as_str(), "DEV-001");

        let id2 = registry.next_session_id(Role::Developer).await;
        assert_eq!(id2.as_str(), "DEV-002");

        let id3 = registry.next_session_id(Role::Architect).await;
        assert_eq!(id3.as_str(), "ARCH-001");
    }

    #[test]
    fn test_save_and_load_metadata() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().join("DEV-001");

        let session_id = SessionId::from_string("DEV-001".to_string());
        let metadata = SessionMetadata::new(
            session_id.clone(),
            Role::Developer,
            "test task".to_string(),
            log_dir.clone(),
        );

        let registry = SessionRegistry::new();
        registry.save_metadata(&metadata).unwrap();

        // Verify file was created
        assert!(log_dir.join("metadata.json").exists());

        // Load it back from file directly
        let metadata_path = log_dir.join("metadata.json");
        let json = fs::read_to_string(metadata_path).unwrap();
        let loaded: SessionMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.id.as_str(), metadata.id.as_str());
        assert_eq!(loaded.task, metadata.task);
    }
}
