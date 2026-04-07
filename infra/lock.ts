export function acquireLock(path: string) {
  try {
    return Bun.file(path).writer();
  } catch {
    throw new Error("Already running");
  }
}