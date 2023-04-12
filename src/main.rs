use rand;
use std::io::Write;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CellState {
    Alive,
    Dead(u8),
}

impl Default for CellState {
    fn default() -> Self {
        CellState::Dead(u8::MAX)
    }
}

#[derive(Default, Clone, Debug)]
struct Cell {
    current: CellState,
    next: CellState,
}

#[derive(Debug, Clone)]
struct Board {
    cells: Vec<Cell>,
    width: usize,
    height: usize,
}

impl Board {
    pub fn from_shape(width: usize, height: usize) -> Self {
        Board {
            width,
            height,
            cells: vec![Cell::default(); width * height],
        }
    }

    pub fn randomize<T>(mut self, probability: f64, generator: &mut T) -> Self
    where
        T: rand::Rng,
    {
        for cell in self.cells.iter_mut() {
            if generator.gen_bool(probability) {
                cell.current = CellState::Alive;
            }
        }
        self
    }

    pub fn index_by_position(&self, x: i32, y: i32) -> usize {
        let board_width = self.width as i32;
        let board_height = self.height as i32;
        (board_width * ((y + board_height) % board_height) + (x + board_width) % board_width)
            as usize
    }

    fn count_cells_around_position(&self, x: i32, y: i32, what_state: CellState) -> u8 {
        let mut counter = 0;

        for x_shift in [-1, 0, 1] {
            for y_shift in [-1, 0, 1] {
                if x_shift == 0 && y_shift == 0 {
                    continue;
                }
                if self
                    .cells
                    .get(self.index_by_position(x + x_shift, y + y_shift))
                    .unwrap()
                    .current
                    == what_state
                {
                    counter += 1;
                }
            }
        }

        counter
    }

    pub fn compute_one_step(&mut self) {
        // Compute the new state
        for x_pos in 0..self.width {
            for y_pos in 0..self.height {
                let alive_around =
                    self.count_cells_around_position(x_pos as i32, y_pos as i32, CellState::Alive);

                let index = self.index_by_position(x_pos as i32, y_pos as i32);
                let cell = self.cells.get_mut(index).unwrap();

                match cell.current {
                    CellState::Alive => {
                        cell.next = match alive_around {
                            // Any live cell with two or three live neighbours survives.
                            2 | 3 => CellState::Alive,
                            // Death by {over,under}crowd
                            _ => CellState::Dead(1),
                        }
                    }
                    CellState::Dead(cycles) => {
                        // Any dead cell with three live neighbours becomes a live cell.
                        if alive_around == 3 {
                            cell.next = CellState::Alive;
                            continue;
                        }
                        cell.next = CellState::Dead(match cycles {
                            u8::MAX => u8::MAX,
                            _ => cycles + 1,
                        })
                    }
                };
            }
        }
        // Swap the new and the old states
        for cell in self.cells.iter_mut() {
            std::mem::swap(&mut cell.current, &mut cell.next);
        }
    }
}

fn main() {
    let (width, height) = termion::terminal_size().unwrap();

    let mut generator = rand::thread_rng();
    let mut board =
        Board::from_shape(width as usize, height as usize).randomize(0.1, &mut generator);

    let mut stdout = std::io::stdout();

    write!(stdout, "{clear}", clear = termion::clear::All).unwrap();
    for x_pos in 0..board.width {
        for y_pos in 0..board.height {
            write!(
                stdout,
                "{}{} ",
                termion::cursor::Goto((x_pos + 1) as u16, (y_pos + 1) as u16),
                termion::color::Bg(termion::color::Rgb(0, 0, 0))
            )
            .unwrap();
        }
    }

    loop {
        let mut terminal_commands = String::with_capacity(board.width * board.height);
        for y_pos in 0..board.height {
            for x_pos in 0..board.width {
                let index = board.index_by_position(x_pos as i32, y_pos as i32);
                let cell = board.cells.get(index).unwrap();

                // Terminals are slooooooooooooow, dont update if possible
                if cell.current == cell.next {
                    continue;
                }

                match board.cells.get(index).unwrap().current {
                    CellState::Alive => terminal_commands.push_str(
                        format!(
                            "{}{} ",
                            termion::cursor::Goto((x_pos + 1) as u16, (y_pos + 1) as u16),
                            termion::color::Bg(termion::color::Rgb(u8::MAX, u8::MAX, u8::MAX,))
                        )
                        .as_str(),
                    ),
                    CellState::Dead(cycles) => {
                        let intencity_multiplier: u16 = 20;
                        let intencity = if cycles as u16 * intencity_multiplier > u8::MAX as u16 {
                            0
                        } else {
                            u8::MAX - cycles * intencity_multiplier as u8
                        };
                        terminal_commands.push_str(
                            format!(
                                "{}{} ",
                                termion::cursor::Goto((x_pos + 1) as u16, (y_pos + 1) as u16),
                                termion::color::Bg(termion::color::Rgb(
                                    intencity / 2,
                                    intencity / 5,
                                    intencity,
                                )),
                            )
                            .as_str(),
                        )
                    }
                }
            }
        }

        write!(
            stdout,
            "{terminal_commands}{reset}",
            reset = termion::color::Bg(termion::color::Black),
        )
        .unwrap();
        board.compute_one_step();

        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
