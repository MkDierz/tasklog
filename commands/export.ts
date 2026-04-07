import { Database } from "bun:sqlite";
import { CONFIG } from "../infra/config";
import { getCurrentISOWeek } from "../core/week";

export function exportCommand(week?: string) {
  const db = new Database(CONFIG.DB_PATH);

  const targetWeek = week || getCurrentISOWeek();

  const rows = db
    .query("SELECT title, content FROM note WHERE title LIKE ?")
    .all(`%${targetWeek}%`) as any[];

  if (!rows.length) {
    console.log(`No logs found for ${targetWeek}`);
    return;
  }

  console.log(`\n📅 WEEK ${targetWeek}\n`);

  for (const row of rows) {
    console.log(row.content);
  }
}