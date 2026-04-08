#!/usr/bin/env bun

import { runCommand } from "./commands/run";
import { exportCommand } from "./commands/export";
import { doctorCommand } from "./commands/doctor";
import { configCommand } from "./commands/config";
import { serviceCommand } from "./commands/service";

const args = process.argv.slice(2);

function help() {
  console.log(`
tasklog - Git → Iotas task logger

Usage:
  tasklog run
  tasklog export [week]      Export logs (default: current week)
  tasklog config             Manage repository configuration (interactive)
  tasklog doctor
  tasklog service [enable|disable|start|stop|status]  Manage background service (run on boot)

Examples:
  tasklog export
  tasklog export 2026-W14
  tasklog config
  tasklog service enable

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

    case "service":
      serviceCommand(args[1]);
      break;

    case "help":
      help();
      break;

    default:
      console.error("Unknown command:", cmd);
      help();
  }
}

main()
  .then(() => process.exit(0))
  .catch((err) => {
    console.error(err);
    process.exit(1);
  });
