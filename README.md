# tasklog

A Git → Iotas task logger. Automatically extracts commit messages from your git repositories and syncs them to the Iotas note-taking app, organized by week.

## Overview

**tasklog** processes git commits from configured repositories and creates weekly task logs in [Iotas](https://world.gnome.org/iotas/), a GNOME note-taking application. It filters commits by author and groups them by ISO week for easy review and export.

## Features

- **Automatic log extraction**: Parse commits from multiple git repositories
- **Weekly organization**: Group commits by ISO week
- **Sync to Iotas**: Store logs directly in the Iotas database
- **Export capability**: Export weekly logs to stdout
- **System verification**: Built-in health check command

## Commands

```bash
tasklog run                    # Process all repos and sync to Iotas
tasklog export [week]          # Export logs for a week (default: current week)
tasklog export 2026-W14        # Export logs for a specific week
tasklog config                 # Interactively manage configured repositories
tasklog doctor                 # Check system configuration
tasklog help                   # Show help message
```

## Setup

### Prerequisites

- [Bun](https://bun.com) runtime
- [Iotas](https://apps.gnome.org/Iotas/) note-taking app installed
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

This will guide you through adding or removing repository paths. You can also manually edit `~/.config/git-tasklog/repos.conf` with one path per line.

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

- **core/**: Core logic for git parsing, data grouping, and Iotas integration
  - `git.ts`: Fetch commits from repositories
  - `parser.ts`: Group commits by week
  - `iotas.ts`: Database operations for Iotas
  - `week.ts`: ISO week utilities
- **commands/**: CLI commands
  - `run.ts`: Main sync command
  - `export.ts`: Export logs for a week
  - `doctor.ts`: System health check
- **infra/**: Infrastructure and utilities
  - `config.ts`: Configuration management
  - `logger.ts`: Logging utilities
  - `lock.ts`: File locking

## Configuration

### Default Paths

- **Config file**: `~/.config/git-tasklog/repos.conf`
- **Iotas DB**: `~/.var/app/org.gnome.World.Iotas/data/iotas/notes.db`
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
- **Database**: SQLite (Iotas database)
- **VCS**: Git
