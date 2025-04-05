use super::{backup_state::BackupState, home_state::HomeState, server_state::ServerState};

pub struct AppState {
    pub current_screen: Screens,
    pub should_quit: bool,
    pub home_state: HomeState,
    pub backup_state: BackupState,
    pub server_state: ServerState,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_screen: Screens::HOME,
            should_quit: false,
            home_state: HomeState::default(),
            backup_state: BackupState::default(),
            server_state: ServerState::default(),
        }
    }
}

pub enum Screens {
    HOME,
    BACKUP,
    SERVER,
}
