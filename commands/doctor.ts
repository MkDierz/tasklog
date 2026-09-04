import { existsSync } from "fs";
import { CONFIG, CSV_PATH } from "../infra/config";

export function doctorCommand() {
  console.log("Checking system...\n");

  console.log("CSV:", existsSync(CSV_PATH) ? "OK" : "MISSING");
  console.log("Config:", existsSync(CONFIG.CONFIG_FILE) ? "OK" : "MISSING");
}