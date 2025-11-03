# claude-man-cli Specification

## Overview

**claude-man-cli** is a command-line orchestration tool that manages multiple Claude AI sessions from a single console interface. It enables high-productivity development workflows by parallelizing work across multiple Claude sessions while maintaining context coherence through documentation artifacts.

### Purpose

The tool addresses key limitations in single-session AI-assisted development:
- **Context Limits**: Individual Claude sessions have finite context windows
- **Sequential Bottlenecks**: Single session can only work on one task at a time
- **Context Fragmentation**: Knowledge gained in one session is lost when starting new sessions
- **Manual Coordination**: Developer must manually coordinate between multiple sessions

### Solution Approach

claude-man-cli acts as an orchestrator that:
1. Spawns and manages multiple Claude sessions on demand
2. Transfers context between sessions via structured documentation artifacts
3. Generates effective prompts that produce both code changes and documentation
4. Enables parallel execution of independent tasks
5. Maintains coherent project state across all sessions

## Core Concepts

### Claude Sessions

A **session** is an independent Claude conversation instance with:
- Unique session ID
- Specific role context (DEVELOPER, ARCHITECT, STAKEHOLDER)
- Task assignment and focus area
- Dedicated workspace/branch (optional)
- Output artifacts (code changes, documentation)

### Context Transfer

**Context transfer** is achieved through documentation artifacts:
- Specifications (what to build)
- Architecture Decision Records (how/why design choices)
- Session summaries (what was done)
- Task results (outcomes and findings)
- Code references (where changes were made)

### Documentation Artifacts

**Artifacts** are structured documents that capture:
- Requirements and acceptance criteria
- Design decisions and rationale
- Implementation details and changes
- Testing results and validation
- Follow-on tasks and dependencies

## Architecture

**Key Design Decision**: claude-man-cli itself runs as a Claude session with the MANAGER role. The MANAGER orchestrates child sessions and adapts based on their outputs.

See [ADR-0002: MANAGER Role Architecture](../adr/0002-manager-role-architecture.md) for detailed rationale.

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    claude-man-cli                            │
│                                                              │
│  ┌────────────────────────────────────────────────────┐    │
│  │         MANAGER Session (Claude)                   │    │
│  │  • Receives user goals                             │    │
│  │  • Plans tasks and spawns child sessions           │    │
│  │  • Reads child outputs as inputs                   │    │
│  │  • Adapts plan based on outcomes/failures          │    │
│  │  • Maintains overall project context               │    │
│  └────────────────┬───────────────────────────────────┘    │
│                   │                                          │
│       ┌───────────┼───────────────┐                         │
│       │           │               │                         │
│  ┌────▼────┐ ┌───▼─────┐   ┌────▼────┐                    │
│  │  ARCH   │ │   DEV   │   │   DEV   │  (Child Sessions)  │
│  │ Session │ │ Session │   │ Session │                     │
│  └────┬────┘ └────┬────┘   └────┬────┘                    │
│       │           │             │                           │
│       └───────────┼─────────────┘                          │
│                   │                                          │
│          ┌────────▼────────┐                                │
│          │   Artifacts     │                                │
│          │  (docs/, code)  │                                │
│          └────────┬────────┘                                │
│                   │                                          │
│          (MANAGER reads artifacts)                          │
│                   │                                          │
│          ┌────────▼────────┐                                │
│          │  MANAGER plans  │                                │
│          │   next actions  │                                │
│          └─────────────────┘                                │
└─────────────────────────────────────────────────────────────┘
```

### Components

#### MANAGER Session
The core orchestrator running as a Claude session with tools:
- **spawn_session**: Create child sessions with role, task, and context
- **attach_session**: Send additional prompts to running sessions
- **stop_session**: Terminate sessions
- **list_sessions**: Check status of all sessions
- **read_artifact**: Read generated documentation
- **write_plan**: Document current plan and progress

The MANAGER operates in a feedback loop:
1. Receive input (user goal or session output)
2. Analyze and update mental model
3. Decide next action
4. Execute via tools
5. Wait for next input

See [ROLES/MANAGER.md](../../ROLES/MANAGER.md) for complete role definition.

#### Child Session Management
Child sessions (ARCHITECT, DEVELOPER, STAKEHOLDER) are:
- Spawned as Claude Code CLI child processes (see [ADR-0001](../adr/0001-claude-cli-wrapper-architecture.md))
- Provided with role context from `ROLES/{ROLE}.md`
- Given specific tasks and relevant context
- Expected to produce code changes and documentation artifacts
- Monitored for completion, errors, and outputs
- Automatically cleaned up on exit to prevent orphans

#### Session Persistence
All sessions log I/O for persistence and debugging:
- I/O logged to `.claude-man/sessions/{SESSION_ID}/io.log` in JSONL format
- Session metadata in `metadata.json`
- Sessions can survive restarts via log replay
- See [ADR-0003: Session Persistence](../adr/0003-session-persistence-io-logging.md)

#### Authentication
claude-man-cli uses environment-based authentication:
- Auth token stored in `CLAUDE_AUTH_TOKEN` environment variable
- Token shared with all child sessions via environment inheritance
- `claude-man login` command for initial authorization via browser OAuth flow
- Token optionally persisted to shell config
- See [ADR-0004: Environment-Based Authentication](../adr/0004-environment-based-authentication.md)

## Features

### 1. Session Management

#### Start Session
```bash
claude-man start [--role ROLE] [--task TASK_ID] [--context CONTEXT_FILES]
```
- Spawn new Claude session
- Apply role context (DEVELOPER/ARCHITECT/STAKEHOLDER)
- Load relevant documentation artifacts
- Generate initialization prompt
- Assign unique session ID

#### List Sessions
```bash
claude-man list [--status active|completed|failed]
```
- Show all active sessions
- Display session ID, role, task, status
- Show elapsed time and activity
- Indicate completion status

#### Attach to Session
```bash
claude-man attach SESSION_ID
```
- Connect to running session
- View conversation history
- Send additional prompts
- Monitor progress

#### Stop Session
```bash
claude-man stop SESSION_ID [--save] [--reason REASON]
```
- Gracefully terminate session
- Capture final state and artifacts
- Generate session summary
- Clean up resources

### 2. Context Transfer

#### Artifact Generation
Sessions are prompted to produce:
- **Specs**: Requirements and acceptance criteria for new features
- **ADRs**: Architecture decisions made during implementation
- **Summaries**: What was accomplished, challenges, outcomes
- **Task Lists**: Follow-on work identified during session
- **Code Maps**: Where changes were made and why

#### Context Loading
When starting a session:
- Load relevant specs from `docs/spec/`
- Load relevant ADRs from `docs/adr/`
- Include summaries from related sessions
- Reference related code changes
- Provide dependency information

#### Smart Context Selection
Context Manager determines relevance by:
- Task dependencies and relationships
- File/module proximity
- Recent changes and activity
- Explicit context hints from user
- Role-specific needs

### 3. Parallel Execution

#### Task Parallelization
```bash
claude-man parallel TASK_FILE
```
- Parse task list with dependencies
- Identify independent tasks
- Spawn multiple sessions in parallel
- Monitor for completion
- Handle inter-task dependencies

#### Dependency Management
- Detect task dependencies
- Ensure dependent tasks wait for prerequisites
- Transfer artifacts from completed tasks
- Fail gracefully on blocking errors

#### Conflict Detection
- Monitor for overlapping file changes
- Detect conflicting design decisions
- Alert user to potential conflicts
- Suggest resolution strategies

### 4. Prompt Generation

#### Effective Prompts
Generated prompts include:
- Role context and responsibilities
- Task description and acceptance criteria
- Relevant documentation artifacts
- Code references and context
- Required output artifacts
- Success criteria

#### Artifact Requirements
Prompts specify required outputs:
```
After completing this task, produce:
1. [docs/spec/feature-name.md] - Specification for future implementation
2. [docs/adr/NNNN-decision-title.md] - ADR for design choices made
3. [session-summary.md] - Summary of changes and outcomes
4. [follow-on-tasks.md] - Additional work identified
```

#### Role-Specific Prompting
- **DEVELOPER**: Focus on implementation, testing, code quality
- **ARCHITECT**: Focus on design, patterns, technical decisions
- **STAKEHOLDER**: Focus on requirements, validation, business value

### 5. Workflow Orchestration

#### Standard Workflows

**Feature Development**
```bash
claude-man workflow feature "User authentication"
```
1. Start ARCHITECT session: Design authentication system, produce ADR
2. Wait for ADR completion
3. Start 2 DEVELOPER sessions in parallel:
   - Session A: Implement backend authentication
   - Session B: Implement frontend login UI
4. Wait for completion
5. Start DEVELOPER session: Integration and testing
6. Produce final spec and summary

**Bug Fix Workflow**
```bash
claude-man workflow bugfix "Fix login timeout issue" --issue 123
```
1. Start DEVELOPER session: Investigate and diagnose
2. Produce diagnosis spec
3. If architectural change needed:
   - Start ARCHITECT session: Design fix, produce ADR
4. Start DEVELOPER session: Implement fix
5. Produce test results and summary

**Code Review Workflow**
```bash
claude-man workflow review --pr 456
```
1. Start ARCHITECT session: Review design and patterns
2. Start DEVELOPER session: Review implementation and tests
3. Aggregate feedback
4. Produce review summary

#### Custom Workflows
Define workflows in YAML:
```yaml
name: custom-workflow
steps:
  - id: design
    role: ARCHITECT
    task: Design the ${FEATURE} system
    outputs:
      - docs/adr/NNNN-${FEATURE}-design.md

  - id: implement
    role: DEVELOPER
    depends_on: design
    parallel: 2
    task: Implement ${FEATURE} based on ADR
    context:
      - docs/adr/NNNN-${FEATURE}-design.md
    outputs:
      - session-summary.md
```

### 6. Monitoring and Reporting

#### Real-time Status
```bash
claude-man status [--watch]
```
- Show active sessions
- Display current activity
- Show recent artifact generation
- Estimate completion time

#### Session Logs
```bash
claude-man logs SESSION_ID [--follow]
```
- View session conversation
- See artifact generation
- Monitor errors and warnings
- Track progress

#### Summary Reports
```bash
claude-man report [--since TIMESTAMP] [--format json|markdown]
```
- Aggregate session outcomes
- List artifacts produced
- Show task completion stats
- Identify blocking issues

## Command-Line Interface

### Core Commands

```bash
# Authentication (see ADR-0004)
claude-man login [--save-to-shell]
claude-man login --refresh
claude-man logout
claude-man auth status

# Primary interaction (starts MANAGER session)
claude-man <goal>
# Example: claude-man "implement user authentication"
# The MANAGER session handles everything from here

# Session management (advanced/manual use)
claude-man list [--status STATUS]
claude-man attach SESSION_ID
claude-man stop SESSION_ID [--reason REASON]

# Monitoring
claude-man status [--watch]
claude-man logs SESSION_ID [--follow]
claude-man report [OPTIONS]

# Artifacts
claude-man artifacts list [--session SESSION_ID]
claude-man artifacts export SESSION_ID OUTPUT_DIR

# Configuration
claude-man config set KEY VALUE
claude-man config get KEY
claude-man config list
```

### Options and Flags

#### Start Options
- `--role ROLE`: Assign role (DEVELOPER/ARCHITECT/STAKEHOLDER)
- `--task TASK_ID`: Assign specific task
- `--context FILES`: Load specific context files
- `--branch BRANCH`: Work on specific git branch
- `--prompt FILE`: Use custom prompt file
- `--artifacts DIR`: Specify artifact output directory

#### List Options
- `--status STATUS`: Filter by status (active/completed/failed)
- `--role ROLE`: Filter by role
- `--since TIMESTAMP`: Show sessions since timestamp

#### Parallel Options
- `--max-sessions N`: Limit concurrent sessions
- `--fail-fast`: Stop all on first failure
- `--continue-on-error`: Continue despite failures

## Context Transfer Protocol

### Artifact Structure

#### Session Summary
```markdown
# Session Summary: SESSION_ID

**Role**: DEVELOPER
**Task**: Implement user authentication
**Status**: Completed
**Duration**: 45 minutes

## What Was Done
- Implemented JWT-based authentication
- Added login/logout endpoints
- Created user session management
- Added authentication middleware

## Changes Made
- `src/auth/jwt.ts`: New JWT token generation and validation
- `src/api/auth.ts`: Login and logout endpoints
- `src/middleware/auth.ts`: Authentication middleware
- `tests/auth.test.ts`: Authentication tests

## Decisions Made
See [ADR-0005: Use JWT for Authentication](../adr/0005-jwt-authentication.md)

## Follow-on Tasks
1. Add password reset functionality
2. Implement refresh token rotation
3. Add rate limiting to login endpoint
4. Add OAuth2 provider integration

## Blockers/Issues
None

## Artifacts Produced
- This summary
- ADR-0005
- Test results
```

#### Task Specification
```markdown
# Task: PASSWORD_RESET

## Context
User authentication system implemented in Session X.
See [Session Summary](./session-X-summary.md)

## Objective
Implement password reset functionality for users who forget their password.

## Requirements
1. Email-based password reset flow
2. Time-limited reset tokens
3. Secure token validation
4. Password update endpoint

## Acceptance Criteria
- [ ] User can request password reset via email
- [ ] Reset email contains secure, time-limited link
- [ ] Token expires after 1 hour
- [ ] User can set new password with valid token
- [ ] Old password is invalidated
- [ ] Tests cover happy path and edge cases

## Context Files
- `src/auth/jwt.ts`: Authentication implementation
- `docs/adr/0005-jwt-authentication.md`: Auth design decisions

## Dependencies
- Requires email service integration (see Task EMAIL_SERVICE)

## Expected Artifacts
1. Implementation code
2. Tests
3. Session summary
4. Update to auth spec if needed
```

### Context Loading Protocol

When starting a session:

1. **Load Role Context**
   - Read `ROLES/{ROLE}.md`
   - Include role responsibilities and focus areas

2. **Load Task Context**
   - Parse task specification
   - Identify dependencies
   - Load referenced context files

3. **Load Related Artifacts**
   - Recent session summaries (last 5)
   - Related specs from `docs/spec/`
   - Related ADRs from `docs/adr/`
   - Code files mentioned in context

4. **Load Project State**
   - Current git branch and status
   - Recent commits (last 10)
   - Open PRs and issues
   - Build/test status

5. **Generate Context Package**
   - Prioritize by relevance
   - Fit within context window
   - Include links to full documents
   - Provide summary of omitted context

## Technical Requirements

### Environment
- **Platform**: Cross-platform (Windows, macOS, Linux)
- **Language**: Rust (see [ADR-0005: Rust Implementation](../adr/0005-rust-implementation.md))
- **Distribution**: Single compiled binary, no runtime dependencies
- **Claude Integration**: Wraps Claude Code CLI as child processes
- **Storage**: Local filesystem for artifacts and session logs
- **Git**: Integration with git for tracking changes

### Configuration
```yaml
# ~/.claude-man/config.yaml
api:
  key: ANTHROPIC_API_KEY
  model: claude-sonnet-4-5
  max_tokens: 8000

sessions:
  max_concurrent: 5
  default_role: DEVELOPER
  timeout_minutes: 60

artifacts:
  output_dir: ./docs
  spec_dir: ./docs/spec
  adr_dir: ./docs/adr
  session_summaries_dir: ./docs/sessions

context:
  max_files: 20
  max_size_mb: 5
  include_git_history: true

workflows:
  custom_dir: ./.claude-man/workflows
```

### API Integration
- Use Anthropic Claude API
- Manage API rate limits
- Handle streaming responses
- Retry on failures
- Track token usage

### Artifact Management
- Create artifacts in structured format
- Version control artifacts via git
- Link artifacts to sessions
- Index artifacts for search
- Archive completed sessions

## Workflows and Use Cases

### Use Case 1: Feature Development

**Scenario**: Build a new "user profile" feature

**Steps**:
```bash
# 1. Architect designs the feature
$ claude-man start --role ARCHITECT --task "Design user profile system"
Session ARCH-001 started
...
ARCH-001 completed
Artifacts:
  - docs/adr/0010-user-profile-design.md
  - docs/spec/user-profile.md

# 2. Parallel implementation
$ claude-man parallel --context docs/spec/user-profile.md << EOF
- task: implement-backend
  role: DEVELOPER
  description: Implement user profile backend API

- task: implement-frontend
  role: DEVELOPER
  description: Implement user profile UI components

- task: implement-storage
  role: DEVELOPER
  description: Implement profile data storage
EOF

3 sessions started in parallel
DEV-002: implement-backend (active)
DEV-003: implement-frontend (active)
DEV-004: implement-storage (active)

...all sessions completed...

# 3. Integration
$ claude-man start --role DEVELOPER \
  --context docs/sessions/DEV-002-summary.md \
  --context docs/sessions/DEV-003-summary.md \
  --context docs/sessions/DEV-004-summary.md \
  --task "Integrate user profile components and test"

Session DEV-005 started
...
DEV-005 completed
```

### Use Case 2: Code Refactoring

**Scenario**: Refactor authentication module

**Steps**:
```bash
# 1. Architect reviews current design
$ claude-man start --role ARCHITECT --task "Review auth module for refactoring opportunities"
Session ARCH-006 started
...
ARCH-006 completed
Artifacts:
  - docs/adr/0011-auth-refactoring-plan.md

# 2. Execute refactoring in parallel
$ claude-man parallel --context docs/adr/0011-auth-refactoring-plan.md << EOF
- task: refactor-jwt
  role: DEVELOPER
  description: Refactor JWT handling per ADR-0011

- task: refactor-middleware
  role: DEVELOPER
  description: Refactor auth middleware per ADR-0011
EOF

# 3. Validate changes
$ claude-man start --role DEVELOPER --task "Validate refactoring and update tests"
```

### Use Case 3: Bug Investigation

**Scenario**: Investigate timeout issue

**Steps**:
```bash
# 1. Investigate in parallel
$ claude-man parallel << EOF
- task: investigate-logs
  role: DEVELOPER
  description: Analyze server logs for timeout patterns

- task: investigate-code
  role: DEVELOPER
  description: Review authentication code for timeout causes

- task: investigate-network
  role: DEVELOPER
  description: Check network configuration and latency
EOF

# 2. Review findings and design fix
$ claude-man start --role ARCHITECT \
  --context docs/sessions/*-investigate-*.md \
  --task "Review investigation findings and design fix"

# 3. Implement fix
$ claude-man start --role DEVELOPER \
  --context docs/adr/NNNN-timeout-fix.md \
  --task "Implement timeout fix per ADR"
```

## Success Criteria

### Functional Success
- ✓ Can spawn and manage multiple Claude sessions
- ✓ Sessions produce required documentation artifacts
- ✓ Context successfully transfers between sessions
- ✓ Parallel execution completes independent tasks
- ✓ Artifacts enable follow-on work without manual context transfer
- ✓ Workflows complete complex tasks with minimal user intervention

### Performance Success
- ✓ Reduces developer time vs. single-session approach
- ✓ Completes parallel tasks in ~1/N time (N = parallelism)
- ✓ Context loading adds < 10% overhead vs. manual setup
- ✓ API usage stays within rate limits
- ✓ Artifact generation doesn't significantly slow sessions

### Quality Success
- ✓ Generated code meets project standards
- ✓ ADRs capture important design decisions
- ✓ Specs are clear and actionable
- ✓ Session summaries accurately reflect work done
- ✓ Context transfer maintains coherence
- ✓ Artifacts are discoverable and useful

### Usability Success
- ✓ CLI is intuitive and well-documented
- ✓ Workflows abstract common patterns
- ✓ Error messages are clear and actionable
- ✓ Configuration is straightforward
- ✓ Monitoring provides useful visibility
- ✓ Learning curve < 1 hour for basic usage

## Future Enhancements

### Phase 2 Features
- **Interactive Mode**: REPL for managing sessions interactively
- **Web Dashboard**: Visual interface for monitoring sessions
- **Session Replay**: Replay and fork previous sessions
- **Smart Conflict Resolution**: Auto-merge compatible changes
- **Cost Tracking**: Monitor and optimize API usage costs

### Phase 3 Features
- **Multi-Project Support**: Manage sessions across multiple projects
- **Team Collaboration**: Share sessions and artifacts with team
- **Template Library**: Pre-built workflows and prompts
- **Plugin System**: Extend with custom functionality
- **CI/CD Integration**: Run workflows in CI/CD pipelines

## Open Questions

1. **Session Persistence**: Should sessions be resumable after CLI restart?
2. **Artifact Versioning**: How to version artifacts vs. using git?
3. **Context Window Management**: Auto-split tasks when context too large?
4. **Error Recovery**: How to handle partial failures in parallel execution?
5. **Prompt Refinement**: How to improve prompts based on outcomes?
6. **Security**: How to handle sensitive data in session context?
7. **Testing**: How to test orchestration without live API calls?
8. **Artifact Format**: Strict schema vs. flexible markdown?

## References

- [Claude API Documentation](https://docs.anthropic.com/claude/reference)
- [ROLES](../../ROLES/) - Team role definitions
- [CLAUDE.md](../../CLAUDE.md) - Project rules and guidelines
- [ADR Template](../adr/template.md) - Architecture decision record format
