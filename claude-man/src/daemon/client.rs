//! IPC client for communicating with the daemon

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

use crate::daemon::protocol::{DaemonRequest, DaemonResponse};
use crate::types::error::{ClaudeManError, Result};

/// Client for communicating with the daemon
pub struct DaemonClient {
    address: String,
}

impl DaemonClient {
    /// Create a new daemon client
    pub fn new(address: String) -> Self {
        Self { address }
    }

    /// Create a client with the default address
    pub fn default() -> Self {
        Self::new(format!("127.0.0.1:{}", crate::daemon::server::DEFAULT_DAEMON_PORT))
    }

    /// Check if daemon is running
    pub async fn is_running(&self) -> bool {
        self.send_request(DaemonRequest::Ping).await.is_ok()
    }

    /// Send a request to the daemon and receive a response
    pub async fn send_request(&self, request: DaemonRequest) -> Result<DaemonResponse> {
        // Connect to daemon
        let stream = TcpStream::connect(&self.address)
            .await
            .map_err(|e| ClaudeManError::Other(format!("Failed to connect to daemon at {}. Is it running? Error: {}", self.address, e)))?;

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        // Send request
        let request_json = serde_json::to_string(&request)?;
        writer.write_all(request_json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;

        // Read response
        let mut line = String::new();
        reader.read_line(&mut line).await?;

        let response: DaemonResponse = serde_json::from_str(&line.trim())
            .map_err(|e| ClaudeManError::Other(format!("Invalid response: {}", e)))?;

        Ok(response)
    }

    /// Spawn a session
    pub async fn spawn(&self, role: String, task: String) -> Result<DaemonResponse> {
        self.send_request(DaemonRequest::Spawn { role, task }).await
    }

    /// List sessions
    pub async fn list(&self) -> Result<DaemonResponse> {
        self.send_request(DaemonRequest::List).await
    }

    /// Get session info
    pub async fn info(&self, session_id: String) -> Result<DaemonResponse> {
        self.send_request(DaemonRequest::Info { session_id }).await
    }

    /// Stop a session
    pub async fn stop(&self, session_id: String) -> Result<DaemonResponse> {
        self.send_request(DaemonRequest::Stop { session_id }).await
    }

    /// Stop all sessions
    pub async fn stop_all(&self) -> Result<DaemonResponse> {
        self.send_request(DaemonRequest::StopAll).await
    }

    /// Send input to a running session
    pub async fn input(&self, session_id: String, text: String) -> Result<DaemonResponse> {
        self.send_request(DaemonRequest::Input { session_id, text }).await
    }

    /// Shutdown the daemon
    pub async fn shutdown(&self) -> Result<DaemonResponse> {
        self.send_request(DaemonRequest::Shutdown).await
    }
}
