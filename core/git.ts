import { CONFIG } from "../infra/config";

export function getCommits(repo: string, since?: string) {
  const args = [
    "git",
    "-C",
    repo,
    "log",
    "main",
    `--author=${CONFIG.AUTHOR}`,
    "--no-merges",
    "--pretty=format:%ad|%s",
    "--date=short",
  ];

  if (since) args.push(`--since=${since}`);

  const res = Bun.spawnSync(args);

  if (res.exitCode !== 0) {
    throw new Error(`Git failed for ${repo}`);
  }

  return res.stdout.toString().trim().split("\n").filter(Boolean);
}