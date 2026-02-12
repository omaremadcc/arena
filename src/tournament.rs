use crate::engine::Engine;
use crate::game::{Game, GameResult, TimeControl};

#[derive(Debug)]
pub struct TournamentResult {
    engine1: String,
    engine2: String,
    games_list: Vec<GameResult>,
    engine1_won: u64,
    engine2_won: u64,
    draws: u64,
    total_games: u64,
}
impl TournamentResult {
    pub fn default() -> Self {
        TournamentResult {
            engine1: String::new(),
            engine2: String::new(),
            games_list: Vec::new(),
            engine1_won: 0,
            engine2_won: 0,
            draws: 0,
            total_games: 0,
        }
    }
    pub fn new(
        engine1: String,
        engine2: String,
        games_list: Vec<GameResult>,
        engine1_won: u64,
        engine2_won: u64,
        draws: u64,
        total_games: u64,
    ) -> Self {
        TournamentResult {
            engine1,
            engine2,
            games_list,
            engine1_won,
            engine2_won,
            draws,
            total_games,
        }
    }
}

pub struct Tournament {
    rounds: i32,
    engine1: Engine,
    engine2: Engine,
    time_control: TimeControl,
}

impl Tournament {
    pub fn new(rounds: i32, engine1: Engine, engine2: Engine, time_control: TimeControl) -> Self {
        Tournament {
            rounds,
            engine1,
            engine2,
            time_control,
        }
    } //

    // pub fn start(&mut self) -> TournamentResult {
    //     let mut tournament_result = TournamentResult::default();
    //     tournament_result.engine1 = self.engine1.name.clone();
    //     tournament_result.engine2 = self.engine2.name.clone();
    //     for i in 0..self.rounds {
    //         let engine1 = Engine::new(&self.engine1.path, &self.engine1.name);
    //         let engine2 = Engine::new(&self.engine2.path, &self.engine2.name);
    //         let mut game;
    //         if i % 2 == 0 {
    //             game = Game::new(engine1, engine2, self.time_control);
    //         } else {
    //             game = Game::new(engine2, engine1, self.time_control);
    //         }
    //         let game_result = game.play();
    //         tournament_result.games_list.push(game_result.clone());
    //         tournament_result.total_games += 1;

    //         if game_result.winner() == self.engine1.name {
    //             tournament_result.engine1_won += 1;
    //         } else if game_result.winner() == self.engine2.name {
    //             tournament_result.engine2_won += 1;
    //         } else {
    //             tournament_result.draws += 1;
    //         }
    //     }
    //     tournament_result
    // } //
}
