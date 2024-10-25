use crate::{config::Config, parallel_run::parallel_run};

pub fn pull(config: Config) -> anyhow::Result<()> {
    if config.repositories.is_empty() {
        println!("No repositories registered: use the 'register' command");
        return Ok(());
    }

    parallel_run(config, "git", &["pull".to_string()], true)
}
