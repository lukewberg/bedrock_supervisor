use ratatui::layout::{Constraint, Direction, Layout};

pub fn get_layout() -> Layout {
    Layout::default().direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Ratio(1,8),
            Constraint::Fill(1),
            Constraint::Ratio(1,8),
        ])
}