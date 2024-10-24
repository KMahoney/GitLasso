use crossterm::{
    style::{self, Stylize},
    terminal::size,
    QueueableCommand,
};

use crate::{config::queue_context_line, config::Config, path::path_to_string};

use std::{
    io::{self, stdout, Write},
    path::PathBuf,
};

pub fn serial_run(config: Config, program: &str, args: &[String]) -> anyhow::Result<()> {
    let paths: Vec<PathBuf> = config.visible_repos().map(|p| p.to_path_buf()).collect();

    queue_context_line(stdout(), &config)?;

    let (width, _) = size()?;

    for path in &paths {
        let header = format!("{:width$}", path_to_string(path), width = width as usize)
            .on_white()
            .black();

        stdout()
            .queue(style::Print("\n"))?
            .queue(style::PrintStyledContent(header))?
            .queue(style::Print("\n"))?
            .flush()?;

        let mut command = std::process::Command::new(program)
            .args(args)
            .current_dir(&path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let mut stdout = command.stdout.take().unwrap();
        let mut stderr = command.stderr.take().unwrap();

        let stdout_handle = std::thread::spawn(move || {
            let _ = io::copy(&mut stdout, &mut io::stdout());
        });

        let stderr_handle = std::thread::spawn(move || {
            let _ = io::copy(&mut stderr, &mut io::stderr());
        });

        stdout_handle.join().unwrap();
        stderr_handle.join().unwrap();

        command.wait()?;
    }

    stdout().flush()?;
    Ok(())
}
