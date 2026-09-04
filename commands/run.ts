import { readFileSync, existsSync, mkdirSync, appendFileSync } from "fs";
import { CONFIG, CSV_PATH, CSV_DIR } from "../infra/config";
import { log } from "../infra/logger";
import { getCommits } from "../core/git";

function csvEscape(value: string): string {
  if (/[",\n]/.test(value)) {
    return `"${value.replace(/"/g, '""')}"`;
  }
  return value;
}

export async function runCommand() {
  const entries = readFileSync(CONFIG.CONFIG_FILE, "utf-8")
    .split("\n")
    .map((l) => l.trim())
    .filter((l) => l && !l.startsWith("#"))
    .map((l) => {
      const parts = l.split(/\s+/);
      const repo = parts.shift()!;
      const branches = parts.length ? parts : ["main"];
      return { repo, branches };
    });

  if (!existsSync(CSV_DIR)) {
    mkdirSync(CSV_DIR, { recursive: true });
  }

  if (!existsSync(CSV_PATH)) {
    appendFileSync(CSV_PATH, "path,commit,date\n");
  }

  const seen = new Set<string>();

  for (const { repo, branches } of entries) {
    log("INFO", `Processing ${repo} [${branches.join(", ")}]`);

    const commits = getCommits(repo, branches);

    const rows = commits
      .map((line) => line.split("|"))
      .filter((parts) => parts.length === 2)
      .map(([date, subject]) => ({ date, subject: subject! }))
      .filter(({ date, subject }) => {
        const key = `${repo}|${date}|${subject}`;
        if (seen.has(key)) return false;
        seen.add(key);
        return true;
      })
      .map(({ date, subject }) =>
        `${csvEscape(repo)},${csvEscape(subject)},${date}`,
      );

    if (rows.length) {
      appendFileSync(CSV_PATH, rows.join("\n") + "\n");
      log("INFO", `  Appended ${rows.length} commits to ${CSV_PATH}`);
    } else {
      log("INFO", `  No commits found for ${repo}`);
    }
  }

  log("INFO", "Done");
}
