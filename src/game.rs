use crate::board::{Board, BoardState};
use crossterm::terminal::size;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::style::{Color, Stylize};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;
use std::io::{stdin, Read, Stdout};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use std::{io, thread};

pub fn run(term: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), io::Error> {
    let input_ch = start_input();
    let alive = Block::default().bg(Color::White);
    let (width, height) = size()?;
    let mut board: Board = Board::new(width as usize / 2, height as usize);
    loop {
        let recent = input_ch.try_recv().unwrap_or(Arc::new([]));
        if recent.contains(&3) {
            break;
        }
        let before_board_update = SystemTime::now();
        board.update();
        let update_time = before_board_update.elapsed().unwrap();
        let stats = Block::default()
            .borders(Borders::ALL)
            .bg(Color::default())
            .title("Stats");
        term.draw(|f| {
            let before_render = SystemTime::now();
            for x in 0..board.get_width() {
                for y in 0..board.get_height() {
                    match board.get(x.clone(), y.clone()) {
                        Some(BoardState::Alive) => {
                            f.render_widget(
                                alive.clone(),
                                Rect::new(x.clone() as u16 * 2, y.clone() as u16, 2, 1),
                            );
                        }
                        _ => {}
                    }
                }
            }
            let render_time = before_render.elapsed().unwrap();
            let elapsed_txt = Text::raw(format!(
                "Update time: {}ms\nRender time: {}ms\nTotal time elapsed: {}ms",
                update_time.as_millis(),
                render_time.as_millis(),
                before_board_update.elapsed().unwrap().as_millis()
            ));
            let stats_para = Paragraph::new(elapsed_txt.clone()).block(stats.clone());
            f.render_widget(
                stats_para,
                Rect::new(
                    0,
                    0,
                    elapsed_txt.width() as u16 + 2,
                    elapsed_txt.height() as u16 + 2,
                ),
            );
        })?;
        thread::sleep(Duration::from_millis(500));
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
