use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProcessMetadata {
    pub pid: u32,
    pub process_name: String,
    pub display_name: Option<String>,
    pub memory_mb: Option<f64>,
    pub uptime_seconds: Option<u64>,
    pub command: Option<String>,
    pub executable_path: Option<String>,
    pub working_directory: Option<String>,
}
