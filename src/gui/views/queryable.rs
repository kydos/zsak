use iced::widget::{button, column, row, text, text_input};
use iced::Element;

use crate::app::{AppState, Message};
use crate::widgets::output_log::output_log;

pub fn view(state: &AppState) -> Element<Message> {
    let toggle_btn = if state.queryable_active {
        button("Stop").on_press(Message::QueryableStop)
    } else {
        button("Start").on_press(Message::QueryableStart)
    };

    column![
        text("Queryable").size(20),
        row![
            text("Key:").width(80),
            text_input("demo/queryable", &state.queryable_key)
                .on_input(Message::QueryableKeyChanged),
        ].spacing(8).align_y(iced::Alignment::Center),
        row![
            text("Reply:").width(80),
            text_input("pong", &state.queryable_reply)
                .on_input(Message::QueryableReplyChanged),
        ].spacing(8).align_y(iced::Alignment::Center),
        toggle_btn,
        output_log(&state.log_lines),
    ]
    .spacing(10)
    .into()
}
