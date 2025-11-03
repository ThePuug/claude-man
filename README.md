# claude-man

A CLI tool for orchestrating multiple Claude AI sessions to enable parallel development workflows with context coherence.

## Overview

claude-man-cli allows developers to parallelize AI-assisted development by managing multiple Claude sessions from a single interface. The tool itself runs as a MANAGER Claude session that spawns and coordinates child sessions (ARCHITECT, DEVELOPER, STAKEHOLDER roles), transferring context between them via documentation artifacts to maintain coherence while maximizing productivity.

## Getting Started

### Prerequisites

List any dependencies, tools, or system requirements needed to run this project.

### Installation

```bash
# Clone the repository
git clone <repository-url>

# Navigate to project directory
cd claude-man

# Install dependencies (adjust based on your project)
# npm install
# pip install -r requirements.txt
# etc.
```

### Usage

Basic usage instructions and examples.

```bash
# Example commands
```

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
