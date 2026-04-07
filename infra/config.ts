const HOME = process.env.HOME!;

function getGitAuthor(): string {
  try {
    const result = Bun.spawnSync(["git", "config", "user.name"]);
    if (result.exitCode === 0) {
      const author = result.stdout.toString().trim();
      return author || "Unknown Author";
    }
  } catch {
    // Fallback if git command fails
  }
  return "Unknown Author";
}

export const CONFIG = {
  AUTHOR: getGitAuthor(),
  DB_PATH: `${HOME}/.var/app/org.gnome.World.Iotas/data/iotas/notes.db`,
  CONFIG_FILE: `${HOME}/.config/git-tasklog/repos.conf`,
  LOCK_FILE: "/tmp/git-tasklog.lock",
};
