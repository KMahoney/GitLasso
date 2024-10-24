use std::path::Path;

use crate::{config::Config, discover::find_git_repos};

pub fn register(mut config: Config, root_path: &Path) -> anyhow::Result<()> {
    let discovered_repo_paths = find_git_repos(root_path);

    if discovered_repo_paths.is_empty() {
        println!(
            "No repositories discovered in '{}'",
            root_path.to_string_lossy()
        );
        return Ok(());
    }

    for repo_path in discovered_repo_paths {
        if config.add_repo(&repo_path) {
            println!("{}: registered", repo_path.to_string_lossy());
        } else {
            println!("{}: already registered", repo_path.to_string_lossy());
        }
    }

    config.write()
}
