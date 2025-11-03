# claude-man - AI Session Orchestration Tool

> **Phase 1 MVP Complete!** 🎉

A Rust-based CLI tool that orchestrates multiple Claude AI sessions to enable parallel development workflows with context coherence.

## 📋 Overview

`claude-man` implements a MANAGER-based orchestration pattern where:
- A MANAGER session coordinates multiple child sessions
- Each session has a specific role (MANAGER, ARCHITECT, DEVELOPER, STAKEHOLDER)
- All I/O is logged to JSONL for persistence and debugging
- Sessions are properly managed with cleanup to prevent orphaned processes

## 🚀 Phase 1 MVP Features

✅ **Core Infrastructure**
- Spawn and manage Claude CLI child processes
- Session lifecycle management (create, run, stop)
- Comprehensive I/O logging to JSONL format
- Graceful process cleanup (no orphaned processes)
- Environment-based authentication

✅ **Commands Implemented**
- `claude-man spawn --role ROLE TASK` - Spawn a new session
- `claude-man list` - List all active sessions
- `claude-man stop SESSION_ID` - Stop a specific session
- `claude-man stop --all` - Stop all sessions
- `claude-man info SESSION_ID` - Get session details

✅ **Code Quality**
- 39 unit tests with >70% coverage
- Comprehensive error handling with `thiserror`
- Full rustdoc API documentation
- CI/CD pipeline with GitHub Actions
- Cross-platform support (Windows, macOS, Linux)

## 📦 Installation

### Prerequisites

1. **Rust toolchain** (1.70+)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Claude Code CLI** - Must be installed and available in PATH
   ```bash
   # Verify Claude CLI is installed
   claude --version
   ```

3. **Claude Auth Token** - Set as environment variable
   ```bash
   export CLAUDE_AUTH_TOKEN="your-token-here"
   ```

### Build from Source

```bash
# Clone the repository
git clone https://github.com/thepuug/claude-man
cd claude-man/claude-man

# Build the project
cargo build --release

# Run tests
cargo test -- --test-threads=1

# Install locally (optional)
cargo install --path .
```

## 🎯 Quick Start

### Basic Usage

```bash
# Set authentication token
export CLAUDE_AUTH_TOKEN="your-token-here"

# Spawn a development session
claude-man spawn --role DEVELOPER "implement a fibonacci function"

# List active sessions
claude-man list

# Get session details
claude-man info DEV-001

# Stop a session
claude-man stop DEV-001

# Stop all sessions
claude-man stop --all
```

### Example Session Workflow

```bash
# Spawn a developer session
$ claude-man spawn --role DEVELOPER "create a REST API for user management"
✓ Session DEV-001 started
Monitoring session... (Ctrl+C to stop)

[DEV-001] I'll create a REST API for user management...
[DEV-001] Creating routes for CRUD operations...
[DEV-001] <complete>

✓ Session DEV-001 completed successfully

# Check the session logs
$ ls -la .claude-man/sessions/DEV-001/
io.log          # JSONL log of all I/O
metadata.json   # Session metadata
```

## 📁 Project Structure

```
claude-man/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library exports
│   ├── cli/
│   │   ├── commands.rs      # Command implementations
│   │   └── output.rs        # Output formatting
│   ├── core/
│   │   ├── auth.rs          # Authentication
│   │   ├── session.rs       # Session management
│   │   ├── process.rs       # Process management
│   │   └── logger.rs        # I/O logging
│   └── types/
│       ├── error.rs         # Error types
│       ├── role.rs          # Role enum
│       └── session.rs       # Session types
├── tests/                   # Integration tests
├── Cargo.toml              # Dependencies
└── README.md               # This file
```

## 🔧 Configuration

### Environment Variables

- `CLAUDE_AUTH_TOKEN` (required) - Claude authentication token
- `RUST_LOG` (optional) - Logging level (e.g., `claude_man=debug`)

### Session Logs

All session I/O is logged to `.claude-man/sessions/{SESSION_ID}/`:
- `io.log` - JSONL format log of all input/output
- `metadata.json` - Session metadata (role, task, timestamps, etc.)

## 📚 Available Roles

- **MANAGER** - Orchestrates and coordinates other sessions
- **ARCHITECT** - Designs system architecture and technical decisions
- **DEVELOPER** - Implements code and features
- **STAKEHOLDER** - Represents business requirements and validation

## 🧪 Testing

```bash
# Run all tests
cargo test -- --test-threads=1

# Run tests with coverage
cargo install cargo-tarpaulin
cargo tarpaulin --verbose --all-features --workspace --timeout 120 --out Html

# Run specific test
cargo test test_spawn_session -- --nocapture

# Run clippy linter
cargo clippy -- -D warnings

# Format code
cargo fmt
```

## 🚦 CI/CD

The project includes a GitHub Actions workflow that:
- Runs on push to `main` and `develop` branches
- Tests on Windows, macOS, and Linux
- Runs `cargo fmt`, `cargo clippy`, and `cargo test`
- Generates code coverage reports with `cargo-tarpaulin`

## 🐛 Troubleshooting

### "CLAUDE_AUTH_TOKEN environment variable not set"
```bash
export CLAUDE_AUTH_TOKEN="your-token-here"
```

### "Session not found"
Check active sessions with `claude-man list` and verify the session ID.

### Process not terminating
Use `claude-man stop --all` to force stop all sessions.

### Tests failing due to environment variables
Run tests with single thread: `cargo test -- --test-threads=1`

## 📖 Documentation

Generate and view the full API documentation:

```bash
cargo doc --open
```

## 🗺️ Roadmap

### Phase 2 - Full Orchestration (Planned)
- MANAGER tool interface (spawn_session, read_artifact, etc.)
- Multiple concurrent sessions
- Context management (loading roles, specs, ADRs)
- Artifact reading and generation
- OAuth authentication flow
- Configuration file support

### Phase 3 - Production Ready (Planned)
- Session persistence and resume
- Monitoring commands (status, logs, report)
- Advanced error handling and recovery
- Performance optimization

### Phase 4 - Advanced (Future)
- Workflow engine with dependency management
- Parallel execution with conflict detection
- VSCode extension integration

## 📝 License

MIT

## 🤝 Contributing

This is a prototype/MVP project. See [CLAUDE.md](../CLAUDE.md) for project rules and contribution guidelines.

## 📞 Support

For issues and feature requests, please use the GitHub issue tracker.

## 🏆 Phase 1 Acceptance Criteria

All Phase 1 acceptance criteria from [SOW-0001](../docs/sow/sow-0001-phase-1-mvp.md) have been met:

✅ User can spawn a Claude session from the CLI
✅ Sessions are properly managed (start, stop, cleanup)
✅ All I/O is logged for debugging and persistence
✅ No orphaned processes after CLI exit
✅ Basic MANAGER session can coordinate a single child session
✅ Code is well-structured, documented, and tested (>70% coverage)

---

**Built with ❤️ using Rust and Claude AI**
