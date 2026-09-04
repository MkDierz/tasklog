import { spawnSync } from "bun";
import { dirname } from "path";

const HOME = process.env.HOME!;

function getGitAuthor(): string {
  try {
    const result = spawnSync(["git", "config", "user.name"]);
    if (result.exitCode === 0) {
      const author = result.stdout.toString().trim();
      return author || "Unknown Author";
    }
  } catch {
    // Fallback if git command fails
  }
  return "Unknown Author";
}

export const CSV_PATH = `${HOME}/.local/share/git-tasklog/tasklog.csv`;
export const CSV_DIR = dirname(CSV_PATH);

export const CONFIG = {
  AUTHOR: getGitAuthor(),
  CONFIG_FILE: `${HOME}/.config/git-tasklog/repos.conf`,
  LOCK_FILE: "/tmp/git-tasklog.lock",
};
