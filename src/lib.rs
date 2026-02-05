use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

struct Engine {
    path: String,
    name: String,
    child_process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

struct Game {
    white: Engine,
    black: Engine,
    moves_list: Vec<String>,
}

impl Engine {
    fn new(path: &str, name: &str) -> Self {
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

        let mut is_uci_ok = false;
        loop {
            let mut line = String::new();
            stdout.read_line(&mut line);
            if line.starts_with("uciok") {
                is_uci_ok = true;
                break;
            }
        }
        if !is_uci_ok {
            panic!("Engine is not UCI compatible");
        }

        Engine {
            path: path.to_str().unwrap().to_string(),
            name: name.to_string(),
            child_process: engine_process,
            stdin,
            stdout,
        }
    } //

    fn send_command(&mut self, command: &str) {
        self.stdin
            .write_all(command.as_bytes())
            .expect("Failed to write command to engine stdin");
        self.stdin.flush().unwrap();
    }

    pub fn read_line(&mut self) -> Option<String> {
        let mut line = String::new();
        self.stdout.read_line(&mut line).ok()?;
        if line.is_empty() { None } else { Some(line) }
    }
} //

struct Tournament {
    engines: Vec<Engine>,
    name: String,
    rounds: u32,
    time_per_move: u32,
} //

mod test {
    use super::*;
    use queenfish::board::{Board, Move, Turn};
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

        let mut board = Board::new();
        let mut game = Game {
            white: engine,
            black: engine2,
            moves_list: Vec::new(),
        };
        loop {
            let valid_moves = board.generate_moves();
            if valid_moves.is_empty() {
                match board.turn {
                    Turn::WHITE => println!("{} wins as black" , game.black.name),
                    Turn::BLACK => println!("{} wins as white" , game.white.name),
                }
                println!("PGN: {}", game.moves_list.join(" "));
                break;
            }
            let engine = match board.turn {
                Turn::WHITE => &mut game.white,
                Turn::BLACK => &mut game.black,
            };
            if game.moves_list.is_empty() {
                engine
                    .send_command(format!("position startpos\n").as_str());
            } else {
                engine.send_command(
                    format!("position startpos moves {}\n", game.moves_list.join(" ")).as_str(),
                );
            }
            engine.send_command("go movetime 10\n");

            loop {
                if let Some(line) = engine.read_line() {
                    if line.starts_with("bestmove") {
                        let best_move = line.split_whitespace().nth(1).unwrap();
                        game.moves_list.push(best_move.to_string());
                        board.make_move(Move::from_uci(best_move, &board));
                        println!("Played {} , fen {}", best_move, board.to_fen());
                        break;
                    } else {
                        // println!("{}", line);
                    }
                }
            }
        }
    }
} //
