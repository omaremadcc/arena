use gpui::{Global, SharedString};
use crate::Engine;
use queenfish::board::Board as QueenFishBoard;

pub struct EnginesServices {
    pub engines: Vec<Engine>,
    pub is_analyzing: bool,
}

impl EnginesServices {
    pub fn new() -> Self {
        EnginesServices {
            engines: vec![],
            is_analyzing: false,
        }
    }
    pub fn toggle_analyze(&mut self, board: &QueenFishBoard) {
        if self.is_analyzing {
            self.is_analyzing = false;
            self.engines.iter_mut().for_each(|engine| {
                engine.send_command("stop\n");
            });
            return;
        }
        self.is_analyzing = true;
        self.engines.iter_mut().for_each(|engine| {
            engine.send_command("stop\n");
            engine.analysis.clear();
            engine.send_command(format!("position fen {} 0 1\n", board.to_fen()).as_str());
            engine.send_command("go\n")
        });
    }
    pub fn poll_engines(&mut self) {
        self.engines
            .iter_mut()
            .for_each(|engine| engine.poll_engine());
    }
}


pub struct SharedState {
    pub fen_string: Option<SharedString>,
    pub engines: EnginesServices,
}
impl Global for SharedState {}

impl SharedState {
    pub fn new() -> Self {
        SharedState {
            fen_string: None,
            engines: EnginesServices::new(),
        }
    }
}