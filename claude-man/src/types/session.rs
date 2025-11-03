//! Session types and lifecycle management
//!
//! Defines the core session data structures and state machine
//! for tracking Claude session lifecycle.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::types::role::Role;

/// Unique identifier for a session (format: {ROLE}-{sequence})
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(String);

impl SessionId {
    /// Create a new session ID from a role and sequence number
    pub fn new(role: Role, sequence: u32) -> Self {
        SessionId(format!("{}-{:03}", role.prefix(), sequence))
    }

    /// Parse a session ID from a string
    pub fn from_string(s: String) -> Self {
        SessionId(s)
    }

    /// Get the string representation of the session ID
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Session lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    /// Session has been created but not yet started
    Created,

    /// Session is actively running
    Running,

    /// Session completed successfully
    Completed,

    /// Session failed or was terminated with error
    Failed,

    /// Session was stopped by user
    Stopped,
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionStatus::Created => write!(f, "created"),
            SessionStatus::Running => write!(f, "running"),
            SessionStatus::Completed => write!(f, "completed"),
            SessionStatus::Failed => write!(f, "failed"),
            SessionStatus::Stopped => write!(f, "stopped"),
        }
    }
}

/// Session metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Unique session identifier
    pub id: SessionId,

    /// Role assigned to this session
    pub role: Role,

    /// Current status of the session
    pub status: SessionStatus,

    /// Task description provided when session was created
    pub task: String,

    /// When the session was created
    pub created_at: DateTime<Utc>,

    /// When the session started running (if applicable)
    pub started_at: Option<DateTime<Utc>>,

    /// When the session ended (if applicable)
    pub ended_at: Option<DateTime<Utc>>,

    /// Process ID of the child Claude process (if running)
    pub pid: Option<u32>,

    /// Directory where session logs are stored
    pub log_dir: PathBuf,
}

impl SessionMetadata {
    /// Create new session metadata
    pub fn new(id: SessionId, role: Role, task: String, log_dir: PathBuf) -> Self {
        Self {
            id,
            role,
            status: SessionStatus::Created,
            task,
            created_at: Utc::now(),
            started_at: None,
            ended_at: None,
            pid: None,
            log_dir,
        }
    }

    /// Mark session as started with the given PID
    pub fn mark_started(&mut self, pid: u32) {
        self.status = SessionStatus::Running;
        self.started_at = Some(Utc::now());
        self.pid = Some(pid);
    }

    /// Mark session as completed
    pub fn mark_completed(&mut self) {
        self.status = SessionStatus::Completed;
        self.ended_at = Some(Utc::now());
        self.pid = None;
    }

    /// Mark session as failed
    pub fn mark_failed(&mut self) {
        self.status = SessionStatus::Failed;
        self.ended_at = Some(Utc::now());
        self.pid = None;
    }

    /// Mark session as stopped
    pub fn mark_stopped(&mut self) {
        self.status = SessionStatus::Stopped;
        self.ended_at = Some(Utc::now());
        self.pid = None;
    }

    /// Check if session is currently active
    pub fn is_active(&self) -> bool {
        matches!(self.status, SessionStatus::Running)
    }

    /// Get the duration of the session (if ended)
    pub fn duration(&self) -> Option<chrono::Duration> {
        match (self.started_at, self.ended_at) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_id_creation() {
        let id = SessionId::new(Role::Developer, 1);
        assert_eq!(id.as_str(), "DEV-001");

        let id = SessionId::new(Role::Manager, 42);
        assert_eq!(id.as_str(), "MGR-042");
    }

    #[test]
    fn test_session_id_display() {
        let id = SessionId::new(Role::Architect, 5);
        assert_eq!(id.to_string(), "ARCH-005");
    }

    #[test]
    fn test_session_status_display() {
        assert_eq!(SessionStatus::Created.to_string(), "created");
        assert_eq!(SessionStatus::Running.to_string(), "running");
        assert_eq!(SessionStatus::Completed.to_string(), "completed");
    }

    #[test]
    fn test_session_metadata_lifecycle() {
        let id = SessionId::new(Role::Developer, 1);
        let mut metadata = SessionMetadata::new(
            id.clone(),
            Role::Developer,
            "test task".to_string(),
            PathBuf::from("/tmp/test"),
        );

        assert_eq!(metadata.status, SessionStatus::Created);
        assert!(!metadata.is_active());

        metadata.mark_started(1234);
        assert_eq!(metadata.status, SessionStatus::Running);
        assert_eq!(metadata.pid, Some(1234));
        assert!(metadata.is_active());
        assert!(metadata.started_at.is_some());

        metadata.mark_completed();
        assert_eq!(metadata.status, SessionStatus::Completed);
        assert!(!metadata.is_active());
        assert!(metadata.ended_at.is_some());
        assert!(metadata.duration().is_some());
    }

    #[test]
    fn test_session_metadata_serialization() {
        let id = SessionId::new(Role::Developer, 1);
        let metadata = SessionMetadata::new(
            id,
            Role::Developer,
            "test".to_string(),
            PathBuf::from("/tmp"),
        );

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: SessionMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(metadata.id.as_str(), deserialized.id.as_str());
        assert_eq!(metadata.role, deserialized.role);
    }
}
