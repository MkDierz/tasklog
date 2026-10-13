use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, Weekday};
use rusqlite::{params, Connection, OpenFlags};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

const CONFIG_FILE: &str = "repos.conf";
const DB_FILE: &str = "tasklog.db";
const DEFAULT_PROJECT_NAME: &str = "[ET] GLI - Alfagift";
/// Overlap added to last_run to catch commits whose committer date is older
/// than the last run (pulls, rebases). Safe because hashes are deduplicated.
const OVERLAP_DAYS: i64 = 7;

/// Column layout consumed by google-form-filler. Do not reorder or rename.
const REPORT_HEADER: [&str; 9] = [
    "Task Completion Date",
    "Project Name",
    "Hours",
    "Task List",
    "Complexity",
    "Urgency",
    "Overtime",
    "Obstacles",
    "Next Target",
];

#[derive(Debug)]
struct RepoEntry {
    repo: String,
    branches: Vec<String>,
}

#[derive(Debug, Clone)]
struct CommitRow {
    hash: String,
    repo: String,
    message: String,
    date: String,
    is_merge: bool,
}

struct ScanResult {
    rows: Vec<CommitRow>,
    failed_branches: usize,
}

#[derive(Default)]
struct RunOptions {
    full: bool,
    since: Option<String>,
}

struct ReportOptions {
    start_date: Option<String>,
    end_date: Option<String>,
    project_name: String,
    output: PathBuf,
    include_merges: bool,
}

// ---------------------------------------------------------------------------
// Paths and database
// ---------------------------------------------------------------------------

fn home_dir() -> PathBuf {
    dirs::home_dir().expect("HOME directory not found")
}

fn config_dir() -> PathBuf {
    let dir = home_dir().join(".config").join("git-tasklog");
    fs::create_dir_all(&dir).expect("failed to create config directory");
    dir
}

fn data_dir() -> PathBuf {
    let dir = home_dir().join(".local").join("share").join("git-tasklog");
    fs::create_dir_all(&dir).expect("failed to create data directory");
    dir
}

fn config_path() -> PathBuf {
    config_dir().join(CONFIG_FILE)
}

fn db_path() -> PathBuf {
    data_dir().join(DB_FILE)
}

fn open_db() -> Connection {
    let conn = Connection::open(db_path()).expect("failed to open database");
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS commits (
            hash     TEXT PRIMARY KEY,
            repo     TEXT NOT NULL,
            message  TEXT NOT NULL,
            date     TEXT NOT NULL,
            is_merge INTEGER NOT NULL DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_commits_date ON commits(date);
        CREATE TABLE IF NOT EXISTS meta (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );",
    )
    .expect("failed to initialize schema");
    conn
}

fn get_meta(conn: &Connection, key: &str) -> Option<String> {
    conn.query_row("SELECT value FROM meta WHERE key = ?1", params![key], |r| r.get(0))
        .ok()
}

fn set_meta(conn: &Connection, key: &str, value: &str) {
    conn.execute(
        "INSERT INTO meta (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )
    .expect("failed to write meta");
}

fn last_run_key(repo: &str) -> String {
    format!("last_run:{repo}")
}

/// Expands a leading `~` so paths like `~/code/app` work.
fn expand_path(path: &str) -> String {
    if path == "~" {
        return home_dir().to_string_lossy().to_string();
    }
    if let Some(rest) = path.strip_prefix("~/") {
        return home_dir().join(rest).to_string_lossy().to_string();
    }
    path.to_string()
}

/// Reads user.name as git resolves it for this repo (local config, then global).
fn git_author(repo: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["-C", repo, "config", "user.name"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

// ---------------------------------------------------------------------------
// Repo config
// ---------------------------------------------------------------------------

fn read_repo_entries() -> Vec<RepoEntry> {
    let path = config_path();
    if !path.exists() {
        return Vec::new();
    }

    let content = fs::read_to_string(path).unwrap_or_default();
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let mut parts = line.split_whitespace();
            let repo = parts.next().unwrap_or_default().to_string();
            let branches = parts.map(str::to_string).collect::<Vec<_>>();
            RepoEntry {
                repo,
                branches: if branches.is_empty() {
                    vec!["main".to_string()]
                } else {
                    branches
                },
            }
        })
        .filter(|entry| !entry.repo.is_empty())
        .collect()
}

fn write_repo_entries(entries: &[RepoEntry]) {
    let path = config_path();
    let content = entries
        .iter()
        .map(|entry| {
            let branches = entry.branches.join(" ");
            if branches.is_empty() {
                entry.repo.clone()
            } else {
                format!("{} {}", entry.repo, branches)
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(path, format!("{content}\n")).expect("failed to write config file");
}

// ---------------------------------------------------------------------------
// run: git -> SQLite
// ---------------------------------------------------------------------------

/// Line format: HASH \t PARENTS \t DATE \t SUBJECT
fn parse_commit_line(line: &str, repo: &str) -> Option<CommitRow> {
    let mut split = line.splitn(4, '\t');
    let hash = split.next()?.trim().to_string();
    let parents = split.next()?.trim().to_string();
    let date = split.next()?.trim().to_string();
    let message = split.next()?.trim().to_string();
    if hash.is_empty() || date.is_empty() || message.is_empty() {
        return None;
    }
    Some(CommitRow {
        hash,
        repo: repo.to_string(),
        message,
        date,
        is_merge: parents.split_whitespace().count() > 1,
    })
}

fn collect_commits(
    repo: &str,
    branches: &[String],
    author: &str,
    since: Option<&str>,
) -> ScanResult {
    let mut rows = Vec::new();
    let mut failed_branches = 0;

    for branch in branches {
        let mut args: Vec<String> = vec![
            "-C".into(),
            repo.into(),
            "log".into(),
            branch.clone(),
            format!("--author={author}"),
            "--pretty=format:%H%x09%P%x09%ad%x09%s".into(),
            "--date=short".into(),
        ];
        if let Some(since) = since {
            args.push(format!("--since={since}"));
        }

        let output = match Command::new("git").args(&args).output() {
            Ok(value) => value,
            Err(err) => {
                eprintln!("  warning: failed to run git: {err}");
                failed_branches += 1;
                continue;
            }
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!(
                "  warning: branch '{branch}' failed: {}",
                stderr.trim().lines().next().unwrap_or("unknown error")
            );
            failed_branches += 1;
            continue;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let before = rows.len();
        for line in stdout.lines() {
            if let Some(row) = parse_commit_line(line, repo) {
                rows.push(row);
            }
        }
        println!("  branch '{branch}': {} commit(s) found", rows.len() - before);
    }

    ScanResult { rows, failed_branches }
}

fn parse_run_options(args: &[String]) -> RunOptions {
    let mut opts = RunOptions::default();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--full" => opts.full = true,
            "--since" => match iter.next() {
                Some(value) => opts.since = Some(value.clone()),
                None => {
                    eprintln!("--since requires a value, e.g. --since 2026-09-01");
                    std::process::exit(1);
                }
            },
            other => {
                eprintln!("Unknown option for run: {other}");
                std::process::exit(1);
            }
        }
    }
    opts
}

/// Decides where to start scanning for a repo. None means all history.
fn resolve_since(conn: &Connection, repo: &str, opts: &RunOptions) -> (Option<String>, String) {
    if opts.full {
        return (None, "all history (--full)".to_string());
    }
    if let Some(since) = &opts.since {
        return (Some(since.clone()), format!("since {since} (--since)"));
    }
    match get_meta(conn, &last_run_key(repo)) {
        Some(stored) => match DateTime::parse_from_rfc3339(&stored) {
            Ok(last) => {
                let start = (last - Duration::days(OVERLAP_DAYS)).to_rfc3339();
                (Some(start.clone()), format!("since {start} (last run {stored})"))
            }
            Err(_) => (None, "all history (unreadable last run)".to_string()),
        },
        None => (None, "all history (first run for this repo)".to_string()),
    }
}

fn run_command(args: &[String]) {
    let opts = parse_run_options(args);
    let entries = read_repo_entries();
    if entries.is_empty() {
        println!("No repositories configured. Run `tasklog config` to add one.");
        return;
    }

    let started = Local::now().to_rfc3339();
    let mut conn = open_db();
    let tx = conn.transaction().expect("failed to start transaction");
    let mut total_inserted = 0usize;

    {
        let mut stmt = tx
            .prepare(
                "INSERT OR IGNORE INTO commits (hash, repo, message, date, is_merge)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )
            .expect("failed to prepare insert");

        for entry in &entries {
            let repo = expand_path(&entry.repo);
            println!("\n{repo}");

            if !PathBuf::from(&repo).exists() {
                eprintln!("  warning: path does not exist, skipping");
                continue;
            }

            let author = match git_author(&repo) {
                Some(value) => value,
                None => {
                    eprintln!("  warning: no git user.name found for this repo, skipping");
                    continue;
                }
            };

            let (since, since_label) = resolve_since(&tx, &repo, &opts);
            println!("  author: {author}");
            println!("  scanning: {since_label}");

            let scan = collect_commits(&repo, &entry.branches, &author, since.as_deref());

            let mut inserted = 0usize;
            for row in &scan.rows {
                inserted += stmt
                    .execute(params![
                        row.hash,
                        row.repo,
                        row.message,
                        row.date,
                        row.is_merge as i64
                    ])
                    .expect("failed to insert commit");
            }
            total_inserted += inserted;
            println!("  {} scanned, {inserted} new", scan.rows.len());

            if scan.failed_branches == 0 {
                set_meta(&tx, &last_run_key(&repo), &started);
            } else {
                eprintln!("  last run not updated because some branches failed");
            }
        }
    }

    tx.commit().expect("failed to commit transaction");
    println!("\nDone. {total_inserted} new commit(s) logged.");
}

// ---------------------------------------------------------------------------
// export: print one day
// ---------------------------------------------------------------------------

fn export_command(date_arg: Option<&str>) {
    if !db_path().exists() {
        println!("No database found at {}", db_path().display());
        println!("Run `tasklog run` first.");
        return;
    }

    let target = match date_arg {
        Some(value) => value.to_string(),
        None => Local::now().format("%Y-%m-%d").to_string(),
    };

    let conn = open_db();
    let mut stmt = conn
        .prepare(
            "SELECT date, repo, hash, message, is_merge
             FROM commits WHERE date = ?1 ORDER BY rowid",
        )
        .expect("failed to prepare query");

    let rows: Vec<(String, String, String, String, bool)> = stmt
        .query_map(params![target], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get::<_, i64>(4)? != 0,
            ))
        })
        .expect("failed to query commits")
        .filter_map(Result::ok)
        .collect();

    if rows.is_empty() {
        println!("No logs found for {target}");
        return;
    }

    println!("\nDay {target}\n");
    for (date, repo, hash, message, is_merge) in rows {
        let short = &hash[..hash.len().min(7)];
        let tag = if is_merge { " [merge]" } else { "" };
        println!("{date} | {repo} | {short} | {message}{tag}");
    }
}

// ---------------------------------------------------------------------------
// report: SQLite -> task report CSV (for google-form-filler)
// ---------------------------------------------------------------------------

fn next_value(iter: &mut std::slice::Iter<String>, name: &str) -> Result<String, String> {
    iter.next()
        .cloned()
        .ok_or_else(|| format!("{name} requires a value"))
}

fn parse_report_options(args: &[String]) -> Result<ReportOptions, String> {
    let mut opts = ReportOptions {
        start_date: None,
        end_date: None,
        project_name: DEFAULT_PROJECT_NAME.to_string(),
        output: PathBuf::from("output.csv"),
        include_merges: false,
    };

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--start-date" => opts.start_date = Some(next_value(&mut iter, "--start-date")?),
            "--end-date" => opts.end_date = Some(next_value(&mut iter, "--end-date")?),
            "--project-name" => opts.project_name = next_value(&mut iter, "--project-name")?,
            "--output" => opts.output = PathBuf::from(next_value(&mut iter, "--output")?),
            "--include-merges" => opts.include_merges = true,
            other => return Err(format!("unknown option for report: {other}")),
        }
    }
    Ok(opts)
}

fn parse_date(label: &str, value: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| {
        format!("{label} must be a valid date in YYYY-MM-DD format (got '{value}').")
    })
}

/// Last path component of the repo path, e.g. /home/me/code/alfagift -> alfagift
fn repo_name(repo: &str) -> &str {
    Path::new(repo.trim_end_matches('/'))
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(repo)
}

fn csv_field(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

/// Per day: the sorted set of unique "<repo-name> <message>" lines.
fn load_report_logs(
    db: &Path,
    start: NaiveDate,
    end: NaiveDate,
    include_merges: bool,
) -> Result<BTreeMap<NaiveDate, BTreeSet<String>>, String> {
    let conn = Connection::open_with_flags(db, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| format!("cannot open {}: {e}", db.display()))?;

    let sql = if include_merges {
        "SELECT date, repo, message FROM commits WHERE date >= ?1 AND date <= ?2"
    } else {
        "SELECT date, repo, message FROM commits
         WHERE date >= ?1 AND date <= ?2 AND is_merge = 0"
    };

    let mut stmt = conn.prepare(sql).map_err(|e| {
        format!("cannot read commits table: {e} (has `tasklog run` been executed?)")
    })?;

    let rows = stmt
        .query_map(
            params![
                start.format("%Y-%m-%d").to_string(),
                end.format("%Y-%m-%d").to_string()
            ],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            },
        )
        .map_err(|e| e.to_string())?;

    let mut logs: BTreeMap<NaiveDate, BTreeSet<String>> = BTreeMap::new();
    for row in rows {
        let (date, repo, message) = row.map_err(|e| e.to_string())?;
        let Ok(date) = NaiveDate::parse_from_str(&date, "%Y-%m-%d") else {
            continue;
        };
        logs.entry(date)
            .or_default()
            .insert(format!("{} {}", repo_name(&repo), message));
    }

    Ok(logs)
}

/// Builds the report CSV. The calendar comes from the requested range, not from
/// the logged dates, so days without entries become "Standby" rows. Weekends are
/// skipped and rows are ordered newest first.
/// Returns (csv, workdays, standby_days).
fn build_report(
    logs: &BTreeMap<NaiveDate, BTreeSet<String>>,
    start: NaiveDate,
    end: NaiveDate,
    project_name: &str,
) -> (String, usize, usize) {
    let mut days = Vec::new();
    let mut day = start;
    while day <= end {
        if !matches!(day.weekday(), Weekday::Sat | Weekday::Sun) {
            days.push(day);
        }
        day = day + Duration::days(1);
    }
    days.reverse();

    let mut out = REPORT_HEADER
        .iter()
        .map(|h| csv_field(h))
        .collect::<Vec<_>>()
        .join(",");
    out.push('\n');

    let mut standby = 0usize;
    for day in &days {
        let (tasks, score) = match logs.get(day) {
            Some(set) if !set.is_empty() => {
                (set.iter().cloned().collect::<Vec<_>>().join("\n"), 3)
            }
            _ => {
                standby += 1;
                ("Standby".to_string(), 0)
            }
        };

        let fields = [
            day.to_string(),
            project_name.to_string(),
            "8".to_string(),
            tasks,
            score.to_string(), // Complexity
            score.to_string(), // Urgency
            String::new(),     // Overtime
            String::new(),     // Obstacles
            String::new(),     // Next Target
        ];

        out.push_str(
            &fields
                .iter()
                .map(|f| csv_field(f))
                .collect::<Vec<_>>()
                .join(","),
        );
        out.push('\n');
    }

    (out, days.len(), standby)
}

fn report_command(args: &[String]) -> Result<(), String> {
    let opts = parse_report_options(args)?;

    let start_raw = opts
        .start_date
        .ok_or("missing required option: --start-date YYYY-MM-DD")?;
    let start = parse_date("--start-date", &start_raw)?;
    let end = match &opts.end_date {
        Some(value) => parse_date("--end-date", value)?,
        None => Local::now().date_naive(),
    };
    if start > end {
        return Err(format!("--start-date ({start}) is after --end-date ({end})."));
    }

    let db = db_path();
    if !db.exists() {
        return Err(format!(
            "database not found at {}. Run `tasklog run` first.",
            db.display()
        ));
    }

    let logs = load_report_logs(&db, start, end, opts.include_merges)?;
    let (report, workdays, standby) = build_report(&logs, start, end, &opts.project_name);

    fs::write(&opts.output, report)
        .map_err(|e| format!("cannot write {}: {e}", opts.output.display()))?;

    println!(
        "\u{2713} Report written to {} ({workdays} workdays, {} with commits, {standby} standby)",
        opts.output.display(),
        workdays - standby
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// config, doctor, service
// ---------------------------------------------------------------------------

fn config_command() {
    let mut repos = read_repo_entries();
    loop {
        println!("\nConfigured repositories:\n");
        if repos.is_empty() {
            println!("No repositories configured.\n");
        } else {
            for (idx, repo) in repos.iter().enumerate() {
                println!("  [{idx}] {} (branches: {})", repo.repo, repo.branches.join(", "));
            }
            println!();
        }

        println!("Commands:");
        println!("  (a)dd     - Add a new repository");
        println!("  (r)emove  - Remove a repository by number");
        println!("  (e)dit    - Edit branches for a repository");
        println!("  (s)ave    - Save and exit");
        println!("  (q)uit    - Exit without saving\n");

        let input = prompt("Choose action [a/r/e/s/q]: ");
        match input.trim().to_ascii_lowercase().as_str() {
            "a" | "add" => {
                let path = prompt("Enter repository path: ");
                if path.is_empty() {
                    continue;
                }
                if repos.iter().any(|repo| repo.repo == path) {
                    println!("Repository already in config.\n");
                    continue;
                }
                let branches = prompt("Enter branches (space-separated, default 'main'): ");
                let branches = if branches.trim().is_empty() {
                    vec!["main".to_string()]
                } else {
                    branches.split_whitespace().map(str::to_string).collect()
                };
                repos.push(RepoEntry { repo: path, branches });
                println!("Added: {}\n", format_entry(&repos[repos.len() - 1]));
            }
            "r" | "remove" => {
                if repos.is_empty() {
                    println!("No repositories to remove.\n");
                    continue;
                }
                let index = match prompt("Enter index: ").parse::<usize>() {
                    Ok(value) => value,
                    Err(_) => {
                        println!("Invalid number.\n");
                        continue;
                    }
                };
                if repos.get(index).is_none() {
                    println!("Invalid number.\n");
                    continue;
                }
                println!("Removed: {}\n", format_entry(&repos[index]));
                repos.remove(index);
            }
            "e" | "edit" => {
                if repos.is_empty() {
                    println!("No repositories to edit.\n");
                    continue;
                }
                let index = match prompt("Enter index: ").parse::<usize>() {
                    Ok(value) => value,
                    Err(_) => {
                        println!("Invalid number.\n");
                        continue;
                    }
                };
                if repos.get(index).is_none() {
                    println!("Invalid number.\n");
                    continue;
                }
                let repo = repos[index].repo.clone();
                let branches = prompt(&format!(
                    "Enter branches for {repo} (space-separated, default 'main'): "
                ));
                let branches = if branches.trim().is_empty() {
                    vec!["main".to_string()]
                } else {
                    branches.split_whitespace().map(str::to_string).collect()
                };
                repos[index].branches = branches;
                println!("Updated: {}\n", format_entry(&repos[index]));
            }
            "s" | "save" => {
                write_repo_entries(&repos);
                println!("Configuration saved.\n");
                return;
            }
            "q" | "quit" => {
                println!("Exiting without saving.\n");
                return;
            }
            _ => println!("Unknown command.\n"),
        }
    }
}

fn format_entry(entry: &RepoEntry) -> String {
    format!("{} {}", entry.repo, entry.branches.join(" "))
        .trim()
        .to_string()
}

fn prompt(question: &str) -> String {
    print!("{question}");
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn doctor_command() {
    println!("Checking system...\n");
    println!("Database: {}", if db_path().exists() { "OK" } else { "MISSING" });
    println!("Config: {}", if config_path().exists() { "OK" } else { "MISSING" });

    let entries = read_repo_entries();
    println!("Repositories configured: {}", entries.len());

    if db_path().exists() {
        let conn = open_db();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM commits", [], |r| r.get(0))
            .unwrap_or(0);
        println!("Commits logged: {count}\n");

        for entry in &entries {
            let repo = expand_path(&entry.repo);
            let exists = if PathBuf::from(&repo).exists() { "ok" } else { "PATH MISSING" };
            let author = git_author(&repo).unwrap_or_else(|| "NOT SET".to_string());
            let last = get_meta(&conn, &last_run_key(&repo))
                .unwrap_or_else(|| "never".to_string());
            println!("  {repo}");
            println!("    path: {exists} | author: {author} | last run: {last}");
        }
    }
}

fn service_command(subcommand: Option<&str>) {
    let Some(subcommand) = subcommand else {
        println!("Usage: tasklog service [enable|disable|start|stop|status]");
        return;
    };

    let args = match subcommand {
        "enable" => vec!["enable", "--now", "tasklog.service"],
        "disable" => vec!["disable", "--now", "tasklog.service"],
        "start" => vec!["start", "tasklog.service"],
        "stop" => vec!["stop", "tasklog.service"],
        "status" => vec!["status", "tasklog.service"],
        _ => {
            println!("Usage: tasklog service [enable|disable|start|stop|status]");
            return;
        }
    };

    let output = Command::new("systemctl").args(["--user"]).args(&args).output();

    match output {
        Ok(result) => {
            if !result.stdout.is_empty() {
                print!("{}", String::from_utf8_lossy(&result.stdout));
            }
            if !result.stderr.is_empty() {
                eprint!("{}", String::from_utf8_lossy(&result.stderr));
            }
            if !result.status.success() {
                std::process::exit(result.status.code().unwrap_or(1));
            }
        }
        Err(err) => {
            eprintln!("Failed to run systemctl: {err}");
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!(
        "
        tasklog - Git → SQLite task logger and report generator

        Usage:
          tasklog run [--full] [--since DATE]
          tasklog report --start-date DATE [options]
          tasklog export [date]      Print logs for one day (default: today)
          tasklog config             Manage repository configuration
          tasklog doctor
          tasklog service [enable|disable|start|stop|status]

        Run behavior:
          By default, scans each repo from its last run (with a {OVERLAP_DAYS}-day
          overlap). A repo with no last run is scanned from the beginning.
          --full                     Rescan all history for every repo
          --since DATE               Scan from DATE (e.g. 2026-09-01)

        Report options:
          --start-date DATE          Start date, YYYY-MM-DD (required, inclusive)
          --end-date DATE            End date, YYYY-MM-DD (default: today, inclusive)
          --project-name NAME        Project name column (default: \"{DEFAULT_PROJECT_NAME}\")
          --output PATH              Output CSV path (default: output.csv)
          --include-merges           Include merge commits (excluded by default)

        Examples:
          tasklog run
          tasklog run --full
          tasklog report --start-date 2026-09-01
          tasklog report --start-date 2026-09-01 --end-date 2026-09-30 --output sept.csv
          tasklog export 2026-09-03

        Options:
          -h, --help                 Show help
        "
    );
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let command = args.first().map(String::as_str);

    match command {
        None | Some("-h") | Some("--help") | Some("help") => print_help(),
        Some("run") => run_command(&args[1..]),
        Some("report") => {
            if let Err(err) = report_command(&args[1..]) {
                eprintln!("\nFailed to generate report: {err}");
                std::process::exit(1);
            }
        }
        Some("export") => export_command(args.get(1).map(String::as_str)),
        Some("config") => config_command(),
        Some("doctor") => doctor_command(),
        Some("service") => service_command(args.get(1).map(String::as_str)),
        _ => {
            eprintln!("Unknown command: {}", command.unwrap_or(""));
            print_help();
            std::process::exit(1);
        }
    }
}