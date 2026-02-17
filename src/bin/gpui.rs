use arena::Score;
use arena::gui::constants::{
    BLACK_BISHOP, BLACK_KING, BLACK_KNIGHT, BLACK_PAWN, BLACK_QUEEN, BLACK_ROOK, WHITE_BISHOP,
    WHITE_KING, WHITE_KNIGHT, WHITE_PAWN, WHITE_QUEEN, WHITE_ROOK,
};
use arena::gui::input::{
    Backspace, Copy, Cut, Delete, End, Home, InputController, InputField, Left, Paste, Right,
    SelectAll, SelectLeft, SelectRight, ShowCharacterPalette,
};
use arena::{AnalysisLine, Engine, EngineOption, gui};
use gpui::{
    App, Application, AsyncApp, Bounds, Context, Corner, Div, ElementId, Entity, Focusable,
    FontWeight, Global, KeyBinding, MouseButton, SharedString, Stateful, TitlebarOptions, Window,
    WindowBounds, WindowOptions, anchored, deferred, div, img, prelude::*, px, rgb, size,
};
use queenfish::board::Move;
use queenfish::board::bishop_magic::init_bishop_magics;
use queenfish::board::rook_magic::init_rook_magics;
use queenfish::board::{Board as QueenFishBoard, UnMakeMove};
use rfd::FileDialog;
use std::{collections::HashSet, path::Path};

pub struct EnginesServices {
    engines: Vec<Engine>,
    is_analyzing: bool,
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
    fen_string: Option<SharedString>,
    engines: EnginesServices,
}
impl Global for SharedState {}

struct EngineOptionsWindow {
    engine_index: usize,
} //

impl Render for EngineOptionsWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let engine = &mut cx.global_mut::<SharedState>().engines.engines[self.engine_index];
        let engine_options = engine.engine_options.clone();
        let engine_is_show = engine.is_show;

        let options = engine_options
            .iter()
            .enumerate()
            .map(|(index, option)| match option {
                EngineOption::CHECK { name, value } => {
                    let value = *value;
                    let name = name.clone();
                    div()
                        .flex()
                        .gap_2()
                        .items_center()
                        .child(name.clone())
                        .child(check_box(value).on_any_mouse_down(cx.listener(
                            move |engine_options_window, _, _, cx| {
                                let state: &mut SharedState = cx.global_mut::<SharedState>();
                                let engine =
                                    &mut state.engines.engines[engine_options_window.engine_index];

                                let (name, new_value) = {
                                    let option = &mut engine.engine_options[index];

                                    match option {
                                        EngineOption::CHECK { value, name } => {
                                            *value = !*value;
                                            (name.clone(), *value)
                                        }
                                        _ => return,
                                    }
                                }; // ← option borrow ends here
                                let _ = engine.send_command(
                                    format!("setoption name {} value {}\n", name, new_value)
                                        .as_str(),
                                );
                                cx.notify();
                            },
                        )))
                }
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
                    .my_2()
                    .text_base()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child("Show Analysis")
                    .child(check_box(engine_is_show))
                    .on_any_mouse_down(cx.listener(|engine_options_window, _, _, cx| {
                        let engine = &mut cx.global_mut::<SharedState>().engines.engines
                            [engine_options_window.engine_index];
                        engine.is_show = !engine.is_show;
                        cx.notify();
                    })),
            )
            .child(
                div()
                    .px_2()
                    .text_base()
                    .font_weight(FontWeight::NORMAL)
                    .text_color(rgb(gui::colors::TEXT))
                    .children(options),
            )
            .child(div().my_2().flex().w_auto().text_xs().child(
                button("Remove Engine").on_any_mouse_down(cx.listener(
                    |engine_options_window, _, window, cx| {
                        window.remove_window();
                        cx.global_mut::<SharedState>()
                            .engines
                            .engines
                            .remove(engine_options_window.engine_index);
                        cx.notify();
                    },
                )),
            ))
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
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .size_full()
            // .py_8()
            .child(format!("Enter FEN:"))
            .child(div().child(self.input_controller.clone()).w_full())
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
    is_analyzing: bool,
    selected_square: Option<u8>,
    unmake_move_history: Vec<UnMakeMove>,
    make_move_history: Vec<Move>,
    current_move_index: usize,
    is_engines_menu_open: bool,
    is_board_flipped: bool,
}

impl Focusable for Board {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Board {
    pub fn select_square(&mut self, square: u8) {
        match self.board.game_result() {
            queenfish::board::GameResult::InProgress => {}
            _ => {
                self.available_moves = Vec::new();
                return;
            }
        }

        let moves = self.board.generate_moves();
        let available_squares = self
            .available_moves
            .iter()
            .map(|mv| mv.1)
            .collect::<Vec<_>>();
        if available_squares.contains(&square) {
            self.selected_square = None;

            match self.board.game_result() {
                queenfish::board::GameResult::InProgress => {}
                _ => {
                    self.available_moves = Vec::new();
                    return;
                }
            }

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

    pub fn new(focus_handle: gpui::FocusHandle) -> Self {
        let board = QueenFishBoard::new();

        let element = Board {
            board,
            focus_handle,
            available_moves: Vec::new(),
            // analysis: Vec::new(),
            // engine_handle: Some(engine_handle),
            is_analyzing: false,
            selected_square: None,
            unmake_move_history: Vec::new(),
            make_move_history: Vec::new(),
            current_move_index: 0,
            is_engines_menu_open: false,
            is_board_flipped: false,
        };

        return element;
    } //

    pub fn reset_board(&mut self) {
        self.board = QueenFishBoard::new();
        self.available_moves = Vec::new();
        self.current_move_index = 0;
        self.make_move_history = Vec::new();
        self.unmake_move_history = Vec::new();
        self.is_analyzing = false;
    } //

    pub fn load_from_fen(&mut self, fen: String) {
        self.board.load_from_fen(fen.as_str());
    } //

    pub fn play_move(&mut self, mv: String) {
        if self.current_move_index != self.make_move_history.len() {
            self.make_move_history.truncate(self.current_move_index);
            self.unmake_move_history.truncate(self.current_move_index);
        }

        self.is_analyzing = false;
        let mv = Move::from_uci(mv.as_str(), &(self.board));
        let unmakemove = self.board.make_move(mv);
        self.make_move_history.push(mv);
        self.unmake_move_history.push(unmakemove);
        self.current_move_index += 1;
    } //

    pub fn move_forward(&mut self) {
        if self.current_move_index as i32 > (self.make_move_history.len() as i32) - 1 {
            return;
        }
        self.is_analyzing = false;
        let mv = self.make_move_history[self.current_move_index];
        self.board.make_move(mv);
        self.current_move_index += 1;
    } //

    pub fn undo_move(&mut self) {
        if self.current_move_index <= 0 {
            return;
        }
        let current_move_index = self.current_move_index - 1;
        self.is_analyzing = false;
        let unmake = self.unmake_move_history[current_move_index];
        self.board.unmake_move(unmake);
        self.current_move_index -= 1;
    } //

    pub fn flip_board_visually(&mut self) {
        self.is_board_flipped = !self.is_board_flipped;
    } //
}

impl Render for Board {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let global = cx.global_mut::<SharedState>();
        if let Some(fen) = global.fen_string.clone() {
            self.load_from_fen(fen.to_string());
            global.fen_string = None;
        }
        global.engines.poll_engines();

        let analysis = global
            .engines
            .engines
            .iter()
            .filter(|engine| engine.is_show)
            .map(|engine| {
                return div()
                    .id(ElementId::named_usize(engine.name.clone(), 0))
                    .overflow_y_scroll()
                    .w_full()
                    .flex_1()
                    .min_h_0()
                    .bg(rgb(gui::colors::SECONDARY_BACKGROUND))
                    .rounded_sm()
                    .py_1()
                    .px_4()
                    .text_color(gpui::white())
                    .child(div().child(engine.name.clone()))
                    .child(seperator(gui::colors::MUTED))
                    .child(
                        div()
                            .px_4()
                            .child(
                                div().child(
                                    div().flex().children(
                                        [
                                            ("Depth", 50),
                                            ("Score", 100),
                                            ("Nodes", 80),
                                            ("Time", 80),
                                            ("Best Move", 100),
                                        ]
                                        .iter()
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
                                                .child(x.0)
                                                .text_color(gpui::white())
                                                .border_r_1()
                                                .border_color(rgb(gui::colors::MUTED))
                                        }),
                                    ),
                                ),
                            )
                            // .when(!self.is_analyzing, |this| this.hidden())
                            .children(engine.analysis.iter().rev().map(|x| match x {
                                AnalysisLine::Move(m) => {
                                    return div()
                                        .flex()
                                        .flex_row()
                                        .gap_2()
                                        .items_center()
                                        .child(format!("Best Move: {}", m))
                                        .text_color(gpui::white());
                                }
                                AnalysisLine::Depth {
                                    depth,
                                    score,
                                    best_move,
                                    nodes,
                                    selective_depth,
                                    time,
                                } => {
                                    let score_text: Option<String> = match score {
                                        Some(Score::Cp(cp)) => Some(format!("{} cp", cp)),
                                        Some(Score::Mate(m)) => Some(format!("Mate in {}", m)),
                                        None => None,
                                    };
                                    div().child(
                                        div().flex().children(
                                            [
                                                (depth, 50),
                                                (&score_text, 100),
                                                (nodes, 80),
                                                (time, 80),
                                                (selective_depth, 20),
                                                (&best_move, 100),
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
                                                    .border_color(rgb(gui::colors::MUTED))
                                            }),
                                        ),
                                    )
                                }
                            })),
                    );
            })
            .collect::<Vec<_>>();

        let losing_tag_index: Option<usize>;
        let winning_tag_index: Option<usize>;
        let draw_tag_index: Option<(usize, usize)>;

        let game_result = self.board.game_result();
        match game_result {
            queenfish::board::GameResult::InProgress => {
                losing_tag_index = None;
                winning_tag_index = None;
                draw_tag_index = None;
            }
            queenfish::board::GameResult::WhiteWin => {
                losing_tag_index = Some(self.board.black_king_sq());
                winning_tag_index = Some(self.board.white_king_sq());
                draw_tag_index = None;
            }
            queenfish::board::GameResult::BlackWin => {
                losing_tag_index = Some(self.board.white_king_sq());
                winning_tag_index = Some(self.board.black_king_sq());
                draw_tag_index = None;
            }
            queenfish::board::GameResult::Draw(_) => {
                losing_tag_index = None;
                winning_tag_index = None;
                draw_tag_index = Some((self.board.white_king_sq(), self.board.black_king_sq()));
            }
        }

        let mut squares = (0..64)
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

                if let Some(index) = winning_tag_index {
                    if index == i {
                        element = element.child(deferred(
                            div()
                                .absolute()
                                .right_neg_1_6()
                                .top_neg_1_6()
                                .flex()
                                .items_center()
                                .justify_center()
                                .bg(rgb(gui::colors::SUCCESS))
                                .rounded_full()
                                .w_1_2() // Adjust size as needed
                                .h_1_2()
                                .child(img(Path::new("svg/crown.svg")).size_full()),
                        ))
                    }
                }

                if let Some(index) = losing_tag_index {
                    if index == i {
                        element = element.child(deferred(
                            div()
                                .absolute()
                                .right_neg_1_6()
                                .top_neg_1_6()
                                .flex()
                                .items_center()
                                .justify_center()
                                .bg(rgb(gui::colors::ERROR))
                                .rounded_full()
                                .w_1_2() // Adjust size as needed
                                .h_1_2()
                                .child(img(Path::new("svg/forfeit.svg")).size_full()),
                        ))
                    }
                }

                if let Some((white_index, black_index)) = draw_tag_index {
                    if white_index == i || black_index == i {
                        element = element.child(deferred(
                            div()
                                .absolute()
                                .right_neg_1_6()
                                .top_neg_1_6()
                                .flex()
                                .items_center()
                                .justify_center()
                                .bg(rgb(gui::colors::MUTED))
                                .rounded_full()
                                .w_1_2() // Adjust size as needed
                                .h_1_2()
                                .child(img(Path::new("svg/half.svg")).size_full()),
                        ))
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
        if self.is_board_flipped {
            squares.reverse();
        }

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
                    .child(
                        menu_button("Engines")
                            .on_mouse_down(
                                gpui::MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    this.is_engines_menu_open = true;
                                    cx.notify();
                                }),
                            )
                            .when(self.is_engines_menu_open, |this| {
                                let children = cx
                                    .global::<SharedState>()
                                    .engines
                                    .engines
                                    .iter()
                                    .enumerate()
                                    .map(|(index, engine)| {
                                        return div()
                                            .child(engine.name.clone())
                                            .py_0p5()
                                            .px_1()
                                            .cursor_pointer()
                                            .hover(|this| this.bg(rgb(gui::colors::MUTED)))
                                            .on_any_mouse_down(cx.listener(move |_, _, _, cx| {
                                                let bounds = Bounds::centered(
                                                    None,
                                                    size(px(300.), px(400.)),
                                                    cx,
                                                );
                                                let options = WindowOptions {
                                                    window_bounds: Some(WindowBounds::Windowed(
                                                        bounds,
                                                    )),
                                                    ..Default::default()
                                                };

                                                let window = cx
                                                    .open_window(options, |_, cx| {
                                                        cx.new(|_| EngineOptionsWindow {
                                                            engine_index: index,
                                                        })
                                                    })
                                                    .unwrap();
                                                window.update(cx, |_, _, cx| cx.entity()).unwrap();
                                            }));
                                    })
                                    .collect::<Vec<_>>();

                                return this.child(
                                    deferred(
                                        anchored()
                                            .anchor(Corner::BottomLeft)
                                            .snap_to_window_with_margin(px(8.))
                                            .child(
                                                div()
                                                    .children(children)
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .items_center()
                                                            .justify_center()
                                                            .gap_0p5()
                                                            .text_xs()
                                                            // .child(img(Path::new("svg/add.svg")).size_3())
                                                            .child("+ Add Engine")
                                                            .py_0p5()
                                                            .px_1()
                                                            .cursor_pointer()
                                                            .hover(|this| {
                                                                this.bg(rgb(gui::colors::MUTED))
                                                            })
                                                            .on_mouse_down(
                                                                MouseButton::Left,
                                                                cx.listener(|_, _, _, cx| {
                                                                    let task = cx.spawn(async move |_, cx: &mut AsyncApp| {
                                                                        let file_path = FileDialog::new()
                                                                            .add_filter(
                                                                                "Engines",
                                                                                &["exe"],
                                                                            )
                                                                            .pick_file();
                                                                        if let Some(file_path) = file_path {
                                                                            let _ = cx.update(move |cx| {
                                                                                let new_engine = Engine::new(
                                                                                    file_path.to_str().unwrap(),
                                                                                    file_path.file_name().unwrap().to_str().unwrap(),
                                                                                );
                                                                                cx.global_mut::<SharedState>().engines.engines.push(new_engine);
                                                                            });
                                                                        }
                                                                    });
                                                                    task.detach();
                                                                }),
                                                            ),
                                                    )
                                                    .text_color(rgb(gui::colors::TEXT))
                                                    .bg(rgb(gui::colors::SECONDARY_BACKGROUND))
                                                    .on_mouse_down_out(cx.listener(
                                                        |this, _, _, cx| {
                                                            this.is_engines_menu_open = false;
                                                            cx.notify();
                                                        },
                                                    )),
                                            ),
                                    )
                                    .priority(0),
                                );
                            }),
                    ),
            ) //
            .child(
                div()
                    .size_full()
                    .p_3()
                    .pt_0()
                    .flex()
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
                                logo_button("svg/brain.svg", 0.).on_any_mouse_down(cx.listener(
                                    move |board, _event, _window, cx| {
                                        // board.analyze(cx);
                                        cx.global_mut::<SharedState>()
                                            .engines
                                            .toggle_analyze(&board.board);
                                    },
                                )),
                            )
                            .child(logo_button("svg/chevron-left.svg", 8.).on_any_mouse_down(
                                cx.listener(move |board, _event, _window, _cx| {
                                    board.undo_move();
                                }),
                            ))
                            .child(logo_button("svg/chevron-right.svg", 8.).on_any_mouse_down(
                                cx.listener(move |board, _event, _window, _cx| {
                                    board.move_forward();
                                }),
                            ))
                            .child(logo_button("svg/flip.svg", 8.).on_any_mouse_down(
                                cx.listener(move |board, _event, _window, _cx| {
                                    board.flip_board_visually();
                                }),
                            )),
                    ) //
                    .child(
                        div()
                            .flex_1()
                            .w_full()
                            .mb_5()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .min_h_0()
                            .children(analysis), //
                    ), //
            ) //
    }
}

fn main() {
    init_bishop_magics();
    init_rook_magics();

    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(600.), px(600.0)), cx);

        let engines = vec![
            Engine::new(
                "C:/Program Files/stockfish/stockfish-windows-x86-64-avx2.exe",
                "Stockfish",
            ),
            Engine::new(
                "C:\\Learn\\LearnRust\\chess\\target\\release\\uci.exe",
                "Queenfish 2",
            ),
        ];
        cx.set_global(SharedState {
            fen_string: None,
            engines: EnginesServices {
                engines,
                is_analyzing: false,
            },
        });

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
        cx.on_app_quit(|cx| {
            // borrow happens here (synchronously)
            let mut engines = cx
                .global_mut::<SharedState>()
                .engines
                .engines
                .drain(..) // optional: take ownership if appropriate
                .collect::<Vec<_>>();

            Box::pin(async move {
                for e in &mut engines {
                    e.disconnect();
                }
            })
        })
        .detach();
    });
}

fn button(text: &str) -> impl IntoElement + InteractiveElement {
    div()
        .id(ElementId::Name(SharedString::new(text).clone()))
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

fn menu_button(text: &str) -> Stateful<Div> {
    div()
        .id(ElementId::Name(SharedString::new(text).clone()))
        .flex()
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
