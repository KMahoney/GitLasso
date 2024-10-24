use crate::{config::Config, parallel_run::parallel_run};

pub fn fetch(config: Config) -> anyhow::Result<()> {
    if config.repositories.is_empty() {
        println!("No repositories registered: use the 'register' command");
        return Ok(());
    }

    parallel_run(config, "git", &["fetch".to_string()])
}
