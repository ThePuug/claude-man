# 0005. Rust for CLI Implementation

Date: 2025-11-03

## Status

Accepted

## Context

We need to choose an implementation language for claude-man-cli. The CLI needs to:
- Spawn and manage multiple child processes reliably
- Handle concurrent session I/O streams
- Parse and log structured data (JSONL)
- Provide responsive CLI experience
- Be cross-platform (Windows, macOS, Linux)
- Potentially be embedded in other tools (VSCode extension)

Language options considered:
1. **Node.js/TypeScript**: Familiar ecosystem, async/await, easy JSON handling
2. **Rust**: Systems programming, safety, performance, excellent async
3. **Go**: Simple concurrency, good CLI libraries, fast compilation
4. **Python**: Rapid development, extensive libraries, slower performance

## Decision

Implement claude-man-cli in **Rust**.

## Consequences

### Positive Consequences

- **Reliability**: Rust's ownership system prevents memory leaks and race conditions critical for long-running process managers
- **Performance**: Compiled binary with minimal runtime overhead, fast startup
- **Cross-platform**: Single binary distribution, no runtime dependencies
- **Process Management**: Excellent support for process spawning, signals, and cleanup
- **Async I/O**: Tokio runtime provides robust async/await for handling multiple session streams
- **Error Handling**: Result types force explicit error handling, reducing bugs
- **Type Safety**: Strong typing catches issues at compile time
- **Binary Size**: Can produce small, optimized binaries
- **Future Extension**: Can be called from VSCode extension via FFI or as subprocess
- **No GC pauses**: Predictable performance without garbage collection
- **Ecosystem**: Excellent CLI libraries (clap), logging (tracing), async (tokio)

### Negative Consequences

- **Steeper Learning Curve**: Rust has a reputation for being harder to learn
- **Slower Compilation**: Rust compiles slower than interpreted languages during development
- **Smaller Ecosystem**: Fewer libraries compared to Node.js or Python
- **Development Speed**: May take longer to implement initially vs. Node.js
- **Borrow Checker**: Can slow down rapid prototyping
- **Less Familiar**: Team may need to learn Rust

### Neutral Consequences

- **Single Responsibility**: CLI focuses on orchestration, not running Claude directly
- **Async Runtime**: Need to use Tokio, which is well-established but adds complexity
- **Error Messages**: Rust error messages are verbose but informative

## Implementation Notes

### Key Libraries

```toml
[dependencies]
# CLI and arguments
clap = { version = "4.0", features = ["derive"] }

# Async runtime for managing concurrent sessions
tokio = { version = "1.0", features = ["full"] }

# Process management
tokio-process = "0.2"

# JSON and JSONL
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Logging and tracing
tracing = "0.1"
tracing-subscriber = "0.3"

# File system operations
tokio-fs = "0.1"

# HTTP for OAuth (login command)
reqwest = { version = "0.11", features = ["json"] }

# Terminal output
colored = "2.0"
indicatif = "0.17"  # Progress bars

# Configuration
config = "0.13"
directories = "5.0"  # Standard user directories

# Error handling
anyhow = "1.0"
thiserror = "1.0"
```

### Project Structure

```
claude-man/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Core library (reusable)
│   ├── cli/                 # CLI-specific code
│   │   ├── mod.rs
│   │   ├── commands.rs      # Command implementations
│   │   └── output.rs        # Terminal output formatting
│   ├── core/                # Core logic (reusable by extension)
│   │   ├── mod.rs
│   │   ├── session.rs       # Session management
│   │   ├── manager.rs       # MANAGER session logic
│   │   ├── process.rs       # Child process handling
│   │   ├── logger.rs        # I/O logging
│   │   ├── auth.rs          # Authentication
│   │   └── config.rs        # Configuration
│   └── types/               # Shared types
│       ├── mod.rs
│       ├── session.rs       # Session metadata
│       └── role.rs          # Role definitions
└── tests/
    ├── integration/
    └── unit/
```

### Process Management Pattern

```rust
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};

async fn spawn_claude_session(role: Role) -> Result<Session> {
    let mut child = Command::new("claude")
        .env("CLAUDE_AUTH_TOKEN", auth_token)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Handle I/O streams asynchronously
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout).lines();

    tokio::spawn(async move {
        while let Some(line) = reader.next_line().await? {
            // Log and process output
        }
    });

    Ok(Session { child, ... })
}
```

### Error Handling Pattern

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClaudeManError {
    #[error("Session {0} not found")]
    SessionNotFound(String),

    #[error("Failed to spawn process: {0}")]
    ProcessError(#[from] std::io::Error),

    #[error("Authentication required. Run: claude-man login")]
    NotAuthenticated,

    #[error("Session {0} failed: {1}")]
    SessionFailed(String, String),
}

type Result<T> = std::result::Result<T, ClaudeManError>;
```

## Alternatives Considered

### Alternative 1: Node.js/TypeScript

**Why rejected**:
- Runtime dependency (need Node.js installed)
- Garbage collection pauses could affect responsiveness
- Less reliable process management (especially on Windows)
- Harder to distribute (npm install vs. single binary)
- Memory usage higher for long-running processes

### Alternative 2: Go

**Why rejected**:
- Good choice, but Rust provides better safety guarantees
- Less mature async ecosystem compared to Tokio
- Larger binary sizes
- GC pauses (though short) could affect process coordination
- Slightly weaker type system

### Alternative 3: Python

**Why rejected**:
- Slow startup time for CLI
- Runtime dependency
- Less reliable for process management
- GIL limits true parallelism
- Distribution complexity (PyInstaller, etc.)
- Async story less mature than Rust

## Migration Path

Phase 1 (MVP):
- Basic CLI structure with clap
- Session spawning and process management
- I/O logging to JSONL
- Authentication stubs (environment variable only)

Phase 2:
- MANAGER session with tool interface
- Context management
- Full authentication flow
- Status and monitoring commands

Phase 3:
- Optimization and refinement
- VSCode extension integration
- Advanced features

## Build and Distribution

### Development
```bash
cargo build
cargo run -- <command>
cargo test
```

### Release
```bash
# Cross-compilation for all platforms
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-pc-windows-gnu
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

### Distribution
- GitHub Releases with binaries for each platform
- Optional: Homebrew tap for macOS
- Optional: Cargo install for Rust users
- Optional: Chocolatey for Windows

## References

- [Rust Programming Language](https://www.rust-lang.org/)
- [Tokio Async Runtime](https://tokio.rs/)
- [Clap CLI Framework](https://docs.rs/clap/)
- [CLI Guidelines](https://clig.dev/)
- [ADR-0001: Claude CLI Wrapper](./0001-claude-cli-wrapper-architecture.md)
