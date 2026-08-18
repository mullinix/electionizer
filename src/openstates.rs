use crate::http_cache::HttpCache;
use crate::redact::redact_secrets;
use anyhow::{Context, Result};
use electionizer_core::models::VoteRecord;
use electionizer_core::openstates::{
    backoff_delay, classify_os_error, extract_votes_for_person, last_name, normalize_name_key,
    pick_person, truncate, vote_sessions, OsErrorKind, StateLegislatorMatch,
};
pub use electionizer_core::openstates::{
    district_from_office, is_rate_limit_error, looks_like_fec_id, state_code_from_jurisdiction,
};
use serde::Deserialize;
use serde_json::Value;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time::sleep;

const OS_ROOT: &str = "https://v3.openstates.org";
const OS_UA: &str = "electionizer/0.1 (voter-info; +https://github.com/local/electionizer)";
const OS_MAX_ATTEMPTS: u32 = 3;

/// OpenStates API client for state legislature identity + roll-calls.
#[derive(Clone)]
pub struct OpenStatesClient {
    client: reqwest::Client,
    api_key: Arc<RwLock<String>>,
    cache: HttpCache,
}

impl OpenStatesClient {
    pub fn new(api_key: Arc<RwLock<String>>, cache: HttpCache) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(OS_UA)
                .timeout(Duration::from_secs(45))
                .connect_timeout(Duration::from_secs(15))
                .build()
                .expect("reqwest client"),
            api_key,
            cache,
        }
    }

    pub fn has_key(&self) -> bool {
        !self.api_key_snapshot().is_empty()
    }

    fn api_key_snapshot(&self) -> String {
        self.api_key
            .read()
            .map(|g| g.trim().to_string())
            .unwrap_or_default()
    }

    /// Resolve a state legislator by name + chamber + district within a USPS state.
    pub async fn resolve_legislator(
        &self,
        name: &str,
        state: &str,
        chamber: &str,
        district: Option<u32>,
    ) -> Result<Option<StateLegislatorMatch>> {
        let key = self.api_key_snapshot();
        if key.is_empty() {
            return Ok(None);
        }
        let state = state.trim().to_ascii_lowercase();
        if state.len() != 2 {
            return Ok(None);
        }
        let want_org = match chamber {
            "state_senate" => "upper",
            "state_house" => "lower",
            _ => return Ok(None),
        };

        let cache_key = format!(
            "os:person:{}:{}:{}:{}",
            state,
            want_org,
            district.map(|d| d.to_string()).unwrap_or_else(|| "-".into()),
            normalize_name_key(name)
        );
        if let Ok(Some(body)) = self.cache.get(&cache_key).await {
            if body == "null" {
                return Ok(None);
            }
            if let Ok(m) = serde_json::from_str::<CachedPerson>(&body) {
                return Ok(Some(StateLegislatorMatch {
                    person_id: m.person_id,
                    name: m.name,
                    profile_url: m.profile_url,
                    jurisdiction: m.jurisdiction,
                    affiliations: m.affiliations,
                    image_url: m.image_url,
                    birth_year: m.birth_year,
                    career_spans: m.career_spans,
                }));
            }
        }

        // Last-name search first; on zero results, one full-name retry.
        let last = last_name(name);
        let body = self
            .fetch_people(&state, &last, &key)
            .await
            .context("OpenStates people")?;
        let mut matched = pick_person(&body, name, want_org, district, &state);

        if matched.is_none() {
            let full = name.trim();
            let full_last = last_name(full);
            if !full.is_empty()
                && full.to_ascii_lowercase() != last.to_ascii_lowercase()
                && full_last.to_ascii_lowercase() != full.to_ascii_lowercase()
            {
                // Only spend a second request when last-name search returned zero rows.
                let root: Value = serde_json::from_str(&body).unwrap_or(Value::Null);
                let empty = root
                    .get("results")
                    .and_then(|v| v.as_array())
                    .map(|a| a.is_empty())
                    .unwrap_or(true);
                if empty {
                    let body2 = self
                        .fetch_people(&state, full, &key)
                        .await
                        .context("OpenStates people (full name)")?;
                    matched = pick_person(&body2, name, want_org, district, &state);
                }
            }
        }

        let store = match &matched {
            Some(m) => serde_json::to_string(&CachedPerson {
                person_id: m.person_id.clone(),
                name: m.name.clone(),
                profile_url: m.profile_url.clone(),
                jurisdiction: m.jurisdiction.clone(),
                affiliations: m.affiliations.clone(),
                image_url: m.image_url.clone(),
                birth_year: m.birth_year,
                career_spans: m.career_spans.clone(),
            })
            .unwrap_or_else(|_| "null".into()),
            None => "null".into(),
        };
        let _ = self
            .cache
            .put(&cache_key, &store, "openstates:people", "application/json")
            .await;
        Ok(matched)
    }

    async fn fetch_people(&self, state: &str, name_q: &str, api_key: &str) -> Result<String> {
        let mut url = reqwest::Url::parse(&format!("{OS_ROOT}/people"))?;
        {
            let mut q = url.query_pairs_mut();
            q.append_pair("jurisdiction", state);
            q.append_pair("name", name_q);
            q.append_pair("per_page", "20");
        }
        self.get_json(url, api_key).await
    }

    /// Legislators near a point (`GET /people.geo`). Empty key → `Ok(None)`.
    pub async fn people_geo(&self, lat: f64, lng: f64) -> Result<Option<String>> {
        let key = self.api_key_snapshot();
        if key.is_empty() {
            return Ok(None);
        }
        let cache_key = format!("os:people.geo:{:.3},{:.3}", lat, lng);
        if let Ok(Some(body)) = self.cache.get(&cache_key).await {
            return Ok(Some(body));
        }
        let mut url = reqwest::Url::parse(&format!("{OS_ROOT}/people.geo"))
            .context("openstates people.geo url")?;
        {
            let mut q = url.query_pairs_mut();
            q.append_pair("lat", &lat.to_string());
            q.append_pair("lng", &lng.to_string());
            q.append_pair("apikey", &key);
        }
        let body = self
            .get_json(url, &key)
            .await
            .context("OpenStates people.geo")?;
        if let Err(err) = self
            .cache
            .put(
                &cache_key,
                &body,
                "openstates:people.geo",
                "application/json",
            )
            .await
        {
            tracing::warn!(error = %err, "OpenStates people.geo cache write failed");
        }
        Ok(Some(body))
    }

    /// Legislature incumbents by state + chamber district (`GET /people`).
    /// `org` is `upper` or `lower`. Returns raw JSON body or `None` if unkeyed.
    pub async fn people_by_district(
        &self,
        state: &str,
        org: &str,
        district: u32,
    ) -> Result<Option<String>> {
        let key = self.api_key_snapshot();
        if key.is_empty() {
            return Ok(None);
        }
        let st = state.trim().to_ascii_lowercase();
        if st.len() != 2 {
            return Ok(None);
        }
        let org = match org {
            "upper" | "lower" => org,
            _ => return Ok(None),
        };
        let cache_key = format!("os:people:{st}:{org}:{district}");
        if let Ok(Some(body)) = self.cache.get(&cache_key).await {
            return Ok(Some(body));
        }
        let mut url =
            reqwest::Url::parse(&format!("{OS_ROOT}/people")).context("openstates people url")?;
        {
            let mut q = url.query_pairs_mut();
            q.append_pair("jurisdiction", &st);
            q.append_pair("org_classification", org);
            q.append_pair("district", &district.to_string());
            q.append_pair("per_page", "10");
            q.append_pair("apikey", &key);
        }
        let body = self
            .get_json(url, &key)
            .await
            .context("OpenStates people by district")?;
        if let Err(err) = self
            .cache
            .put(&cache_key, &body, "openstates:people", "application/json")
            .await
        {
            tracing::warn!(error = %err, "OpenStates people cache write failed");
        }
        Ok(Some(body))
    }

    /// people.geo, or district /people merge when geo has no results.
    pub async fn people_for_geo(&self, geo: &crate::models::GeoResolution) -> Result<Option<String>> {
        if let (Some(lat), Some(lng)) = (geo.latitude, geo.longitude) {
            match self.people_geo(lat, lng).await {
                Ok(Some(body)) if people_results_len(&body) > 0 => return Ok(Some(body)),
                Ok(_) => {}
                Err(err) => {
                    tracing::warn!(error = %err, "Open States people.geo failed; trying district");
                }
            }
        }
        let mut results = Vec::new();
        let mut seen = std::collections::HashSet::new();
        if let Some(sd) = geo.state_senate_district {
            if let Ok(Some(body)) = self.people_by_district(&geo.state, "upper", sd).await {
                append_people_results(&body, &mut results, &mut seen);
            }
        }
        if let Some(hd) = geo.state_house_district {
            if let Ok(Some(body)) = self.people_by_district(&geo.state, "lower", hd).await {
                append_people_results(&body, &mut results, &mut seen);
            }
        }
        if results.is_empty() {
            return Ok(None);
        }
        let wrapped = serde_json::json!({ "results": results });
        Ok(Some(wrapped.to_string()))
    }

    /// Recent floor votes for an OpenStates person id within a jurisdiction abbr.
    /// `cycle` is the federal/election cycle year (e.g. 2026); we query cycle-1, cycle, cycle-2.
    pub async fn recent_votes(
        &self,
        person_id: &str,
        jurisdiction: &str,
        cycle: i32,
        limit: usize,
    ) -> Result<Vec<VoteRecord>> {
        let key = self.api_key_snapshot();
        if key.is_empty() {
            return Ok(Vec::new());
        }
        let limit = limit.clamp(1, 20);
        let sessions = vote_sessions(cycle);
        let sessions_tag = sessions
            .iter()
            .map(|y| y.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let list_key = format!("os:votes:{person_id}:{sessions_tag}:{limit}");
        if let Ok(Some(body)) = self.cache.get(&list_key).await {
            if let Ok(rows) = serde_json::from_str::<Vec<VoteRecord>>(&body) {
                return Ok(rows);
            }
        }

        let mut out = Vec::new();
        let mut seen = std::collections::HashSet::<String>::new();

        for session in &sessions {
            if out.len() >= limit {
                break;
            }
            let mut url = reqwest::Url::parse(&format!("{OS_ROOT}/bills"))?;
            {
                let mut q = url.query_pairs_mut();
                q.append_pair("jurisdiction", jurisdiction);
                q.append_pair("session", &session.to_string());
                q.append_pair("sort", "updated_desc");
                q.append_pair("per_page", "20");
                q.append_pair("include", "votes");
            }
            let body = match self.get_json(url, &key).await {
                Ok(b) => b,
                Err(err) => {
                    if !out.is_empty() {
                        // Partial success across sessions — keep what we have.
                        break;
                    }
                    // Empty so far: hard errors bubble; soft failures also Err so UI can note 429.
                    return Err(err).context("OpenStates bills");
                }
            };
            let batch = extract_votes_for_person(&body, person_id, limit);
            for row in batch {
                let dedupe = format!("{}|{}|{}", row.date, row.question, row.position);
                if seen.insert(dedupe) {
                    out.push(row);
                    if out.len() >= limit {
                        break;
                    }
                }
            }
        }

        if let Ok(json) = serde_json::to_string(&out) {
            let _ = self
                .cache
                .put(&list_key, &json, "openstates:votes", "application/json")
                .await;
        }
        Ok(out)
    }

    async fn get_json(&self, mut url: reqwest::Url, api_key: &str) -> Result<String> {
        // Auth: header + query (API accepts either).
        {
            let mut q = url.query_pairs_mut();
            q.append_pair("apikey", api_key);
        }

        let mut last_err = None::<anyhow::Error>;
        for attempt in 1..=OS_MAX_ATTEMPTS {
            match self.get_json_once(url.clone(), api_key).await {
                Ok(body) => return Ok(body),
                Err(err) => {
                    let kind = classify_os_error(&err);
                    let retryable = matches!(kind, OsErrorKind::RateLimit | OsErrorKind::Server)
                        && attempt < OS_MAX_ATTEMPTS;
                    if retryable {
                        let delay = backoff_delay(attempt, kind == OsErrorKind::RateLimit);
                        tracing::debug!(
                            attempt,
                            delay_ms = delay.as_millis() as u64,
                            error = %redact_secrets(&format!("{err:#}")),
                            "OpenStates retry"
                        );
                        sleep(delay).await;
                        last_err = Some(err);
                        continue;
                    }
                    return Err(err);
                }
            }
        }
        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("OpenStates request failed")))
    }

    async fn get_json_once(&self, url: reqwest::Url, api_key: &str) -> Result<String> {
        let resp = self
            .client
            .get(url)
            .header("X-API-KEY", api_key)
            .send()
            .await
            .map_err(|e| {
                let msg = redact_secrets(&e.without_url().to_string());
                anyhow::anyhow!("OpenStates network error: {msg}")
            })?;
        let status = resp.status();
        if !status.is_success() {
            let t = resp.text().await.unwrap_or_default();
            let safe = redact_secrets(&truncate(&t, 180));
            anyhow::bail!("OpenStates HTTP {status}: {safe}");
        }
        resp.text()
            .await
            .map_err(|e| anyhow::anyhow!("OpenStates body: {}", redact_secrets(&e.to_string())))
    }
}


#[derive(Debug, Clone, serde::Serialize, Deserialize)]
struct CachedPerson {
    person_id: String,
    name: String,
    profile_url: String,
    jurisdiction: String,
    #[serde(default)]
    affiliations: Vec<electionizer_core::models::AffiliationSpan>,
    #[serde(default)]
    image_url: Option<String>,
    #[serde(default)]
    birth_year: Option<i32>,
    #[serde(default)]
    career_spans: Vec<electionizer_core::bio::CareerSpan>,
}

fn people_results_len(body: &str) -> usize {
    let Ok(v) = serde_json::from_str::<Value>(body) else {
        return 0;
    };
    v.get("results")
        .and_then(|r| r.as_array())
        .map(|a| a.len())
        .unwrap_or(0)
}

fn append_people_results(
    body: &str,
    out: &mut Vec<Value>,
    seen: &mut std::collections::HashSet<String>,
) {
    let Ok(v) = serde_json::from_str::<Value>(body) else {
        return;
    };
    let Some(arr) = v.get("results").and_then(|r| r.as_array()) else {
        return;
    };
    for p in arr {
        let id = p
            .get("id")
            .and_then(|x| x.as_str())
            .or_else(|| p.get("name").and_then(|x| x.as_str()))
            .unwrap_or("")
            .to_string();
        if id.is_empty() || !seen.insert(id) {
            continue;
        }
        out.push(p.clone());
    }
}
