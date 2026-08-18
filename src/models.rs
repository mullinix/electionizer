//! Native model surface: re-exports core domain types plus SQLx row types.

pub use electionizer_core::models::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ZipRow {
    pub zip: String,
    pub status: String,
    pub last_built_at: Option<String>,
    pub error: Option<String>,
    pub coverage_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BuildJob {
    pub id: String,
    pub zip: String,
    pub status: String,
    pub stage: String,
    pub progress_pct: i64,
    pub message: String,
    pub error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub finished_at: Option<String>,
}
