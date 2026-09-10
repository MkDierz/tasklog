# tasklog

A Rust-based Git → CSV task logger. It reads commit messages from configured repositories, filters them by the configured Git author, and appends them to a CSV file.

## Overview

**tasklog** scans each configured repository, collects commits on the configured branches, and writes rows in the format `path,commit,date` to `~/.local/share/git-tasklog/tasklog.csv`.

## Features

- Git commit extraction from multiple repositories
- Support for multiple branches per repository
- CSV output with deduplication across the current run
- Export by day from the generated CSV
- Simple config and doctor commands
- Systemd user-service support

## Commands

```bash
tasklog run
ntasklog export [date]
tasklog config
tasklog doctor
tasklog service [enable|disable|start|stop|status]
```

## Setup

### Prerequisites

- Rust toolchain
- Git repositories to monitor

### Configure repositories

Create or edit `~/.config/git-tasklog/repos.conf`:

```text
/path/to/repo main dev
# or just one branch:
/another/repo main
```

One repository per line. The first token is the path; remaining tokens are space-separated branches. `main` is used by default when omitted.

### Build and install

```bash
cargo build
make build
make install
```

You can also run it directly from Cargo:

```bash
cargo run -- run
```

## Default Paths

- Config file: `~/.config/git-tasklog/repos.conf`
- CSV output: `~/.local/share/git-tasklog/tasklog.csv`

## Author

The tool reads your Git author from:

```bash
git config user.name
```

If it is not set, the app falls back to `Unknown Author`.

## Development

```bash
make help
make build
make release
make run
```
