use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn is_git_repo(path: &Path) -> bool {
    path.join(".git").exists()
}

pub fn find_git_repos(root_dir: &Path) -> Vec<PathBuf> {
    // An explicit iterator & loop is used here to short circuit the recursion when
    // a git repo is found.

    let mut paths: Vec<PathBuf> = Vec::new();

    // Iterate through directories
    let mut it = WalkDir::new(root_dir)
        .into_iter()
        .filter_entry(|entry| entry.file_type().is_dir());

    loop {
        let entry = match it.next() {
            None => break,
            Some(Err(_)) => continue,
            Some(Ok(entry)) => entry,
        };

        // If this is a git repo, stop iterating through its children
        if is_git_repo(entry.path()) {
            if let Ok(full_path) = entry.path().canonicalize() {
                paths.push(full_path);
                it.skip_current_dir();
            }
        }
    }

    return paths;
}
