# 0004. Environment-Based Authentication with Login Command

Date: 2025-11-03

## Status

Accepted

## Context

claude-man-cli needs to authenticate with Claude.ai to spawn and manage sessions. The MANAGER session and all child sessions need access to the same authentication credentials.

Requirements:
- Secure storage of authentication tokens
- Share credentials between MANAGER and child sessions
- Simple setup process for users
- No credentials in code or config files (git-safe)
- Work across different machines/environments

Options considered:
1. **Environment Variables**: Store tokens in env vars
2. **Config File**: Store encrypted tokens in config file
3. **OS Keychain**: Use system credential store
4. **Per-Session Auth**: Each session authenticates independently

## Decision

Use **environment variables** for authentication tokens, with a **login command** for initial authorization.

### Authentication Flow

```
User runs: claude-man login
    ↓
Opens browser to claude.ai
    ↓
User logs in and authorizes
    ↓
claude-man receives auth token
    ↓
Token stored in environment variable
    ↓
All sessions (MANAGER + children) use same token
```

### Environment Variable

```bash
CLAUDE_AUTH_TOKEN=<token>
```

This token is:
- Set by `claude-man login` command
- Inherited by all child processes
- Used by Claude CLI instances
- Persisted in user's shell config (optional)

### Login Command

```bash
$ claude-man login

Opening browser for authentication...
Please log in to claude.ai and authorize claude-man-cli

✓ Authentication successful!
Token saved to environment.

Add this to your shell config to persist:
  export CLAUDE_AUTH_TOKEN="<token>"

Or use:
  claude-man login --save-to-shell
```

### Token Sharing

MANAGER and child sessions all use the same token:

```typescript
// When spawning child session
const child = spawn('claude', args, {
  env: {
    ...process.env,
    CLAUDE_AUTH_TOKEN: process.env.CLAUDE_AUTH_TOKEN
  }
});
```

All sessions share the token via environment variable inheritance.

### Token Management

**Refresh**:
```bash
claude-man login --refresh
```

**Logout**:
```bash
claude-man logout
# Unsets CLAUDE_AUTH_TOKEN
```

**Status**:
```bash
claude-man auth status
# Shows if authenticated, token expiry, user info
```

## Consequences

### Positive Consequences

- **Simple implementation**: Environment variables are standard and well-understood
- **Secure**: Token not stored in files that might be committed to git
- **Shared automatically**: Child processes inherit environment
- **No external dependencies**: No keychain libraries needed
- **Cross-platform**: Works on Windows, macOS, Linux
- **Standard pattern**: Familiar to developers (like AWS_ACCESS_KEY_ID)
- **Easy debugging**: Can check env var to verify auth
- **Shell agnostic**: Works with bash, zsh, fish, PowerShell, etc.

### Negative Consequences

- **Not encrypted at rest**: Token visible in process environment
- **Manual persistence**: User must add to shell config to persist
- **Process visibility**: Token visible to other processes (ps aux)
- **No automatic refresh**: User must re-login when token expires
- **Single token**: All sessions share one token (can't use different accounts)

### Neutral Consequences

- **Session scope**: Token lives for shell session unless persisted
- **No token rotation**: Token remains same until user re-authenticates

## Implementation Notes

### Login Command Implementation

```typescript
async function loginCommand(options: LoginOptions): Promise<void> {
  // 1. Start local HTTP server to receive OAuth callback
  const server = createCallbackServer();
  const callbackUrl = await server.start();

  // 2. Open browser to Claude.ai auth page
  const authUrl = `https://claude.ai/oauth/authorize?redirect_uri=${callbackUrl}`;
  await openBrowser(authUrl);

  console.log('Opening browser for authentication...');
  console.log('Please log in to claude.ai and authorize claude-man-cli');

  // 3. Wait for callback with auth token
  const token = await server.waitForCallback(timeout = 300000); // 5 min

  // 4. Set environment variable
  process.env.CLAUDE_AUTH_TOKEN = token;

  console.log('✓ Authentication successful!');

  // 5. Optionally save to shell config
  if (options.saveToShell) {
    await appendToShellConfig(`export CLAUDE_AUTH_TOKEN="${token}"`);
    console.log('Token saved to shell config');
  } else {
    console.log('\nAdd this to your shell config to persist:');
    console.log(`  export CLAUDE_AUTH_TOKEN="${token}"`);
  }

  await server.stop();
}
```

### Token Validation

Before starting sessions:
```typescript
function validateAuth(): void {
  const token = process.env.CLAUDE_AUTH_TOKEN;

  if (!token) {
    console.error('Not authenticated. Please run: claude-man login');
    process.exit(1);
  }

  // Optionally validate token is not expired
  // (requires calling Claude API to check)
}
```

### Security Best Practices

1. **Never log token**: Redact in logs and output
2. **Warn about visibility**: Inform user that env vars are process-visible
3. **Session isolation**: Each user should have their own token
4. **Token expiry**: Prompt re-login on expired token
5. **Clear on logout**: Unset env var on logout

### Error Handling

- **No token**: Clear error message to run `claude-man login`
- **Expired token**: Detect and prompt re-login
- **Invalid token**: Clear error and suggest re-login
- **Network errors**: Graceful fallback with retry options

## Alternatives Considered

### Alternative 1: Config File with Encryption

Store encrypted token in `~/.claude-man/config.json`.

**Why rejected**:
- More complex (need encryption key management)
- Chicken-and-egg: where to store encryption key?
- File-based secrets can accidentally be committed
- Not significantly more secure than env vars

### Alternative 2: OS Keychain

Use platform-specific credential stores.

**Why rejected**:
- Platform-specific code (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- Complex implementation with multiple libraries
- May require GUI for initial setup
- Child process access is complicated
- Overkill for command-line tool

### Alternative 3: Per-Session Authentication

Each session authenticates independently.

**Why rejected**:
- Significant overhead (multiple auth flows)
- Poor user experience (multiple logins)
- Complicates session spawning
- No benefit for single-user tool

### Alternative 4: Direct API Key

User provides Anthropic API key directly.

**Why rejected**:
- Requires user to have API key (not all Claude users do)
- Misses claude.ai-specific features
- Want to use Claude Code CLI, not direct API

## Future Enhancements

### Phase 2
- **Automatic token refresh**: Refresh before expiry
- **Multiple profiles**: Switch between accounts
- **Token encryption**: Optional encrypted storage
- **Session tokens**: Different tokens for different projects

### Phase 3
- **Team authentication**: Shared team tokens
- **SSO integration**: Enterprise SSO support
- **Token auditing**: Track token usage
- **Scoped tokens**: Different permissions per session

## Migration and Rollout

### Initial Setup

1. User installs claude-man-cli
2. Runs `claude-man login`
3. Authenticates via browser
4. Optionally saves to shell config
5. Can now use all claude-man commands

### Existing Claude Users

If user already has Claude CLI configured:
- Check for existing Claude CLI auth
- Offer to use existing credentials
- Or go through login flow

### Documentation

Provide clear docs on:
- Why authentication is needed
- How login works
- How to persist token
- Security considerations
- Troubleshooting auth issues

## References

- [OAuth 2.0 Specification](https://oauth.net/2/)
- [Environment Variables Best Practices](https://12factor.net/config)
- [Claude.ai Authentication](https://claude.ai/docs/authentication)
- [ADR-0001: Claude CLI Wrapper](./0001-claude-cli-wrapper-architecture.md)
