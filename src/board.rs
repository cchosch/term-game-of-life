use anyhow::Context;
use rand::Rng;
use std::rc::Rc;
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

const SEG_S: usize = 8;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Copy)]
pub enum BoardState {
    Alive,
    Dead,
}

#[derive(Clone, Debug)]
pub struct BoardSegment {
    grid: [[BoardState; SEG_S]; SEG_S],
    queued: Arc<RwLock<[[BoardState; SEG_S]; SEG_S]>>,
    position: (usize, usize),
}

#[derive(Clone, Debug)]
pub struct Board {
    grid: Vec<Vec<BoardSegment>>,
    width: usize,
    height: usize,
}

impl Board {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            grid: (0..=height / SEG_S)
                .map(|x| {
                    (0..=width / SEG_S)
                        .map(|y| BoardSegment::random(x, y))
                        .collect()
                })
                .collect(),
            width,
            height,
        }
    }

    pub fn get_dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn get(&self, x: usize, y: usize) -> Option<BoardState> {
        if !(0..self.width).contains(&x) {
            return None;
        }
        if !(0..self.height).contains(&y) {
            return None;
        }
        return Some(self.grid[y / SEG_S][x / SEG_S].get(x % SEG_S, y % SEG_S));
    }

    pub fn update(&mut self) {
        let ref_board = Arc::new(self.clone());
        let mut recvers: Vec<Receiver<(usize, usize)>> = vec![];
        for row in 0..=(self.height / SEG_S) {
            for x in 0..=(self.width / SEG_S) {
                recvers.push(self.grid[row][x].queue_update(ref_board.clone()));
            }
        }
        for recver in recvers {
            let (y, x) = recver.recv().unwrap();
            self.grid[y][x].swap().unwrap();
        }
    }
}

impl BoardSegment {
    pub fn random(x: usize, y: usize) -> Self {
        Self {
            grid: (0..SEG_S)
                .map(|_| {
                    (0..SEG_S)
                        .map(|_| BoardState::random())
                        .collect::<Vec<BoardState>>()
                        .try_into()
                        .unwrap()
                })
                .collect::<Vec<[BoardState; SEG_S]>>()
                .try_into()
                .unwrap(),
            queued: Arc::new(RwLock::new(
                (0..SEG_S)
                    .map(|_| {
                        (0..SEG_S)
                            .map(|_| BoardState::Dead)
                            .collect::<Vec<BoardState>>()
                            .try_into()
                            .unwrap()
                    })
                    .collect::<Vec<[BoardState; SEG_S]>>()
                    .try_into()
                    .unwrap(),
            )),
            position: (x, y),
        }
    }
    pub fn get(&self, x: usize, y: usize) -> BoardState {
        self.grid[y][x]
    }

    pub fn queue_update(&mut self, board: Arc<Board>) -> Receiver<(usize, usize)> {
        let (ch_send, ch_recv) = channel();
        let queued_grid = self.queued.clone();
        let (seg_x, seg_y) = self.position.clone();
        thread::spawn(move || {
            let mut new_vals = queued_grid.read().unwrap().clone();
            for x in 0..SEG_S {
                for y in 0..SEG_S {
                    new_vals[y][x] = new_value(seg_x + x, seg_y + y, board.clone());
                }
            }
            let mut qg = queued_grid.write().unwrap();
            *qg = new_vals;
            drop(qg);
            let _ = ch_send.send((seg_x, seg_y));
        });
        ch_recv
    }

    pub fn swap(&mut self) -> anyhow::Result<()> {
        let qg = self.queued.read().unwrap();
        for x in 0..SEG_S {
            for y in 0..SEG_S {
                self.grid[x][y] = qg[x][y];
            }
        }
        drop(qg);
        Ok(())
    }
}

impl BoardState {
    pub fn random() -> Self {
        match rand::thread_rng().gen_range(0..2) {
            0 => BoardState::Alive,
            1 => BoardState::Dead,
            _ => unreachable!(),
        }
    }
}

fn update_row(row: usize, board: Arc<Board>) -> Receiver<(usize, Vec<BoardState>)> {
    let (ch_send, ch_recv) = channel();
    thread::spawn(move || {
        let mut new_vals = Vec::with_capacity(board.width);
        for x in 0..board.width {
            new_vals.push(new_value(x, row, board.clone()));
        }
        ch_send.send((row, new_vals)).unwrap();
    });
    ch_recv
}

fn new_value(x: usize, y: usize, board: Arc<Board>) -> BoardState {
    let current = board.get(x, y);
    if let None = current {
        return BoardState::Dead;
    }
    let current = current.unwrap();
    let x = x as i32;
    let y = y as i32;

    let mut alive = 0;
    for i in -1..=1 {
        for j in -1..=1 {
            if i == 0 && j == 0 {
                continue;
            }
            alive += if board
                .get((x + i) as usize, (y + j) as usize)
                .unwrap_or(BoardState::Dead)
                == BoardState::Alive
            {
                1
            } else {
                0
            };
        }
    }
    if alive == 3 || (current == BoardState::Alive && alive == 2) {
        return BoardState::Alive;
    }
    return BoardState::Dead;
}
