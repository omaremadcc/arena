use gpui::{
    Div, ElementId,
    FontWeight, SharedString, Stateful,
    div, img, prelude::*, px, rgb,
};

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