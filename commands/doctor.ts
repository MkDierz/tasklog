import { existsSync } from "fs";
import { CONFIG } from "../infra/config";

export function doctorCommand() {
  console.log("Checking system...\n");

  console.log("DB:", existsSync(CONFIG.DB_PATH) ? "OK" : "MISSING");
  console.log("Config:", existsSync(CONFIG.CONFIG_FILE) ? "OK" : "MISSING");
}