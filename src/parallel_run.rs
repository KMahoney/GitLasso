use crossterm::style::Stylize;
use crossterm::terminal::size;
use crossterm::{cursor, style, QueueableCommand};
use std::collections::HashMap;
use std::io::{self, stdout, Write};
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::RecvTimeoutError;
use std::thread;
use std::time::Duration;

use crate::config::queue_context_line;
use crate::config::Config;
use crate::path::path_to_string;

const SPINNER_CHARS: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

#[derive(PartialEq)]
enum ProcessStatus {
    Running,
    Finished(String),
    Error(String),
}

type ProcessStatuses = HashMap<PathBuf, ProcessStatus>;

/// Run a program on all selected repositories in parallel. Show a spinner for each repository as
/// the program is running, and then show any errors.
pub fn parallel_run(config: Config, program: &str, args: &[String]) -> anyhow::Result<()> {
    let paths = config.visible_repos();

    let mut results: ProcessStatuses = paths
        .iter()
        .map(|p| (p.clone(), ProcessStatus::Running))
        .collect();

    // Each thread sends its result back through this channel.
    let (tx, rx) = mpsc::channel();

    // For each repo, kick off a thread executing the command.
    for path in &paths {
        let thread_path = path.clone();
        let thread_tx = tx.clone();
        let thread_program = program.to_string();
        let thread_args = Vec::from(args);
        thread::spawn(move || {
            let output = std::process::Command::new(thread_program)
                .args(&thread_args)
                .current_dir(&thread_path)
                .output();

            let result = match output {
                Ok(output) => {
                    if output.status.success() {
                        ProcessStatus::Finished(
                            String::from_utf8_lossy(&output.stdout).into_owned(),
                        )
                    } else {
                        ProcessStatus::Error(String::from_utf8_lossy(&output.stderr).into_owned())
                    }
                }
                Err(err) => ProcessStatus::Error(err.to_string()),
            };

            // This should only fail to send when the receiver has hung up.
            // In theory this cannot happen.
            thread_tx
                .send((thread_path.clone(), result))
                .expect("could not send");
        });
    }

    // This has been cloned for each thread, so drop the original.
    // When the threads have all dropped their clone, the channel will close.
    drop(tx);

    let (width, height) = size()?;

    // Show a compact spinner if there isn't enough space to show a spinner for each repo.
    let compact = paths.len() >= height as usize;

    wait_for_results(config, &paths, &mut results, rx, compact)?;

    // Print out errors
    for path in paths {
        match results.get(&path) {
            Some(ProcessStatus::Error(err)) => {
                let header = format!("{:width$}", path_to_string(&path), width = width as usize)
                    .on_red()
                    .black();
                stdout()
                    .queue(style::Print("\n"))?
                    .queue(style::PrintStyledContent(header))?
                    .queue(style::Print("\n"))?
                    .queue(style::Print(err))?;
            }
            Some(ProcessStatus::Finished(out)) => {
                if !out.is_empty() {
                    let header =
                        format!("{:width$}", path_to_string(&path), width = width as usize)
                            .on_white()
                            .black();
                    stdout()
                        .queue(style::Print("\n"))?
                        .queue(style::PrintStyledContent(header))?
                        .queue(style::Print("\n"))?
                        .queue(style::Print(out))?;
                }
            }
            _ => {}
        }
    }
    stdout().flush()?;

    Ok(())
}

fn wait_for_results(
    config: Config,
    paths: &[PathBuf],
    results: &mut HashMap<PathBuf, ProcessStatus>,
    rx: mpsc::Receiver<(PathBuf, ProcessStatus)>,
    compact: bool,
) -> io::Result<()> {
    let mut spinner_index = 0;
    let mut out = stdout();

    queue_context_line(&out, &config)?;
    out.queue(cursor::Hide)?;
    if !compact {
        // An initial print, so that when the cursor is moved up it goes to the correct place.
        queue_update_progress(&out, paths, &*results, spinner_index)?;
    }
    out.flush()?;

    // Receive results until the channel disconnects (i.e. all threads have finished).
    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok((repo_path, result)) => {
                results.insert(repo_path.clone(), result);
            }
            Err(RecvTimeoutError::Timeout) => {
                if compact {
                    queue_update_progress_compact(&out, paths, results, spinner_index)?;
                } else {
                    out.queue(cursor::MoveUp(paths.len() as u16))?;
                    queue_update_progress(&out, paths, results, spinner_index)?;
                }
                out.flush()?;
                spinner_index = (spinner_index + 1) % SPINNER_CHARS.len();
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }

    // One last update.
    if compact {
        queue_update_progress_compact(&out, paths, results, spinner_index)?;
        out.queue(style::Print("\r\n"))?;
    } else {
        out.queue(cursor::MoveUp(paths.len() as u16))?;
        queue_update_progress(&out, paths, &*results, spinner_index)?;
    }
    out.queue(cursor::Show)?;
    out.flush()?;
    Ok(())
}

fn queue_update_progress_compact(
    mut out: impl QueueableCommand,
    paths: &[PathBuf],
    results: &ProcessStatuses,
    spinner_index: usize,
) -> io::Result<()> {
    out.queue(style::Print("\r"))?;
    out.queue(style::PrintStyledContent(
        SPINNER_CHARS[spinner_index].bold(),
    ))?;
    let total = paths.len();
    let finished = paths
        .into_iter()
        .filter(|p| results.get(*p) != Some(&ProcessStatus::Running))
        .count();
    out.queue(style::Print(format!(
        " running [{}/{} complete]",
        finished, total
    )))?;
    Ok(())
}

fn queue_update_progress(
    mut out: impl QueueableCommand,
    paths: &[PathBuf],
    results: &ProcessStatuses,
    spinner_index: usize,
) -> io::Result<()> {
    for path in paths {
        if let Some(result) = results.get(path) {
            out.queue(style::PrintStyledContent(match *result {
                ProcessStatus::Finished(_) => '✓'.dark_green(),
                ProcessStatus::Running => SPINNER_CHARS[spinner_index].bold(),
                ProcessStatus::Error(_) => 'X'.dark_red(),
            }))?
            .queue(style::Print(format!(" {}\n", path_to_string(path))))?;
        }
    }
    Ok(())
}
