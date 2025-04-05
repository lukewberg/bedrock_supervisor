use crossterm::event::Event;
use ratatui::Frame;

use crate::state::app_state::AppState;

pub mod backup_view;
pub mod home_view;
pub mod root_view;

pub trait View {
    fn render(state: &mut AppState, frame: &mut Frame, event: Event);
}
