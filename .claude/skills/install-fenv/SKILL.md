---
name: install-fenv
description: Install fenv for development and testing purposes
allowed-tools: Bash
argument-hint: [latest|snapshot|version]
---

# Install fenv

Install fenv using one of three modes based on the argument provided.

## Mode Detection

- No argument or `latest` → Install latest release from remote
- `snapshot` → Build from local source and install
- Any other value (e.g., `0.3.0`, `v0.3.0`) → Install specific version

## Instructions

### If mode is `latest` (or no argument):

Run:

```bash
curl -fsSL "https://fenv-install.jerry.company" | bash
```

### If mode is `snapshot`:

Build from local source and install:

```bash
cargo build && cp target/debug/fenv ${FENV_ROOT:-~/.fenv}/bin/ \
  && cp shims/* ${FENV_ROOT:-~/.fenv}/shims/
```

### If mode is a version (e.g., `0.3.0` or `v0.3.0`):

Install the specific version. Auto-prefix `v` if the version doesn't start with
`v`:

```bash
curl -fsSL "https://fenv-install.jerry.company" |
  FENV_VERSION=v{version} bash
```

For example:

- `0.3.0` → `FENV_VERSION=v0.3.0`
- `v0.3.0` → `FENV_VERSION=v0.3.0`
