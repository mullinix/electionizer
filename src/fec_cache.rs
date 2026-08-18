use anyhow::Result;
use chrono::{Duration as ChronoDuration, Utc};
use sqlx::SqlitePool;

/// SQLite-backed cache for OpenFEC candidate list responses.
#[derive(Clone)]
pub struct FecCache {
    pool: SqlitePool,
    ttl_hours: i64,
}

impl FecCache {
    pub fn new(pool: SqlitePool, ttl_hours: i64) -> Self {
        Self {
            pool,
            ttl_hours: ttl_hours.max(1),
        }
    }

    pub fn cache_key(state: &str, office: &str, district: Option<u32>, cycle: i32) -> String {
        let d = match district {
            Some(0) => "00".to_string(),
            Some(n) => n.to_string(),
            None => "-".to_string(),
        };
        format!(
            "{}:{}:{}:{}",
            state.to_ascii_uppercase(),
            office.to_ascii_uppercase(),
            d,
            cycle
        )
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let row: Option<(String, String)> = sqlx::query_as(
            r#"
            SELECT body_json, fetched_at FROM fec_cache WHERE cache_key = ?
            "#,
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
            // Expired — drop so table stays small
            sqlx::query("DELETE FROM fec_cache WHERE cache_key = ?")
                .bind(key)
                .execute(&self.pool)
                .await?;
            Ok(None)
        }
    }

    pub async fn put(
        &self,
        key: &str,
        state: &str,
        office: &str,
        district: Option<u32>,
        cycle: i32,
        body_json: &str,
    ) -> Result<()> {
        let d = district.map(|n| {
            if n == 0 {
                "00".to_string()
            } else {
                n.to_string()
            }
        });
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        sqlx::query(
            r#"
            INSERT INTO fec_cache (cache_key, state, office, district, cycle, body_json, fetched_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(cache_key) DO UPDATE SET
                body_json = excluded.body_json,
                fetched_at = excluded.fetched_at,
                state = excluded.state,
                office = excluded.office,
                district = excluded.district,
                cycle = excluded.cycle
            "#,
        )
        .bind(key)
        .bind(state.to_ascii_uppercase())
        .bind(office.to_ascii_uppercase())
        .bind(d)
        .bind(cycle)
        .bind(body_json)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub fn ttl_hours(&self) -> i64 {
        self.ttl_hours
    }

    /// Cache arbitrary JSON under a free-form key (e.g. totals:{id}:{cycle}).
    pub async fn get_raw(&self, key: &str) -> Result<Option<String>> {
        self.get(key).await
    }

    pub async fn put_raw(&self, key: &str, cycle: i32, body_json: &str) -> Result<()> {
        self.put(key, "XX", "TOT", None, cycle, body_json).await
    }

    pub fn totals_key(candidate_id: &str, cycle: i32) -> String {
        format!("totals:{}:{}", candidate_id.to_ascii_uppercase(), cycle)
    }

    pub fn ie_key(candidate_id: &str, cycle: i32) -> String {
        format!("ie:{}:{}", candidate_id.to_ascii_uppercase(), cycle)
    }

    pub fn sched_a_ind_key(candidate_id: &str, cycle: i32) -> String {
        format!("sched_a_ind:{}:{}", candidate_id.to_ascii_uppercase(), cycle)
    }

    pub fn sched_a_cmte_key(candidate_id: &str, cycle: i32) -> String {
        format!(
            "sched_a_cmte:{}:{}",
            candidate_id.to_ascii_uppercase(),
            cycle
        )
    }

    pub fn size_key(candidate_id: &str, cycle: i32) -> String {
        format!("size:{}:{}", candidate_id.to_ascii_uppercase(), cycle)
    }

    pub fn committees_key(candidate_id: &str, cycle: i32) -> String {
        format!(
            "committees:{}:{}",
            candidate_id.to_ascii_uppercase(),
            cycle
        )
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
        })
        .or_else(|| {
            chrono::NaiveDateTime::parse_from_str(fetched_at, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|n| n.and_utc())
        });

    match parsed {
        Some(ts) => Utc::now() < ts + ChronoDuration::hours(ttl_hours),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_stable() {
        assert_eq!(
            FecCache::cache_key("fl", "H", Some(8), 2026),
            "FL:H:8:2026"
        );
        assert_eq!(
            FecCache::cache_key("AK", "H", Some(0), 2026),
            "AK:H:00:2026"
        );
        assert_eq!(FecCache::cache_key("CA", "S", None, 2026), "CA:S:-:2026");
    }

    #[test]
    fn fresh_timestamp() {
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        assert!(is_fresh(&now, 24));
        assert!(!is_fresh("2020-01-01T00:00:00Z", 24));
    }

    #[tokio::test]
    async fn put_get_roundtrip() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query(
            r#"
            CREATE TABLE fec_cache (
                cache_key TEXT PRIMARY KEY NOT NULL,
                state TEXT NOT NULL,
                office TEXT NOT NULL,
                district TEXT,
                cycle INTEGER NOT NULL,
                body_json TEXT NOT NULL,
                fetched_at TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let cache = FecCache::new(pool, 24);
        let key = FecCache::cache_key("FL", "H", Some(8), 2026);
        cache
            .put(&key, "FL", "H", Some(8), 2026, r#"[{"name":"TEST"}]"#)
            .await
            .unwrap();
        let got = cache.get(&key).await.unwrap().unwrap();
        assert!(got.contains("TEST"));
    }
}
