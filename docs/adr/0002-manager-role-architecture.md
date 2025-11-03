# 0002. MANAGER Role and Session Orchestration

Date: 2025-11-03

## Status

Accepted

## Context

claude-man-cli needs to coordinate multiple Claude sessions, decide what work to do, spawn appropriate sessions, and handle their outputs. We need to determine who/what makes these orchestration decisions.

Traditional approaches:
- **Imperative orchestration**: Hard-coded workflows and decision trees
- **Declarative workflows**: YAML/config files defining task flows
- **User-directed**: User manually decides what to spawn and when

The challenge:
- Pre-defined workflows are rigid and can't adapt to findings
- User-directed orchestration defeats the purpose of automation
- Hard-coded decision trees become complex and brittle
- Need to handle dynamic task discovery and failures intelligently

## Decision

**claude-man-cli itself runs as a Claude session with the MANAGER role.**

The MANAGER:
- Is a Claude session that orchestrates other Claude sessions
- Receives outputs from child sessions as inputs
- Plans and decides what actions to take next
- Spawns new sessions with appropriate roles and context
- Treats failures as inputs that trigger re-evaluation
- Maintains overall project context and goals

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│              claude-man-cli (MANAGER)                    │
│  ┌──────────────────────────────────────────────────┐  │
│  │        Claude Session (MANAGER Role)             │  │
│  │  - Reads child session outputs                   │  │
│  │  - Plans next actions                            │  │
│  │  - Spawns/controls child sessions                │  │
│  │  - Handles failures and re-planning              │  │
│  └──────────────────────────────────────────────────┘  │
│                         │                                │
│         ┌───────────────┼───────────────┐               │
│         │               │               │               │
│    ┌────▼────┐    ┌────▼────┐    ┌────▼────┐          │
│    │ ARCH    │    │  DEV    │    │  DEV    │          │
│    │ Session │    │ Session │    │ Session │          │
│    └────┬────┘    └────┬────┘    └────┬────┘          │
│         │               │               │               │
│         └───────────────┼───────────────┘               │
│                         │                                │
│                  (outputs/artifacts)                     │
│                         │                                │
│         ┌───────────────▼───────────────┐               │
│         │     MANAGER processes:         │               │
│         │  - Session summaries           │               │
│         │  - Generated artifacts         │               │
│         │  - Failure reports             │               │
│         │  - Decides next actions        │               │
│         └────────────────────────────────┘               │
└─────────────────────────────────────────────────────────┘
```

### MANAGER Capabilities

The MANAGER has access to tools:
- `spawn_session(role, task, context)` - Create new child session
- `attach_session(session_id)` - Send additional prompts to running session
- `stop_session(session_id, reason)` - Terminate session
- `list_sessions()` - Get status of all sessions
- `read_artifact(path)` - Read generated artifacts
- `write_plan(plan)` - Document current plan

### Feedback Loop

```
User provides goal
    ↓
MANAGER creates plan
    ↓
MANAGER spawns sessions (ARCHITECT, DEVELOPER, etc.)
    ↓
Sessions produce outputs (code, artifacts, errors)
    ↓
MANAGER receives outputs as inputs
    ↓
MANAGER evaluates outcomes:
    - Success? Proceed to next step
    - Failure? Re-plan and retry
    - New discovery? Adapt plan
    ↓
Repeat until goal achieved or blocked
```

### Example Flow

**User**: "Implement user authentication"

**MANAGER thinks**:
- Need architectural design first
- Spawn ARCHITECT session with task

**ARCHITECT completes**:
- Produces ADR-0005: JWT Authentication Design
- Identifies 3 parallel implementation tasks

**MANAGER receives** ARCHITECT summary and ADR:
- Reads ADR-0005
- Understands 3 tasks can run in parallel
- Spawns 3 DEVELOPER sessions with appropriate context

**DEV-001 fails**: Database connection error

**MANAGER receives** failure report:
- Analyzes error
- Decides database setup is prerequisite
- Spawns DEV-004 to set up database
- Will retry DEV-001 after DEV-004 completes

**All sessions complete**:
- MANAGER reviews all outputs
- Spawns integration/testing session
- Reports completion to user

## Consequences

### Positive Consequences

- **Adaptive planning**: MANAGER can adjust based on findings, not locked into pre-defined workflows
- **Intelligent error handling**: Failures are analyzed and handled contextually, not just retried blindly
- **Natural language interface**: User describes goals, MANAGER figures out how to achieve them
- **Context coherence**: MANAGER maintains big picture across all sessions
- **Dynamic task discovery**: New tasks can be identified during execution
- **Simplified implementation**: No complex workflow engine needed - just prompt engineering
- **Explainable decisions**: MANAGER's reasoning is visible in its conversation
- **Handles ambiguity**: Can ask user for clarification when needed

### Negative Consequences

- **API costs**: MANAGER session runs continuously, consuming tokens
- **Latency**: MANAGER must think/plan between steps (adds overhead)
- **Non-deterministic**: MANAGER might make different decisions each run
- **Complexity in testing**: Hard to unit test an AI decision-maker
- **Potential for poor decisions**: MANAGER might make suboptimal choices
- **Context limits**: MANAGER context can fill up with session outputs
- **Dependency on Claude quality**: Orchestration quality depends on Claude's reasoning

### Neutral Consequences

- **Meta-architecture**: System architecture is AI managing AI
- **Prompt engineering critical**: MANAGER effectiveness depends on role definition and prompting
- **Observability**: Need to log MANAGER's decision-making process

## Implementation Notes

### MANAGER Role Definition

The MANAGER role (see `ROLES/MANAGER.md`) includes:
- Responsibilities: Plan, orchestrate, coordinate, handle failures
- Available tools: Session management, artifact access, planning
- Decision authority: When to spawn sessions, what roles, context to provide
- Success criteria: Achieve user goals efficiently with quality results

### Tool Implementation

Tools available to MANAGER are implemented as:
- **Node.js functions** that the CLI executes on behalf of MANAGER
- **Formatted output** that MANAGER can parse and understand
- **Structured commands** that MANAGER can issue

Example MANAGER command:
```markdown
I need to spawn an ARCHITECT session to design the authentication system.

<spawn_session>
role: ARCHITECT
task: Design JWT-based authentication system
context_files:
  - docs/spec/user-requirements.md
  - ROLES/ARCHITECT.md
artifacts_required:
  - docs/adr/NNNN-authentication-design.md
  - docs/spec/authentication-api.md
</spawn_session>
```

CLI parses this, spawns session, returns session ID to MANAGER.

### Context Management

MANAGER context includes:
- User's original goal
- Current plan and progress
- Active session summaries (abbreviated)
- Recently completed artifacts (links or summaries)
- Recent failures and decisions made

To prevent context overflow:
- Summarize completed work
- Prune old session details
- Keep only relevant artifacts in full
- Use references/links instead of full content

### Failure Handling Pattern

When session fails:
1. MANAGER receives failure output
2. MANAGER analyzes error
3. MANAGER decides:
   - Retry with same approach?
   - Change approach?
   - Spawn helper session to resolve blocker?
   - Escalate to user?
4. MANAGER documents decision
5. MANAGER executes decision

## Alternatives Considered

### Alternative 1: Declarative Workflows

Define workflows in YAML, execute deterministically.

**Why rejected**:
- Too rigid, can't adapt to findings
- Complex workflow DSL needed
- Still need to handle errors programmatically
- Defeats purpose of having AI assistance

### Alternative 2: User as Orchestrator

User manually decides what sessions to spawn and when.

**Why rejected**:
- High cognitive load on user
- Defeats automation purpose
- User must track all session states
- Not scalable to complex tasks

### Alternative 3: Imperative Code Orchestration

Write TypeScript/JavaScript orchestration logic.

**Why rejected**:
- Inflexible
- Requires code changes for new patterns
- Poor handling of dynamic discoveries
- Doesn't leverage AI reasoning

## Migration Path

Phase 1 (MVP):
- MANAGER with basic commands (spawn, stop, list)
- Simple task execution
- Manual user guidance when stuck

Phase 2 (Enhanced):
- More sophisticated failure handling
- Context optimization
- Better artifact summarization
- Proactive blocker detection

Phase 3 (Advanced):
- Multi-project orchestration
- Cost optimization strategies
- Learning from past executions
- Team collaboration features

## References

- [ROLES/MANAGER.md](../../ROLES/MANAGER.md) - MANAGER role definition
- [Claude Tool Use Documentation](https://docs.anthropic.com/claude/docs/tool-use)
- [ADR-0001: Claude CLI Wrapper](./0001-claude-cli-wrapper-architecture.md)
