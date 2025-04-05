use crate::{
    state::app_state::{AppState, Screens},
    views::View,
};
use crossterm::event::Event;
use ratatui::Frame;

use super::home_view::HomeView;

pub struct RootView {}

impl View for RootView {
    fn render(state: &mut AppState, frame: &mut Frame, event: Event) {
        match &mut state.current_screen {
            Screens::HOME => {
                HomeView::render(state, frame, event);
            }
            Screens::BACKUP => {
                todo!()
            }
            Screens::SERVER => {
                todo!()
            }
        };
    }
}
