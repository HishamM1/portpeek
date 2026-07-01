use std::sync::Mutex;

use crate::domain::settings::types::Settings;

pub struct AppState {
    pub settings: Mutex<Settings>,
}

impl AppState {
    pub fn new(settings: Settings) -> Self {
        Self {
            settings: Mutex::new(settings),
        }
    }
}
