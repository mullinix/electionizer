use anyhow::Result;
use chrono::{Duration as ChronoDuration, Utc};
use sqlx::SqlitePool;

/// Generic SQLite HTTP response cache (HTML/JSON rosters, etc.).
#[derive(Clone)]
pub struct HttpCache {
    pool: SqlitePool,
    ttl_hours: i64,
}

impl HttpCache {
    pub fn new(pool: SqlitePool, ttl_hours: i64) -> Self {
        Self {
            pool,
            ttl_hours: ttl_hours.max(1),
        }
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let row: Option<(String, String)> = sqlx::query_as(
            "SELECT body, fetched_at FROM http_cache WHERE cache_key = ?",
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        let Some((body, fetched_at)) = row else {
            return Ok(None);
        };
        if is_fresh(&fetched_at, self.ttl_hours) {
            Ok(Some(body))
        } else {
            sqlx::query("DELETE FROM http_cache WHERE cache_key = ?")
                .bind(key)
                .execute(&self.pool)
                .await?;
            Ok(None)
        }
    }

    pub async fn put(&self, key: &str, body: &str, source_url: &str, content_type: &str) -> Result<()> {
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        sqlx::query(
            r#"
            INSERT INTO http_cache (cache_key, body, content_type, source_url, fetched_at)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(cache_key) DO UPDATE SET
                body = excluded.body,
                content_type = excluded.content_type,
                source_url = excluded.source_url,
                fetched_at = excluded.fetched_at
            "#,
        )
        .bind(key)
        .bind(body)
        .bind(content_type)
        .bind(source_url)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

fn is_fresh(fetched_at: &str, ttl_hours: i64) -> bool {
    let parsed = chrono::DateTime::parse_from_rfc3339(fetched_at)
        .map(|d| d.with_timezone(&Utc))
        .ok()
        .or_else(|| {
            chrono::NaiveDateTime::parse_from_str(fetched_at, "%Y-%m-%dT%H:%M:%SZ")
                .ok()
                .map(|n| n.and_utc())
        });
    match parsed {
        Some(ts) => Utc::now() < ts + ChronoDuration::hours(ttl_hours),
        None => false,
    }
}
