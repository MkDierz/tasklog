const { spawnSync } = Bun;

function runSystemctl(args: string[]): boolean {
  const result = spawnSync(["systemctl", "--user", ...args]);
  if (!result.success) {
    process.stderr.write(
      result.stderr?.toString() || "Failed to run systemctl\n",
    );
    return false;
  }
  process.stdout.write(result.stdout?.toString() || "");
  return true;
}

export function serviceCommand(subcmd?: string) {
  switch (subcmd) {
    case "enable":
      runSystemctl(["enable", "--now", "tasklog.service"]);
      break;
    case "disable":
      runSystemctl(["disable", "--now", "tasklog.service"]);
      break;
    case "start":
      runSystemctl(["start", "tasklog.service"]);
      break;
    case "stop":
      runSystemctl(["stop", "tasklog.service"]);
      break;
    case "status":
      runSystemctl(["status", "tasklog.service"]);
      break;
    default:
      console.log(`Usage: tasklog service [enable|disable|start|stop|status]`);
  }
}
