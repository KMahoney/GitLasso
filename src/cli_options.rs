use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(ValueEnum, Clone)]
pub enum CompletionShell {
    Bash,
    Fish,
    Zsh,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Scan a directory for git repositories and register them
    Register {
        /// Path to register
        #[arg(value_name = "PATH")]
        path: PathBuf,
    },

    /// Scan a directory for git repositories and unregister them
    Unregister {
        /// Keep the current context and discard unselected repositories
        #[arg(long = "keep-context")]
        keep_context: bool,
    },

    /// Update all git repositories
    Fetch,

    /// Exec a git command on all repositories
    Git {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Execute a command on all repositories
    Exec {
        /// Run command in parallel
        #[arg(short = 'p')]
        parallel: bool,

        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Select which repositories commands will apply to
    Context,

    /// Print completions for various shells
    Completions {
        #[arg(value_name = "SHELL")]
        shell: CompletionShell,

        #[arg(long = "binary", default_value = "gitlasso")]
        binary_name: String,
    },
}
