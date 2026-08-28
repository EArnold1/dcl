use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::materialization::Materialization;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub name: String,
    pub destination: PathBuf,
    pub source: PathBuf,
    pub revision: String,
    pub mode: Materialization,
    pub status: InstanceStatus,
    pub created_at: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceStatus {
    InProgress,
    Ready,
    Failed { reason: String },
}

impl Instance {
    pub fn new(
        name: String,
        destination: PathBuf,
        source: PathBuf,
        revision: String,
        mode: Materialization,
    ) -> Self {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self {
            name,
            destination,
            source,
            revision,
            mode,
            status: InstanceStatus::InProgress,
            created_at,
        }
    }
}
