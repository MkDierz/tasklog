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

/* ------------------ DISPLAY ------------------ */

function displayRepos(repos: string[]) {
  if (repos.length === 0) {
    console.log("No repositories configured.\n");
    return;
  }

  console.log("\nConfigured repositories:\n");
  repos.forEach((repo, i) => {
    console.log(`  [${i}] ${repo}`);
  });
  console.log();
}

/* ------------------ PROMPT (FIXED) ------------------ */

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
});

function promptUser(question: string): Promise<string> {
  return new Promise((resolve) => {
    rl.question(question, (answer) => {
      resolve(answer.trim());
    });
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
    console.log("  (s)ave    - Save and exit");
    console.log("  (q)uit    - Exit without saving\n");

    const cmd = await promptUser("Choose action [a/r/s/q]: ");

    switch (cmd.toLowerCase()) {
      case "a":
      case "add": {
        const path = await promptUser("Enter repository path: ");

        if (!path) break;

        if (repos.includes(path)) {
          console.log("✗ Repository already in config.\n");
          break;
        }

        repos.push(path);
        console.log(`✓ Added: ${path}\n`);
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

      case "s":
      case "save":
        await writeRepos(repos);
        console.log("✓ Configuration saved.\n");
        rl.close(); // ✅ important
        return;

      case "q":
      case "quit":
        console.log("Exiting without saving.\n");
        rl.close(); // ✅ important
        return;

      default:
        console.log("✗ Unknown command.\n");
    }
  }
}
