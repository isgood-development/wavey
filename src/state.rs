use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: String,
    pub volume: f32,
    pub ffmpeg_path: String,
    pub rpc_enabled: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "Dark".to_string(),
            volume: 0.5,
            ffmpeg_path: "".to_string(),
            rpc_enabled: false,
        }
    }
}
