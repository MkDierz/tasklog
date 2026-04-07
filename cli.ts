#!/usr/bin/env bun

import { runCommand } from "./commands/run";
import { exportCommand } from "./commands/export";
import { doctorCommand } from "./commands/doctor";
import { configCommand } from "./commands/config";

const args = process.argv.slice(2);

function help() {
  console.log(`
tasklog - Git → Iotas task logger

Usage:
  tasklog run
  tasklog export [week]      Export logs (default: current week)
  tasklog config             Manage repository configuration (interactive)
  tasklog doctor

Examples:
  tasklog export
  tasklog export 2026-W14
  tasklog config

Options:
  -h, --help                 Show help
`);
}

async function main() {
  const cmd = args[0];

  if (!cmd || cmd === "-h" || cmd === "--help") {
    help();
    return;
  }

  switch (cmd) {
    case "run":
      await runCommand();
      break;

    case "export":
      exportCommand(args[1]);
      break;

    case "config":
      await configCommand();
      break;

    case "doctor":
      doctorCommand();
      break;

    case "help":
      help();
      break;

    default:
      console.error("Unknown command:", cmd);
      help();
  }
}

main();
