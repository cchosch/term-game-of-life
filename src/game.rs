use std::io::{Read, stdin, Stdout};
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver};
use std::{io, thread};
use crossterm::terminal::size;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::style::{Color, Stylize};
use ratatui::Terminal;
use ratatui::widgets::Block;
use crate::board::Board;

pub fn run(term: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), io::Error> {
    let input_ch = start_input();
    let alive = Block::default().bg(Color::White);
    let (width, height) = size()?;
    let mut board: Board = Board::new(width as usize, height as usize);
    loop {
        let recent = input_ch.try_recv().unwrap_or(Arc::new([]));
        if recent.contains(&3) {
            break
        }
        board.update();
        term.draw(|f| {
            f.render_widget(alive.clone(), Rect::new(0, 0, 1, 1));
        })?;
    }
    Ok(())
}

fn start_input() -> Receiver<Arc<[u8]>> {
    let (ch_send, ch_recv) = channel();
    thread::spawn(move || {
        let mut input = stdin();
        let mut buffer: Box<[u8]>;
        loop {
            buffer = Box::new([0; 32]);
            input.read(&mut buffer).unwrap();
            ch_send.send(buffer.into()).unwrap();
        }
    });
    ch_recv
}

