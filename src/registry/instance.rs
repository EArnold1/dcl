use serde::{Deserialize, Serialize};
use std::fmt;
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

impl fmt::Display for InstanceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstanceStatus::InProgress => write!(f, "in_progress"),
            InstanceStatus::Ready => write!(f, "ready"),
            InstanceStatus::Failed { reason } => write!(f, "failed ({})", reason),
        }
    }
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
