// GUI binary entry point.
// Shared logic (action, parser, types) lives in the zsak lib crate.
// GUI-only modules (app, bridge, session, views, widgets) are declared here.

mod app;
mod bridge;
mod session;
mod views;
mod widgets;

// Re-export shared modules under `crate::` so GUI sub-modules can use
// `crate::action`, `crate::types`, `crate::gui` paths.
use zsak::action;
use zsak::types;

fn main() -> iced::Result {
    env_logger::init();
    iced::application(
        app::AppState::title,
        app::AppState::update,
        app::AppState::view,
    )
    .theme(app::AppState::theme)
    .window(iced::window::Settings {
        size: iced::Size::new(1200.0, 800.0),
        ..Default::default()
    })
    .run_with(app::AppState::new)
}
