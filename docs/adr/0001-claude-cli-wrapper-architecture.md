# 0001. Claude CLI Wrapper Architecture

Date: 2025-11-03

## Status

Accepted

## Context

claude-man-cli needs to spawn and manage multiple Claude sessions for parallel development work. We need to decide how to interface with Claude - either through direct API integration or by wrapping existing Claude CLI tools.

Key requirements:
- Spawn multiple concurrent Claude sessions
- Manage session lifecycle (create, monitor, terminate)
- Ensure no orphaned sessions remain after claude-man-cli exits
- Access to Claude's full capabilities (file operations, code execution, etc.)
- Reliable process management

Options considered:
1. **Direct API Integration**: Use Anthropic Claude API via SDK/REST
2. **Claude Code CLI Wrapper**: Spawn Claude Code CLI instances as child processes
3. **Hybrid**: Use API for orchestration, CLI for actual work

## Decision

We will use a **Claude Code CLI wrapper** with **direct child process** management.

claude-man-cli will:
- Spawn Claude Code CLI instances as child processes
- Communicate via stdin/stdout/stderr
- Monitor process health and status
- Ensure graceful cleanup of all child processes on exit
- Kill orphaned sessions if claude-man-cli restarts

### Process Management Strategy

- Use Node.js `child_process` module for spawning
- Track all child PIDs in a registry
- Implement cleanup handlers for SIGINT, SIGTERM, process exit
- On startup, check for and clean up any orphaned processes
- Use process groups for reliable cleanup

### Session Lifecycle

```
claude-man-cli start
    ↓
Spawn child process (claude CLI)
    ↓
Register PID and session metadata
    ↓
Monitor stdout/stderr
    ↓
Session completes or claude-man-cli exits
    ↓
Gracefully terminate child process
    ↓
Cleanup session resources
```

## Consequences

### Positive Consequences

- **Leverage existing capabilities**: Don't need to reimplement file operations, tool calling, etc.
- **Simpler implementation**: Less code to maintain vs. full API integration
- **Full feature access**: Get all Claude Code features out of the box
- **Familiar environment**: Sessions run in the same environment users are familiar with
- **No orphaned sessions**: Strict process management ensures cleanup
- **Resource efficiency**: Direct child processes are lightweight vs. containers

### Negative Consequences

- **Dependency on Claude Code**: Requires Claude Code CLI to be installed
- **Parsing complexity**: Must parse terminal output to understand session state
- **Less control**: Can't access low-level API features directly
- **IPC limitations**: Communication limited to stdin/stdout patterns
- **Testing complexity**: Need to mock/test process spawning

### Neutral Consequences

- **Process limits**: Limited by OS process limits (typically sufficient)
- **Single machine**: All sessions run on local machine (distributed not needed for v1)
- **Terminal output**: Need to handle ANSI codes and formatting

## Implementation Notes

### Child Process Management

```typescript
// Pseudocode
const sessions = new Map<SessionId, ChildProcess>();

function spawnSession(config: SessionConfig): Session {
  const child = spawn('claude', args, {
    stdio: ['pipe', 'pipe', 'pipe'],
    detached: false, // Keep in same process group for cleanup
  });

  sessions.set(sessionId, child);

  // Cleanup handlers
  child.on('exit', () => handleSessionExit(sessionId));

  return createSession(sessionId, child);
}

// Cleanup on exit
process.on('exit', () => cleanupAllSessions());
process.on('SIGINT', () => {
  cleanupAllSessions();
  process.exit(0);
});
```

### Orphan Detection

On startup:
1. Check for PID file from previous run
2. Verify if processes are still running
3. Attempt graceful termination
4. Force kill if needed
5. Clean up stale state

### Process Communication

- **Input to session**: Write to stdin
- **Output from session**: Parse stdout
- **Status/errors**: Monitor stderr
- **Control signals**: Use process signals (SIGTERM, SIGINT)

## References

- [Node.js child_process documentation](https://nodejs.org/api/child_process.html)
- [Claude Code CLI documentation](https://docs.claude.com/claude-code)
- [Process Management Best Practices](https://nodejs.org/en/docs/guides/nodejs-docker-webapp/#handling-kernel-signals)
