# Project Rules

## Code Standards

### General Guidelines
- Follow consistent coding style across the project
- Write clear, self-documenting code
- Keep functions small and focused on a single responsibility
- Use meaningful variable and function names

### Documentation
- Document all public APIs and interfaces
- Keep documentation up-to-date with code changes
- Include examples in documentation where appropriate

### Version Control
- Write clear, descriptive commit messages
- Keep commits atomic and focused
- Reference issue numbers in commit messages when applicable
- Never commit sensitive information (credentials, API keys, etc.)

### Code Review
- All code must be reviewed before merging to main branch
- Address all review comments before merging
- Ensure CI/CD checks pass before requesting review

### Testing
- Write tests for new functionality
- Maintain or improve code coverage
- Run tests locally before pushing

### Architecture
- Follow established architectural patterns
- Consult with architects before making significant architectural changes
- Document architectural decisions in ADRs (see docs/adr)

## Workflow

### Branching Strategy
- `main` - production-ready code
- `develop` - integration branch for features
- `feature/*` - feature branches
- `bugfix/*` - bug fix branches
- `hotfix/*` - urgent production fixes

### Development Process
1. Create feature branch from appropriate base branch
2. Implement changes with tests
3. Update documentation as needed
4. Submit pull request for review
5. Address review feedback
6. Merge after approval

## Communication
- Use issue tracker for bug reports and feature requests
- Keep team informed of blockers and dependencies
- Document decisions and share knowledge
