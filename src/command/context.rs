use std::io::{self, stdout, Write};

use crossterm::{
    cursor, event,
    style::{Color, ContentStyle, Print, PrintStyledContent, StyledContent, Stylize},
    terminal::{self, disable_raw_mode, enable_raw_mode, size},
    ExecutableCommand, QueueableCommand,
};

use crate::{config::Config, path::path_to_string};

/// Show the user an interactive checkbox UI for selecting repositories.
///
/// The user can move the repository selection using the arrow keys and toggle
/// the selection using the space bar.
///
/// If the number of repositories is greater than the terminal height, the list
/// is paginated.
pub fn context_ui(mut config: Config) -> anyhow::Result<()> {
    if config.repositories.is_empty() {
        println!("No repositories registered: use the 'register' command");
        return Ok(());
    }

    let repo_count = config.repositories.len();
    let (_, height) = size()?;

    // Not enough space for the UI
    if height <= 5 {
        return Ok(());
    }

    // The max page height is the terminal height with a little bit of space for the info
    // bar, the page indicator, and a blank line at the bottom.
    let max_page_height = height as usize - 3;

    let page_size = repo_count.min(max_page_height);
    let mut out = stdout();

    // Long repo names can muck up the redraw
    out.execute(terminal::DisableLineWrap)?;

    // Perform the first draw of the UI so that when the event_loop moves the cursor up it
    // moves to the correct place.
    queue_info_bar(&mut out)?;
    queue_page_info(&out, &config, 0, page_size)?;
    queue_repo_list(&out, &config, 0, page_size)?;

    event_loop(&mut out, &mut config, page_size)?;

    out.execute(terminal::EnableLineWrap)?;

    config.write()
}

fn event_loop(
    out: &mut io::Stdout,
    config: &mut Config,
    page_size: usize,
) -> Result<(), anyhow::Error> {
    let repo_count = config.repositories.len();
    let mut selected = 0;
    out.execute(cursor::Hide)?;
    enable_raw_mode()?;

    loop {
        match event::read()? {
            event::Event::Key(event) => match event.code {
                event::KeyCode::Char('+') => config
                    .repositories
                    .iter_mut()
                    .for_each(|r| r.visible = true),
                event::KeyCode::Char('-') => config
                    .repositories
                    .iter_mut()
                    .for_each(|r| r.visible = false),
                event::KeyCode::Up | event::KeyCode::Char('k') => {
                    if selected > 0 {
                        selected -= 1
                    }
                }
                event::KeyCode::Down | event::KeyCode::Char('j') => {
                    if selected < repo_count - 1 {
                        selected += 1
                    }
                }
                event::KeyCode::Left | event::KeyCode::Char('h') => {
                    if selected >= page_size {
                        selected -= page_size;
                    } else {
                        selected = 0;
                    }
                }
                event::KeyCode::Right | event::KeyCode::Char('l') => {
                    selected = (selected + page_size).min(repo_count - 1);
                }
                event::KeyCode::Enter => break,
                event::KeyCode::Char(' ') => {
                    config.repositories[selected].visible = !config.repositories[selected].visible
                }
                _ => {}
            },
            _ => {}
        }
        out.queue(cursor::MoveUp(page_size as u16 + 1))?;
        queue_page_info(&*out, config, selected, page_size)?;
        queue_repo_list(&*out, config, selected, page_size)?;
        out.flush()?;
    }

    disable_raw_mode()?;
    out.execute(cursor::Show)?;
    Ok(())
}

fn queue_page_info(
    mut out: impl QueueableCommand,
    config: &Config,
    selected: usize,
    page_size: usize,
) -> io::Result<()> {
    let repo_count = config.repositories.len();
    let page_count = ((repo_count - 1) / page_size) + 1;
    let selected_count = config
        .repositories
        .iter()
        .filter(|repo| repo.visible)
        .count();
    let selected_page = selected / page_size;

    if page_count > 1 {
        for i in 0..page_count {
            out.queue(Print(if i == selected_page { "⦿" } else { "○" }))?;
        }
        out.queue(Print(format!(
            " [Page {}/{}] ",
            selected_page + 1,
            page_count
        )))?;
    }

    out.queue(Print(format!(
        "[Selected {}/{}]\r\n",
        selected_count, repo_count
    )))?;
    Ok(())
}

fn queue_info_bar(mut out: impl QueueableCommand) -> Result<(), anyhow::Error> {
    const KEYS: [(&str, &str); 5] = [
        ("up/down", "move"),
        ("enter", "confirm"),
        ("space", "toggle"),
        ("+", "all"),
        ("-", "none"),
    ];

    let styled_keys: Vec<String> = KEYS
        .iter()
        .map(|(key, description)| format!("{} {}", key.dark_yellow(), description.dark_grey()))
        .collect();

    out.queue(Print(styled_keys.join(&format!("{}", ", ".dark_grey()))))?
        .queue(Print("\r\n"))?;

    Ok(())
}

fn queue_repo_list(
    mut out: impl QueueableCommand,
    config: &Config,
    selected: usize,
    page_size: usize,
) -> io::Result<()> {
    let repo_count = config.repositories.len();
    let page = selected / page_size;
    let page_start = page * page_size;
    let page_end = (page_start + page_size).min(repo_count);
    let repos = &config.repositories[page_start..page_end];
    for (i, repo) in repos.iter().enumerate() {
        // Construct the checkbox with the repo name
        let display = format!(
            "[{}] {}",
            if repo.visible { "✓" } else { " " },
            path_to_string(&repo.path)
        );
        let mut style = ContentStyle::new();
        style.foreground_color = if repo.visible {
            Some(Color::White)
        } else {
            Some(Color::Grey)
        };
        style.background_color = if i + page_start == selected {
            Some(Color::DarkBlue)
        } else {
            None
        };

        // 'Clear line' is not supported by crossterm, so use the ANSI code.
        // We are potentially overwriting a page, so if it had a longer repo name we need to clear it.
        const CLEAR_LINE: &str = "\x1B[2K";

        out.queue(Print(CLEAR_LINE))?
            .queue(PrintStyledContent(StyledContent::new(style, display)))?
            .queue(Print("\n\r"))?;
    }

    // Add blank lines to fill out page.
    // This keeps the pages a consistent size, so that when we move the cursor upwards
    // by the page size, it goes to the top of the list.
    for _ in 0..(page_size - (page_end - page_start)) {
        out.queue(Print("\x1B[2K\n\r"))?;
    }
    Ok(())
}
