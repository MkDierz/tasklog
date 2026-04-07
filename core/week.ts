export function getISOWeek(date: string) {
  const d = new Date(date);
  const yearStart = new Date(Date.UTC(d.getUTCFullYear(), 0, 1));
  const week = Math.ceil(
    ((d.getTime() - yearStart.getTime()) / 86400000 +
      yearStart.getUTCDay() +
      1) /
    7
  );

  return `${d.getUTCFullYear()}-W${String(week).padStart(2, "0")}`;
}

export function getMonday() {
  const now = new Date();
  const day = now.getDay() || 7;
  now.setDate(now.getDate() - day + 1);
  return now.toISOString().slice(0, 10);
}

export function getCurrentISOWeek() {
  const now = new Date();
  const yearStart = new Date(Date.UTC(now.getUTCFullYear(), 0, 1));
  const week = Math.ceil(
    ((now.getTime() - yearStart.getTime()) / 86400000 +
      yearStart.getUTCDay() +
      1) /
    7
  );

  return `${now.getUTCFullYear()}-W${String(week).padStart(2, "0")}`;
}