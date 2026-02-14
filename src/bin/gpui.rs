use arena::gui::constants::{
    BLACK_BISHOP, BLACK_KING, BLACK_KNIGHT, BLACK_PAWN, BLACK_QUEEN, BLACK_ROOK, WHITE_BISHOP,
    WHITE_KING, WHITE_KNIGHT, WHITE_PAWN, WHITE_QUEEN, WHITE_ROOK,
};
use arena::gui::input::{
    Backspace, Copy, Cut, Delete, End, Home, InputController, InputField, Left, Paste, Right,
    SelectAll, SelectLeft, SelectRight, ShowCharacterPalette,
};
use arena::{Engine, EngineHandle, EngineOption, gui};
use gpui::{
    App, Application, Bounds, Context, Entity, Focusable, FontWeight, Global, KeyBinding,
    SharedString, TitlebarOptions, Window, WindowBounds, WindowOptions, div, img, prelude::*, px,
    rgb, size,
};
use queenfish::board::Move;
use queenfish::board::bishop_magic::init_bishop_magics;
use queenfish::board::rook_magic::init_rook_magics;
use queenfish::board::{Board as QueenFishBoard, UnMakeMove};
use std::sync::mpsc::Sender;
use std::{collections::HashSet, path::Path};

enum AnalysisLine {
    Move(String),
    Depth {
        depth: Option<String>,
        selective_depth: Option<String>,
        score: Option<String>,
        best_move: Option<String>,
        nodes: Option<String>,
        time: Option<String>,
    },
}
impl AnalysisLine {
    fn new(line: String) -> Option<AnalysisLine> {
        let line = line.trim().replace("\n", "");
        let args = line.split_whitespace().collect::<Vec<_>>();
        if line.starts_with("bestmove") {
            return Some(AnalysisLine::Move(args[1].to_string()));
        } else if line.starts_with("info") {
            let mut depth = None;
            let mut nodes = None;
            let mut best_move = None;
            let mut time = None;
            let mut score = None;

            let depth_index = args.iter().position(|str| str == &"depth");
            if let Some(depth_index) = depth_index {
                if let Some(depth_str) = args.get(depth_index + 1) {
                    depth = Some(depth_str.to_string());
                }
            }
            let score_index = args.iter().position(|str| str == &"cp" || str == &"mate");
            if let Some(score_index) = score_index {
                if let Some(score_str) = args.get(score_index + 1) {
                    score = Some(score_str.to_string());
                }
            }
            let nodes_index = args.iter().position(|str| str == &"nodes");
            if let Some(nodes_index) = nodes_index {
                if let Some(nodes_str) = args.get(nodes_index + 1) {
                    nodes = Some(nodes_str.to_string());
                }
            }
            let best_move_index = args.iter().position(|str| str == &"pv");
            if let Some(best_move_index) = best_move_index {
                if let Some(best_move_str) = args.get(best_move_index + 1) {
                    best_move = Some(best_move_str.to_string());
                }
            }
            let time_index = args.iter().position(|str| str == &"time");
            if let Some(time_index) = time_index {
                if let Some(time_str) = args.get(time_index + 1) {
                    time = Some(time_str.to_string());
                }
            }

            return Some(AnalysisLine::Depth {
                depth: depth,
                selective_depth: None,
                score,
                best_move: best_move,
                nodes,
                time,
            });
        }
        None
    }
}

pub struct SharedState {
    fen_string: Option<SharedString>,
}
impl Global for SharedState {}

struct EngineOptionsWindow {
    engine_tx: Sender<String>,
    focus_handle: gpui::FocusHandle,
    options: Vec<EngineOption>,
}

impl Render for EngineOptionsWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {

        let options = self.options.iter().map(|option| match option {
            EngineOption::CHECK { name, value } => {
                let value = *value;
                let name = name.clone();
            div()
                .flex()
                .gap_2()
                .items_center()
                .child(name.clone())
                .child(
                    check_box(value)
                        .on_any_mouse_down(cx.listener(move |engine_options_window, _, _, cx| {
                            let engine_tx = engine_options_window.engine_tx.clone();
                            if let Some(option) = engine_options_window.options.iter_mut().find(|o| match o {EngineOption::CHECK { name: name_inner, .. } => *name_inner == name, _ => false}) {
                                if let EngineOption::CHECK { value, .. } = option {
                                    *value = !*value;
                                }
                            }
                            let _ = engine_tx.send(format!("setoption name {} value {}\n", name.clone(), !(value)));
                            cx.notify();
                        })),
                )
            },
            EngineOption::SPIN {
                name,
                value,
                min,
                max,
            } => div().child(format!(
                "{}: {} ({}/{})",
                name,
                value,
                min.unwrap_or(0),
                max.unwrap_or(0)
            )),
        });
        div()
            .id("engine_options_window")
            .overflow_y_scroll()
            .size_full()
            .bg(rgb(gui::colors::BACKGROUND))
            .text_color(gpui::white())
            .text_2xl()
            .font_weight(FontWeight::BOLD)
            .flex_col()
            .items_center()
            .justify_center()
            .py_8()
            .px_6()
            .child(format!("Engine Options:"))
            .child(
                div()
                    .px_2()
                    .text_base()
                    .font_weight(FontWeight::NORMAL)
                    .text_color(rgb(gui::colors::TEXT))
                    .children(options),
            )
    }
}

struct FenWindow {
    input_controller: Entity<InputController>,
    focus_handle: gpui::FocusHandle,
}
impl Focusable for FenWindow {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for FenWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .bg(rgb(gui::colors::BACKGROUND))
            .text_color(rgb(gui::colors::TEXT))
            .flex_col()
            .items_center()
            .justify_center()
            .py_8()
            .child(format!("Enter FEN:"))
            .child(self.input_controller.clone())
            .child(
                button("Load").on_any_mouse_down(cx.listener(|this, _, _, cx| {
                    let input_controller = this.input_controller.clone().read(cx);
                    let input_field = input_controller.text_input.clone().read(cx);
                    let content = input_field.content.as_str().to_string();
                    cx.global_mut::<SharedState>().fen_string =
                        Some(SharedString::from(content.clone()));
                    cx.notify();
                })),
            )
    }
}

struct Board {
    board: QueenFishBoard,
    focus_handle: gpui::FocusHandle,
    available_moves: Vec<(u8, u8)>,
    analysis: Vec<AnalysisLine>,
    engine_handle: Option<EngineHandle>,
    is_analyzing: bool,
    selected_square: Option<u8>,
    unmake_move_history: Vec<UnMakeMove>,
    make_move_history: Vec<Move>,
    current_move_index: usize,
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
            self.selected_square = None;
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
            self.play_move(mv.to_uci());
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
            if self.board.piece_at[square as usize].is_some() {
                self.selected_square = Some(square);
            } else {
                self.selected_square = None;
            }
        }
    } //

    pub fn analyze(&mut self, cx: &mut Context<Self>) {
        let Some(handle) = self.engine_handle.as_mut() else {
            return;
        };

        if self.is_analyzing {
            handle.send_command("stop\n");
            self.analysis.clear();
            self.is_analyzing = false;
        } else {
            handle.send_command("stop\n");
            self.analysis.clear();
            self.is_analyzing = true;
            let command = format!("position fen {} 0 1\n", self.board.to_fen());
            handle.send_command(&command);
            handle.send_command("go\n");
        }

        cx.notify();
    } //

    pub fn new(focus_handle: gpui::FocusHandle) -> Self {
        let board = QueenFishBoard::new();

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
            selected_square: None,
            unmake_move_history: Vec::new(),
            make_move_history: Vec::new(),
            current_move_index: 0,
        };

        return element;
    } //

    pub fn poll_engine(&mut self, cx: &mut Context<Self>) {
        if let Some(handle) = self.engine_handle.as_mut() {
            while let Some(line) = handle.try_read_line() {
                if let Some(analysis) = AnalysisLine::new(line) {
                    self.analysis.push(analysis);
                    cx.notify();
                }
            }
        }
    } //

    pub fn reset_board(&mut self) {
        self.board = QueenFishBoard::new();
        self.available_moves = Vec::new();
        self.current_move_index = 0;
        self.make_move_history = Vec::new();
        self.unmake_move_history = Vec::new();
        if let Some(handle) = self.engine_handle.as_mut() {
            handle.send_command("stop\n");
        }
        self.analysis.clear();
        self.is_analyzing = false;
    } //

    pub fn load_from_fen(&mut self, fen: String) {
        self.board.load_from_fen(fen.as_str());
    } //

    pub fn play_move(&mut self, mv: String) {
        if self.current_move_index != self.make_move_history.len() {
            println!("Truncating move history");
            self.make_move_history.truncate(self.current_move_index);
            self.unmake_move_history.truncate(self.current_move_index);
        }

        self.is_analyzing = false;
        self.analysis.clear();
        self.engine_handle.as_mut().unwrap().send_command("stop\n");
        let mv = Move::from_uci(mv.as_str(), &(self.board));
        let unmakemove = self.board.make_move(mv);
        self.make_move_history.push(mv);
        self.unmake_move_history.push(unmakemove);
        self.current_move_index += 1;
    } //

    pub fn move_forward(&mut self) {
        println!("move forward {}", self.current_move_index);
        println!(
            "{:?}",
            self.make_move_history
                .iter()
                .map(|mv| mv.to_uci())
                .collect::<Vec<String>>()
        );
        if self.current_move_index as i32 > (self.make_move_history.len() as i32) - 1 {
            return;
        }
        self.is_analyzing = false;
        self.analysis.clear();
        self.engine_handle.as_mut().unwrap().send_command("stop\n");
        let mv = self.make_move_history[self.current_move_index];
        self.board.make_move(mv);
        self.current_move_index += 1;
    } //

    pub fn undo_move(&mut self) {
        println!("undo move {}", self.current_move_index);
        if self.current_move_index <= 0 {
            return;
        }
        let current_move_index = self.current_move_index - 1;
        self.is_analyzing = false;
        self.analysis.clear();
        self.engine_handle.as_mut().unwrap().send_command("stop\n");
        let unmake = self.unmake_move_history[current_move_index];
        self.board.unmake_move(unmake);
        self.current_move_index -= 1;
    } //
}

impl Render for Board {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(fen) = cx.global::<SharedState>().fen_string.clone() {
            self.load_from_fen(fen.to_string());
            cx.global_mut::<SharedState>().fen_string = None;
            cx.notify();
        }

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

                let mut color = if (file + rank) % 2 == 0 {
                    gui::colors::BOARD_LIGHT
                } else {
                    gui::colors::BOARD_DARK
                };

                if let Some(selected_square) = self.selected_square {
                    if selected_square == i as u8 {
                        color = gui::colors::SQUARE_SELECTION;
                    }
                }

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
                    .bg(rgb(color))
                    .p_0p5()
                    .flex()
                    .items_center()
                    .justify_center()
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
                                        .bg(rgb(gui::colors::SQUARE_SELECTION))
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

        let window_bounds = _window.window_bounds().get_bounds().size;
        let window_width = window_bounds.width;
        let window_height = window_bounds.height;
        let board_size = window_width.min(window_height) * 0.5;

        div()
            .id("board")
            .bg(rgb(gui::colors::BACKGROUND))
            .size_full()
            .child(
                div()
                    .w_full()
                    .bg(rgb(gui::colors::SECONDARY_BACKGROUND))
                    .group("top_menu")
                    .flex()
                    .py(px(2.))
                    .px_2()
                    .child(menu_button("Reset Board").on_any_mouse_down(cx.listener(
                        |board, _, _, cx| {
                            board.reset_board();
                            cx.notify();
                        },
                    )))
                    .child(
                        menu_button("Load FEN").on_any_mouse_down(cx.listener(|_, _, _, cx| {
                            let bounds = Bounds::centered(None, size(px(500.), px(150.)), cx);
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
                                    cx.new(|cx| FenWindow {
                                        input_controller,
                                        focus_handle: cx.focus_handle(),
                                    })
                                })
                                .unwrap();

                            let view = window.update(cx, |_, _, cx| cx.entity()).unwrap();
                            cx.observe_keystrokes(move |_, ev, _, cx| {
                                view.update(cx, |view, cx| {
                                    view.input_controller
                                        .as_mut(cx)
                                        .recent_keystrokes
                                        .push(ev.keystroke.clone());
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
                        })),
                    )
                    .child(menu_button("Engine Options").on_any_mouse_down(cx.listener(
                        |board, _, _, cx| {
                            let bounds = Bounds::centered(None, size(px(300.), px(400.)), cx);
                            let options = WindowOptions {
                                window_bounds: Some(WindowBounds::Windowed(bounds)),
                                ..Default::default()
                            };
                            let engine_handle = board.engine_handle.as_mut().unwrap();
                            let engine_options = engine_handle.detect_engine_options().clone();

                            let window = cx
                                .open_window(options, |_, cx| {
                                    cx.new(|cx| EngineOptionsWindow {
                                        engine_tx: engine_handle.tx.clone(),
                                        options: engine_options,
                                        focus_handle: cx.focus_handle(),
                                    })
                                })
                                .unwrap();
                            window.update(cx, |_, _, cx| cx.entity()).unwrap();
                        },
                    ))),
            ) //
            .child(
                div()
                    .size_full()
                    .p_3()
                    .pt_0()
                    .flex()
                    .flex_grow()
                    .flex_col()
                    .gap_2()
                    .child(
                        div().flex().child(
                            div()
                                .w(board_size)
                                .h(board_size)
                                .grid()
                                .grid_cols(8)
                                .grid_rows(8)
                                .gap(px(-1.))
                                .children(squares)
                                .on_mouse_down_out(cx.listener(|board, _, _, cx| {
                                    board.selected_square = None;
                                    cx.notify();
                                })),
                        ),
                    ) //
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(
                                logo_button(
                                    "C:/Learn/LearnRust/Chess Arena/arena/svg/brain.svg",
                                    0.,
                                )
                                .on_any_mouse_down(cx.listener(
                                    move |board, _event, _window, cx| {
                                        board.analyze(cx);
                                    },
                                )),
                            )
                            .child(
                                logo_button(
                                    "C:/Learn/LearnRust/Chess Arena/arena/svg/chevron-left.svg",
                                    8.,
                                )
                                .on_any_mouse_down(cx.listener(
                                    move |board, _event, _window, cx| {
                                        board.undo_move();
                                    },
                                )),
                            )
                            .child(
                                logo_button(
                                    "C:/Learn/LearnRust/Chess Arena/arena/svg/chevron-right.svg",
                                    8.,
                                )
                                .on_any_mouse_down(cx.listener(
                                    move |board, _event, _window, cx| {
                                        board.move_forward();
                                    },
                                )),
                            ),
                    ) //
                    .child(
                        div()
                            .id("analysis")
                            .overflow_y_scroll()
                            .w_full()
                            .h_full()
                            .mb_3()
                            .bg(rgb(gui::colors::SECONDARY_BACKGROUND))
                            .rounded_sm()
                            .py_1()
                            .px_4()
                            .text_color(gpui::white())
                            .child(div().child(format!(
                                "{}",
                                self.engine_handle.as_ref().unwrap().engine.name
                            )))
                            .child(seperator(gui::colors::MUTED))
                            .child(
                                div()
                                    .px_4()
                                    .when(!self.is_analyzing, |this| this.hidden())
                                    .children(self.analysis.iter().rev().map(
                                        |x: &AnalysisLine| {
                                            match x {
                                                AnalysisLine::Move(m) => {
                                                    let mv = m.clone();
                                                    return div()
                                                        .flex()
                                                        .flex_row()
                                                        .gap_2()
                                                        .items_center()
                                                        .child(format!("Best Move: {}", m))
                                                        .child(
                                                            button("Play This Move")
                                                                .on_any_mouse_down(cx.listener(
                                                                    move |board, _, _, cx| {
                                                                        board.play_move(
                                                                            mv.split(" ")
                                                                                .collect::<Vec<_>>(
                                                                                )[0]
                                                                            .to_string(),
                                                                        );
                                                                        cx.notify();
                                                                    },
                                                                )),
                                                        )
                                                        .text_color(gpui::white());
                                                }
                                                AnalysisLine::Depth {
                                                    depth,
                                                    score,
                                                    best_move: _best_move,
                                                    nodes,
                                                    selective_depth,
                                                    time,
                                                } => div().child(
                                                    div().flex().children(
                                                        [
                                                            (depth, 30),
                                                            (score, 50),
                                                            (nodes, 80),
                                                            (time, 80),
                                                            (selective_depth, 20),
                                                        ]
                                                        .iter()
                                                        .filter(|x| x.0.is_some())
                                                        .map(|x| {
                                                            div()
                                                                .flex()
                                                                .flex_row()
                                                                .gap_2()
                                                                .items_center()
                                                                .w(px(x.1 as f32))
                                                                .px_2()
                                                                .flex()
                                                                .items_center()
                                                                .justify_center()
                                                                .child(x.0.clone().unwrap())
                                                                .text_color(gpui::white())
                                                                .border_r_1()
                                                                .border_color(rgb(
                                                                    gui::colors::MUTED,
                                                                ))
                                                        }),
                                                    ),
                                                ), // .child(seperator(gui::colors::BACKGROUND)),
                                            }
                                        },
                                    )),
                            ),
                    ), //
            ) //
    }
}

fn main() {
    init_bishop_magics();
    init_rook_magics();

    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);

        cx.set_global(SharedState { fen_string: None });

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

fn button(text: &str) -> impl IntoElement + InteractiveElement {
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
} //

fn menu_button(text: &str) -> impl IntoElement + InteractiveElement {
    div()
        .flex_none()
        .px(px(2.))
        .hover(|this| this.bg(gpui::white()))
        .font_weight(FontWeight::MEDIUM)
        .text_xs()
        .border(px(1.))
        .border_color(gpui::black())
        .bg(rgb(gui::colors::TEXT))
        .text_color(rgb(gui::colors::BACKGROUND))
        .cursor_pointer()
        .child(text.to_string())
} //

fn seperator(color: u32) -> impl IntoElement + InteractiveElement {
    div().w_full().h(px(1.)).bg(rgb(color))
} //

fn logo_button(path: &str, padding: f32) -> impl IntoElement + InteractiveElement {
    div()
        .size(px(30.))
        .rounded_sm()
        .bg(rgb(gui::colors::TEXT))
        .flex()
        .gap_2()
        .items_center()
        .justify_between()
        .p(px(padding))
        .child(img(Path::new(path)).size_full())
        .hover(|this| this.bg(gpui::white()))
        .cursor_pointer()
        .text_color(gpui::black())
} //

fn check_box(state: bool) -> impl IntoElement + InteractiveElement {
    div()
        .w(px(12.))
        .h(px(12.))
        .flex_none()
        .bg(rgb(0xf7f7f7))
        .text_color(gpui::black())
        .border_1()
        .border_color(rgb(0xe0e0e0))
        .rounded_sm()
        .cursor_pointer()
        .when(state, |this| this.bg(rgb(0x3b82f6)))
} //
