use iced::widget::{column, container, scrollable, text};
use iced::{Element, Length};

/// A simple scrollable text log. Pass a slice of log lines.
pub fn output_log<'a, Msg: 'a>(lines: &'a [String]) -> Element<'a, Msg> {
    let content = column(
        lines
            .iter()
            .map(|line| text(line.as_str()).size(13).into())
            .collect::<Vec<_>>(),
    )
    .spacing(2);

    container(
        scrollable(content)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(container::dark)
    .padding(8)
    .into()
}
