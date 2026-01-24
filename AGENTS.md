# AGENTS.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

fenv is a Rust CLI tool for managing multiple Flutter SDK versions. It's a single static-linked executable with no external dependencies (not even Dart). Inspired by pyenv.

## Build & Test Commands

```bash
cargo build                    # Development build
cargo build --release          # Release build
cargo test                     # Run all tests
cargo test <test_name>         # Run specific test
tools/coverage.sh              # Generate LCOV coverage report
tools/coverage.sh --html       # Generate HTML coverage report
```

## Architecture

- `src/main.rs` - Entry point, sets up context and runs CLI
- `src/lib.rs` - Command dispatch logic for 14+ subcommands
- `src/args.rs` - Clap CLI argument definitions
- `src/context.rs` - `FenvContext` trait defining paths (home, fenv_root, versions dir, etc.)
- `src/sdk_service/` - SDK discovery and management
  - `local_repository.rs` - Manages locally installed SDKs
  - `remote_repository.rs` - Fetches remote SDK list from Flutter APIs
- `src/service/` - Business logic for each command (install, uninstall, global, local, etc.)
- `src/external/` - External command execution (flutter, git)

Key environment variables:
- `FENV_ROOT` - Installation root (default: `$HOME/.fenv`)
- `FENV_DIR` - Working directory (default: `$PWD`)

## Testing Requirements

This project aims for high test coverage. No manual testing should be required—all behavior must be verified by automated tests.

1. **Refactoring (no behavior change)**: All existing tests must pass.

2. **Bug fixes**: Add a new test case that proves the bug is fixed.

3. **Behavior changes**: Update existing test cases to reflect the new behavior and ensure tests pass.

4. **New CLI-accessible features**: Add integration tests in `*_service.rs` using `try_run("fenv", "...")` to verify stdout/stderr output. All testable scenarios must have integration test coverage.

5. **New internal components (not yet CLI-accessible)**: Add unit tests. If mocking external dependencies is needed, consult the user first.

## Commit Rules

Write simple, direct commit messages. Do NOT use conventional commit prefixes.

Good:
```
Add dark mode support
Fix login timeout issue
Update dependencies
Remove deprecated API
```

Bad:
```
feat: Add dark mode support
fix(auth): Fix login timeout issue
chore: Update dependencies
```

Keep messages short (under 72 characters). One sentence is enough.

## Pull Request Rules

PR titles MUST use conventional commit prefixes (without scope):

```
feat: Add dark mode support
fix: Resolve login timeout issue
docs: Update README
refactor: Simplify authentication flow
test: Add unit tests for parser
chore: Update dependencies
ci: Fix GitHub Actions workflow
```

PR body should contain only a brief summary (one or two bullet points). No section headers.
