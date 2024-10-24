use crate::{config::Config, serial_run::serial_run};

pub fn run(config: Config, args: &Vec<String>) -> anyhow::Result<()> {
    if config.repositories.is_empty() {
        println!("No repositories registered: use the 'register' command");
        return Ok(());
    }

    serial_run(config, "git", args)
}
