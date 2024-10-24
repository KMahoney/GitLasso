use anyhow::Error;
use clap::Parser;
use cli_options::{Cli, Commands};
use directories::ProjectDirs;

mod cli_options;
mod command;
mod config;
mod discover;
mod parallel_run;
mod path;
mod serial_run;
mod tui;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let project_dirs = ProjectDirs::from("", "", "GitLasso")
        .ok_or(Error::msg("could not find configuration directory"))?;

    let repositories_path = project_dirs.config_dir().join("repositories");

    let config = config::read(&repositories_path)?;

    match cli.command {
        Some(Commands::Register { path }) => command::register::register(config, &path),
        Some(Commands::Unregister { path }) => command::unregister::unregister(config, &path),
        Some(Commands::Fetch) => command::fetch::fetch(config),
        Some(Commands::Git { args }) => command::git::run(config, &args),
        Some(Commands::Exec { parallel, args }) => command::exec::run(config, parallel, &args),
        Some(Commands::Context) => command::context::context_ui(config),
        Some(Commands::Completions { shell, binary_name }) => {
            command::completions::completions(shell, &binary_name)
        }
        None => command::status::status(config),
    }
}
