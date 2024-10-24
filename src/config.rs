use anyhow::Context;
use crossterm::style::{Print, PrintStyledContent, Stylize};
use crossterm::QueueableCommand;
use std::fs::{read_to_string, File};
use std::io::Result;
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct Config {
    pub path: PathBuf,
    pub repositories: Vec<RepoConfig>,
}

pub struct RepoConfig {
    pub path: PathBuf,
    pub visible: bool,
}

pub fn read(repositories_path: &Path) -> anyhow::Result<Config> {
    if !Path::new(repositories_path).exists() {
        return Ok(Config {
            path: repositories_path.to_path_buf(),
            repositories: Vec::new(),
        });
    }

    let str = read_to_string(repositories_path)
        .with_context(|| "failed to read the repositories file")?;

    let repositories = str
        .lines()
        .map(|line| match line.strip_prefix("#") {
            Some(str_path) => RepoConfig {
                path: PathBuf::from(str_path),
                visible: false,
            },
            None => RepoConfig {
                path: PathBuf::from(line),
                visible: true,
            },
        })
        .collect();

    Ok(Config {
        path: repositories_path.to_path_buf(),
        repositories,
    })
}

impl Config {
    pub fn write(&self) -> anyhow::Result<()> {
        let repositories_string = self
            .repositories
            .iter()
            .map(|repo| {
                if repo.visible {
                    format!("{}", repo.path.to_string_lossy())
                } else {
                    format!("#{}", repo.path.to_string_lossy())
                }
            })
            .collect::<Vec<String>>()
            .join("\n");
        let mut file =
            File::create(&self.path).with_context(|| "failed to create the configuration file")?;
        file.write_all(repositories_string.as_bytes())
            .with_context(|| "failed to write the configuration file")
    }

    pub fn add_repo(&mut self, repo_path: &Path) -> bool {
        if self.repositories.iter().any(|r| r.path == repo_path) {
            return false;
        }

        self.repositories.push(RepoConfig {
            path: repo_path.to_owned(),
            visible: true,
        });
        return true;
    }

    pub fn remove_repo(&mut self, repo_path: &Path) -> bool {
        let exists = self.repositories.iter().any(|r| r.path == repo_path);
        self.repositories.retain(|r| r.path != repo_path);
        exists
    }

    pub fn visible_repos(&self) -> impl Iterator<Item = &Path> {
        self.repositories
            .iter()
            .filter(|&r| r.visible)
            .map(|r| r.path.as_path())
    }
}

pub fn queue_context_line(mut f: impl QueueableCommand, config: &Config) -> Result<()> {
    let visible = config.visible_repos().count();
    let total = config.repositories.len();
    if visible == total {
        return Ok(());
    }
    f.queue(PrintStyledContent(
        format!("context: {} of {} repositories", visible, total).dark_grey(),
    ))?;
    f.queue(Print("\r\n"))?;
    Ok(())
}
