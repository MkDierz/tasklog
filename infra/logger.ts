export function log(level: "INFO" | "WARN" | "ERROR", msg: string) {
  const ts = new Date().toISOString();
  console.log(`[${ts}] [${level}] ${msg}`);
}