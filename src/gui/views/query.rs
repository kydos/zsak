use iced::widget::{button, column, row, text, text_input};
use iced::Element;

use crate::app::{AppState, Message};
use crate::widgets::output_log::output_log;

pub fn view(state: &AppState) -> Element<Message> {
    column![
        text("Query").size(20),
        row![
            text("Expr:").width(80),
            text_input("demo/**", &state.query_expr)
                .on_input(Message::QueryExprChanged),
        ].spacing(8).align_y(iced::Alignment::Center),
        row![
            text("Body:").width(80),
            text_input("(optional)", &state.query_body)
                .on_input(Message::QueryBodyChanged),
        ].spacing(8).align_y(iced::Alignment::Center),
        button("Query").on_press(Message::QueryRun),
        output_log(&state.log_lines),
    ]
    .spacing(10)
    .into()
}
