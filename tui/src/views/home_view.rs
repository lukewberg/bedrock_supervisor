use crossterm::event::Event;
use ratatui::{Frame, widgets::Tabs};

use crate::{
    layout::home_layout,
    state::{app_state::AppState, home_state::HomeState},
};

use super::View;

pub struct HomeView {}

impl View for HomeView {
    fn render(state: &mut AppState, frame: &mut Frame, event: Event) {
        match event {
            Event::FocusGained => {}
            Event::FocusLost => {}
            Event::Key(key_event) => {
                state.should_quit = true;
            }
            Event::Mouse(mouse_event) => {}
            Event::Paste(_) => {}
            Event::Resize(_, _) => {}
        }
        // Get home layout
        let layout = home_layout::get_layout().split(frame.area());

        // Top bar
        let top_nav = Tabs::new(vec!["Home", "Backups", "Server I/O"]);
        frame.render_widget(top_nav, layout[0]);
    }
}
