# 0003. Session Persistence via I/O Logging

Date: 2025-11-03

## Status

Accepted

## Context

Sessions need to survive claude-man-cli restarts for several reasons:
- Developer might need to restart the tool
- System crashes or power loss
- Long-running tasks that exceed single session duration
- Ability to inspect and replay sessions for debugging

We need a way to:
- Capture complete session state
- Resume sessions after restart
- Inspect historical sessions
- Debug issues by reviewing conversation flow

Options for persistence:
1. **No persistence**: Sessions lost on restart
2. **State snapshots**: Serialize session state at intervals
3. **I/O logging**: Log all input/output during session
4. **Database storage**: Store structured session data

## Decision

Use **I/O logging** to persist sessions. Every input sent to a session and every output received is logged to disk in append-only log files.

### Log Structure

Each session has a log directory:
```
.claude-man/
  sessions/
    MANAGER-main/
      io.log          # Complete I/O transcript
      metadata.json   # Session metadata
      artifacts/      # Links/copies of artifacts produced
    ARCH-001/
      io.log
      metadata.json
      artifacts/
    DEV-002/
      io.log
      metadata.json
      artifacts/
```

### I/O Log Format

Append-only JSONL (JSON Lines) format:

```jsonl
{"timestamp":"2025-11-03T10:15:30.123Z","direction":"in","content":"Please design authentication system"}
{"timestamp":"2025-11-03T10:15:31.456Z","direction":"out","content":"I'll design a JWT-based auth system..."}
{"timestamp":"2025-11-03T10:16:45.789Z","direction":"in","content":"<user>Proceed with implementation</user>"}
{"timestamp":"2025-11-03T10:16:46.012Z","direction":"out","content":"Creating ADR document..."}
{"timestamp":"2025-11-03T10:17:20.345Z","direction":"artifact","path":"docs/adr/0005-jwt-auth.md","action":"created"}
{"timestamp":"2025-11-03T10:18:00.000Z","direction":"out","content":"<session_complete>Authentication design ADR created</session_complete>"}
```

### Metadata File

```json
{
  "session_id": "ARCH-001",
  "role": "ARCHITECT",
  "status": "completed",
  "created_at": "2025-11-03T10:15:30.123Z",
  "completed_at": "2025-11-03T10:18:00.000Z",
  "parent_session": "MANAGER-main",
  "task": "Design authentication system",
  "context_files": [
    "docs/spec/user-requirements.md"
  ],
  "artifacts_produced": [
    "docs/adr/0005-jwt-auth.md",
    "docs/spec/authentication-api.md"
  ],
  "pid": 12345,
  "exit_code": 0
}
```

### Resuming Sessions

On restart:
1. Scan `.claude-man/sessions/` for active sessions
2. Load `metadata.json` to check status
3. If status is "active" but process is dead:
   - Mark session as "interrupted"
   - Option to resume: replay I/O log to new process
4. If status is "active" and process exists:
   - Attempt to reconnect (if possible)
   - Otherwise, mark as interrupted

### Session Replay

To resume an interrupted session:
1. Create new Claude CLI child process
2. Read `io.log` sequentially
3. Send all "in" messages to new process
4. Continue from last state
5. Optionally verify "out" messages match (for debugging)

## Consequences

### Positive Consequences

- **Crash recovery**: Can resume after unexpected shutdown
- **Debugging**: Complete transcript available for inspection
- **Audit trail**: Know exactly what was sent/received
- **Replay capability**: Can replay sessions for testing
- **Simple implementation**: Append-only logs are simple and reliable
- **Human readable**: JSONL is easy to inspect
- **Git-friendly**: Text-based logs can be committed (if desired)
- **No data loss**: Captures everything

### Negative Consequences

- **Disk usage**: Logs can grow large for long sessions
- **Replay accuracy**: Replay might not produce identical results (non-deterministic)
- **No perfect resume**: Claude might generate different responses
- **Log rotation needed**: Need to clean up old logs
- **Race conditions**: Concurrent writers need careful handling

### Neutral Consequences

- **Append-only**: Logs only grow, never modified (good for reliability)
- **Plaintext storage**: Logs contain full conversation (security consideration)

## Implementation Notes

### Logging Implementation

```typescript
class SessionLogger {
  private logFile: WriteStream;
  private metadata: SessionMetadata;

  async logInput(content: string): Promise<void> {
    const entry = {
      timestamp: new Date().toISOString(),
      direction: 'in',
      content
    };
    await this.appendLog(entry);
  }

  async logOutput(content: string): Promise<void> {
    const entry = {
      timestamp: new Date().toISOString(),
      direction: 'out',
      content
    };
    await this.appendLog(entry);
  }

  async logArtifact(path: string, action: string): Promise<void> {
    const entry = {
      timestamp: new Date().toISOString(),
      direction: 'artifact',
      path,
      action
    };
    await this.appendLog(entry);
    await this.updateMetadata({ artifacts: [...this.metadata.artifacts, path] });
  }

  private async appendLog(entry: LogEntry): Promise<void> {
    await this.logFile.write(JSON.stringify(entry) + '\n');
  }
}
```

### Resume Strategy

For MVP:
- No automatic resume, but logs preserved
- User can inspect logs to understand what happened
- User can restart task manually with context

For v2:
- Automatic detection of interrupted sessions
- Prompt user to resume or start fresh
- Smart resume: load conversation context, continue from last state

### Log Rotation and Cleanup

- Keep logs for completed sessions (configurable retention)
- Archive old logs (gzip for compression)
- Option to delete logs for failed/aborted sessions
- Never delete MANAGER logs (critical for understanding orchestration)

### Security Considerations

- Logs may contain sensitive information from code
- Don't log API keys or credentials (filter/redact)
- Warn user if logs might contain secrets
- Option to encrypt logs at rest (future enhancement)
- `.gitignore` for session logs by default

## Alternatives Considered

### Alternative 1: In-Memory Only

No persistence, sessions lost on restart.

**Why rejected**:
- No crash recovery
- Can't inspect historical sessions
- Poor debugging experience
- Users lose work on unexpected shutdown

### Alternative 2: State Snapshots

Serialize session state periodically.

**Why rejected**:
- Complex: need to serialize all conversation state
- Incomplete: lose state between snapshots
- Claude API doesn't expose full conversation state
- Harder to implement than I/O logging

### Alternative 3: Database Storage

Store all messages in SQLite/Postgres.

**Why rejected**:
- More complex than needed
- Schema management overhead
- Overkill for append-only access pattern
- Harder to inspect than text files

## Future Enhancements

### Phase 2
- Automatic session resume on restart
- Smart context reconstruction from logs
- Log compression for old sessions
- Better log analysis tools

### Phase 3
- Session fork (start from mid-conversation)
- Log search and query
- Session comparison (diff two runs)
- Export logs in different formats

## References

- [JSONL Format Specification](http://jsonlines.org/)
- [Node.js fs.createWriteStream](https://nodejs.org/api/fs.html#fs_fs_createwritestream_path_options)
- [ADR-0001: Claude CLI Wrapper](./0001-claude-cli-wrapper-architecture.md)
- [ADR-0002: MANAGER Role](./0002-manager-role-architecture.md)
