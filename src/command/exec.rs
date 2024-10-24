use crate::{config::Config, parallel_run::parallel_run, serial_run::serial_run};

pub fn run(config: Config, parallel: bool, args: &Vec<String>) -> anyhow::Result<()> {
    if args.is_empty() {
        eprintln!("at least one command argument is required.");
        return Ok(());
    }

    if config.repositories.is_empty() {
        println!("No repositories registered: use the 'register' command");
        return Ok(());
    }

    let program = &args[0];
    let args = &args[1..];

    if parallel {
        parallel_run(config, program, args)
    } else {
        serial_run(config, program, args)
    }
}
