use model::config::{ServiceConfig, read_config};
use std::{
    thread,
    fs::File,
    error::Error,
    io::{BufReader, stdout, Result as IOResult},
    path::Path,
    time::Duration
};
use tui::{
    backend::CrosstermBackend,
    widgets::{Widget, Block, Borders},
    layout::{Layout, Constraint, Direction},
    Terminal
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::style::{Color, Style};

fn main() -> IOResult<()> {
    let config = read_config(Path::new("./sampleConfig.yml"));

    enable_raw_mode()?;

    let stdout = stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut executing = true;

    while executing {
        terminal.draw(|f| {
            let size = f.size();

            let block = Block::default()
                .style(
                    Style::default()
                        .bg(Color::Black)
                ).title(format!("Block"))
                .borders(Borders::ALL);
            f.render_widget(block, size);
        })?;

        let has_event = event::poll(Duration::from_millis(20))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Esc => {
                    executing = false;
                },
                _ => {}
            }
        }
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}