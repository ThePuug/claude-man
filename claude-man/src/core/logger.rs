//! I/O logging module for session persistence
//!
//! Logs all session I/O to JSONL format for debugging and future session resume.
//! Log structure: `.claude-man/sessions/{SESSION_ID}/io.log`

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::types::error::Result;
use crate::types::session::{SessionId, SessionStatus};

/// Type of I/O event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum IoEventType {
    /// Input sent to the session (stdin)
    Input,

    /// Output received from the session (stdout)
    Output,

    /// Error output received from the session (stderr)
    Error,

    /// Session lifecycle event
    Lifecycle,
}

/// A single I/O event logged to JSONL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoEvent {
    /// Timestamp when the event occurred
    pub timestamp: DateTime<Utc>,

    /// Type of event
    pub event_type: IoEventType,

    /// The actual content of the event
    pub content: String,

    /// Optional metadata (for lifecycle events, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl IoEvent {
    /// Create a new I/O event
    pub fn new(event_type: IoEventType, content: String) -> Self {
        Self {
            timestamp: Utc::now(),
            event_type,
            content,
            metadata: None,
        }
    }

    /// Create a new I/O event with metadata
    pub fn with_metadata(event_type: IoEventType, content: String, metadata: serde_json::Value) -> Self {
        Self {
            timestamp: Utc::now(),
            event_type,
            content,
            metadata: Some(metadata),
        }
    }

    /// Create a lifecycle event
    pub fn lifecycle(status: SessionStatus, message: String) -> Self {
        let metadata = serde_json::json!({
            "status": status.to_string(),
        });
        Self::with_metadata(IoEventType::Lifecycle, message, metadata)
    }
}

/// Session I/O logger
pub struct SessionLogger {
    session_id: SessionId,
    log_file: File,
    log_path: PathBuf,
}

impl SessionLogger {
    /// Create a new session logger
    ///
    /// Creates the log directory and opens the io.log file for appending
    pub fn new(session_id: SessionId, log_dir: &Path) -> Result<Self> {
        // Create log directory if it doesn't exist
        create_dir_all(log_dir)?;

        let log_path = log_dir.join("io.log");

        // Open log file in append mode
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;

        Ok(Self {
            session_id,
            log_file,
            log_path,
        })
    }

    /// Log an I/O event to the JSONL file
    pub fn log_event(&mut self, event: IoEvent) -> Result<()> {
        let json = serde_json::to_string(&event)?;
        writeln!(self.log_file, "{}", json)?;
        self.log_file.flush()?;
        Ok(())
    }

    /// Log input sent to the session
    pub fn log_input(&mut self, content: String) -> Result<()> {
        self.log_event(IoEvent::new(IoEventType::Input, content))
    }

    /// Log output received from the session
    pub fn log_output(&mut self, content: String) -> Result<()> {
        self.log_event(IoEvent::new(IoEventType::Output, content))
    }

    /// Log error output received from the session
    pub fn log_error(&mut self, content: String) -> Result<()> {
        self.log_event(IoEvent::new(IoEventType::Error, content))
    }

    /// Log a lifecycle event
    pub fn log_lifecycle(&mut self, status: SessionStatus, message: String) -> Result<()> {
        self.log_event(IoEvent::lifecycle(status, message))
    }

    /// Get the path to the log file
    pub fn log_path(&self) -> &Path {
        &self.log_path
    }

    /// Get the session ID
    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }
}

/// Get the default log directory for sessions
pub fn default_log_dir() -> PathBuf {
    PathBuf::from(".claude-man").join("sessions")
}

/// Get the log directory for a specific session
pub fn session_log_dir(session_id: &SessionId) -> PathBuf {
    default_log_dir().join(session_id.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_io_event_creation() {
        let event = IoEvent::new(IoEventType::Input, "test input".to_string());
        assert_eq!(event.event_type, IoEventType::Input);
        assert_eq!(event.content, "test input");
        assert!(event.metadata.is_none());
    }

    #[test]
    fn test_lifecycle_event() {
        let event = IoEvent::lifecycle(
            SessionStatus::Running,
            "Session started".to_string(),
        );
        assert_eq!(event.event_type, IoEventType::Lifecycle);
        assert!(event.metadata.is_some());
    }

    #[test]
    fn test_io_event_serialization() {
        let event = IoEvent::new(IoEventType::Output, "test output".to_string());
        let json = serde_json::to_string(&event).unwrap();

        // Deserialize and verify
        let deserialized: IoEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.event_type, IoEventType::Output);
        assert_eq!(deserialized.content, "test output");
    }

    #[test]
    fn test_session_logger_creation() {
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().join("DEV-001");
        let session_id = SessionId::from_string("DEV-001".to_string());

        let logger = SessionLogger::new(session_id.clone(), &log_dir).unwrap();
        assert!(log_dir.exists());
        assert!(logger.log_path().exists());
        assert_eq!(logger.session_id(), &session_id);
    }

    #[test]
    fn test_session_logger_logging() {
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().join("DEV-002");
        let session_id = SessionId::from_string("DEV-002".to_string());

        let mut logger = SessionLogger::new(session_id, &log_dir).unwrap();

        logger.log_input("test input".to_string()).unwrap();
        logger.log_output("test output".to_string()).unwrap();
        logger.log_error("test error".to_string()).unwrap();
        logger.log_lifecycle(SessionStatus::Running, "started".to_string()).unwrap();

        // Read the log file and verify
        let log_contents = fs::read_to_string(logger.log_path()).unwrap();
        let lines: Vec<&str> = log_contents.lines().collect();

        assert_eq!(lines.len(), 4);

        // Verify each line is valid JSON
        for line in lines {
            let event: IoEvent = serde_json::from_str(line).unwrap();
            assert!(!event.content.is_empty());
        }
    }

    #[test]
    fn test_session_log_dir() {
        let session_id = SessionId::from_string("DEV-003".to_string());
        let log_dir = session_log_dir(&session_id);

        assert!(log_dir.to_string_lossy().contains("DEV-003"));
        assert!(log_dir.to_string_lossy().contains(".claude-man"));
    }
}
