import { getISOWeek } from "./week";

export function groupByWeek(repo: string, lines: string[]) {
  const map = new Map<string, string>();

  let currentWeek = "";
  let content = "";
  let lastDate = "";

  for (const line of lines) {
    const [date, subject] = line.split("|");
    const week = getISOWeek(date ?? '');

    if (week !== currentWeek) {
      if (currentWeek) map.set(currentWeek, content);

      currentWeek = week;
      content = `## Task Logs - ${repo} - ${week}\n\n`;
      lastDate = "";
    }

    if (date !== lastDate) {
      content += `### ${date}\n`;
      lastDate = date ?? '';
    }

    content += `- ${subject}\n`;
  }

  if (currentWeek) map.set(currentWeek, content);

  return map;
}