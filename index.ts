#!/usr/bin/env bun

import { Database } from "bun:sqlite";
import { existsSync, mkdirSync, readFileSync, appendFileSync } from "fs";
import { join } from "path";

const AUTHOR = "Syibbran Mulaesyi";

const HOME = process.env.HOME!;
const DB_PATH = `${HOME}/.var/app/org.gnome.World.Iotas/data/iotas/notes.db`;
const CONFIG_FILE = `${HOME}/.config/git-tasklog/repos.conf`;
const LOG_FILE = `${HOME}/.local/share/git-tasklog/run.log`;

/* ------------------ UTIL ------------------ */

function log(msg: string) {
  const line = `[${new Date().toISOString().replace("T", " ").slice(0, 19)}] ${msg}`;
  console.log(line);
  appendFileSync(LOG_FILE, line + "\n");
}

function ensureSetup() {
  mkdirSync(join(LOG_FILE, ".."), { recursive: true });

  if (!existsSync(CONFIG_FILE)) {
    log(`Error: config file not found at ${CONFIG_FILE}`);
    process.exit(1);
  }

  if (!existsSync(DB_PATH)) {
    log(`Error: Iotas DB not found at ${DB_PATH}`);
    process.exit(1);
  }
}

function isIotasRunning(): boolean {
  try {
    const proc = Bun.spawnSync(["pgrep", "-x", "iotas"]);
    return proc.exitCode === 0;
  } catch {
    return false;
  }
}

function getWeek(date: string) {
  const d = new Date(date);
  const yearStart = new Date(Date.UTC(d.getUTCFullYear(), 0, 1));
  const week = Math.ceil(
    ((d.getTime() - yearStart.getTime()) / 86400000 +
      yearStart.getUTCDay() +
      1) /
      7,
  );
  return `${d.getUTCFullYear()}-W${String(week).padStart(2, "0")}`;
}

function getMondayOfWeek() {
  const now = new Date();
  const day = now.getDay() || 7;
  now.setDate(now.getDate() - day + 1);
  return now.toISOString().slice(0, 10);
}

/* ------------------ DB ------------------ */

const db = new Database(DB_PATH);

const findNoteStmt = db.prepare(
  `SELECT id, content FROM note WHERE title = ? LIMIT 1`,
);

const insertStmt = db.prepare(`
  INSERT INTO note 
  (title, content, excerpt, last_modified, favourite, category, remote_id, dirty, locally_deleted, etag, read_only, failed_pushes)
  VALUES (?, ?, ?, ?, 0, ?, -1, 1, 0, '', 0, NULL)
`);

const updateStmt = db.prepare(`
  UPDATE note SET 
    content = ?, 
    excerpt = ?, 
    last_modified = ?, 
    category = ?, 
    dirty = 1,
    etag = ''
  WHERE id = ?
`);

/* ------------------ CORE ------------------ */

function upsertNote(repo: string, week: string, content: string) {
  const title = `Task Logs - ${repo} - ${week}`;
  const excerpt = content.replace(/\n/g, " ").slice(0, 200);
  const nowTs = Math.floor(Date.now() / 1000);

  const existing = findNoteStmt.get(title) as
    | { id: number; content: string }
    | undefined;

  if (!existing) {
    insertStmt.run(title, content, excerpt, nowTs, repo);
    return;
  }

  if (existing.content === content) return;

  updateStmt.run(content, excerpt, nowTs, repo, existing.id);
}

async function processRepo(repoPathRaw: string) {
  let repoPath = repoPathRaw.replace(/^~(?=$|\/)/, HOME).replace(/\/$/, "");
  const repoName = repoPath.split("/").pop()!;

  if (!existsSync(`${repoPath}/.git`)) {
    log(`Skip: ${repoPath} is not a git repo`);
    return;
  }

  log(`Processing: ${repoName}`);

  const weekStart = getMondayOfWeek();

  const lastTsRow = db
    .prepare(`SELECT MAX(last_modified) as ts FROM note WHERE title LIKE ?`)
    .get(`Task Logs - ${repoName} - %`) as { ts?: number };

  let sinceArg: string[] = [];

  if (!lastTsRow?.ts) {
    log(`  First run for ${repoName}`);
  } else {
    const lastDate = new Date(lastTsRow.ts * 1000).toISOString().slice(0, 10);

    const since = lastDate > weekStart ? weekStart : lastDate;

    sinceArg = [`--since=${since}`];
    log(`  Incremental run from ${since}`);
  }

  const git = Bun.spawnSync([
    "git",
    "-C",
    repoPath,
    "log",
    "main",
    `--author=${AUTHOR}`,
    "--no-merges",
    "--pretty=format:%ad|%s",
    "--date=short",
    ...sinceArg,
  ]);

  const output = git.stdout.toString();
  if (!output) return;

  const lines = output.split("\n");

  let currentWeek = "";
  let lastDate = "";
  let content = "";

  for (const line of lines) {
    const [dateStr, subject] = line.split("|");
    const week = getWeek(dateStr ?? "");

    if (week !== currentWeek) {
      if (currentWeek) {
        upsertNote(repoName, currentWeek, content);
      }
      currentWeek = week;
      lastDate = "";
      content = `## Task Logs - ${repoName} - ${week}\n\n`;
    }

    if (dateStr !== lastDate) {
      content += `### ${dateStr}\n`;
      lastDate = dateStr ?? "";
    }

    content += `- ${subject}\n`;
  }

  if (currentWeek) {
    upsertNote(repoName, currentWeek, content);
  }

  log(`  Done: ${repoName}`);
}

/* ------------------ MAIN ------------------ */

async function main() {
  log("─── Starting run ───");

  ensureSetup();

  if (isIotasRunning()) {
    log("Warning: Iotas is running — skipping");
    process.exit(0);
  }

  const repos = readFileSync(CONFIG_FILE, "utf-8")
    .split("\n")
    .map((l) => l.trim())
    .filter((l) => l && !l.startsWith("#"));

  for (const repo of repos) {
    await processRepo(repo);
  }

  log("─── Run complete ───");
}

main();
