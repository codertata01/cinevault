use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnumeratedVolume {
    pub serial: String,
    pub guid: String,
    pub label: String,
    pub filesystem: String,
    pub drive_letter: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeRecord {
    pub id: i64,
    pub serial: String,
    pub guid: String,
    pub label: String,
    pub filesystem: String,
    pub last_drive_letter: Option<String>,
    pub auto_remap: bool,
    pub is_online: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemapCandidate {
    pub volume_id: i64,
    pub serial: String,
    pub guid: String,
    pub label: String,
    pub previous_drive_letter: String,
    pub current_drive_letter: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeSyncResult {
    pub volumes: Vec<VolumeRecord>,
    pub remap_candidates: Vec<RemapCandidate>,
}

pub(crate) fn format_serial(serial: u32) -> String {
    format!("{serial:08X}")
}

pub fn normalize_drive_letter(letter: &str) -> String {
    let trimmed = letter.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let upper = trimmed.to_ascii_uppercase();
    if upper.ends_with(':') {
        upper
    } else {
        format!("{}:", upper.chars().next().unwrap_or('?'))
    }
}