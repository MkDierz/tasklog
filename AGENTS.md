# AGENTS.md

## Runtime & Toolchain

- **Runtime**: Bun (not Node). `node:fs` for file I/O is fine here; no SQLite is used on this branch (Iotas DB was removed).
- **Module system**: ESM (`"type": "module"` in package.json). `verbatimModuleSyntax` is on — use `import type` for type-only imports.
- **Build**: `make build` (debug) or `make release` (minified). Output goes to `dist/`.
- **No test framework, linter, or typecheck script** is configured. There are no tests to run.

## Quick Commands

```
bun run cli.ts <command>     # dev run
make dev                     # same, via Makefile
make build                   # compile to dist/tasklog
make release                 # optimized compile
```

## Architecture

Single entry point:

- `cli.ts` — the CLI entry, dispatches to `commands/`. Used by `make build`.

Module layout:

```
commands/   CLI subcommands (run, export, config, doctor, service)
core/       Git parsing (iotas.ts, parser.ts, week.ts all deleted on this branch)
infra/      Config, logger, file locking
```

## Key Gotchas

- **Configurable branches**: `core/git.ts` runs `git log <branch>` per configured branch. Config lines are `path [branch branch ...]` (space-separated; `main` is the default if omitted). Commits are deduped across branches by `repo|date|subject` in `commands/run.ts`, but rows still append across runs (no cross-run dedup).
- **Author from git config**: `infra/config.ts` reads `git config user.name` at startup. If unset, falls back to `"Unknown Author"` and nothing will match.
- **CSV output (diverge branch)**: `run` dumps commit data to `~/.local/share/git-tasklog/tasklog.csv` (columns `path,commit,date`) instead of the Iotas DB. `export` filters by **date** (default today). `core/iotas.ts` and `core/week.ts` are deleted; `core/parser.ts` is dead code. `commands/run.ts` appends rows on each run — no cross-run dedup.
- **No concurrency guard**: The lock file (`infra/lock.ts`) exists but is never called. Runs are not protected against overlap.
- **Systemd service**: `make service-install` copies `tasklog.service` to `~/.config/systemd/user/` and enables it. The service runs `~/.local/bin/tasklog run`.

## Config Files

| Purpose | Path |
|---|---|
| Repo list | `~/.config/git-tasklog/repos.conf` (`path [branch ...]` per line, `#` comments) |
| CSV output | `~/.local/share/git-tasklog/tasklog.csv` |
| Logs | `~/.local/share/git-tasklog/run.log` |
| Lock | `/tmp/git-tasklog.lock` |
