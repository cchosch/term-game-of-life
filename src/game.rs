use crate::board::{Board, BoardSegment, BoardState};
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

const FRAME_DELAY: Duration = Duration::from_millis(50);

pub fn run(term: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), io::Error> {
    let input_ch = start_input();
    let alive = Block::default().bg(Color::White);
    let (width, height) = size()?;
    let mut board: Board = Board::new(width as usize / 2, height as usize);
    let mut label = String::from("Press ESC to exit");
    loop {
        let recent = input_ch.try_recv().unwrap_or(Arc::new([]));
        if recent.contains(&27) {
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
            let render_time = SystemTime::now();
            let (width, height) = board.get_dimensions();
            for x in 0..width {
                for y in 0..height {
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
            let render_time = render_time.elapsed().unwrap();
            let elapsed_txt = Text::raw(format!(
                "Update time: {}ms\nRender time: {}ms\nTotal time elapsed: {}ms\n{label}",
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
        thread::sleep(FRAME_DELAY);
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
