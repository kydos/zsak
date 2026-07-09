use iced::widget::{button, column, row, text, text_input};
use iced::Element;

use crate::app::{AppState, Message};
use crate::widgets::output_log::output_log;

pub fn view(state: &AppState) -> Element<Message> {
    let toggle_btn = if state.subscribe_active {
        button("Stop").on_press(Message::SubscribeStop)
    } else {
        button("Start").on_press(Message::SubscribeStart)
    };

    column![
        text("Subscribe").size(20),
        row![
            text("Key:").width(80),
            text_input("demo/**", &state.subscribe_key)
                .on_input(Message::SubscribeKeyChanged),
        ].spacing(8).align_y(iced::Alignment::Center),
        toggle_btn,
        output_log(&state.log_lines),
    ]
    .spacing(10)
    .into()
}
