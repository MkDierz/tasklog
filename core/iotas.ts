import { Database } from "bun:sqlite";

export class IotasDB {
  constructor(private db: Database) {
    this.db.run(`
      PRAGMA journal_mode=WAL;
      PRAGMA synchronous=NORMAL;
    `);
  }

  upsert(repoName: string, week: string, content: string) {
    const title = `Task Logs - ${repoName} - ${week}`;
    const category = `tasklog/${repoName}`;
    const excerpt = content.replace(/\n/g, " ").slice(0, 100);
    const now = Math.floor(Date.now() / 1000);

    const existing = this.db
      .query("SELECT id, content FROM note WHERE title = ?")
      .get(title) as any;

    if (!existing) {
      this.db.run(
        `INSERT INTO note (title, content, excerpt, last_modified, favourite, category, remote_id, dirty, locally_deleted, etag, read_only, failed_pushes)
        VALUES (?, ?, ?, ?, 0, ?, -1, 1, 0, '', 0, NULL)`,
        [title, content, excerpt, now, category],
      );
      return;
    }

    if (existing.content === content) return;

    this.db.run(
      `UPDATE note SET content=?, excerpt=?, last_modified=?, category=?, dirty=1, etag=''
      WHERE id=?`,
      [content, excerpt, now, category, existing.id],
    );
  }
}
