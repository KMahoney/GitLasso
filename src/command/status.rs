use std::io::stdout;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::thread::{self, JoinHandle};

use crossterm::style::Stylize;
use crossterm::terminal::size;
use git2::{Repository, StatusOptions};
use serde::{Deserialize, Serialize};

use crate::config::queue_context_line;
use crate::config::Config;
use crate::path::path_to_string;
use crate::tui::table::queue_table;
use crate::tui::table::Cell;
use crate::tui::table::Table;

pub fn status(config: Config) -> anyhow::Result<()> {
    if config.repositories.is_empty() {
        println!("No repositories registered: use the 'register' command");
        return Ok(());
    }

    let paths = config.visible_repos();

    // Fetch repository info in parallel
    let info_threads: Vec<JoinHandle<(PathBuf, Result<RepoInfo, git2::Error>)>> = paths
        .iter()
        .map(|path| {
            let thread_path = path.clone();
            thread::spawn(move || (thread_path.clone(), fetch_info(&thread_path)))
        })
        .collect();

    // Collect threads, printing any errors to stderr
    let info_repos: Vec<RepoInfo> = info_threads
        .into_iter()
        .filter_map(|handle| {
            let (path, repo_info) = handle.join().unwrap();
            match repo_info {
                Ok(info) => Some(info),
                Err(err) => {
                    let name = path.to_string_lossy();
                    eprintln!("Error {}: {}", name, err);
                    None
                }
            }
        })
        .collect();

    // Display the status table
    let (width, _) = size()?;
    queue_context_line(stdout(), &config)?;
    queue_table(stdout(), build_table(info_repos, width as usize))?;
    stdout().flush()?;
    Ok(())
}

fn build_table(repos: Vec<RepoInfo>, width: usize) -> Table {
    let mut rows: Vec<Vec<Cell>> = Vec::new();

    // Header
    let headers = ["path", "name", "branch", "status", "upstream", "", "commit"];
    rows.push(
        headers
            .into_iter()
            .map(|h| Cell::new([h.to_owned().bold()]))
            .collect(),
    );

    // Body
    for repo in repos {
        let row: Vec<Cell> = vec![
            Cell::plain(repo.parent_path.unwrap_or("-".to_owned())),
            Cell::new([repo.name.bold()]),
            Cell::plain(repo.branch_name),
            Cell::new([match repo.status {
                RepoStatus::Clean => "clean".to_string().stylize(),
                RepoStatus::Modified(n) => format!("{n} modified").red(),
            }]),
            repo.upstream_remote_info
                .map(|remote_info| {
                    Cell::plain(format!("{}@{}", remote_info.url, remote_info.branch))
                })
                .unwrap_or(Cell::plain(repo.upstream.unwrap_or("-".to_owned()))),
            match repo.ahead_behind {
                Some((ahead, behind)) => {
                    let ahead_string = format!("+{}", ahead);
                    let behind_string = format!("-{}", behind);
                    Cell::new([
                        if ahead > 0 {
                            ahead_string.green()
                        } else {
                            ahead_string.stylize()
                        },
                        "/".to_string().stylize(),
                        if behind > 0 {
                            behind_string.red()
                        } else {
                            behind_string.stylize()
                        },
                    ])
                }
                None => Cell::plain(""),
            },
            Cell::plain(format!(
                "{} {}",
                &repo.latest_commit_hash.chars().take(7).collect::<String>(),
                repo.latest_commit_message
            )),
        ];
        rows.push(row);
    }

    Table { width, rows }
}

#[derive(Serialize, Deserialize)]
struct RepoInfo {
    parent_path: Option<String>,
    name: String,
    branch_name: String,
    status: RepoStatus,
    upstream: Option<String>,
    upstream_remote_info: Option<RemoteInfo>,
    ahead_behind: Option<(usize, usize)>,
    latest_commit_hash: String,
    latest_commit_message: String,
}

#[derive(Serialize, Deserialize)]
enum RepoStatus {
    Clean,
    Modified(usize),
}

#[derive(Serialize, Deserialize)]
struct RemoteInfo {
    url: String,
    branch: String,
}

/// Fetch info on a git repository
fn fetch_info(repo_path: &Path) -> Result<RepoInfo, git2::Error> {
    let repo = Repository::open(repo_path)?;

    let name = repo_path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or("-".to_owned());

    let parent_path = repo_path.parent().map(path_to_string);

    let head = match repo.head() {
        Ok(head) => head,
        Err(_) => {
            // No 'head' means this is probably an empty repository
            return Ok(RepoInfo {
                name,
                parent_path,
                branch_name: "-".to_string(),
                status: RepoStatus::Clean,
                upstream: None,
                upstream_remote_info: None,
                ahead_behind: None,
                latest_commit_hash: "-".to_string(),
                latest_commit_message: "-".to_string(),
            });
        }
    };

    let mut status_options = StatusOptions::new();
    status_options.include_ignored(false);
    status_options.include_untracked(false);

    let modified_count = repo
        .statuses(Some(&mut status_options))?
        .iter()
        .filter(|status| status.status() != git2::Status::CURRENT)
        .count();

    let status = if modified_count > 0 {
        RepoStatus::Modified(modified_count)
    } else {
        RepoStatus::Clean
    };

    let branch_ref_name = head.name().unwrap_or("?").to_string();

    let branch_shorthand = head.shorthand().unwrap_or("?").to_string();

    let upstream_ref_name = repo
        .branch_upstream_name(&branch_ref_name)
        .ok()
        .and_then(|name| name.as_str().map(str::to_string));

    let upstream_reference = upstream_ref_name
        .as_ref()
        .and_then(|name| repo.find_reference(&name).ok());

    let ahead_behind = match (head.target(), upstream_reference.and_then(|r| r.target())) {
        (Some(head_oid), Some(upstream_oid)) => {
            repo.graph_ahead_behind(head_oid, upstream_oid).ok()
        }
        _ => None,
    };

    let upstream_remote_info = upstream_ref_name
        .as_ref()
        .and_then(|name| name.strip_prefix("refs/remotes/"))
        .and_then(|stripped_name| stripped_name.split_once('/'))
        .and_then(|(remote_name, branch)| {
            repo.find_remote(remote_name)
                .ok()
                .and_then(|remote| remote.url().map(|url| url.to_string()))
                .map(|url| RemoteInfo {
                    url: url,
                    branch: branch.to_string(),
                })
        });

    let head_commit = head.peel_to_commit()?;

    Ok(RepoInfo {
        name,
        parent_path,
        branch_name: branch_shorthand,
        status,
        upstream: upstream_ref_name,
        upstream_remote_info,
        ahead_behind: ahead_behind,
        latest_commit_hash: head_commit.id().to_string(),
        latest_commit_message: head_commit.summary().unwrap_or("-").to_string(),
    })
}
