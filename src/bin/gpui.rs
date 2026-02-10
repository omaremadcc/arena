use arena::gpui::input::{
    Backspace, Copy, Cut, Delete, End, Home, InputController, InputField, Left, Paste, Right,
    SelectAll, SelectLeft, SelectRight, ShowCharacterPalette,
};
use arena::{Engine, EngineHandle};
use gpui::{
    App, Application, Bounds, Context, Entity, Focusable, KeyBinding, Rgba, SharedString, TitlebarOptions, Window, WindowBounds, WindowOptions, div, img, prelude::*, px, rgb, size
};
use queenfish::board::Board as QueenFishBoard;
use queenfish::board::bishop_magic::init_bishop_magics;
use queenfish::board::rook_magic::init_rook_magics;
use std::{collections::HashSet, path::Path};

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

struct FenWindow {
    input_controller: Entity<InputController>
}

impl Render for FenWindow {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .child(self.input_controller.clone())
    }
}

struct Board {
    board: QueenFishBoard,
    focus_handle: gpui::FocusHandle,
    available_moves: Vec<(u8, u8)>,
    analysis: Vec<String>,
    engine_handle: Option<EngineHandle>,
    is_analyzing: bool,
    is_load_fen_window_opened: bool,
}

impl Focusable for Board {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Board {
    pub fn select_square(&mut self, square: u8) {
        let moves = self.board.generate_moves();
        let available_squares = self
            .available_moves
            .iter()
            .map(|mv| mv.1)
            .collect::<Vec<_>>();
        if available_squares.contains(&square) {
            self.engine_handle.as_mut().unwrap().send_command("stop\n");
            self.analysis.clear();

            let selected_mv = self
                .available_moves
                .iter()
                .find(|mv| mv.1 == square)
                .unwrap();
            let mv = moves
                .iter()
                .find(|mv| (mv.from() as u8, mv.to() as u8) == *selected_mv)
                .unwrap();
            self.board.make_move(*mv);
            self.available_moves = Vec::new();
            return;
        } else {
            let avail_squares = moves
                .iter()
                .filter(|&x| x.from() == square as usize)
                .map(|&x| (x.from() as u8, x.to() as u8))
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();
            self.available_moves = avail_squares;
        }
    } //

    pub fn analyze(&mut self, cx: &mut Context<Self>) {
        let Some(handle) = self.engine_handle.as_mut() else {
            eprintln!("engine handle missing");
            return;
        };

        if self.is_analyzing {
            handle.send_command("stop\n");
            self.analysis.clear();
            self.is_analyzing = false;
        } else {
            handle.send_command("stop\n");
            self.analysis.clear();
            handle.send_command(dbg!(
                format!("position fen {} 0 1\ngo\n", self.board.to_fen()).as_str()
            ));
            self.is_analyzing = true;
        }

        cx.notify();
    } //

    pub fn new(focus_handle: gpui::FocusHandle) -> Self {
        let mut board = QueenFishBoard::new();
        board.load_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");

        let engine = Engine::new(
            "C:\\Learn\\LearnRust\\chess\\target\\release\\uci.exe",
            "Queenfish 2",
        );
        let engine_handle = engine.spawn_handle();

        let element = Board {
            board,
            focus_handle,
            available_moves: Vec::new(),
            analysis: Vec::new(),
            engine_handle: Some(engine_handle),
            is_analyzing: false,
            is_load_fen_window_opened: false,
        };

        return element;
    } //

    pub fn poll_engine(&mut self, cx: &mut Context<Self>) {
        if let Some(handle) = self.engine_handle.as_mut() {
            while let Some(line) = handle.try_read_line() {
                self.analysis.push(line);
                cx.notify();
            }
        }
    } //

    pub fn reset_board(&mut self, cx: &mut Context<Self>) {
        self.board = QueenFishBoard::new();
        self.available_moves = Vec::new();
        if let Some(handle) = self.engine_handle.as_mut() {
            handle.send_command("stop\n");
        }
        self.analysis.clear();
        self.is_analyzing = false;
        cx.notify();
    }
}

impl Render for Board {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.poll_engine(cx);

        let squares = (0..64)
            .collect::<Vec<_>>()
            .chunks(8)
            .rev()
            .flatten()
            .copied()
            .map(|i| {
                let file = i % 8;
                let rank = i / 8;

                let color = if (file + rank) % 2 == 0 {
                    light_board_color()
                } else {
                    dark_board_color()
                };
                let mut piece_image = "";
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

                let mut element = div()
                    .size_full()
                    .bg(color)
                    .p_0p5()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(40.)) // Adjust size as needed
                    .h(px(40.))
                    .child(img(Path::new(piece_image)).size_full());

                if self
                    .available_moves
                    .iter()
                    .map(|x| x.1)
                    .collect::<Vec<u8>>()
                    .contains(&(i as u8))
                {
                    if self.board.piece_at[i].is_some() {
                        element = element.child(
                            div()
                                .absolute()
                                .size_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    div()
                                        .border_4()
                                        .border_color(rgb(0xaeb187))
                                        .rounded_full()
                                        .w_full() // Adjust size as needed
                                        .h_full(),
                                ),
                        );
                    } else {
                        element = element.child(
                            div()
                                .absolute()
                                .size_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    div()
                                        .bg(rgb(0xaeb187))
                                        .rounded_full()
                                        .w_1_3() // Adjust size as needed
                                        .h_1_3(),
                                ),
                        );
                    }
                }

                element = element.on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(move |board, _event, _window, cx| {
                        board.select_square(i as u8);
                        cx.notify();
                    }),
                );
                return element;
            })
            .collect::<Vec<_>>();

        div()
            .bg(rgb(0x161512))
            .size_full()
            .p_3()
            .pt_0()
            .flex()
            .flex_grow()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .w_full()
                    .bg(gpui::green())
                    .group("top_menu")
                    .flex()
                    .gap_2()
                    .py(px(2.))
                    .px_2()
                    .child(
                        div()
                            .child(format!("Reset Board"))
                            .text_xs()
                            .text_color(gpui::white())
                            .invisible()
                            .group("top_menu")
                            .group_hover("top_menu", |el| el.visible())
                            .on_mouse_down(
                                gpui::MouseButton::Left,
                                cx.listener(move |board, _event, _window, cx| {
                                    board.reset_board(cx);
                                }),
                            ),
                    )
                    .child(
                        div()
                            .child(format!("Load FEN"))
                            .text_xs()
                            .text_color(gpui::white())
                            .invisible()
                            .group("top_menu")
                            .group_hover("top_menu", |el| el.visible())
                            .on_mouse_down(
                                gpui::MouseButton::Left,
                                cx.listener(move |board, _event, _window, cx| {
                                    board.is_load_fen_window_opened =
                                        !board.is_load_fen_window_opened;
                                    cx.notify();
                                }),
                            ),
                    )
                    .child(button("Omar", move |_, cx| {
                        let bounds = Bounds::centered(None, size(px(500.), px(100.)), cx);
                        let options = WindowOptions {
                            window_bounds: Some(WindowBounds::Windowed(bounds)),
                            ..Default::default()
                        };

                        let text_input = cx.new(|cx| InputField::new(cx));
                        let input_controller = cx.new(|cx| InputController {
                            recent_keystrokes: Vec::new(),
                            focus_handle: cx.focus_handle(),
                            text_input,
                        });

                        let window = cx
                            .open_window(options, |_, cx| {
                                cx.new(|_cx| FenWindow {
                                    input_controller
                                })
                            })
                            .unwrap();

                        let view = window.update(cx, |_, _, cx| cx.entity()).unwrap();
                        cx.observe_keystrokes(move |ev, _, cx| {
                            view.update(cx, |view, cx| {
                                view.input_controller.as_mut(cx).recent_keystrokes.push(ev.keystroke.clone());
                                cx.notify();
                            })
                        })
                        .detach();
                        cx.on_keyboard_layout_change({
                            move |cx| {
                                window.update(cx, |_, _, cx| cx.notify()).ok();
                            }
                        })
                        .detach();

                        // window
                        //     .update(cx, |view, window, cx| {
                        //         window.focus(&view.input_controller.as_mut(cx).text_input.focus_handle(cx));
                        //         cx.activate(true);
                        //     })
                        //     .unwrap();
                    })),
            )
            .child(
                div()
                    .w(px(8. * 40.))
                    .h(px(8. * 40.))
                    .grid()
                    .grid_cols(8)
                    .grid_rows(8)
                    .children(squares),
            )
            .child(
                div()
                    .h(px(10.))
                    .w_full()
                    .rounded_md()
                    .bg(rgb(0x262421))
                    .flex()
                    .flex_row()
                    .gap_2()
                    .items_center()
                    .justify_between()
                    .p_1()
                    .child(format!("Analyze board"))
                    .text_color(gpui::white())
                    .on_any_mouse_down(cx.listener(move |board, _event, _window, cx| {
                        board.analyze(cx);
                    })),
            )
            .child(
                div()
                    .id("analysis")
                    .overflow_y_scroll()
                    .w_full()
                    .h_1_3()
                    .bg(rgb(0x262421))
                    .rounded_md()
                    .p_1()
                    .gap_neg_112()
                    .child(format!("analysis"))
                    .text_color(gpui::white())
                    .children(
                        self.analysis
                            .iter()
                            .rev()
                            .map(|x| div().child(x.clone()).text_color(gpui::white()).text_sm()),
                    ),
            )
    }
}

fn main() {
    init_bishop_magics();
    init_rook_magics();

    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);

        cx.bind_keys([
            KeyBinding::new("backspace", Backspace, None),
            KeyBinding::new("delete", Delete, None),
            KeyBinding::new("left", Left, None),
            KeyBinding::new("right", Right, None),
            KeyBinding::new("shift-left", SelectLeft, None),
            KeyBinding::new("shift-right", SelectRight, None),
            KeyBinding::new("ctrl-a", SelectAll, None),
            KeyBinding::new("ctrl-v", Paste, None),
            KeyBinding::new("ctrl-c", Copy, None),
            KeyBinding::new("ctrl-x", Cut, None),
            KeyBinding::new("home", Home, None),
            KeyBinding::new("end", End, None),
            KeyBinding::new("ctrl-cmd-space", ShowCharacterPalette, None),
        ]);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some(SharedString::from("Arena")),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|cx| Board::new(cx.focus_handle())),
        )
        .unwrap();
        cx.activate(true);
    });
}

fn button(text: &str, on_click: impl Fn(&mut Window, &mut App) + 'static) -> impl IntoElement {
    div()
        .flex_none()
        .px_2()
        .bg(rgb(0xf7f7f7))
        .text_color(gpui::black())
        .border_1()
        .border_color(rgb(0xe0e0e0))
        .rounded_sm()
        .cursor_pointer()
        .child(text.to_string())
        .on_any_mouse_down(move |_, window, cx| on_click(window, cx))
}
