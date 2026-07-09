use iced::widget::{button, column, row, text, text_input};
use iced::Element;

use crate::app::{AppState, Message};
use crate::widgets::output_log::output_log;

pub fn view(state: &AppState) -> Element<Message> {
    column![
        text("Delete").size(20),
        row![
            text("Key:").width(80),
            text_input("key/expr", &state.delete_key)
                .on_input(Message::DeleteKeyChanged),
        ].spacing(8).align_y(iced::Alignment::Center),
        button("Delete").on_press(Message::DeleteRun),
        output_log(&state.log_lines),
    ]
    .spacing(10)
    .into()
}
