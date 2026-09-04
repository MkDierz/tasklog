# tasklog

A Git → CSV task logger. Automatically extracts commit messages from your git repositories, filtered by author and configured branches, and dumps them to a CSV file organized by day.

## Overview

**tasklog** reads commits from configured repositories and appends each one as a row (`path,commit,date`) to a CSV file. It filters commits by author (from `git config user.name`) and supports multiple configurable branches per repository.

## Features

- **Automatic log extraction**: Parse commits from multiple git repositories
- **Configurable branches**: Read from `main`, `dev`, or any branch — multiple per repo
- **CSV output**: Append commit data to a daily CSV file
- **Export by date**: Filter and print logs for a specific day
- **System verification**: Built-in health check command

## Commands

```bash
tasklog run                    # Process all repos and append commits to CSV
tasklog export [date]          # Export logs for a day (default: today)
tasklog export 2026-09-03      # Export logs for a specific date
tasklog config                 # Interactively manage configured repositories
tasklog doctor                 # Check system configuration
tasklog help                   # Show help message
```

## Setup

### Prerequisites

- [Bun](https://bun.com) runtime
- Git repositories to monitor

### Installation

1. Install dependencies:

```bash
bun install
```

2. Configure repositories using the interactive config tool:

```bash
tasklog config
```

This will guide you through adding, removing, or editing repositories. You can also manually edit `~/.config/git-tasklog/repos.conf`:

```
/path/to/repo main dev
# or just one branch:
/another/repo main
```

One repository per line. The first token is the path; remaining tokens are space-separated branches (`main` is the default if omitted). Lines starting with `#` are ignored.

3. Build the executable:

```bash
make build
# or for optimized release:
make release
```

4. Install globally:

```bash
make install
```

## Development

### Quick start

```bash
make dev              # Run in development mode
make help             # Show available commands
```

### Building

```bash
make build            # Create debug binary
make release          # Create optimized binary
make clean            # Remove build artifacts
```

## Architecture

- **core/**: Core logic for git parsing
  - `git.ts`: Fetch commits from repositories
- **commands/**: CLI commands
  - `run.ts`: Fetch commits and append to CSV
  - `export.ts`: Export logs for a given date
  - `config.ts`: Interactive repository management
  - `doctor.ts`: System health check
  - `service.ts`: Systemd user service control
- **infra/**: Infrastructure and utilities
  - `config.ts`: Configuration constants and paths
  - `logger.ts`: Logging utilities
  - `lock.ts`: File locking

## Configuration

### Default Paths

- **Config file**: `~/.config/git-tasklog/repos.conf`
- **CSV output**: `~/.local/share/git-tasklog/tasklog.csv`
- **Log file**: `~/.local/share/git-tasklog/run.log`

### Author

The tool automatically reads the author name from your git configuration using `git config user.name`. Make sure your git config is properly set:

```bash
git config --global user.name "Your Name"
```

Tasklog will use this name to filter commits from your configured repositories.

## Technical Stack

- **Runtime**: [Bun](https://bun.com)
- **Language**: TypeScript
- **Output**: CSV
- **VCS**: Git