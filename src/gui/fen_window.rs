use gpui::{Entity, Focusable, App, Render, Window, Context, IntoElement, rgb ,div, SharedString, prelude::*};
use crate::gui::input::{InputController};
use crate::gui::state::SharedState;
use super::components::button;



pub struct FenWindow {
    pub input_controller: Entity<InputController>,
    pub focus_handle: gpui::FocusHandle,
}
impl FenWindow {
    pub fn new(input_controller: Entity<InputController>, cx: &mut Context<SharedState>) -> Self {
        let focus_handle = cx.focus_handle();
        FenWindow {
            input_controller,
            focus_handle,
        }
    }
}

impl Focusable for FenWindow {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for FenWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .bg(rgb(super::colors::BACKGROUND))
            .text_color(rgb(super::colors::TEXT))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .size_full()
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