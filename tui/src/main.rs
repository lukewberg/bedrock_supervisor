pub mod layout;
pub mod state;
pub mod views;

use color_eyre::Result;
use crossterm::event::{self};
use ratatui::DefaultTerminal;
use state::app_state::AppState;
use views::View;
use views::root_view::RootView;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let mut app_state = AppState::default();
    while !app_state.should_quit {
        let event = event::read()?;
        terminal.draw(|frame| {
            RootView::render(&mut app_state, frame, event);
        })?;
        // if matches!(event::read()?, Event::Key(_)) {
        //     break Ok(());
        // }
    }
    Ok(())
}

// fn render(frame: &mut Frame) {
//     frame.render_widget("hello world", frame.area());
// }
