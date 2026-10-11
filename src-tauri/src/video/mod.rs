//! Sizqo streaming video contract. Video bytes never enter EngineResult.
use serde::{Deserialize, Serialize};
use std::path::Path;
pub mod capabilities;
pub mod plan;
pub mod presets;
pub mod probe;
pub mod progress;
pub mod transaction;
pub mod validation;
#[cfg(not(feature = "inproc-backends"))]
pub mod ffmpeg_backend;
#[cfg(not(feature = "inproc-backends"))]
pub mod process;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum MediaKind { #[default] Image, Video }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct VideoOptions {
    pub encoder: String, pub mode: String, pub crf: Option<f64>,
    pub bitrate_kbps: Option<u64>, pub target_mi_b: Option<f64>,
    pub preset: String, pub resolution: u32, pub frame_rate: Option<f64>,
    pub pixel_format: Option<String>, pub audio_policy: String,
    pub audio_bitrate_kbps: u64, pub subtitle_policy: String,
    pub metadata_policy: String, pub output_mode: String,
    pub output_dir: Option<String>, pub output_suffix: String,
    pub allow_larger_output: bool, pub container: String, pub threads: usize,
}
impl Default for VideoOptions {
    fn default() -> Self { Self { encoder: "auto".into(), mode: "abr".into(),
        crf: None, bitrate_kbps: None, target_mi_b: None, preset: "medium".into(),
        resolution: 0, frame_rate: None, pixel_format: None, audio_policy: "copy".into(),
        audio_bitrate_kbps: 160, subtitle_policy: "copy".into(), metadata_policy: "preserve".into(),
        output_mode: "suffix".into(), output_dir: None, output_suffix: "_compressed".into(),
        allow_larger_output: false, container: "mp4".into(), threads: 3,
    } }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoResult {
    pub success: bool, pub task_id: String, pub source_path: String,
    pub output_path: Option<String>, pub backup_path: Option<String>,
    pub original_bytes: u64, pub output_bytes: Option<u64>, pub elapsed_ms: u64,
    pub encoder_actual: String, pub backend_actual: String, pub savings: Option<f64>,
    pub reason: Option<String>, pub warnings: Vec<String>, pub diagnostics_log_path: Option<String>,
    pub output_mode: String, pub history_id: Option<String>, pub stage: String,
}
impl VideoResult {
    pub fn pending(task_id: &str, path: &Path, options: &VideoOptions) -> Self {
        Self { success: false, task_id: task_id.into(), source_path: path.to_string_lossy().into(),
            output_path: None, backup_path: None, original_bytes: std::fs::metadata(path).map(|m|m.len()).unwrap_or(0),
            output_bytes: None, elapsed_ms: 0, encoder_actual: String::new(), backend_actual: "ffmpeg".into(),
            savings: None, reason: None, warnings: vec![], diagnostics_log_path: None,
            output_mode: options.output_mode.clone(), history_id: None, stage: "failed".into() }
    }
}
pub fn is_video_extension(path:&Path) -> bool {
    matches!(path.extension().and_then(|x|x.to_str()).unwrap_or("").to_lowercase().as_str(),
        "mp4"|"mov"|"mkv"|"webm"|"m4v"|"avi"|"ts"|"mts"|"m2ts"|"flv"|"wmv"|"mpg"|"mpeg"|"vob"|"rm"|"rmvb")
}
