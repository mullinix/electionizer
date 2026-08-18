use crate::fec_cache::FecCache;
use crate::redact::redact_secrets;
use anyhow::Result;
use electionizer_core::fec::{
    parse_ie_json, parse_principal_committee, parse_sched_a_json, parse_size_json, parse_totals_json,
};
use electionizer_core::models::{
    CandidateFinance, CommitteeLink, ContributorRow, OutsideSpendRow, SizeBucketRow,
};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time::sleep;

const FEC_MAX_ATTEMPTS: u32 = 3;
const FEC_TIMEOUT: Duration = Duration::from_secs(60);

/// Shared OpenFEC HTTP helper for detail-page enrichers.
#[derive(Clone)]
pub struct FecClient {
    client: reqwest::Client,
    api_key: Arc<RwLock<String>>,
    cycle: i32,
    cache: FecCache,
}

impl FecClient {
    pub fn new(api_key: Arc<RwLock<String>>, cycle: i32, cache: FecCache) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("electionizer/0.1 (voter-info; +https://github.com/local/electionizer)")
                .timeout(FEC_TIMEOUT)
                .connect_timeout(Duration::from_secs(15))
                .build()
                .expect("reqwest client"),
            api_key,
            cycle,
            cache,
        }
    }

    pub fn cycle(&self) -> i32 {
        self.cycle
    }

    fn api_key(&self) -> String {
        self.api_key
            .read()
            .map(|g| g.clone())
            .unwrap_or_else(|_| "DEMO_KEY".into())
    }

    /// Cycle finance totals for one FEC candidate id.
    pub async fn candidate_totals(&self, candidate_id: &str) -> Result<Option<CandidateFinance>> {
        let cid = candidate_id.trim();
        if cid.is_empty() {
            return Ok(None);
        }
        let key = FecCache::totals_key(cid, self.cycle);

        if let Ok(Some(body)) = self.cache.get_raw(&key).await {
            tracing::debug!(%key, "FEC totals cache hit");
            return Ok(parse_totals_json(&body, cid, self.cycle));
        }

        let mut last_err = String::new();
        for attempt in 1..=FEC_MAX_ATTEMPTS {
            match self.fetch_totals_once(cid).await {
                Ok(json) => {
                    if let Err(err) = self.cache.put_raw(&key, self.cycle, &json).await {
                        tracing::warn!(%key, error = %err, "FEC totals cache write failed");
                    }
                    return Ok(parse_totals_json(&json, cid, self.cycle));
                }
                Err(err) => {
                    last_err = err;
                    let retryable = last_err.contains("429")
                        || last_err.contains("rate limit")
                        || last_err.contains("timed out")
                        || last_err.contains("5");
                    if attempt < FEC_MAX_ATTEMPTS && retryable {
                        let delay = Duration::from_millis(400 * 2u64.pow(attempt - 1));
                        sleep(delay).await;
                        continue;
                    }
                    break;
                }
            }
        }
        anyhow::bail!("{}", redact_secrets(&last_err));
    }

    /// Top independent expenditures supporting/opposing a candidate (Schedule E).
    /// Non-fatal for callers: returns empty vec on empty results; Err on hard failure.
    pub async fn candidate_outside_spending(
        &self,
        candidate_id: &str,
    ) -> Result<Vec<OutsideSpendRow>> {
        let cid = candidate_id.trim();
        if cid.is_empty() {
            return Ok(Vec::new());
        }
        let key = FecCache::ie_key(cid, self.cycle);
        let json = self
            .cached_get(&key, "IE", || self.fetch_ie_once(cid))
            .await?;
        Ok(parse_ie_json(&json))
    }

    /// Largest itemized individual Schedule A lines (not unique-donor totals).
    pub async fn candidate_top_individual_contributors(
        &self,
        candidate_id: &str,
        limit: u32,
    ) -> Result<Vec<ContributorRow>> {
        let cid = candidate_id.trim();
        if cid.is_empty() {
            return Ok(Vec::new());
        }
        let limit = limit.clamp(1, 25);
        let key = FecCache::sched_a_ind_key(cid, self.cycle);
        let json = self
            .cached_get(&key, "SchedA-ind", || {
                self.fetch_sched_a_once(cid, true, limit)
            })
            .await?;
        Ok(parse_sched_a_json(&json, limit as usize))
    }

    /// Largest itemized committee/PAC Schedule A lines.
    pub async fn candidate_top_committee_contributors(
        &self,
        candidate_id: &str,
        limit: u32,
    ) -> Result<Vec<ContributorRow>> {
        let cid = candidate_id.trim();
        if cid.is_empty() {
            return Ok(Vec::new());
        }
        let limit = limit.clamp(1, 25);
        let key = FecCache::sched_a_cmte_key(cid, self.cycle);
        let json = self
            .cached_get(&key, "SchedA-cmte", || {
                self.fetch_sched_a_once(cid, false, limit)
            })
            .await?;
        Ok(parse_sched_a_json(&json, limit as usize))
    }

    /// Contribution size distribution (OpenFEC by_size/by_candidate).
    pub async fn candidate_contribution_sizes(
        &self,
        candidate_id: &str,
    ) -> Result<Vec<SizeBucketRow>> {
        let cid = candidate_id.trim();
        if cid.is_empty() {
            return Ok(Vec::new());
        }
        let key = FecCache::size_key(cid, self.cycle);
        let json = self
            .cached_get(&key, "size", || self.fetch_size_once(cid))
            .await?;
        Ok(parse_size_json(&json))
    }

    /// Principal campaign committee when available.
    pub async fn candidate_principal_committee(
        &self,
        candidate_id: &str,
    ) -> Result<Option<CommitteeLink>> {
        let cid = candidate_id.trim();
        if cid.is_empty() {
            return Ok(None);
        }
        let key = FecCache::committees_key(cid, self.cycle);
        let json = self
            .cached_get(&key, "committees", || self.fetch_committees_once(cid))
            .await?;
        Ok(parse_principal_committee(&json))
    }

    async fn cached_get<F, Fut>(&self, key: &str, label: &str, fetch: F) -> Result<String>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<String, String>>,
    {
        if let Ok(Some(body)) = self.cache.get_raw(key).await {
            tracing::debug!(%key, label, "FEC cache hit");
            return Ok(body);
        }
        let mut last_err = String::new();
        for attempt in 1..=FEC_MAX_ATTEMPTS {
            match fetch().await {
                Ok(json) => {
                    if let Err(err) = self.cache.put_raw(key, self.cycle, &json).await {
                        tracing::warn!(%key, error = %err, label, "FEC cache write failed");
                    }
                    return Ok(json);
                }
                Err(err) => {
                    last_err = err;
                    let retryable = last_err.contains("429")
                        || last_err.contains("rate limit")
                        || last_err.contains("timed out")
                        || last_err.contains("5");
                    if attempt < FEC_MAX_ATTEMPTS && retryable {
                        let delay = Duration::from_millis(400 * 2u64.pow(attempt - 1));
                        sleep(delay).await;
                        continue;
                    }
                    break;
                }
            }
        }
        anyhow::bail!("{}", redact_secrets(&last_err));
    }

    async fn fetch_totals_once(&self, candidate_id: &str) -> Result<String, String> {
        let mut url = reqwest::Url::parse(&format!(
            "https://api.open.fec.gov/v1/candidate/{candidate_id}/totals/"
        ))
        .map_err(|e| e.to_string())?;
        {
            let mut q = url.query_pairs_mut();
            q.append_pair("api_key", &self.api_key());
            q.append_pair("cycle", &self.cycle.to_string());
            q.append_pair("per_page", "20");
            q.append_pair("sort", "-cycle");
        }
        self.get_json(url, "totals").await
    }

    async fn fetch_ie_once(&self, candidate_id: &str) -> Result<String, String> {
        let mut url = reqwest::Url::parse(
            "https://api.open.fec.gov/v1/schedules/schedule_e/by_candidate/",
        )
        .map_err(|e| e.to_string())?;
        {
            let mut q = url.query_pairs_mut();
            q.append_pair("api_key", &self.api_key());
            q.append_pair("candidate_id", candidate_id);
            q.append_pair("cycle", &self.cycle.to_string());
            q.append_pair("per_page", "5");
            q.append_pair("sort", "-total");
        }
        self.get_json(url, "IE").await
    }

    async fn fetch_sched_a_once(
        &self,
        candidate_id: &str,
        individual: bool,
        limit: u32,
    ) -> Result<String, String> {
        let mut url =
            reqwest::Url::parse("https://api.open.fec.gov/v1/schedules/schedule_a/")
                .map_err(|e| e.to_string())?;
        {
            let mut q = url.query_pairs_mut();
            q.append_pair("api_key", &self.api_key());
            q.append_pair("candidate_id", candidate_id);
            q.append_pair("two_year_transaction_period", &self.cycle.to_string());
            q.append_pair("per_page", &limit.to_string());
            q.append_pair("sort", "-contribution_receipt_amount");
            q.append_pair("is_individual", if individual { "true" } else { "false" });
            q.append_pair("hide_null", "true");
        }
        self.get_json(url, if individual { "SchedA-ind" } else { "SchedA-cmte" })
            .await
    }

    async fn fetch_size_once(&self, candidate_id: &str) -> Result<String, String> {
        let mut url = reqwest::Url::parse(
            "https://api.open.fec.gov/v1/schedules/schedule_a/by_size/by_candidate/",
        )
        .map_err(|e| e.to_string())?;
        {
            let mut q = url.query_pairs_mut();
            q.append_pair("api_key", &self.api_key());
            q.append_pair("candidate_id", candidate_id);
            q.append_pair("cycle", &self.cycle.to_string());
            q.append_pair("per_page", "20");
            q.append_pair("sort", "size");
        }
        self.get_json(url, "size").await
    }

    async fn fetch_committees_once(&self, candidate_id: &str) -> Result<String, String> {
        let mut url = reqwest::Url::parse(&format!(
            "https://api.open.fec.gov/v1/candidate/{candidate_id}/committees/"
        ))
        .map_err(|e| e.to_string())?;
        {
            let mut q = url.query_pairs_mut();
            q.append_pair("api_key", &self.api_key());
            q.append_pair("cycle", &self.cycle.to_string());
            q.append_pair("per_page", "20");
        }
        self.get_json(url, "committees").await
    }

    async fn get_json(&self, url: reqwest::Url, label: &str) -> Result<String, String> {
        let resp = self.client.get(url).send().await.map_err(|e| {
            if e.is_timeout() {
                format!("FEC {label} request timed out")
            } else {
                format!(
                    "FEC {label} network error: {}",
                    redact_secrets(&e.without_url().to_string())
                )
            }
        })?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            let safe = redact_secrets(&truncate(&body, 200));
            if status.as_u16() == 429 {
                return Err(format!("FEC HTTP 429 rate limit: {safe}"));
            }
            return Err(format!("FEC HTTP {status}: {safe}"));
        }

        resp.text()
            .await
            .map_err(|e| format!("FEC {label} read body: {}", redact_secrets(&e.to_string())))
    }
}

fn truncate(s: &str, max: usize) -> String {
    electionizer_core::fec::truncate(s, max)
}
