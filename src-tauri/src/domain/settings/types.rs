use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub show_system_ports: bool,
    pub refresh_interval_ms: u64,
    pub theme: Theme,
    pub launch_at_startup: bool,
    pub confirm_before_kill: bool,
    pub minimize_on_blur: bool,
    pub default_open_protocol: OpenProtocol,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_system_ports: false,
            refresh_interval_ms: 2_000,
            theme: Theme::System,
            launch_at_startup: false,
            confirm_before_kill: true,
            minimize_on_blur: false,
            default_open_protocol: OpenProtocol::Http,
        }
    }
}

impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if !(500..=60_000).contains(&self.refresh_interval_ms) {
            return Err("refresh interval must be between 500 and 60000 milliseconds".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OpenProtocol {
    Http,
    Https,
}

impl OpenProtocol {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_refresh_interval_boundary() {
        let mut settings = Settings::default();
        assert!(settings.validate().is_ok());
        settings.refresh_interval_ms = 100;
        assert!(settings.validate().is_err());
    }
}
