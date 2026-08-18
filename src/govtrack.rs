use crate::http_cache::HttpCache;
use crate::models::VoteRecord;
use anyhow::{Context, Result};
use electionizer_core::govtrack::{
    build_fec_index, VoteDetail, VoteRef, VoteVoterResponse,
};
pub use electionizer_core::govtrack::{ballot_affiliations, LegislatorMatch};
use std::collections::HashMap;
use std::time::Duration;

const LEGISLATORS_URL: &str =
    "https://unitedstates.github.io/congress-legislators/legislators-current.json";
const LEGISLATORS_CACHE_KEY: &str = "congress:legislators-current";
const LEGISLATORS_HISTORICAL_URL: &str =
    "https://unitedstates.github.io/congress-legislators/legislators-historical.json";
const LEGISLATORS_HISTORICAL_CACHE_KEY: &str = "congress:legislators-historical";
const GT_UA: &str = "electionizer/0.1 (voter-info; +https://github.com/local/electionizer)";

/// Open data helpers: congress-legislators map + GovTrack roll-calls.
#[derive(Clone)]
pub struct LegislatorClient {
    client: reqwest::Client,
    cache: HttpCache,
}

impl LegislatorClient {
    pub fn new(cache: HttpCache) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(GT_UA)
                .timeout(Duration::from_secs(45))
                .connect_timeout(Duration::from_secs(15))
                .build()
                .expect("reqwest client"),
            cache,
        }
    }

    /// Resolve a federal candidate via FEC id → congress-legislators (current, then historical).
    pub async fn resolve_by_fec(&self, fec_id: &str) -> Result<Option<LegislatorMatch>> {
        let fec = fec_id.trim().to_ascii_uppercase();
        if fec.is_empty() {
            return Ok(None);
        }
        let map = self.fec_index().await?;
        if let Some(m) = map.get(&fec).cloned() {
            return Ok(Some(m));
        }
        // Former members may only appear in historical (same FEC id). Never invent spans.
        let hist = self.fec_index_historical().await?;
        Ok(hist.get(&fec).cloned())
    }

    /// Recent roll-call votes for a GovTrack person id.
    pub async fn recent_votes(&self, govtrack_id: i64, limit: usize) -> Result<Vec<VoteRecord>> {
        let limit = limit.clamp(1, 20);
        let list_key = format!("gt:votes:{govtrack_id}:{limit}");
        if let Ok(Some(body)) = self.cache.get(&list_key).await {
            if let Ok(rows) = serde_json::from_str::<Vec<VoteRecord>>(&body) {
                tracing::debug!(%list_key, "GovTrack votes cache hit");
                return Ok(rows);
            }
        }

        let url = format!(
            "https://www.govtrack.us/api/v2/vote_voter?person={govtrack_id}&limit={limit}&order_by=-created"
        );
        let body = self
            .get_text(&url)
            .await
            .with_context(|| format!("GovTrack vote_voter {govtrack_id}"))?;
        let parsed: VoteVoterResponse =
            serde_json::from_str(&body).context("parse GovTrack vote_voter")?;
        let mut out = Vec::new();
        for row in parsed.objects.unwrap_or_default() {
            let position = row
                .option
                .as_ref()
                .and_then(|o| o.value.clone())
                .unwrap_or_else(|| "—".into());
            let date = row
                .created
                .as_deref()
                .map(|s| s.chars().take(10).collect::<String>())
                .unwrap_or_default();
            let vote_id = row
                .vote
                .as_ref()
                .and_then(|v| match v {
                    VoteRef::Id(i) => Some(*i),
                    VoteRef::Obj(o) => o.id,
                })
                .or_else(|| row.option.as_ref().and_then(|o| o.vote));

            let (question, result, url) = if let Some(vid) = vote_id {
                match self.vote_detail(vid).await {
                    Ok(d) => d,
                    Err(err) => {
                        tracing::debug!(vote_id = vid, error = %err, "vote detail skip");
                        (
                            format!("Vote #{vid}"),
                            None,
                            format!("https://www.govtrack.us/congress/votes/{vid}"),
                        )
                    }
                }
            } else {
                ("Congressional vote".into(), None, "https://www.govtrack.us/congress/votes".into())
            };

            out.push(VoteRecord {
                date,
                question,
                position,
                result,
                url,
            });
        }

        if let Ok(json) = serde_json::to_string(&out) {
            let _ = self
                .cache
                .put(&list_key, &json, "govtrack:vote_voter", "application/json")
                .await;
        }
        Ok(out)
    }

    async fn vote_detail(&self, vote_id: i64) -> Result<(String, Option<String>, String)> {
        let key = format!("gt:vote:{vote_id}");
        let body = if let Ok(Some(b)) = self.cache.get(&key).await {
            b
        } else {
            let url = format!("https://www.govtrack.us/api/v2/vote/{vote_id}");
            let b = self.get_text(&url).await?;
            let _ = self
                .cache
                .put(&key, &b, &url, "application/json")
                .await;
            b
        };
        let v: VoteDetail = serde_json::from_str(&body).context("parse vote detail")?;
        let question = v
            .question
            .filter(|s| !s.trim().is_empty())
            .or(v.question_details)
            .unwrap_or_else(|| format!("Vote #{vote_id}"));
        let result = v.result.filter(|s| !s.trim().is_empty());
        let url = v
            .link
            .unwrap_or_else(|| format!("https://www.govtrack.us/congress/votes/{vote_id}"));
        Ok((question, result, url))
    }

    async fn fec_index(&self) -> Result<HashMap<String, LegislatorMatch>> {
        let json = self.legislators_json().await?;
        Ok(build_fec_index(&json))
    }

    async fn fec_index_historical(&self) -> Result<HashMap<String, LegislatorMatch>> {
        let json = self.legislators_historical_json().await?;
        Ok(build_fec_index(&json))
    }

    async fn legislators_json(&self) -> Result<String> {
        if let Ok(Some(body)) = self.cache.get(LEGISLATORS_CACHE_KEY).await {
            tracing::debug!("congress-legislators cache hit");
            return Ok(body);
        }
        let body = self
            .get_text(LEGISLATORS_URL)
            .await
            .context("fetch congress-legislators")?;
        let _ = self
            .cache
            .put(
                LEGISLATORS_CACHE_KEY,
                &body,
                LEGISLATORS_URL,
                "application/json",
            )
            .await;
        Ok(body)
    }

    async fn legislators_historical_json(&self) -> Result<String> {
        if let Ok(Some(body)) = self.cache.get(LEGISLATORS_HISTORICAL_CACHE_KEY).await {
            tracing::debug!("congress-legislators historical cache hit");
            return Ok(body);
        }
        let body = self
            .get_text(LEGISLATORS_HISTORICAL_URL)
            .await
            .context("fetch congress-legislators historical")?;
        let _ = self
            .cache
            .put(
                LEGISLATORS_HISTORICAL_CACHE_KEY,
                &body,
                LEGISLATORS_HISTORICAL_URL,
                "application/json",
            )
            .await;
        Ok(body)
    }

    async fn get_text(&self, url: &str) -> Result<String> {
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .with_context(|| format!("GET {url}"))?;
        if !resp.status().is_success() {
            anyhow::bail!("HTTP {} from {url}", resp.status());
        }
        resp.text().await.context("read body")
    }
}
