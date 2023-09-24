pub mod board;
mod game;

use crate::game::run;
use crossterm::event::DisableMouseCapture;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::Terminal;
use std::io;
use std::io::Stdout;

fn setup() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn main() -> io::Result<()> {
    let mut term = setup()?;
    term.resize(Rect::new(10, 10, 1, 1)).unwrap();
    enable_raw_mode()?;

    run(&mut term)?;

    disable_raw_mode()?;
    execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;
    Ok(())
}
