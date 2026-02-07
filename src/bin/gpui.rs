use gpui::{
    App, Application, Bounds, Context, Focusable, KeyBinding, MouseDownEvent, Rgba, Window, WindowBounds, WindowOptions, actions, div, img, prelude::*, px, rgb, size
};
use std::path::Path;
use queenfish::board::Board as QueenFishBoard;

const WHITE_PAWN: &str = "C:\\Learn\\LearnRust\\Chess Arena\\arena\\pieces\\wP.svg";
const WHITE_KNIGHT: &str = "C:\\Learn\\LearnRust\\Chess Arena\\arena\\pieces\\wN.svg";
const WHITE_BISHOP: &str = "C:\\Learn\\LearnRust\\Chess Arena\\arena\\pieces\\wB.svg";
const WHITE_ROOK: &str = "C:\\Learn\\LearnRust\\Chess Arena\\arena\\pieces\\wR.svg";
const WHITE_QUEEN: &str = "C:\\Learn\\LearnRust\\Chess Arena\\arena\\pieces\\wQ.svg";
const WHITE_KING: &str = "C:\\Learn\\LearnRust\\Chess Arena\\arena\\pieces\\wK.svg";

const BLACK_PAWN: &str = "C:\\Learn\\LearnRust\\Chess Arena\\arena\\pieces\\bP.svg";
const BLACK_KNIGHT: &str = "C:\\Learn\\LearnRust\\Chess Arena\\arena\\pieces\\bN.svg";
const BLACK_BISHOP: &str = "C:\\Learn\\LearnRust\\Chess Arena\\arena\\pieces\\bB.svg";
const BLACK_ROOK: &str = "C:\\Learn\\LearnRust\\Chess Arena\\arena\\pieces\\bR.svg";
const BLACK_QUEEN: &str = "C:\\Learn\\LearnRust\\Chess Arena\\arena\\pieces\\bQ.svg";
const BLACK_KING: &str = "C:\\Learn\\LearnRust\\Chess Arena\\arena\\pieces\\bK.svg";

fn light_board_color() -> Rgba {
    rgb(0xf0d9b5)
}
fn dark_board_color() -> Rgba {
    rgb(0xb58863)
}

actions!(play, [Play]);

struct Board {
    board: QueenFishBoard,
    focus_handle: gpui::FocusHandle,
}
impl Focusable for Board {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Board {
    pub fn play(&mut self, _: &Play , _: &mut Window, cx: &mut Context<Self>) {
        let board = &mut self.board;
        let moves = board.generate_moves();
        board.make_move(moves[0]);
        cx.notify();
    }
}


impl Render for Board {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {

        let squares = (0..64).map(|i| {
            let file = i % 8;
            let rank = i / 8;

            let color = if (file + rank) % 2 == 0 {
                light_board_color()
            } else {
                dark_board_color()
            };
            let mut piece_image= "";
            if let Some(piece) = self.board.piece_at[i] {
                piece_image = match piece as usize {
                    0 => WHITE_PAWN,
                    1 => WHITE_KNIGHT,
                    2 => WHITE_BISHOP,
                    3 => WHITE_ROOK,
                    4 => WHITE_QUEEN,
                    5 => WHITE_KING,
                    6 => BLACK_PAWN,
                    7 => BLACK_KNIGHT,
                    8 => BLACK_BISHOP,
                    9 => BLACK_ROOK,
                    10 => BLACK_QUEEN,
                    11 => BLACK_KING,
                    _ => "",
                };
            }

            return div().size_full().bg(color).p_0p5().child(img(Path::new(piece_image)).size_full());
        }).rev().collect::<Vec<_>>();

        div().key_context("board").track_focus(&self.focus_handle(cx))
        .size_full().grid().grid_cols(8).grid_rows(8).children(squares).on_action(cx.listener(Self::play))
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);
        cx.bind_keys([
            KeyBinding::new("space", Play, Some("board")),
            KeyBinding::new("enter", Play, Some("board"))
        ]);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|cx| Board { board: QueenFishBoard::new(), focus_handle: cx.focus_handle() }),
        )
        .unwrap();
        cx.activate(true);
    });
}
