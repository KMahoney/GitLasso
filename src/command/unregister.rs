use std::path::PathBuf;

use crate::config::Config;

pub fn unregister(mut config: Config, keep_context: bool) -> anyhow::Result<()> {
    let repos: Vec<PathBuf> = if keep_context {
        config.invisible_repos()
    } else {
        config.visible_repos()
    };

    for path in repos {
        if config.remove_repo(&path) {
            println!("{}: unregistered", path.to_string_lossy());
        } else {
            println!("{}: not registered", path.to_string_lossy());
        }
    }

    config.write()
}
