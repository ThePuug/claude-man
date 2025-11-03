//! Daemon server and IPC protocol
//!
//! The daemon runs as a long-lived background process that manages
//! all Claude sessions. CLI commands communicate with the daemon via IPC.

pub mod client;
pub mod protocol;
pub mod server;

pub use client::DaemonClient;
pub use protocol::{DaemonRequest, DaemonResponse};
pub use server::DaemonServer;
