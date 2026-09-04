import { spawnSync } from "bun";
import { CONFIG } from "../infra/config";

export function getCommits(
  repo: string,
  branches: string[] = ["main"],
  since?: string,
) {
  const lines: string[] = [];

  for (const branch of branches) {
    const args = [
      "git",
      "-C",
      repo,
      "log",
      branch,
      `--author=${CONFIG.AUTHOR}`,
      "--no-merges",
      "--pretty=format:%ad|%s",
      "--date=short",
    ];

    if (since) args.push(`--since=${since}`);

    const res = spawnSync(args);

    if (res.exitCode !== 0) {
      throw new Error(`Git failed for ${repo} on branch ${branch}`);
    }

    lines.push(...res.stdout.toString().trim().split("\n").filter(Boolean));
  }

  return lines;
}