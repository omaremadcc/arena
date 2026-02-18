use arena::Score;
use arena::gui::input::{
    Backspace, Copy, Cut, Delete, End, Home, InputController, InputField, Left, Paste, Right,
    SelectAll, SelectLeft, SelectRight, ShowCharacterPalette,
};
use arena::{AnalysisLine, Engine, gui};
use gpui::{
    App, Application, AsyncApp, Bounds, Context, Corner, ElementId, Focusable,
    KeyBinding, MouseButton, SharedString,  TitlebarOptions, Window,
    WindowBounds, WindowOptions, anchored, deferred, div, prelude::*, px, rgb, size,
};
use queenfish::board::Move;
use queenfish::board::bishop_magic::init_bishop_magics;
use queenfish::board::rook_magic::init_rook_magics;
use queenfish::board::{Board as QueenFishBoard, UnMakeMove};
use rfd::FileDialog;
use std::{collections::HashSet};
use arena::gui::fen_window::FenWindow;
use arena::gui::state::SharedState;
use arena::gui::components::{board_square, logo_button, menu_button, seperator};
use arena::gui::state::EnginesServices;
use arena::gui::engine_options::EngineOptionsWindow;


struct Board {
    board: QueenFishBoard,
    focus_handle: gpui::FocusHandle,
    available_moves: Vec<(usize, usize)>,
    is_analyzing: bool,
    selected_square: Option<usize>,
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
    pub fn select_square(&mut self, square: usize) {
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
                .find(|mv| (mv.from(), mv.to()) == *selected_mv)
                .unwrap();
            self.play_move(mv.to_uci());
            self.available_moves = Vec::new();
            return;
        } else {
            let avail_squares = moves
                .iter()
                .filter(|&x| x.from() == square)
                .map(|&x| (x.from(), x.to()))
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

        let is_king_in_check = self.board.is_king_in_check(self.board.turn);
        let current_turn_king_sq = match self.board.turn {
            queenfish::board::Turn::WHITE => self.board.white_king_sq(),
            queenfish::board::Turn::BLACK => self.board.black_king_sq(),
        };

        let mut squares = (0..64)
            .collect::<Vec<_>>()
            .chunks(8)
            .rev()
            .flatten()
            .copied()
            .map(|i| {
                let mut element = board_square(i, self.selected_square, self.board.piece_at[i], self.is_board_flipped, is_king_in_check, current_turn_king_sq, &self.available_moves, winning_tag_index, losing_tag_index, draw_tag_index);

                element = element.on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(move |board, _event, _window, cx| {
                        board.select_square(i);
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
} //
