use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub show_system_ports: bool,
    pub refresh_interval_ms: u64,
    pub theme: Theme,
    pub launch_at_startup: bool,
    pub confirm_before_kill: bool,
    pub default_open_protocol: OpenProtocol,
    #[serde(default)]
    pub pinned_ports: Vec<u16>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_system_ports: false,
            refresh_interval_ms: 2_000,
            theme: Theme::System,
            launch_at_startup: false,
            confirm_before_kill: true,
            default_open_protocol: OpenProtocol::Http,
            pinned_ports: Vec::new(),
        }
    }
}

impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if !(500..=60_000).contains(&self.refresh_interval_ms) {
            return Err("refresh interval must be between 500 and 60000 milliseconds".into());
        }
        if self.pinned_ports.len() > 100 || self.pinned_ports.contains(&0) {
            return Err("pinned ports must contain at most 100 valid port numbers".into());
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
        settings.refresh_interval_ms = 2_000;
        settings.pinned_ports = vec![0];
        assert!(settings.validate().is_err());
    }
}
