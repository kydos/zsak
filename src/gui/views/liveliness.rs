use iced::widget::{button, column, pick_list, row, text, text_input};
use iced::Element;

use crate::app::{AppState, Message};
use crate::widgets::output_log::output_log;

const ACTIONS: &[&str] = &["query", "subscribe", "declare"];

pub fn view(state: &AppState) -> Element<Message> {
    let action = state.liveliness_action.as_str();

    let run_btn: Element<Message> = match action {
        "query" => button("Run Query").on_press(Message::LivelinessRun).into(),
        "subscribe" => {
            if state.liveliness_active {
                button("Stop").on_press(Message::LivelinessStop).into()
            } else {
                button("Start Subscribe").on_press(Message::LivelinessStart).into()
            }
        }
        "declare" => {
            if state.liveliness_active {
                button("Undeclare").on_press(Message::LivelinessStop).into()
            } else {
                button("Declare Token").on_press(Message::LivelinessStart).into()
            }
        }
        _ => button("Run").into(),
    };

    column![
        text("Liveliness").size(20),
        row![
            text("Action:").width(80),
            pick_list(
                ACTIONS,
                Some(action),
                |s| Message::LivelinessActionChanged(s.to_string()),
            ),
        ].spacing(8).align_y(iced::Alignment::Center),
        row![
            text("Key:").width(80),
            text_input("demo/**", &state.liveliness_key)
                .on_input(Message::LivelinessKeyChanged),
        ].spacing(8).align_y(iced::Alignment::Center),
        run_btn,
        output_log(&state.log_lines),
    ]
    .spacing(10)
    .into()
}
