import { readFileSync } from "fs";
import { Database } from "bun:sqlite";
import { CONFIG } from "../infra/config";
import { log } from "../infra/logger";
import { getCommits } from "../core/git";
import { groupByWeek } from "../core/parser";
import { IotasDB } from "../core/iotas";

export async function runCommand() {
  const repos = readFileSync(CONFIG.CONFIG_FILE, "utf-8")
    .split("\n")
    .map((l) => l.trim())
    .filter((l) => l && !l.startsWith("#"));

  const db = new Database(CONFIG.DB_PATH);
  const iotas = new IotasDB(db);

  const tx = db.transaction((entries) => {
    for (const e of entries) {
      iotas.upsert(e.repoName, e.week, e.content);
    }
  });

  for (const repo of repos) {
    log("INFO", `Processing ${repo}`);

    const commits = getCommits(repo);
    const grouped = groupByWeek(repo.split("/").pop()!, commits);
    const repoName = repo.split("/").pop()!;

    const entries = [...grouped.entries()].map(([week, content]) => ({
      repoPath: repo,
      repoName,
      repo,
      week,
      content,
    }));

    tx(entries);
  }

  log("INFO", "Done");
}
