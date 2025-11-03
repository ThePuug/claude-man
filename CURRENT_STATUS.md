# Claude-Man - Current Status

## ✅ Working Features

### Session Management
- **Spawn**: Create Claude sessions that run until completion
- **List**: View all active sessions (loaded from disk on startup)
- **Logs**: View complete session history (with --follow option)
- **Stop**: Terminate running sessions
- **Info**: View detailed session information

### Process Handling
- ✅ Cross-platform process spawning (Windows/Linux/macOS)
- ✅ Real-time output capture and display
- ✅ Process monitoring with lifecycle tracking
- ✅ Graceful process termination

### Persistence
- ✅ Session metadata saved to `.claude-man/sessions/{SESSION_ID}/metadata.json`
- ✅ Complete I/O logs in JSONL format at `.claude-man/sessions/{SESSION_ID}/io.log`
- ✅ Sessions loaded from disk on startup
- ✅ Automatic detection of dead processes

## 📋 Current Workflow

```bash
# 1. Spawn a session (blocks until complete)
$ claude-man spawn --role DEVELOPER "Write a hello world function"
✓ Session DEV-001 started (PID: 12345)

[DEV-001] Here's a hello world function...
[DEV-001] ...

✓ Session DEV-001 completed successfully

View logs:  claude-man logs DEV-001

# 2. View logs of completed session
$ claude-man logs DEV-001
[DEV-001] Here's a hello world function...
[DEV-001] ...

# 3. View last 10 lines only
$ claude-man logs DEV-001 -n 10

# 4. List active sessions (none in this case)
$ claude-man list
ℹ No active sessions

# 5. Multiple sessions in parallel (run in different terminals)
Terminal 1: $ claude-man spawn --role DEVELOPER "Task 1"
Terminal 2: $ claude-man spawn --role ARCHITECT "Task 2"
Terminal 3: $ claude-man list  # Shows both running
```

## 🎯 Design Decisions

### Current: Blocking Spawn
- **Why**: Ensures all output is captured properly
- **Behavior**: `spawn` command waits for Claude task to complete before returning
- **Benefit**: Simple, reliable, no orphaned processes
- **Limitation**: Can't manage multiple sessions from single terminal

### Session Persistence
- All sessions are saved to disk immediately on creation
- On startup, claude-man loads all sessions marked as "running"
- Dead processes are automatically marked as "failed"

### Log Format
- **JSONL** (JSON Lines): One event per line
- Event types: `output`, `error`, `input`, `lifecycle`
- Timestamped with ISO 8601 format
- Parseable for future analysis/replay

## 🏗️ Architecture

```
┌──────────────┐
│  User CLI    │
└──────┬───────┘
       │
       v
┌──────────────────┐
│ SessionRegistry  │  ←─ Loads sessions from disk
│  (In-memory)     │  ←─ Tracks active sessions
└────────┬─────────┘
         │
         v
    ┌────────────┐
    │  Session   │
    │  Handle    │
    └────┬───────┘
         │
    ┌────v────────────┐
    │ Monitoring Task │  ←─ tokio::spawn
    │  (Background)   │
    └────┬────────────┘
         │
    ┌────v─────────┐
    │ Claude CLI   │  ←─ Child process
    │   Process    │
    └──────────────┘
```

## 📁 File Structure

```
.claude-man/
└── sessions/
    ├── DEV-001/
    │   ├── metadata.json      # Session info, status, timestamps
    │   └── io.log             # JSONL event log
    ├── DEV-002/
    │   ├── metadata.json
    │   └── io.log
    └── ARCH-001/
        ├── metadata.json
        └── io.log
```

### metadata.json Example
```json
{
  "id": "DEV-001",
  "role": "DEVELOPER",
  "status": "completed",
  "task": "Write a hello world function",
  "created_at": "2025-11-03T17:14:27Z",
  "started_at": "2025-11-03T17:14:27Z",
  "ended_at": "2025-11-03T17:14:33Z",
  "pid": 29896,
  "log_dir": ".claude-man/sessions/DEV-001"
}
```

### io.log Example (JSONL)
```json
{"timestamp":"2025-11-03T17:14:27Z","event_type":"lifecycle","content":"Session started (PID: 29896)","metadata":{"status":"running"}}
{"timestamp":"2025-11-03T17:14:33Z","event_type":"output","content":"Hello! Here's a hello world function..."}
{"timestamp":"2025-11-03T17:14:33Z","event_type":"lifecycle","content":"Session completed successfully (exit code: 0)","metadata":{"status":"completed"}}
```

## 🔄 Next Steps (Future Phases)

### Phase 2: Background Sessions
**Goal**: Non-blocking spawn, manage multiple sessions from one terminal

**Requirements**:
- Daemon process or persistent background workers
- Session attach/detach capability
- IPC mechanism for monitoring output

**Approach**:
1. Run claude-man as a daemon/server
2. CLI commands communicate via IPC (Unix sockets/named pipes)
3. Sessions run independently of CLI process

### Phase 3: Interactive Sessions
**Goal**: stdin forwarding for interactive Claude conversations

**Requirements**:
- PTY (pseudo-terminal) management
- Input/output multiplexing
- Terminal mode switching

### Phase 4: Advanced Features
- Web UI for monitoring
- Session templates
- Resource limits
- Collaborative sessions
- Session replay

## 🧪 Testing

```bash
# Quick test
cd claude-man
cargo run -- spawn --role DEVELOPER "Say hello and goodbye"

# Expected output:
# ✓ Session DEV-001 started (PID: xxxxx)
# [DEV-001] Hello! ...
# [DEV-001] Goodbye! ...
# ✓ Session DEV-001 completed successfully
# View logs:  claude-man logs DEV-001

# View logs
cargo run -- logs DEV-001

# Test session persistence
cargo run -- list  # Loads sessions from disk
```

## 📊 Metrics

| Metric | Value |
|--------|-------|
| Total LOC | ~2,000 |
| Core Features | 5/5 ✅ |
| Test Coverage | Manual (comprehensive) |
| Build Time | ~2-3s (incremental) |
| Binary Size | ~2MB (release) |
| Platform Support | Windows ✅ Linux ✅ macOS ✅ |

## 🐛 Known Limitations

### Current Phase (Blocking Mode)
1. **Single terminal**: Can't manage multiple sessions from one terminal
2. **No background**: Sessions block the terminal until complete
3. **No attach**: Can't reconnect to running sessions mid-execution

### Future Fixes
- These will be resolved in Phase 2 with daemon architecture
- For now, use multiple terminals for parallel sessions
- Or run in background with `&` (but won't see output)

## 🎉 What Works Great

✅ **Output Capture**: All stdout/stderr is captured in real-time
✅ **Session History**: Complete logs persisted to disk
✅ **Process Management**: Clean process lifecycle tracking
✅ **Cross-Platform**: Works on Windows, Linux, macOS
✅ **Persistence**: Sessions survive program restarts
✅ **Log Viewing**: Flexible log viewing with tail/follow

## 🚀 Ready to Use!

The tool is fully functional for its current design:
- Perfect for sequential task execution
- Great for logging and reviewing session history
- Reliable process management
- Complete output capture

Try it:
```bash
cd claude-man
cargo run -- spawn --role DEVELOPER "Help me understand this codebase"
```

---

**Last Updated**: November 3, 2025
**Version**: 0.1.0 (Phase 1 Complete)
**Next Phase**: Background sessions with daemon architecture
