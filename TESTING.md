# Claude-Man Testing Guide

## Current Status: ✅ FULLY FUNCTIONAL

All core features have been implemented and tested successfully on Windows.

## Prerequisites

1. **Claude CLI must be installed and authenticated:**
   ```bash
   claude /login
   ```

2. **Build the project:**
   ```bash
   cd claude-man
   cargo build --release
   ```

## Available Commands

### 1. Spawn a Session
```bash
cargo run -- spawn --role DEVELOPER "Your task here"
cargo run -- spawn --role ARCHITECT "Design the authentication system"
cargo run -- spawn --role MANAGER "Review the project timeline"
cargo run -- spawn --role STAKEHOLDER "Provide feedback on UX"
```

**What happens:**
- Creates a new session with a unique ID (e.g., DEV-001, ARCH-001)
- Spawns a Claude CLI process in the background
- Monitors the process and logs all output
- Displays output to console in real-time
- Press Ctrl+C to stop monitoring (session continues in background)

### 2. List Active Sessions
```bash
cargo run -- list
```

**Output example:**
```
┌─────────┬────────────┬────────────┬─────────┬──────────────────────┐
│ ID      │ Role       │ Status     │ PID     │ Task                 │
├─────────┼────────────┼────────────┼─────────┼──────────────────────┤
│ DEV-001 │ DEVELOPER  │ running    │ 12345   │ Fix the auth bug     │
│ ARCH-001│ ARCHITECT  │ running    │ 12346   │ Design new API       │
└─────────┴────────────┴────────────┴─────────┴──────────────────────┘
```

### 3. Get Session Info
```bash
cargo run -- info DEV-001
```

**Shows:**
- Session ID and role
- Current status
- Task description
- Start time and duration
- Process ID
- Log file location

### 4. Stop a Session
```bash
# Stop specific session
cargo run -- stop DEV-001

# Stop all sessions
cargo run -- stop --all
```

**What happens:**
- Terminates the Claude process gracefully (SIGTERM on Unix, taskkill on Windows)
- Updates session metadata to "stopped"
- Cleans up monitoring tasks

## File Structure

Sessions are stored in `.claude-man/sessions/`:

```
.claude-man/
└── sessions/
    ├── DEV-001/
    │   ├── metadata.json    # Session metadata (ID, status, timestamps, PID)
    │   └── io.log          # JSONL log of all I/O events
    ├── DEV-002/
    │   ├── metadata.json
    │   └── io.log
    └── ARCH-001/
        ├── metadata.json
        └── io.log
```

### Log Format (JSONL)

Each line in `io.log` is a JSON object:

```json
{"timestamp":"2025-11-03T16:53:45.163105600Z","event_type":"lifecycle","content":"Session started (PID: 34760)","metadata":{"status":"running"}}
{"timestamp":"2025-11-03T16:53:51.312773700Z","event_type":"output","content":"Hello! 👋"}
{"timestamp":"2025-11-03T16:53:51.330641900Z","event_type":"lifecycle","content":"Session completed successfully (exit code: 0)","metadata":{"status":"completed"}}
```

Event types:
- `lifecycle` - Session state changes
- `output` - stdout from Claude
- `error` - stderr from Claude
- `input` - stdin sent to Claude (future feature)

## Test Scenarios

### Basic Workflow Test
```bash
# 1. Start a session
cargo run -- spawn --role DEVELOPER "Write a hello world function in Rust"

# 2. List sessions (should show DEV-001 as running)
cargo run -- list

# 3. Check session details
cargo run -- info DEV-001

# 4. View logs
cat .claude-man/sessions/DEV-001/io.log

# 5. Stop the session
cargo run -- stop DEV-001

# 6. List again (should be empty or show stopped)
cargo run -- list
```

### Multi-Session Test
```bash
# Start multiple sessions
cargo run -- spawn --role DEVELOPER "Task 1" &
cargo run -- spawn --role ARCHITECT "Task 2" &
cargo run -- spawn --role MANAGER "Task 3" &

# Wait a moment for them to start
sleep 2

# List all sessions
cargo run -- list

# Stop all at once
cargo run -- stop --all
```

### Long-Running Session Test
```bash
# Start a session with a long task
cargo run -- spawn --role DEVELOPER "Research best practices for async Rust error handling and write a summary"

# Let it run in background with Ctrl+C
# Check it's still running
cargo run -- list

# View live logs
tail -f .claude-man/sessions/DEV-001/io.log

# Stop when done
cargo run -- stop DEV-001
```

## Known Limitations

### Phase 1 Implementation
- **No stdin forwarding**: Sessions run non-interactively
- **No session resume**: Can't reconnect to a running session
- **Memory-only registry**: Sessions lost on program restart (logs persist)
- **No process recovery**: If claude-man crashes, orphaned Claude processes continue

### Windows-Specific
- Process termination uses `taskkill /F` (forceful)
- No graceful shutdown attempt (unlike Unix SIGTERM)

### Future Enhancements
- Session persistence (restore sessions on restart)
- Interactive mode (stdin forwarding)
- Session attach/detach
- Web UI for monitoring
- Session templates
- Resource limits per session

## Troubleshooting

### "Claude CLI not found"
```bash
# Verify Claude CLI is in PATH
where claude  # Windows
which claude  # Unix

# If not found, install Claude CLI first
```

### "Not authenticated with Claude CLI"
```bash
# Login first
claude /login
```

### "Session not found"
```bash
# List active sessions
cargo run -- list

# Check if session exists in logs
ls .claude-man/sessions/
```

### Process doesn't stop
```bash
# On Windows, manually kill
taskkill /F /PID <pid>

# On Unix
kill -9 <pid>
```

## Verification

All features tested successfully on Windows:
- ✅ Session spawning
- ✅ Process monitoring
- ✅ Real-time output display
- ✅ Log file creation (JSONL format)
- ✅ Metadata persistence
- ✅ Session listing
- ✅ Session info display
- ✅ Process termination
- ✅ Batch session management (stop --all)

## Test Output Example

```bash
$ cargo run -- spawn --role DEVELOPER "Say hello and then exit"
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.13s
     Running `target\debug\claude-man.exe spawn --role DEVELOPER 'Say hello and then exit'`
[2025-11-03T16:53:45.153331Z] INFO Executing spawn command: role=DEVELOPER, task=Say hello and then exit
[2025-11-03T16:53:45.153435Z] INFO Spawning session DEV-001 with role Developer
[2025-11-03T16:53:45.163041Z] INFO Session DEV-001 started successfully
[2025-11-03T16:53:45.163053Z] INFO Monitoring process 34760 for session DEV-001

✓ Session DEV-001 started
Monitoring session... (Ctrl+C to stop)
[DEV-001] Hello! 👋
[DEV-001]
[DEV-001] I'm Claude, ready to help you with your software engineering tasks in the claude-man project. I can assist with coding, debugging, refactoring, documentation, and more.
[DEV-001]
[DEV-001] Feel free to ask me anything or let me know what you'd like to work on!

[2025-11-03T16:53:51.330603Z] INFO Process 34760 exited with code: 0
```
