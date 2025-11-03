//! claude-man CLI entry point
//!
//! Main entry point for the claude-man command-line interface.

use clap::{Parser, Subcommand};
use claude_man::cli::commands;
use claude_man::core::auth;
use claude_man::core::SessionRegistry;
use claude_man::daemon::{DaemonClient, DaemonServer};
use claude_man::types::{ClaudeManError, Result, Role, SessionId};
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};

/// claude-man - AI Session Orchestration Tool
#[derive(Parser)]
#[command(name = "claude-man")]
#[command(about = "Manage multiple Claude AI sessions from a single CLI", long_about = None)]
#[command(version)]
struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Spawn a new Claude session
    Spawn {
        /// Role for the session (MANAGER, ARCHITECT, DEVELOPER, STAKEHOLDER)
        #[arg(short, long)]
        role: String,

        /// Task description for the session
        task: String,
    },

    /// Resume an existing Claude session with additional input
    Resume {
        /// Session ID to resume
        session_id: String,

        /// Additional message/input to provide
        message: String,
    },

    /// List all active sessions
    List,

    /// Stop a session
    Stop {
        /// Session ID to stop, or --all to stop all sessions
        #[arg(conflicts_with = "all")]
        session_id: Option<String>,

        /// Stop all sessions
        #[arg(short, long)]
        all: bool,
    },

    /// Get detailed information about a session
    Info {
        /// Session ID
        session_id: String,
    },

    /// View session logs
    Logs {
        /// Session ID
        session_id: String,

        /// Follow log output (like tail -f)
        #[arg(short, long)]
        follow: bool,

        /// Number of lines to show (default: 50, use 0 for all)
        #[arg(short = 'n', long, default_value = "50")]
        lines: usize,
    },

    /// Attach to a running session (view live output)
    Attach {
        /// Session ID
        session_id: String,
    },

    /// Send input to a running session
    Input {
        /// Session ID
        session_id: String,

        /// Input text to send
        text: String,
    },

    /// Start the daemon server
    Daemon,

    /// Shutdown the daemon server
    Shutdown,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("claude_man=info"));

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_writer(std::io::stderr)
        .init();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Run the appropriate command
    if let Err(e) = run(cli).await {
        error!("Error: {}", e);
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    // Handle daemon commands separately (don't require auth validation)
    match &cli.command {
        Some(Commands::Daemon) => {
            // Start daemon in foreground
            let daemon = DaemonServer::default();
            println!("Starting daemon on {}", daemon.address());
            return daemon.start().await;
        }
        Some(Commands::Shutdown) => {
            // Shutdown daemon
            let client = DaemonClient::default();
            match client.shutdown().await {
                Ok(_) => {
                    println!("Daemon shut down successfully");
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("Error shutting down daemon: {}", e);
                    std::process::exit(1);
                }
            }
        }
        _ => {}
    }

    // Validate authentication for all other commands
    auth::validate_auth()?;

    // Check if daemon is running
    let client = DaemonClient::default();
    let use_daemon = client.is_running().await;

    if use_daemon {
        info!("Using daemon mode");
        return run_with_daemon(cli, client).await;
    } else {
        info!("Running in direct mode (no daemon)");
        return run_without_daemon(cli).await;
    }
}

/// Run command using daemon
async fn run_with_daemon(cli: Cli, client: DaemonClient) -> Result<()> {
    match cli.command {
        Some(Commands::Spawn { role, task }) => {
            match client.spawn(role, task).await {
                Ok(response) => {
                    use claude_man::daemon::DaemonResponse;
                    match response {
                        DaemonResponse::Ok { session_id, pid, .. } => {
                            if let Some(sid) = session_id {
                                println!("✓ Session {} started{}", sid,
                                    pid.map(|p| format!(" (PID: {})", p)).unwrap_or_default());
                                println!();
                                println!("View output: claude-man logs {}", sid);
                            }
                        }
                        DaemonResponse::Error { message } => {
                            eprintln!("Error: {}", message);
                            std::process::exit(1);
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Some(Commands::Resume { session_id, message }) => {
            match client.resume(session_id.clone(), message).await {
                Ok(response) => {
                    use claude_man::daemon::DaemonResponse;
                    match response {
                        DaemonResponse::Ok { message: Some(msg), .. } => {
                            println!("✓ {}", msg);
                        }
                        DaemonResponse::Error { message } => {
                            eprintln!("Error: {}", message);
                            std::process::exit(1);
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Some(Commands::List) => {
            match client.list().await {
                Ok(response) => {
                    use claude_man::daemon::DaemonResponse;
                    match response {
                        DaemonResponse::Ok { sessions: Some(sessions), .. } => {
                            commands::print_sessions_list(&sessions);
                        }
                        DaemonResponse::Error { message } => {
                            eprintln!("Error: {}", message);
                            std::process::exit(1);
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Some(Commands::Stop { session_id, all }) => {
            if all {
                match client.stop_all().await {
                    Ok(_) => println!("✓ All sessions stopped"),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            } else if let Some(id) = session_id {
                match client.stop(id.clone()).await {
                    Ok(_) => println!("✓ Session {} stopped", id),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                eprintln!("Must specify either session ID or --all");
                std::process::exit(1);
            }
        }

        Some(Commands::Info { session_id }) => {
            match client.info(session_id).await {
                Ok(response) => {
                    use claude_man::daemon::DaemonResponse;
                    match response {
                        DaemonResponse::Ok { session: Some(metadata), .. } => {
                            commands::print_session_info(&metadata);
                        }
                        DaemonResponse::Error { message } => {
                            eprintln!("Error: {}", message);
                            std::process::exit(1);
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Some(Commands::Logs { .. }) => {
            // Logs command reads from disk, doesn't need daemon
            return run_without_daemon(cli).await;
        }

        Some(Commands::Attach { .. }) => {
            // Attach command reads from disk, doesn't need daemon
            return run_without_daemon(cli).await;
        }

        Some(Commands::Input { session_id, text }) => {
            match client.input(session_id.clone(), text).await {
                Ok(response) => {
                    use claude_man::daemon::DaemonResponse;
                    match response {
                        DaemonResponse::Ok { message: Some(msg), .. } => {
                            println!("✓ {}", msg);
                        }
                        DaemonResponse::Error { message } => {
                            eprintln!("Error: {}", message);
                            std::process::exit(1);
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Some(Commands::Daemon) | Some(Commands::Shutdown) => {
            unreachable!("Handled above")
        }

        None => {
            eprintln!("No command specified. Use --help for usage information.");
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Run command without daemon (direct mode)
async fn run_without_daemon(cli: Cli) -> Result<()> {
    // Create session registry and load existing sessions
    let registry = Arc::new(SessionRegistry::new());
    registry.load_from_disk().await?;

    // Setup signal handlers for cleanup
    setup_signal_handlers(registry.clone())?;

    // Execute command
    match cli.command {
        Some(Commands::Spawn { role, task }) => {
            let role = role.parse::<Role>()?;
            commands::spawn_session(registry.clone(), role, task).await?;
        }

        Some(Commands::Resume { session_id, message }) => {
            let session_id = SessionId::from_string(session_id);
            registry.resume_session(session_id, message).await?;
            println!("✓ Session resumed");
        }

        Some(Commands::List) => {
            commands::list_sessions(registry.clone()).await?;
        }

        Some(Commands::Stop { session_id, all }) => {
            if all {
                commands::stop_all_sessions(registry.clone()).await?;
            } else if let Some(id) = session_id {
                let session_id = SessionId::from_string(id);
                commands::stop_session(registry.clone(), session_id).await?;
            } else {
                return Err(ClaudeManError::InvalidInput(
                    "Must specify either session ID or --all".to_string(),
                ));
            }
        }

        Some(Commands::Info { session_id }) => {
            let session_id = SessionId::from_string(session_id);
            commands::get_session_info(registry.clone(), session_id).await?;
        }

        Some(Commands::Logs { session_id, follow, lines }) => {
            let session_id = SessionId::from_string(session_id);
            commands::view_logs(registry.clone(), session_id, follow, lines).await?;
        }

        Some(Commands::Attach { session_id }) => {
            let session_id = SessionId::from_string(session_id);
            commands::attach_session(registry.clone(), session_id).await?;
        }

        Some(Commands::Input { session_id, text }) => {
            let session_id = SessionId::from_string(session_id);
            registry.send_input(&session_id, text).await?;
            println!("✓ Input sent to session {}", session_id);
        }

        Some(Commands::Daemon) | Some(Commands::Shutdown) => {
            unreachable!("Daemon commands handled earlier in run()")
        }

        None => {
            eprintln!("No command specified. Use --help for usage information.");
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Setup signal handlers for graceful shutdown
fn setup_signal_handlers(registry: Arc<SessionRegistry>) -> Result<()> {
    // Spawn a task to handle Ctrl+C for cleanup
    let registry_clone = registry.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            info!("Received shutdown signal, cleaning up sessions...");
            if let Err(e) = registry_clone.stop_all_sessions().await {
                error!("Error stopping sessions: {}", e);
            }
        }
    });

    Ok(())
}
