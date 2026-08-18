use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::SqlitePool;
use std::path::Path;
use std::str::FromStr;

/// Remove SQLite main DB + WAL sidecars if present.
pub fn wipe_db_files(path: &Path) -> Result<()> {
    let mut paths = vec![path.to_path_buf()];
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        let dir = path.parent().unwrap_or_else(|| Path::new("."));
        paths.push(dir.join(format!("{name}-wal")));
        paths.push(dir.join(format!("{name}-shm")));
    }

    for p in paths {
        if p.exists() {
            std::fs::remove_file(&p).with_context(|| format!("remove {}", p.display()))?;
            tracing::info!(path = %p.display(), "wiped db file");
        }
    }
    Ok(())
}

pub async fn connect(path: &Path) -> Result<SqlitePool> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create db dir {}", parent.display()))?;
        }
    }

    let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .with_context(|| format!("connect sqlite {}", path.display()))?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("run migrations")?;

    Ok(pool)
}
