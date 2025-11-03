//! IPC protocol definitions for daemon communication

use serde::{Deserialize, Serialize};
use crate::types::session::{SessionId, SessionMetadata};

/// Request from CLI client to daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "lowercase")]
pub enum DaemonRequest {
    /// Spawn a new session
    Spawn {
        role: String,
        task: String,
    },

    /// Resume an existing session with additional input
    Resume {
        session_id: String,
        message: String,
    },

    /// List all active sessions
    List,

    /// Get info about a specific session
    Info {
        session_id: String,
    },

    /// Stop a session
    Stop {
        session_id: String,
    },

    /// Stop all sessions
    StopAll,

    /// Attach to session output stream
    Attach {
        session_id: String,
    },

    /// Send input to a running session
    Input {
        session_id: String,
        text: String,
    },

    /// Shutdown the daemon
    Shutdown,

    /// Ping to check if daemon is alive
    Ping,
}

/// Response from daemon to CLI client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum DaemonResponse {
    /// Success response
    Ok {
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        session_id: Option<SessionId>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pid: Option<u32>,

        #[serde(skip_serializing_if = "Option::is_none")]
        sessions: Option<Vec<SessionMetadata>>,

        #[serde(skip_serializing_if = "Option::is_none")]
        session: Option<SessionMetadata>,
    },

    /// Error response
    Error {
        message: String,
    },

    /// Output event (for attach command)
    Output {
        session_id: SessionId,
        content: String,
        event_type: String,
    },

    /// Session ended (for attach command)
    SessionEnded {
        session_id: SessionId,
        exit_code: i32,
    },
}

impl DaemonResponse {
    /// Create a success response
    pub fn ok() -> Self {
        Self::Ok {
            message: None,
            session_id: None,
            pid: None,
            sessions: None,
            session: None,
        }
    }

    /// Create a success response with a message
    pub fn ok_with_message(message: String) -> Self {
        Self::Ok {
            message: Some(message),
            session_id: None,
            pid: None,
            sessions: None,
            session: None,
        }
    }

    /// Create a success response for spawn
    pub fn spawned(session_id: SessionId, pid: u32) -> Self {
        Self::Ok {
            message: None,
            session_id: Some(session_id),
            pid: Some(pid),
            sessions: None,
            session: None,
        }
    }

    /// Create a success response for list
    pub fn sessions(sessions: Vec<SessionMetadata>) -> Self {
        Self::Ok {
            message: None,
            session_id: None,
            pid: None,
            sessions: Some(sessions),
            session: None,
        }
    }

    /// Create a success response for info
    pub fn session_info(session: SessionMetadata) -> Self {
        Self::Ok {
            message: None,
            session_id: None,
            pid: None,
            sessions: None,
            session: Some(session),
        }
    }

    /// Create an error response
    pub fn error(message: String) -> Self {
        Self::Error { message }
    }

    /// Create an output event
    pub fn output(session_id: SessionId, content: String, event_type: String) -> Self {
        Self::Output {
            session_id,
            content,
            event_type,
        }
    }

    /// Create a session ended event
    pub fn session_ended(session_id: SessionId, exit_code: i32) -> Self {
        Self::SessionEnded {
            session_id,
            exit_code,
        }
    }
}
