# Statement of Work: claude-man-cli Phase 1 MVP

**Project**: claude-man-cli - AI Session Orchestration Tool
**Phase**: Phase 1 - Minimum Viable Product (MVP)
**Version**: 1.0
**Date**: 2025-11-03
**Status**: Draft

---

## 1. Executive Summary

This Statement of Work defines the scope, deliverables, and success criteria for Phase 1 MVP of claude-man-cli, a Rust-based CLI tool that orchestrates multiple Claude AI sessions to enable parallel development workflows with context coherence.

Phase 1 focuses on **proving the core architectural concept** by implementing the fundamental infrastructure needed to spawn, manage, and coordinate Claude sessions. The MVP will demonstrate that the MANAGER-based orchestration pattern is viable and provides a solid foundation for advanced features in future phases.

---

## 2. Project Objectives

### Primary Objectives
1. **Validate Architecture**: Prove that the MANAGER orchestration pattern works in practice
2. **Establish Foundation**: Build core infrastructure for session management and process control
3. **Enable Development**: Create a stable base for implementing advanced features in Phase 2+
4. **Demonstrate Value**: Show tangible benefits of multi-session orchestration

### Success Criteria
- ✅ User can spawn a Claude session from the CLI
- ✅ Sessions are properly managed (start, stop, cleanup)
- ✅ All I/O is logged for debugging and persistence
- ✅ No orphaned processes after CLI exit
- ✅ Basic MANAGER session can coordinate a single child session
- ✅ Code is well-structured, documented, and tested

---

## 3. Scope Definition

### 3.1 In Scope - Phase 1 MVP

#### Core Infrastructure

**Project Setup**
- ✅ Rust project initialization with Cargo
- ✅ Project structure following modular architecture (core, cli, types)
- ✅ Development tooling setup (rustfmt, clippy, testing framework)
- ✅ CI/CD pipeline basics (GitHub Actions for build and test)
- ✅ Documentation structure (rustdoc comments)

**CLI Framework**
- ✅ Argument parsing using clap
- ✅ Basic command structure: `claude-man <command>`
- ✅ Help text and usage documentation
- ✅ Version information
- ✅ Error message formatting

**Authentication**
- ✅ Read `CLAUDE_AUTH_TOKEN` from environment variable
- ✅ Validate token exists before operations
- ✅ Clear error messages when not authenticated
- ❌ OAuth login flow (deferred to Phase 2)
- ❌ Token persistence to shell config (deferred to Phase 2)

**Process Management**
- ✅ Spawn Claude Code CLI as child process
- ✅ Pass environment variables (including auth token) to child
- ✅ Capture stdout/stderr streams
- ✅ Graceful process termination (SIGTERM)
- ✅ Forced termination after timeout (SIGKILL)
- ✅ Cleanup handlers for CLI exit (SIGINT, SIGTERM)
- ✅ Prevention of orphaned processes
- ✅ PID tracking and management

**Session Management - Basic**
- ✅ Session data structure (ID, role, status, metadata)
- ✅ Session lifecycle: created → running → completed/failed
- ✅ Unique session ID generation (format: `{ROLE}-{sequence}`)
- ✅ In-memory session registry
- ✅ Basic session commands:
  - `claude-man spawn --role DEVELOPER "task description"` - Start child session
  - `claude-man list` - Show active sessions
  - `claude-man stop SESSION_ID` - Stop specific session
  - `claude-man stop --all` - Stop all sessions

**I/O Logging**
- ✅ JSONL log format for session I/O
- ✅ Log directory structure: `.claude-man/sessions/{SESSION_ID}/`
- ✅ Log files: `io.log` (JSONL) and `metadata.json`
- ✅ Append-only logging (never modify existing logs)
- ✅ Timestamped log entries
- ✅ Log both input (stdin) and output (stdout/stderr)
- ✅ Log session lifecycle events (created, started, completed, failed)

**Basic MANAGER Session**
- ✅ MANAGER session runs as primary CLI process
- ✅ MANAGER can spawn a single child session
- ✅ MANAGER receives child output as input (proof of concept)
- ✅ MANAGER prints child outputs to console
- ✅ MANAGER handles child session completion
- ❌ MANAGER tool interface (spawn_session, read_artifact, etc.) - deferred to Phase 2
- ❌ MANAGER decision-making and planning logic - deferred to Phase 2

**Configuration**
- ✅ Read configuration from environment variables
- ✅ Sensible defaults (session timeout, log location)
- ❌ Configuration file parsing - deferred to Phase 2

**Error Handling**
- ✅ Structured error types using thiserror
- ✅ Result types throughout codebase
- ✅ Clear, actionable error messages
- ✅ Proper error propagation
- ✅ Logging of errors to stderr

**Testing**
- ✅ Unit tests for core logic (>70% coverage)
- ✅ Integration tests for CLI commands
- ✅ Mock process spawning for tests
- ✅ Test fixtures for JSONL logs

**Documentation**
- ✅ README with installation and usage instructions
- ✅ API documentation (rustdoc)
- ✅ Update feature matrix with implemented features
- ✅ Basic troubleshooting guide

#### Implemented Features Summary

From the [Feature Matrix](../spec/claude-man-cli-feature-matrix.md), Phase 1 implements:

**Core Architecture** (Partial)
- ✅ Child Session Spawning (basic)
- ✅ Process Management & Cleanup
- ✅ Session I/O Logging (JSONL)
- ✅ Environment-Based Auth (read only)
- 🚧 MANAGER Session (basic proof of concept only)
- ❌ Session Persistence & Resume (logs created but not resumable yet)

**Commands Implemented**
- ✅ `claude-man spawn --role ROLE TASK` (spawn single child)
- ✅ `claude-man list` (list active sessions)
- ✅ `claude-man stop SESSION_ID` (stop session)
- ✅ `claude-man --version` (show version)
- ✅ `claude-man --help` (show help)

**Technical Infrastructure**
- ✅ Rust CLI framework (clap)
- ✅ Async runtime (tokio)
- ✅ Process spawning (tokio::process)
- ✅ JSON/JSONL serialization (serde)
- ✅ Error types (thiserror, anyhow)
- ✅ Logging framework (tracing)

### 3.2 Out of Scope - Phase 1 MVP

The following features are explicitly **deferred to future phases**:

**Phase 2 Features** (Full Orchestration)
- ❌ MANAGER tool interface (spawn_session, read_artifact, etc.)
- ❌ Multiple concurrent sessions
- ❌ Context management (loading roles, specs, ADRs)
- ❌ Artifact reading and generation
- ❌ Smart session coordination
- ❌ OAuth login flow
- ❌ Configuration file support
- ❌ Session persistence and resume

**Phase 3 Features** (Production Ready)
- ❌ Monitoring commands (status, logs, report)
- ❌ Artifact commands
- ❌ Advanced error handling and recovery
- ❌ Performance optimization
- ❌ Cross-compilation and distribution

**Phase 4 Features** (Advanced)
- ❌ Workflow engine
- ❌ Parallel execution with dependency management
- ❌ Conflict detection
- ❌ VSCode extension

---

## 4. Deliverables

### 4.1 Code Deliverables

**Repository Structure**
```
claude-man/
├── Cargo.toml                 # Rust project manifest
├── Cargo.lock                 # Dependency lock file
├── .github/
│   └── workflows/
│       └── ci.yml             # CI/CD pipeline
├── src/
│   ├── main.rs                # CLI entry point
│   ├── lib.rs                 # Core library export
│   ├── cli/
│   │   ├── mod.rs
│   │   ├── commands.rs        # Command implementations
│   │   └── output.rs          # Terminal output formatting
│   ├── core/
│   │   ├── mod.rs
│   │   ├── session.rs         # Session data structures and registry
│   │   ├── process.rs         # Child process management
│   │   ├── logger.rs          # I/O logging to JSONL
│   │   └── auth.rs            # Authentication (env var reading)
│   └── types/
│       ├── mod.rs
│       ├── session.rs         # Session types and enums
│       ├── role.rs            # Role enum (MANAGER, ARCHITECT, DEVELOPER, STAKEHOLDER)
│       └── error.rs           # Error types
├── tests/
│   ├── integration/
│   │   ├── spawn_tests.rs
│   │   ├── list_tests.rs
│   │   └── stop_tests.rs
│   └── fixtures/
│       └── mock_sessions/
└── docs/
    └── sow/
        └── phase-1-mvp.md     # This document
```

**Compiled Binary**
- Single Rust binary: `claude-man` (or `claude-man.exe` on Windows)
- Debug build for development
- Release build for distribution

### 4.2 Documentation Deliverables

- ✅ Updated [README.md](../../README.md) with:
  - Installation instructions (cargo install)
  - Prerequisites (Rust, Claude Code CLI)
  - Quick start guide
  - Basic usage examples
- ✅ Updated [Feature Matrix](../spec/claude-man-cli-feature-matrix.md) showing Phase 1 complete
- ✅ API documentation via rustdoc
- ✅ Phase 1 implementation notes (lessons learned, design decisions)

### 4.3 Testing Deliverables

- ✅ Unit test suite with >70% code coverage
- ✅ Integration test suite covering all CLI commands
- ✅ CI/CD pipeline running tests on every commit
- ✅ Test documentation and examples

---

## 5. Technical Approach

### 5.1 Architecture

Phase 1 implements the foundation of the architecture defined in [ADR-0001](../adr/0001-claude-cli-wrapper-architecture.md) and [ADR-0002](../adr/0002-manager-role-architecture.md).

**Simplified Phase 1 Architecture**:
```
┌─────────────────────────────────────────┐
│        claude-man CLI (MANAGER)         │
│  • Spawns child Claude sessions         │
│  • Logs all I/O to JSONL                │
│  • Manages session lifecycle            │
│  • Prevents orphaned processes          │
└────────────────┬────────────────────────┘
                 │
                 │ spawn & monitor
                 ↓
         ┌───────────────┐
         │  Child Claude │
         │    Session    │
         │  (any role)   │
         └───────┬───────┘
                 │
                 │ writes
                 ↓
         ┌───────────────┐
         │  Session Logs │
         │   (JSONL)     │
         └───────────────┘
```

### 5.2 Technology Stack

As defined in [ADR-0005: Rust Implementation](../adr/0005-rust-implementation.md):

**Core Dependencies**:
```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
anyhow = "1.0"
thiserror = "1.0"
chrono = "0.4"  # For timestamps
uuid = "1.6"     # For session IDs

[dev-dependencies]
tokio-test = "0.4"
tempfile = "3.8"
```

### 5.3 Development Workflow

1. **Setup**: Initialize Rust project with `cargo init`
2. **Iterative Development**:
   - Implement feature
   - Write tests
   - Run `cargo test`
   - Run `cargo clippy` (linter)
   - Run `cargo fmt` (formatter)
3. **Commit**: Push to git with clear commit messages
4. **CI**: GitHub Actions runs tests automatically
5. **Documentation**: Update docs as features are completed

---

## 6. Success Criteria & Acceptance

### 6.1 Functional Acceptance Criteria

**Must Pass All**:

1. ✅ **Spawn Session**:
   ```bash
   export CLAUDE_AUTH_TOKEN="test-token"
   claude-man spawn --role DEVELOPER "write a hello world function"
   # Output: "Session DEV-001 started"
   # Verify: Claude CLI process is running
   # Verify: .claude-man/sessions/DEV-001/io.log exists
   ```

2. ✅ **List Sessions**:
   ```bash
   claude-man list
   # Output shows DEV-001 with status "running"
   ```

3. ✅ **Stop Session**:
   ```bash
   claude-man stop DEV-001
   # Output: "Session DEV-001 stopped"
   # Verify: Process terminated gracefully
   # Verify: io.log contains completion event
   ```

4. ✅ **No Orphans on Exit**:
   ```bash
   claude-man spawn --role DEVELOPER "task"
   # Press Ctrl+C
   # Verify: All child processes terminated
   # Verify: No zombie processes
   ```

5. ✅ **I/O Logging**:
   ```bash
   claude-man spawn --role DEVELOPER "task"
   # Wait for output
   cat .claude-man/sessions/DEV-001/io.log
   # Verify: JSONL format
   # Verify: Timestamps present
   # Verify: Both input and output logged
   ```

6. ✅ **Error Handling**:
   ```bash
   # No auth token
   unset CLAUDE_AUTH_TOKEN
   claude-man spawn --role DEVELOPER "task"
   # Output: Clear error message about missing token

   # Invalid session
   claude-man stop INVALID-999
   # Output: Clear error message about unknown session
   ```

### 6.2 Non-Functional Acceptance Criteria

1. ✅ **Performance**:
   - CLI startup time < 500ms
   - Session spawn time < 2s (excluding Claude CLI startup)
   - No memory leaks over 1 hour of operation

2. ✅ **Reliability**:
   - No crashes during normal operation
   - Graceful degradation on errors
   - All resources cleaned up on exit

3. ✅ **Code Quality**:
   - >70% test coverage
   - Zero clippy warnings on default lints
   - All public APIs documented
   - Code passes `cargo fmt --check`

4. ✅ **Usability**:
   - `--help` text is clear and complete
   - Error messages are actionable
   - Commands follow CLI conventions

### 6.3 Test Cases

**Unit Tests** (minimum 20 tests):
- Session ID generation is unique
- Session state transitions are valid
- Process cleanup handlers are registered
- JSONL serialization is correct
- Error types convert correctly

**Integration Tests** (minimum 10 tests):
- End-to-end session spawn and stop
- Multiple sessions can coexist
- Logs are created correctly
- Cleanup on Ctrl+C works
- Auth validation works

---

## 7. Timeline & Milestones

### Phase 1 Timeline: 2-3 Weeks

**Week 1: Foundation**
- Day 1-2: Project setup, cargo init, CI/CD
- Day 3-4: CLI framework (clap), basic commands
- Day 5-7: Process spawning, cleanup handlers

**Week 2: Core Features**
- Day 8-10: Session management (spawn, list, stop)
- Day 11-12: I/O logging (JSONL)
- Day 13-14: Testing, bug fixes

**Week 3: Polish & Documentation**
- Day 15-16: Integration tests, code coverage
- Day 17-18: Documentation, examples
- Day 19-21: Final testing, acceptance criteria validation

### Milestones

| Milestone | Deliverable | Target |
|-----------|-------------|--------|
| M1: Foundation | Project compiles, help text works | End of Week 1 |
| M2: Basic Sessions | Can spawn and stop sessions | Mid Week 2 |
| M3: I/O Logging | All I/O logged to JSONL | End of Week 2 |
| M4: Testing Complete | >70% coverage, all tests pass | Mid Week 3 |
| M5: Phase 1 Complete | All acceptance criteria met | End of Week 3 |

---

## 8. Dependencies & Prerequisites

### 8.1 External Dependencies

**Required**:
- ✅ Rust toolchain (1.70+)
- ✅ Claude Code CLI installed and in PATH
- ✅ Git for version control
- ✅ GitHub account (for CI/CD)

**Development**:
- ✅ Code editor with Rust support (VS Code + rust-analyzer recommended)
- ✅ `cargo-watch` for auto-rebuild during development
- ✅ `cargo-tarpaulin` or similar for coverage

### 8.2 Internal Dependencies

**Prerequisites**:
- ✅ All Phase 0 deliverables complete (specs, ADRs, roles) ✅ DONE
- ✅ Feature matrix established ✅ DONE
- ✅ Architecture decisions documented ✅ DONE

---

## 9. Risks & Mitigations

### Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Process cleanup fails on Windows | Medium | High | Implement platform-specific handlers, test on all platforms |
| JSONL logs become very large | Low | Medium | Document log rotation strategy (manual for Phase 1) |
| Claude CLI changes interface | Low | High | Version pin Claude CLI, document compatible versions |
| Tokio async complexity | Medium | Medium | Start simple, add complexity incrementally |

### Schedule Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Scope creep (adding Phase 2 features) | High | High | Strict adherence to SOW, defer features explicitly |
| Testing takes longer than expected | Medium | Low | Allocate full week for testing and polish |
| Platform-specific issues | Medium | Medium | Test on Windows/macOS/Linux early |

---

## 10. Out of Scope - Explicit Exclusions

The following are **explicitly not included** in Phase 1:

1. ❌ Multiple concurrent sessions (Phase 2)
2. ❌ MANAGER decision-making logic (Phase 2)
3. ❌ Context management (loading files, roles) (Phase 2)
4. ❌ Artifact generation and reading (Phase 2)
5. ❌ OAuth login flow (Phase 2)
6. ❌ Configuration file parsing (Phase 2)
7. ❌ Session resume from logs (Phase 3)
8. ❌ Monitoring commands (status, logs, report) (Phase 3)
9. ❌ Workflow engine (Phase 4)
10. ❌ VSCode extension (Phase 4)

---

## 11. Phase 1 Success Definition

**Phase 1 is considered successful when**:

✅ All functional acceptance criteria pass
✅ All non-functional acceptance criteria pass
✅ Test coverage >70%
✅ CI/CD pipeline is green
✅ Documentation is complete and accurate
✅ Feature matrix is updated
✅ Code is production-quality (no TODOs, no hacks)
✅ Demonstrates the core value proposition: spawning and managing Claude sessions

**Outcome**: A solid foundation for Phase 2 implementation, with core infrastructure proven and working.

---

## 12. Next Steps After Phase 1

Upon successful completion of Phase 1, proceed to:

**Phase 2: Full Orchestration**
- MANAGER tool interface
- Multiple concurrent sessions
- Context management
- Artifact reading
- OAuth authentication
- Configuration file support

See future SOW documents for Phase 2+ details.

---

## Appendix A: Example Session

**Complete Phase 1 workflow**:

```bash
# Set authentication
export CLAUDE_AUTH_TOKEN="my-token"

# Spawn a development session
$ claude-man spawn --role DEVELOPER "implement fibonacci function"
✓ Session DEV-001 started
Monitoring session... (Ctrl+C to stop)

[DEV-001] I'll implement a fibonacci function for you...
[DEV-001] Here's an efficient implementation...
[DEV-001] <complete>

✓ Session DEV-001 completed successfully

# Check session logs
$ ls -la .claude-man/sessions/DEV-001/
io.log          # JSONL log of all I/O
metadata.json   # Session metadata

# List all sessions
$ claude-man list
SESSION-ID  ROLE        STATUS      STARTED
DEV-001     DEVELOPER   completed   2025-11-03T10:15:00Z

# Spawn another and stop it manually
$ claude-man spawn --role ARCHITECT "design auth system" &
$ claude-man list
SESSION-ID  ROLE        STATUS      STARTED
DEV-001     DEVELOPER   completed   2025-11-03T10:15:00Z
ARCH-002    ARCHITECT   running     2025-11-03T10:20:00Z

$ claude-man stop ARCH-002
✓ Session ARCH-002 stopped
```

---

**Document Status**: Draft
**Last Updated**: 2025-11-03
**Author**: Claude (AI) + Project Team
**Approval Required**: Yes
**Next Review**: After Phase 1 Completion
