use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

const CONFIG_FILE: &str = "repos.conf";

#[derive(Debug)]
struct RepoEntry {
    repo: String,
    branches: Vec<String>,
}

#[derive(Debug, Clone)]
struct CommitRow {
    path: String,
    commit: String,
    date: String,
}

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

fn csv_path() -> PathBuf {
    data_dir().join("tasklog.csv")
}

fn git_author() -> String {
    let output = Command::new("git")
        .args(["config", "user.name"])
        .output();

    match output {
        Ok(result) if result.status.success() => {
            let value = String::from_utf8_lossy(&result.stdout).trim().to_owned();
            if !value.is_empty() { value } else { "Unknown Author".to_string() }
        }
        _ => "Unknown Author".to_string(),
    }
}

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

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn parse_commit_line(line: &str) -> Option<(String, String)> {
    let mut split = line.splitn(2, '|');
    let date = split.next()?.trim().to_string();
    let subject = split.next()?.trim().to_string();
    if date.is_empty() || subject.is_empty() {
        return None;
    }
    Some((date, subject))
}

fn collect_commits(repo: &str, branches: &[String]) -> Vec<CommitRow> {
    let mut rows = Vec::new();
    let author = git_author();

    for branch in branches {
        let output = Command::new("git")
            .args([
                "-C",
                repo,
                "log",
                branch,
                &format!("--author={author}"),
                "--no-merges",
                "--pretty=format:%ad|%s",
                "--date=short",
            ])
            .output();

        let output = match output {
            Ok(value) => value,
            Err(err) => panic!("failed to run git for {repo}: {err}"),
        };

        if !output.status.success() {
            panic!("git failed for {repo} on branch {branch}");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if let Some((date, subject)) = parse_commit_line(line) {
                rows.push(CommitRow {
                    path: repo.to_string(),
                    commit: subject,
                    date,
                });
            }
        }
    }

    rows
}

fn ensure_csv() {
    let path = csv_path();
    let parent = path.parent().unwrap();
    fs::create_dir_all(parent).expect("failed to create data directory");
    if !path.exists() {
        fs::write(path, "path,commit,date\n").expect("failed to create csv");
    }
}

fn run_command() {
    let entries = read_repo_entries();
    ensure_csv();
    let csv = csv_path();
    let mut seen = std::collections::HashSet::new();

    for entry in entries {
        let rows = collect_commits(&entry.repo, &entry.branches);
        let mut new_rows = Vec::new();

        for row in rows {
            let key = format!("{}|{}|{}", row.path, row.date, row.commit);
            if seen.insert(key) {
                new_rows.push(row);
            }
        }

        if !new_rows.is_empty() {
            let mut file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&csv)
                .expect("failed to open csv file");

            for row in new_rows {
                use std::io::Write;
                writeln!(
                    file,
                    "{},{},{}",
                    csv_escape(&row.path),
                    csv_escape(&row.commit),
                    row.date
                )
                .expect("failed to append csv row");
            }
        }
    }
}

fn export_command(date_arg: Option<&str>) {
    let csv = csv_path();
    if !csv.exists() {
        println!("No CSV file found at {}", csv.display());
        println!("Run `tasklog run` first.");
        return;
    }

    let target = match date_arg {
        Some(value) => value.to_string(),
        None => chrono::Local::now().format("%Y-%m-%d").to_string(),
    };

    let content = fs::read_to_string(csv).unwrap_or_default();
    let mut lines = content.lines();
    let header = lines.next();
    if header != Some("path,commit,date") {
        println!("Unrecognized CSV format");
        return;
    }

    let rows: Vec<_> = lines
        .filter_map(|line| {
            let mut fields = Vec::new();
            let mut current = String::new();
            let mut chars = line.chars().peekable();
            let mut in_quotes = false;

            while let Some(ch) = chars.next() {
                if ch == '"' {
                    if in_quotes && chars.peek() == Some(&'"') {
                        current.push('"');
                        chars.next();
                    } else {
                        in_quotes = !in_quotes;
                    }
                } else if ch == ',' && !in_quotes {
                    fields.push(current.clone());
                    current.clear();
                } else {
                    current.push(ch);
                }
            }
            fields.push(current);

            if fields.len() != 3 {
                return None;
            }

            let path = fields[0].clone();
            let commit = fields[1].clone();
            let date = fields[2].clone();
            if path.is_empty() || commit.is_empty() || date.is_empty() {
                return None;
            }
            Some((path, commit, date))
        })
        .filter(|(_, _, date)| date == &target)
        .collect();

    if rows.is_empty() {
        println!("No logs found for {target}");
        return;
    }

    println!("\nDay {target}\n");
    for (path, commit, date) in rows {
        println!("{date} | {path} | {commit}");
    }
}

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
                let index = prompt("Enter index: ");
                let index = match index.parse::<usize>() {
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
                let index = prompt("Enter index: ");
                let index = match index.parse::<usize>() {
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
                let branches = prompt(&format!("Enter branches for {repo} (space-separated, default 'main'): "));
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
    format!("{} {}", entry.repo, entry.branches.join(" ")).trim().to_string()
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
    println!("CSV: {}", if csv_path().exists() { "OK" } else { "MISSING" });
    println!("Config: {}", if config_path().exists() { "OK" } else { "MISSING" });
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

    let output = Command::new("systemctl")
        .args(["--user"])
        .args(&args)
        .output();

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
        tasklog - Git → CSV task logger

        Usage:
          tasklog run
          tasklog export [date]      Export logs (default: today)
          tasklog config             Manage repository configuration
          tasklog doctor
          tasklog service [enable|disable|start|stop|status]

        Examples:
          tasklog export
          tasklog export 2026-09-03
          tasklog config
          tasklog service enable

        Options:
          -h, --help                 Show help
        "
    );
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let command = args.first().map(String::as_str);

    match command {
        None | Some("-h") | Some("--help") => print_help(),
        Some("run") => run_command(),
        Some("export") => export_command(args.get(1).map(String::as_str)),
        Some("config") => config_command(),
        Some("doctor") => doctor_command(),
        Some("service") => service_command(args.get(1).map(String::as_str)),
        Some("help") => print_help(),
        _ => {
            eprintln!("Unknown command: {}", command.unwrap_or(""));
            print_help();
            std::process::exit(1);
        }
    }
}
