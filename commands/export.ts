import { readFileSync, existsSync } from "fs";
import { CSV_PATH } from "../infra/config";

function today(): string {
  return new Date().toISOString().slice(0, 10);
}

function parseCSVLine(line: string): { path: string; commit: string; date: string } | null {
  let i = 0;
  const fields: string[] = [];
  let current = "";

  while (i < line.length) {
    const char = line[i];

    if (char === '"') {
      i++;
      while (i < line.length && line[i] !== '"') {
        current += line[i];
        i++;
      }
      if (line[i] === '"') i++;
      if (line[i] === '"') {
        current += '"';
        i++;
      }
    } else if (char === ",") {
      fields.push(current);
      current = "";
      i++;
    } else {
      current += char;
      i++;
    }
  }

  fields.push(current);

  if (fields.length !== 3) return null;

  const [path, commit, date] = fields;
  if (!path || !commit || !date) return null;

  return { path, commit, date };
}

export function exportCommand(date?: string) {
  if (!existsSync(CSV_PATH)) {
    console.log(`No CSV file found at ${CSV_PATH}`);
    console.log("Run `tasklog run` first.");
    return;
  }

  const targetDate = date || today();
  const lines = readFileSync(CSV_PATH, "utf-8").trim().split("\n");

  const headers = lines[0];
  if (headers !== "path,commit,date") {
    console.log("Unrecognized CSV format");
    return;
  }

  const rows = lines
    .slice(1)
    .map(parseCSVLine)
    .filter((r): r is { path: string; commit: string; date: string } => r !== null)
    .filter((r) => r.date === targetDate);

  if (!rows.length) {
    console.log(`No logs found for ${targetDate}`);
    return;
  }

  console.log(`\nDay ${targetDate}\n`);

  for (const row of rows) {
    console.log(`${row.date} | ${row.path} | ${row.commit}`);
  }
}