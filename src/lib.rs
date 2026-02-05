use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use queenfish::board::{Board, Move, Turn};

#[derive(Debug)]
enum EngineOption {
    CHECK {
        name: String,
        value: bool
    },
    SPIN {
        name: String,
        value: i32,
        min: Option<i32>,
        max: Option<i32>,
    }
} //


struct Engine {
    path: String,
    name: String,
    engine_options: Vec<EngineOption>,
    child_process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
} //

enum TimeControl {
    INFINITE,
    TIME_PER_MOVE(i32), // in ms
}

struct Game {
    white: Engine,
    black: Engine,
    moves_list: Vec<String>,
    board: Board,
    time_control: TimeControl
}

#[derive(Debug)]
struct GameResult {
    white: String,
    black: String,
    moves_list: Vec<String>,
    white_won: bool,
}

impl Game {
    pub fn new(white: Engine, black: Engine, time_control: TimeControl) -> Self {
        Game {
            white,
            black,
            moves_list: Vec::new(),
            board: Board::new(),
            time_control
        }
    } //

    pub fn play(&mut self) -> GameResult {
        let start_time = std::time::Instant::now();
        loop {
            let valid_moves = self.board.generate_moves();
            if valid_moves.is_empty() {
                match self.board.turn {
                    Turn::WHITE => println!("{} wins as black", self.black.name),
                    Turn::BLACK => println!("{} wins as white", self.white.name),
                }
                return GameResult {
                    white: self.white.name.clone(),
                    black: self.black.name.clone(),
                    moves_list: self.moves_list.clone(),
                    white_won: self.board.turn == Turn::BLACK,
                }
            }
            let engine = match self.board.turn {
                Turn::WHITE => &mut self.white,
                Turn::BLACK => &mut self.black,
            };
            if self.moves_list.is_empty() {
                engine.send_command(format!("position startpos\n").as_str());
            } else {
                engine.send_command(
                    format!("position startpos moves {}\n", self.moves_list.join(" ")).as_str(),
                );
            }

            match self.time_control {
                TimeControl::INFINITE => {
                    engine.send_command("go infinite\n");
                }
                TimeControl::TIME_PER_MOVE(time) => {
                    engine.send_command(format!("go movetime {}\n", time).as_str());
                }
            }

            loop {
                if let Some(line) = engine.read_line() {
                    if line.starts_with("bestmove") {
                        let best_move = line.split_whitespace().nth(1).unwrap();
                        self.moves_list.push(best_move.to_string());
                        self.board.make_move(Move::from_uci(best_move, &self.board));
                        break;
                    } else {
                        // println!("{}", line);
                    }
                }
            }
        };

    } //
} //

impl Engine {
    pub fn new(path: &str, name: &str) -> Self {
        let path = Path::new(path);

        if !path.exists() {
            panic!("Engine path does not exist");
        } else if !path.is_file() {
            panic!("Engine path is not a file");
        }
        if let Some(extension) = path.extension() {
            if extension != "exe" && extension != "" {
                panic!("Engine file is not an executable");
            }
        } else {
            panic!("Engine file has no extension");
        }

        let mut engine_process = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to start engine process");

        let mut stdin = engine_process
            .stdin
            .take()
            .expect("Failed to take engine stdin");
        let mut stdout = BufReader::new(
            engine_process
                .stdout
                .take()
                .expect("Failed to take engine stdout"),
        );

        stdin
            .write_all("uci\n".as_bytes())
            .expect("Failed to write 'uci' to engine stdin");

        let is_uci_ok;
        loop {
            let mut line = String::new();
            let _ = stdout.read_line(&mut line);
            if line.starts_with("uciok") {
                is_uci_ok = true;
                break;
            }
        }
        if !is_uci_ok {
            panic!("Engine is not UCI compatible");
        }

        let mut engine = Engine {
            path: path.to_str().unwrap().to_string(),
            name: name.to_string(),
            engine_options: Vec::new(),
            child_process: engine_process,
            stdin,
            stdout,
        };

        engine.detect_engine_options();
        engine
    } //

    pub fn send_command(&mut self, command: &str) {
        self.stdin
            .write_all(command.as_bytes())
            .expect("Failed to write command to engine stdin");
        self.stdin.flush().unwrap();
    } //

    pub fn read_line(&mut self) -> Option<String> {
        let mut line = String::new();
        self.stdout.read_line(&mut line).ok()?;
        if line.is_empty() { None } else { Some(line) }
    } //

    pub fn detect_engine_options(&mut self) {
        self.send_command("uci\n");
        let mut options = vec![];
        loop {
            if let Some(str) = self.read_line() {
                println!("line: {}" , str);
                if str.starts_with("option") {
                    let args = str.split_whitespace().collect::<Vec<_>>();
                    let option_type;
                    let value;
                    let name;

                    if let Some(name_index) = args.iter().position(|w| w == &"name") {
                        name = args[name_index + 1].to_string();
                    } else {
                        continue;
                    }
                    if let Some(default_index) = args.iter().position(|w| w == &"default") {
                        value = args[default_index + 1].to_string();
                    } else {
                        continue;
                    }
                    if let Some(option_type_index) = args.iter().position(|w| w == &"type") {
                        option_type = args[option_type_index + 1].to_string();
                    } else {
                        continue;
                    }

                    match option_type.as_str() {
                        "check" => {
                            options.push(EngineOption::CHECK {
                                name,
                                value: value.parse::<bool>().unwrap(),
                            });
                        }
                        "spin" => {
                            let mut min = None;
                            let mut max = None;
                            if let Some(min_index) = args.iter().position(|w| w == &"min") {
                                min = Some(args[min_index + 1].parse::<i32>().unwrap());
                            }
                            if let Some(max_index) = args.iter().position(|w| w == &"max") {
                                max = Some(args[max_index + 1].parse::<i32>().unwrap());
                            }
                            options.push(EngineOption::SPIN {
                                name,
                                value: value.parse::<i32>().unwrap(),
                                min,
                                max
                            });
                        }
                        _ => {}
                    }
                } else if str.contains("uciok") {
                    break;
                }
            } else {
                break;
            }
        }
        self.engine_options = options;
    } //
} //

#[cfg(test)]
mod test {
    use super::*;
    use queenfish::board::bishop_magic::init_bishop_magics;
    use queenfish::board::rook_magic::init_rook_magics;

    #[test]
    fn it_works() {
        init_bishop_magics();
        init_rook_magics();

        let engine = Engine::new(
            "C:\\Learn\\LearnRust\\chess\\target\\release\\uci.exe",
            "Queenfish 2",
        );

        let engine2 = Engine::new(
            "C:\\Program Files\\stockfish\\stockfish-windows-x86-64-avx2.exe",
            "Stockfish",
        );

        let mut game = Game::new(engine , engine2, TimeControl::INFINITE);
        // dbg!(game.play());

    }
} //
