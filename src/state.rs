use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: String,
    pub volume: f32,
    pub ffmpeg_path: String,
    pub rpc_enabled: bool,
}

pub struct PlayerState {
    pub active_video_id: String,
    pub display_name: String,
    pub total_duration: u64,

    pub is_paused: bool,
    pub seconds_passed: u64,
    pub queued_tracks: Vec<HashMap<String, String>>,
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

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            display_name: "Nothing is playing.".to_string(),
            is_paused: true,
            seconds_passed: 0,
            total_duration: 0,
            queued_tracks: Vec::new(),
            active_video_id: String::new(),
        }
    }
}
