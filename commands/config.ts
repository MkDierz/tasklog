import { CONFIG } from "../infra/config";
import readline from "readline";

/* ------------------ FILE ------------------ */

async function readRepos(): Promise<string[]> {
  const file = Bun.file(CONFIG.CONFIG_FILE);

  if (!(await file.exists())) return [];

  const text = await file.text();

  return text
    .split("\n")
    .map((l) => l.trim())
    .filter((l) => l && !l.startsWith("#"));
}

async function writeRepos(repos: string[]) {
  await Bun.write(CONFIG.CONFIG_FILE, repos.join("\n") + "\n");
}

function parseEntry(line: string): { repo: string; branches: string[] } {
  const parts = line.split(/\s+/);
  const repo = parts.shift()!;
  const branches = parts.length ? parts : ["main"];
  return { repo, branches };
}

function formatEntry(repo: string, branches: string[]) {
  return `${repo} ${branches.join(" ")}`.trim();
}

/* ------------------ DISPLAY ------------------ */

function displayRepos(repos: string[]) {
  if (repos.length === 0) {
    console.log("No repositories configured.\n");
    return;
  }

  console.log("\nConfigured repositories:\n");
  repos.forEach((entry, i) => {
    const { repo, branches } = parseEntry(entry);
    console.log(`  [${i}] ${repo} (branches: ${branches.join(", ")})`);
  });
  console.log();
}

/* ------------------ PROMPT (FIXED) ------------------ */

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
});

let pendingPrompt: ((line: string) => void) | null = null;

rl.on("line", (line) => {
  if (pendingPrompt) {
    const resolve = pendingPrompt;
    pendingPrompt = null;
    resolve(line);
  }
});

function promptUser(question: string): Promise<string> {
  process.stdout.write(question);

  return new Promise((resolve) => {
    pendingPrompt = (line: string) => resolve(line.trim());
  });
}

/* ------------------ COMMAND ------------------ */

export async function configCommand() {
  let repos = await readRepos();

  while (true) {
    displayRepos(repos);

    console.log("Commands:");
    console.log("  (a)dd     - Add a new repository");
    console.log("  (r)emove  - Remove a repository by number");
    console.log("  (e)dit    - Edit branches for a repository");
    console.log("  (s)ave    - Save and exit");
    console.log("  (q)uit    - Exit without saving\n");

    const cmd = await promptUser("Choose action [a/r/e/s/q]: ");

    switch (cmd.toLowerCase()) {
      case "a":
      case "add": {
        const path = await promptUser("Enter repository path: ");

        if (!path) break;

        if (repos.some((e) => parseEntry(e).repo === path)) {
          console.log("✗ Repository already in config.\n");
          break;
        }

        const branchesRaw = await promptUser(
          "Enter branches (space-separated, default 'main'): ",
        );
        const branches = branchesRaw ? branchesRaw.split(/\s+/).filter(Boolean) : ["main"];

        repos.push(formatEntry(path, branches));
        console.log(`✓ Added: ${formatEntry(path, branches)}\n`);
        break;
      }

      case "r":
      case "remove": {
        if (!repos.length) {
          console.log("✗ No repositories to remove.\n");
          break;
        }

        const index = Number(await promptUser("Enter index: "));

        if (Number.isNaN(index) || !repos[index]) {
          console.log("✗ Invalid number.\n");
          break;
        }

        console.log(`✓ Removed: ${repos[index]}\n`);
        repos.splice(index, 1);
        break;
      }

      case "e":
      case "edit": {
        if (!repos.length) {
          console.log("✗ No repositories to edit.\n");
          break;
        }

        const index = Number(await promptUser("Enter index: "));

        if (Number.isNaN(index) || !repos[index]) {
          console.log("✗ Invalid number.\n");
          break;
        }

        const { repo } = parseEntry(repos[index]);
        const branchesRaw = await promptUser(
          `Enter branches for ${repo} (space-separated, default 'main'): `,
        );
        const branches = branchesRaw ? branchesRaw.split(/\s+/).filter(Boolean) : ["main"];

        repos[index] = formatEntry(repo, branches);
        console.log(`✓ Updated: ${formatEntry(repo, branches)}\n`);
        break;
      }

      case "s":
      case "save":
        await writeRepos(repos);
        console.log("✓ Configuration saved.\n");
        rl.close();
        return;

      case "q":
      case "quit":
        console.log("Exiting without saving.\n");
        rl.close();
        return;

      default:
        console.log("✗ Unknown command.\n");
    }
  }
}
