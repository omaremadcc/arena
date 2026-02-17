use gpui::{Render, Window, Context, IntoElement, rgb ,div, prelude::*, FontWeight};
use crate::gui::state::SharedState;
use super::components::{check_box, button};
use crate::engine::EngineOption;


pub struct EngineOptionsWindow {
    pub engine_index: usize,
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
            .bg(rgb(super::colors::BACKGROUND))
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
                    .text_color(rgb(super::colors::TEXT))
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
