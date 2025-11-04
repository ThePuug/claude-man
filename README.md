> [!WARNING]
> **END OF LIFE**: claude code's sub-agents effectively make this obsolete

# claude-man

**AI Session Orchestration Tool** - Manage multiple Claude AI sessions from a single CLI, enabling parallel development workflows with MANAGER-led coordination.

## Overview

claude-man enables **one AI to manage many AIs**. Spawn a MANAGER session that orchestrates multiple child sessions (DEVELOPER, ARCHITECT, STAKEHOLDER) working in parallel on complex goals. Each session runs independently with full I/O logging, and MANAGER coordinates using `spawn`, `logs`, and `resume` commands.

**Key Features:**
- 🤖 MANAGER orchestration - One AI coordinates multiple specialized AIs
- 🚀 Daemon architecture - Background process managing all sessions
- 📊 Full observability - Complete I/O logging for every session
- 🔄 Multi-turn workflows - Resume sessions for interactive coordination
- 🌲 Session hierarchy - Parent-child relationship tracking
- 💾 Persistence - Sessions survive daemon restarts
- 🖥️ Cross-platform - Windows and Unix support

## Prerequisites

- **Rust** (1.70+) - [Install Rust](https://rustup.rs/)
- **Claude CLI** - Must be installed and authenticated
  ```bash
  npm install -g @anthropic/claude
  claude login  # Authenticate first
  ```

## Installation

```bash
# Clone the repository
git clone https://github.com/ThePuug/claude-man.git
cd claude-man/claude-man

# Install globally (adds to ~/.cargo/bin/)
cargo install --path .

# Verify installation
claude-man --help
```

After installation, `claude-man` will be available from any directory.

## Quick Start

```bash
# 1. One-time setup (creates auto-approval hook)
claude-man init

# 2. Start daemon (in background)
claude-man daemon &

# 3. Spawn a MANAGER to orchestrate a complex task
claude-man spawn --role MANAGER "Build user authentication system"

# 4. Monitor progress
claude-man list                    # See all sessions
claude-man logs MGR-001            # View MANAGER output
claude-man attach MGR-001          # Watch live

# 5. When done, cleanup
claude-man shutdown
```

## Usage

### Core Commands

```bash
# Initialize project (one-time)
claude-man init

# Start/stop daemon
claude-man daemon                  # Start in foreground
claude-man daemon &                # Start in background
claude-man shutdown                # Stop daemon + all sessions

# Spawn sessions
claude-man spawn --role MANAGER "coordinate feature development"
claude-man spawn --role DEVELOPER "implement auth API"
claude-man spawn --role ARCHITECT "design database schema"

# Resume sessions (multi-turn workflows)
claude-man resume DEV-001 "use JWT tokens"

# Monitor sessions
claude-man list                    # Table of all sessions
claude-man info DEV-001            # Detailed metadata
claude-man logs DEV-001 -n 50      # Last 50 lines
claude-man logs DEV-001 --follow   # Live tail
claude-man attach DEV-001          # Stream from beginning

# Control sessions
claude-man stop DEV-001            # Stop specific session
claude-man stop --all              # Stop all sessions
```

## MANAGER Orchestration Example

The killer feature: **one AI managing multiple AIs in parallel**.

```bash
# MANAGER coordinates a complex feature
claude-man spawn --role MANAGER "Build complete authentication system"

# MANAGER reads role-context.md and orchestrates:
# 1. Spawns: claude-man spawn --role ARCHITECT "Design auth system"
# 2. Reads result: claude-man logs ARCH-001
# 3. Spawns parallel:
#    claude-man spawn --role DEVELOPER "Implement backend auth"
#    claude-man spawn --role DEVELOPER "Implement frontend auth"
# 4. Monitors: claude-man list
# 5. Reads results: claude-man logs DEV-001, claude-man logs DEV-002
# 6. If needed, resumes: claude-man resume DEV-001 "use JWT tokens"
# 7. Reports completion to user

# All sessions run in background, MANAGER coordinates autonomously!
```

**How It Works:**
1. MANAGER reads `role-context.md` with orchestration instructions
2. MANAGER spawns child sessions via `claude-man spawn`
3. Children exit when done (task-oriented, not long-running)
4. MANAGER reads outputs via `claude-man logs`
5. MANAGER resumes sessions via `claude-man resume` for multi-turn
6. Repeat until goal achieved

**No stdin needed!** The spawn→logs→resume pattern handles all coordination.

## Architecture

- **Daemon Server** - TCP server (port 47520) managing all sessions
- **Session Registry** - In-memory + disk persistence
- **IPC Protocol** - JSON over TCP for client-daemon communication
- **Process Monitoring** - Async monitoring with proper cleanup
- **JSONL Logging** - Full stdout/stderr/lifecycle capture
- **File-based Context** - role-context.md for role instructions

## Project Structure

```
claude-man/
├── docs/
│   ├── spec/                # Technical specifications
│   │   ├── claude-man-cli.md
│   │   └── claude-man-cli-feature-matrix.md
│   └── adr/                 # Architecture Decision Records
│       ├── template.md
│       ├── 0001-claude-cli-wrapper-architecture.md
│       ├── 0002-manager-role-architecture.md
│       ├── 0003-session-persistence-io-logging.md
│       ├── 0004-environment-based-authentication.md
│       └── 0005-rust-implementation.md
├── ROLES/               # Team roles and responsibilities
│   ├── MANAGER.md       # Orchestrator role (claude-man-cli itself)
│   ├── ARCHITECT.md     # Architecture and design role
│   ├── DEVELOPER.md     # Implementation role
│   └── STAKEHOLDER.md   # Product and validation role
├── CLAUDE.md            # Project rules and guidelines
└── README.md            # This file
```

## Documentation

- [Project Rules](CLAUDE.md) - Coding standards, workflow, and best practices

### Roles and Responsibilities
- [MANAGER](ROLES/MANAGER.md) - Orchestrator role (claude-man-cli itself)
- [ARCHITECT](ROLES/ARCHITECT.md) - Architecture and design role
- [DEVELOPER](ROLES/DEVELOPER.md) - Implementation role
- [STAKEHOLDER](ROLES/STAKEHOLDER.md) - Product and validation role

### Specifications
- [claude-man-cli Specification](docs/spec/claude-man-cli.md) - Complete tool specification
- [Feature Matrix](docs/spec/claude-man-cli-feature-matrix.md) - **Implementation status and roadmap** ⚠️ Nothing implemented yet

### Architecture Decisions (ADRs)
- [ADR-0001: Claude CLI Wrapper Architecture](docs/adr/0001-claude-cli-wrapper-architecture.md)
- [ADR-0002: MANAGER Role Architecture](docs/adr/0002-manager-role-architecture.md)
- [ADR-0003: Session Persistence via I/O Logging](docs/adr/0003-session-persistence-io-logging.md)
- [ADR-0004: Environment-Based Authentication](docs/adr/0004-environment-based-authentication.md)
- [ADR-0005: Rust Implementation](docs/adr/0005-rust-implementation.md)

## Development

### Contributing

Information about how to contribute to the project:
1. Create a feature branch
2. Make your changes
3. Write or update tests
4. Submit a pull request

See [CLAUDE.md](CLAUDE.md) for detailed development guidelines.

### Team Roles

See the [ROLES](ROLES/) directory for detailed information about team structure and responsibilities.

## Testing

Instructions for running tests.

```bash
# Example test commands
```

## License

Specify your license here.

## Contact

Information about how to reach the team or project maintainers.
