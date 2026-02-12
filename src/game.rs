use crate::engine::Engine;
use queenfish::board::{Board, Turn, Move};

#[derive(Debug, Clone, Copy)]
pub enum TimeControl {
    Infinite,
    TimePerMove(i32), // in ms
}

pub struct Game {
    white: Engine,
    black: Engine,
    moves_list: Vec<String>,
    board: Board,
    time_control: TimeControl,
}

#[derive(Debug, Clone)]
pub struct GameResult {
    white: String,
    black: String,
    moves_list: Vec<String>,
    result: i32,
}
impl GameResult {
    pub fn winner(&self) -> String {
        match self.result {
            1 => self.white.clone(),
            -1 => self.black.clone(),
            _ => String::new(),
        }
    }
}

impl Game {
    pub fn new(white: Engine, black: Engine, time_control: TimeControl) -> Self {
        Game {
            white,
            black,
            moves_list: Vec::new(),
            board: Board::new(),
            time_control,
        }
    } //

    // pub fn play(&mut self) -> GameResult {
    //     let start_time = std::time::Instant::now();
    //     let mut white_process = self.white.spawn_process();
    //     let mut black_process = self.black.spawn_process();

    //     loop {
    //         let valid_moves = self.board.generate_moves();
    //         if valid_moves.is_empty() {
    //             match self.board.turn {
    //                 Turn::WHITE => println!("{} wins as black", self.black.name),
    //                 Turn::BLACK => println!("{} wins as white", self.white.name),
    //             }
    //             let result: i32;
    //             if self.board.is_king_in_check(self.board.turn) {
    //                 match self.board.turn {
    //                     Turn::WHITE => result = -1,
    //                     Turn::BLACK => result = 1,
    //                 }
    //             } else {
    //                 result = 0;
    //             }
    //             return GameResult {
    //                 white: self.white.name.clone(),
    //                 black: self.black.name.clone(),
    //                 moves_list: self.moves_list.clone(),
    //                 result,
    //             };
    //         }
    //         let engine_process = match self.board.turn {
    //             Turn::WHITE => &mut white_process,
    //             Turn::BLACK => &mut black_process,
    //         };
    //         if self.moves_list.is_empty() {
    //             engine_process.send_command(format!("position startpos\n").as_str());
    //         } else {
    //             engine_process.send_command(
    //                 format!("position startpos moves {}\n", self.moves_list.join(" ")).as_str(),
    //             );
    //         }

    //         match self.time_control {
    //             TimeControl::Infinite => {
    //                 engine_process.send_command("go infinite\n");
    //             }
    //             TimeControl::TimePerMove(time) => {
    //                 engine_process.send_command(format!("go movetime {}\n", time).as_str());
    //             }
    //         }

    //         loop {
    //             if let Some(line) = engine_process.read_line() {
    //                 if line.starts_with("bestmove") {
    //                     let best_move = line.split_whitespace().nth(1).unwrap();
    //                     self.moves_list.push(best_move.to_string());
    //                     self.board.make_move(Move::from_uci(best_move, &self.board));
    //                     break;
    //                 } else {
    //                     // println!("{}", line);
    //                 }
    //             }
    //         }
    //     } //
    //     white_process.disconnect();
    //     black_process.disconnect();
    // } //
} //
