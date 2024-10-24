use std::io::{stdout, Write};

use clap::CommandFactory;
use clap_complete::{aot, generate};

use crate::cli_options::{Cli, CompletionShell};

pub fn completions(shell: CompletionShell, binary_name: &str) -> anyhow::Result<()> {
    let mut buffer = Vec::new();

    match shell {
        CompletionShell::Bash => generate(aot::Bash, &mut Cli::command(), binary_name, &mut buffer),
        CompletionShell::Fish => {
            generate(aot::Fish, &mut Cli::command(), binary_name, &mut buffer);
            let ext = include_str!("../../completions/gitlasso.fish");
            let _ = writeln!(&mut buffer, "{}", ext.replace("<BINARY>", binary_name));
        }
        CompletionShell::Zsh => generate(aot::Zsh, &mut Cli::command(), binary_name, &mut buffer),
    }

    let mut out = stdout();
    out.write(&buffer)?;
    out.flush()?;
    Ok(())
}
