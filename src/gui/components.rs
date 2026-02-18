use crate::gui::constants::{
    BLACK_BISHOP, BLACK_KING, BLACK_KNIGHT, BLACK_PAWN, BLACK_QUEEN, BLACK_ROOK, WHITE_BISHOP,
    WHITE_KING, WHITE_KNIGHT, WHITE_PAWN, WHITE_QUEEN, WHITE_ROOK,
};
use gpui::{Div, ElementId, FontWeight, SharedString, Stateful, div, img, prelude::*, px, rgb, deferred};
use queenfish::board::pieces::PieceType;

use std::path::Path;

pub fn button(text: &str) -> impl IntoElement + InteractiveElement {
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

pub fn menu_button(text: &str) -> Stateful<Div> {
    div()
        .id(ElementId::Name(SharedString::new(text).clone()))
        .flex()
        .px(px(2.))
        .hover(|this| this.bg(gpui::white()))
        .font_weight(FontWeight::MEDIUM)
        .text_xs()
        .border(px(1.))
        .border_color(gpui::black())
        .bg(rgb(super::colors::TEXT))
        .text_color(rgb(super::colors::BACKGROUND))
        .cursor_pointer()
        .child(text.to_string())
} //

pub fn seperator(color: u32) -> impl IntoElement + InteractiveElement {
    div().w_full().h(px(1.)).bg(rgb(color))
} //

pub fn logo_button(path: &str, padding: f32) -> impl IntoElement + InteractiveElement {
    div()
        .size(px(30.))
        .rounded_sm()
        .bg(rgb(super::colors::TEXT))
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

pub fn check_box(state: bool) -> impl IntoElement + InteractiveElement {
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

pub fn board_square(
    i: usize,
    selected_square: Option<usize>,
    piece: Option<PieceType>,
    is_board_flipped: bool,
    is_king_in_check: bool,
    current_turn_king_sq: usize,
    available_moves: &Vec<(usize, usize)>,
    winning_tag_index: Option<usize>,
    losing_tag_index: Option<usize>,
    draw_tag_index: Option<(usize, usize)>,
) -> impl IntoElement + InteractiveElement {
    let file = i % 8;
    let rank = i / 8;

    let mut color = if (file + rank) % 2 == 0 {
        super::colors::BOARD_LIGHT
    } else {
        super::colors::BOARD_DARK
    };

    if let Some(selected_square) = selected_square {
        if selected_square == i {
            color = super::colors::SQUARE_SELECTION;
        }
    }
    let mut piece_image = "";
    if let Some(piece) = piece {
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
        .p(px(2.))
        .flex()
        .items_center()
        .justify_center()
        .child(img(Path::new(piece_image)).size_full());

    if (i < 8 && !is_board_flipped) || (i > 55 && is_board_flipped) {
        element = element.child(
            div()
                .absolute()
                .right(px(3.))
                .bottom_0()
                .text_color(match color {
                    super::colors::BOARD_DARK => rgb(super::colors::BOARD_LIGHT),
                    _ => rgb(super::colors::BOARD_DARK),
                })
                .text_size(px(10.))
                .child(((b'a' + (i as u8 % 8)) as char).to_string()),
        );
    } //
    if (i % 8 == 0 && !is_board_flipped) || (i % 8 == 7 && is_board_flipped) {
        element = element.child(
            div()
                .absolute()
                .left(px(3.))
                .top_0()
                .text_color(match color {
                    super::colors::BOARD_DARK => rgb(super::colors::BOARD_LIGHT),
                    _ => rgb(super::colors::BOARD_DARK),
                })
                .text_size(px(10.))
                .child(((i / 8) + 1).to_string()),
        );
    } //
    if is_king_in_check && i == current_turn_king_sq as usize {
        element = element.child(
            div()
                .absolute()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .bg(rgb(super::colors::ERROR))
                .opacity(0.5),
        );
    } //

    if available_moves
        .iter()
        .map(|x| x.1)
        .collect::<Vec<usize>>()
        .contains(&i)
    {
        if piece.is_some() {
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
                            .bg(rgb(super::colors::SQUARE_SELECTION))
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
                    .bg(rgb(super::colors::SUCCESS))
                    .rounded_full()
                    .w_1_2() // Adjust size as needed
                    .h_1_2()
                    .child(img(Path::new("svg/crown.svg")).size_full()),
            ))
        }
    } //

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
                    .bg(rgb(super::colors::ERROR))
                    .rounded_full()
                    .w_1_2() // Adjust size as needed
                    .h_1_2()
                    .child(img(Path::new("svg/forfeit.svg")).size_full()),
            ))
        }
    } //

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
                    .bg(rgb(super::colors::MUTED))
                    .rounded_full()
                    .w_1_2() // Adjust size as needed
                    .h_1_2()
                    .child(img(Path::new("svg/half.svg")).size_full()),
            ))
        }
    } //

    // element = element.on_mouse_down(
    //     gpui::MouseButton::Left,
    //     cx.listener(move |board, _event, _window, cx| {
    //         board.select_square(i as u8);
    //         cx.notify();
    //     }),
    // );
    return element;
}
