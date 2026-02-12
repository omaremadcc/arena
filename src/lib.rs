pub mod engine;
pub mod game;
pub mod tournament;
pub mod gui;

pub use engine::*;
pub use game::*;
pub use tournament::*;

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

        let mut tournament = Tournament::new(5, engine , engine2 , TimeControl::TimePerMove(50));
        // let tournament_result = tournament.start();
        // dbg!(tournament_result);
        // dbg!(game.play());
    }
} //
