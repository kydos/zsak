use iced::widget::{button, column, text};
use iced::Element;

use crate::app::{AppState, Message};
use crate::widgets::output_log::output_log;

pub fn view(state: &AppState) -> Element<Message> {
    column![
        text("Scout").size(20),
        text("Scouts for Zenoh runtimes for 5 seconds.").size(13),
        button("Scout Now").on_press(Message::ScoutRun),
        output_log(&state.log_lines),
    ]
    .spacing(10)
    .into()
}
