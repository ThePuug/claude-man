//! Daemon server implementation
//!
//! Runs as a long-lived background process managing all Claude sessions.

use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use crate::core::SessionRegistry;
use crate::daemon::protocol::{DaemonRequest, DaemonResponse};
use crate::types::error::{ClaudeManError, Result};
use crate::types::{Role, SessionId};

/// Default daemon port
pub const DEFAULT_DAEMON_PORT: u16 = 47520;

/// Daemon server managing all sessions
pub struct DaemonServer {
    /// Session registry
    registry: Arc<SessionRegistry>,

    /// TCP port to listen on
    port: u16,

    /// Shutdown flag
    shutdown: Arc<RwLock<bool>>,
}

impl DaemonServer {
    /// Create a new daemon server
    pub fn new(port: u16) -> Self {
        Self {
            registry: Arc::new(SessionRegistry::new()),
            port,
            shutdown: Arc::new(RwLock::new(false)),
        }
    }

    /// Create a daemon server with the default port
    pub fn default() -> Self {
        Self::new(DEFAULT_DAEMON_PORT)
    }

    /// Get the daemon address
    pub fn address(&self) -> String {
        format!("127.0.0.1:{}", self.port)
    }

    /// Check if daemon should shutdown
    async fn should_shutdown(&self) -> bool {
        *self.shutdown.read().await
    }

    /// Start the daemon server
    pub async fn start(&self) -> Result<()> {
        let addr = self.address();
        info!("Starting daemon server at {}", addr);

        // Load existing sessions from disk
        self.registry.load_from_disk().await?;

        // Bind to TCP port
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| ClaudeManError::Other(format!("Failed to bind to {}: {}", addr, e)))?;

        info!("Daemon listening on {}", addr);

        // Accept connections
        loop {
            if self.should_shutdown().await {
                info!("Shutdown signal received, stopping daemon");
                break;
            }

            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let registry = self.registry.clone();
                    let shutdown = self.shutdown.clone();

                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_client(stream, registry, shutdown).await {
                            error!("Error handling client: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }

        // Cleanup
        info!("Stopping all sessions...");
        self.registry.stop_all_sessions().await?;

        info!("Daemon stopped");
        Ok(())
    }

    /// Handle a client connection
    async fn handle_client(
        stream: TcpStream,
        registry: Arc<SessionRegistry>,
        shutdown: Arc<RwLock<bool>>,
    ) -> Result<()> {
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        // Read request
        reader.read_line(&mut line).await?;
        let request: DaemonRequest = serde_json::from_str(&line.trim())
            .map_err(|e| ClaudeManError::Other(format!("Invalid request: {}", e)))?;

        debug!("Received request: {:?}", request);

        // Handle request
        let response = Self::handle_request(request, registry, shutdown).await;

        // Send response
        let response_json = serde_json::to_string(&response)?;
        writer.write_all(response_json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;

        Ok(())
    }

    /// Handle a daemon request
    async fn handle_request(
        request: DaemonRequest,
        registry: Arc<SessionRegistry>,
        shutdown: Arc<RwLock<bool>>,
    ) -> DaemonResponse {
        match request {
            DaemonRequest::Ping => {
                DaemonResponse::ok_with_message("pong".to_string())
            }

            DaemonRequest::Spawn { role, task } => {
                // Parse role
                let role = match role.parse::<Role>() {
                    Ok(r) => r,
                    Err(e) => return DaemonResponse::error(format!("Invalid role: {}", e)),
                };

                // Spawn session
                match registry.spawn_session(role, task).await {
                    Ok(session_id) => {
                        // Get PID
                        let pid = registry
                            .get_session(&session_id)
                            .await
                            .and_then(|m| m.pid)
                            .unwrap_or(0);

                        DaemonResponse::spawned(session_id, pid)
                    }
                    Err(e) => DaemonResponse::error(format!("Failed to spawn session: {}", e)),
                }
            }

            DaemonRequest::Resume { session_id, message } => {
                let session_id = SessionId::from_string(session_id);

                match registry.resume_session(session_id, message).await {
                    Ok(_) => DaemonResponse::ok_with_message("Session resumed".to_string()),
                    Err(e) => DaemonResponse::error(format!("Failed to resume session: {}", e)),
                }
            }

            DaemonRequest::List => {
                let sessions = registry.list_sessions().await;
                DaemonResponse::sessions(sessions)
            }

            DaemonRequest::Info { session_id } => {
                let session_id = SessionId::from_string(session_id);
                match registry.get_session(&session_id).await {
                    Some(metadata) => DaemonResponse::session_info(metadata),
                    None => DaemonResponse::error(format!("Session not found: {}", session_id)),
                }
            }

            DaemonRequest::Stop { session_id } => {
                let session_id = SessionId::from_string(session_id);
                match registry.stop_session(&session_id).await {
                    Ok(_) => DaemonResponse::ok_with_message(format!("Session {} stopped", session_id)),
                    Err(e) => DaemonResponse::error(format!("Failed to stop session: {}", e)),
                }
            }

            DaemonRequest::StopAll => {
                match registry.stop_all_sessions().await {
                    Ok(_) => DaemonResponse::ok_with_message("All sessions stopped".to_string()),
                    Err(e) => DaemonResponse::error(format!("Failed to stop sessions: {}", e)),
                }
            }

            DaemonRequest::Attach { session_id } => {
                let session_id = SessionId::from_string(session_id);

                // Check if session exists
                if registry.get_session(&session_id).await.is_none() {
                    return DaemonResponse::error(format!("Session not found: {}", session_id));
                }

                // Signal that attach is starting (client will handle streaming)
                DaemonResponse::ok_with_message(format!("Attaching to session {}", session_id))
            }

            DaemonRequest::Input { session_id, text } => {
                let session_id = SessionId::from_string(session_id);

                match registry.send_input(&session_id, text).await {
                    Ok(_) => DaemonResponse::ok_with_message(format!("Input sent to session {}", session_id)),
                    Err(e) => DaemonResponse::error(format!("Failed to send input: {}", e)),
                }
            }

            DaemonRequest::Shutdown => {
                info!("Shutdown requested");
                let mut s = shutdown.write().await;
                *s = true;
                DaemonResponse::ok_with_message("Daemon shutting down".to_string())
            }
        }
    }
}
