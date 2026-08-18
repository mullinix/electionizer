use super::states;
use super::BallotProvider;
use crate::fec_cache::FecCache;
use crate::http_cache::HttpCache;
use crate::models::{federal_election_date, BallotSnapshot, GeoResolution};
use crate::openstates::OpenStatesClient;
use crate::redact::{fec_source_url_public, redact_secrets};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use electionizer_core::{
    filter_fec_candidates, geo_from_zippo_and_census, map_fec_candidates,
    openstates_extras_from_people_geo, parse_cd_number, parse_census_coordinates_json,
    parse_fec_candidates_json, parse_zippo_json, FecCandidateRow,
};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time::sleep;

const FEC_MAX_ATTEMPTS: u32 = 3;
const FEC_TIMEOUT: Duration = Duration::from_secs(60);

/// Live provider: Zippopotam + Census geographies → OpenFEC federal + state extras.
pub struct LiveProvider {
    client: reqwest::Client,
    fec_api_key: Arc<RwLock<String>>,
    openstates: OpenStatesClient,
    cycle: i32,
    cache: FecCache,
    http_cache: HttpCache,
}

impl LiveProvider {
    pub fn new(
        fec_api_key: Arc<RwLock<String>>,
        openstates_api_key: Arc<RwLock<String>>,
        cycle: i32,
        cache: FecCache,
        http_cache: HttpCache,
    ) -> Self {
        let openstates = OpenStatesClient::new(openstates_api_key, http_cache.clone());
        Self {
            client: reqwest::Client::builder()
                .user_agent("electionizer/0.1 (voter-info; +https://github.com/local/electionizer)")
                .timeout(FEC_TIMEOUT)
                .connect_timeout(Duration::from_secs(15))
                .cookie_store(true)
                .build()
                .expect("reqwest client"),
            fec_api_key,
            openstates,
            cycle,
            cache,
            http_cache,
        }
    }

    fn api_key(&self) -> String {
        self.fec_api_key
            .read()
            .map(|g| g.clone())
            .unwrap_or_else(|_| "DEMO_KEY".into())
    }

    async fn resolve_geo(&self, zip: &str) -> Result<GeoResolution> {
        let zippo = self.zippopotam(zip).await?;
        let census = self
            .census_by_coordinates(zippo.longitude, zippo.latitude)
            .await?;
        Ok(geo_from_zippo_and_census(zip, &zippo, &census))
    }

    async fn zippopotam(&self, zip: &str) -> Result<electionizer_core::ZippoPlace> {
        let url = format!("https://api.zippopotam.us/us/{zip}");
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("zippopotam request")?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            bail!("ZIP {zip} not found");
        }
        if !resp.status().is_success() {
            bail!("zippopotam HTTP {}", resp.status());
        }

        let text = resp.text().await.context("zippopotam body")?;
        parse_zippo_json(&text).map_err(|e| anyhow::anyhow!(e))
    }

    async fn census_by_coordinates(&self, lon: f64, lat: f64) -> Result<electionizer_core::CensusGeo> {
        let url = format!(
            "https://geocoding.geo.census.gov/geocoder/geographies/coordinates?x={lon}&y={lat}&benchmark=Public_AR_Current&vintage=Current_Current&format=json"
        );
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("census coordinates request")?;
        if !resp.status().is_success() {
            bail!("census geocoder HTTP {}", resp.status());
        }

        let text = resp.text().await.context("census body")?;
        parse_census_coordinates_json(&text).map_err(|e| anyhow::anyhow!(e))
    }

    async fn fec_candidates(
        &self,
        state: &str,
        office: &str,
        district: Option<u32>,
    ) -> Result<(Vec<FecCandidateRow>, String)> {
        let public_url = fec_source_url_public(state, office, district, self.cycle);
        let cache_key = FecCache::cache_key(state, office, district, self.cycle);

        if let Ok(Some(cached)) = self.cache.get(&cache_key).await {
            match parse_fec_candidates_json(&cached) {
                Ok(rows) => {
                    tracing::info!(
                        %cache_key,
                        count = rows.len(),
                        ttl_hours = self.cache.ttl_hours(),
                        "FEC cache hit"
                    );
                    let filtered = filter_fec_candidates(rows, self.cycle);
                    return Ok((filtered, public_url));
                }
                Err(err) => {
                    tracing::warn!(%cache_key, error = %err, "FEC cache JSON corrupt; refetching");
                }
            }
        }

        let mut url = reqwest::Url::parse("https://api.open.fec.gov/v1/candidates/")
            .expect("fec base url");
        {
            let key = self.api_key();
            let mut q = url.query_pairs_mut();
            q.append_pair("api_key", &key);
            q.append_pair("state", state);
            q.append_pair("office", office);
            q.append_pair("cycle", &self.cycle.to_string());
            q.append_pair("election_year", &self.cycle.to_string());
            q.append_pair("candidate_status", "C");
            q.append_pair("per_page", "100");
            q.append_pair("sort", "name");
            if let Some(d) = district {
                let dstr = if d == 0 {
                    "00".to_string()
                } else {
                    d.to_string()
                };
                q.append_pair("district", &dstr);
            }
        }

        let mut last_err = String::from("FEC request failed");
        for attempt in 1..=FEC_MAX_ATTEMPTS {
            match self.fec_candidates_once(url.clone()).await {
                Ok(rows) => {
                    if let Ok(json) = serde_json::to_string(&rows) {
                        if let Err(err) = self
                            .cache
                            .put(&cache_key, state, office, district, self.cycle, &json)
                            .await
                        {
                            tracing::warn!(%cache_key, error = %err, "FEC cache write failed");
                        } else {
                            tracing::info!(%cache_key, count = rows.len(), "FEC cache store");
                        }
                    }
                    let filtered = filter_fec_candidates(rows, self.cycle);
                    return Ok((filtered, public_url));
                }
                Err(err) => {
                    last_err = err;
                    let retryable = last_err.contains("timed out")
                        || last_err.contains("timeout")
                        || last_err.contains("HTTP 5")
                        || last_err.contains("HTTP 429")
                        || last_err.contains("rate limit")
                        || last_err.contains("network");
                    if !retryable || attempt == FEC_MAX_ATTEMPTS {
                        break;
                    }
                    let backoff = Duration::from_secs(2u64.pow(attempt)); // 2s, 4s
                    tracing::warn!(
                        attempt,
                        backoff_secs = backoff.as_secs(),
                        office,
                        state,
                        error = %last_err,
                        "FEC request failed; retrying"
                    );
                    sleep(backoff).await;
                }
            }
        }

        if last_err.contains("429") || last_err.to_ascii_lowercase().contains("rate limit") {
            bail!(
                "FEC rate limit exceeded after {FEC_MAX_ATTEMPTS} attempts. \
                 Set a personal API key in Settings (https://api.open.fec.gov/developers/) or wait and retry."
            );
        }
        if last_err.contains("timed out") || last_err.contains("timeout") {
            bail!(
                "FEC timed out after {FEC_MAX_ATTEMPTS} attempts ({state} {office}). \
                 The OpenFEC API may be slow — try again in a minute."
            );
        }
        bail!("{last_err}");
    }

    async fn fec_candidates_once(&self, url: reqwest::Url) -> Result<Vec<FecCandidateRow>, String> {
        let resp = match self.client.get(url).send().await {
            Ok(r) => r,
            Err(e) => {
                // Never include raw reqwest error (embeds full URL + api_key).
                if e.is_timeout() {
                    return Err("FEC request timed out".into());
                }
                if e.is_connect() {
                    return Err("FEC network connect error".into());
                }
                return Err(format!(
                    "FEC network error: {}",
                    redact_secrets(&e.without_url().to_string())
                ));
            }
        };

        let status = resp.status();
        let text = match resp.text().await {
            Ok(t) => t,
            Err(e) => {
                return Err(format!(
                    "FEC body error: {}",
                    redact_secrets(&e.to_string())
                ))
            }
        };
        if !status.is_success() {
            let safe_body = redact_secrets(&truncate(&text, 160));
            if status.as_u16() == 429 {
                return Err(format!("FEC HTTP 429 rate limit: {safe_body}"));
            }
            return Err(format!("FEC HTTP {status}: {safe_body}"));
        }

        parse_fec_candidates_json(&text).map_err(|e| {
            format!(
                "FEC response JSON error: {}",
                redact_secrets(&e)
            )
        })
    }
}

#[async_trait]
impl BallotProvider for LiveProvider {
    fn name(&self) -> &'static str {
        "live"
    }

    async fn resolve_zip(&self, zip: &str) -> Result<GeoResolution> {
        self.resolve_geo(zip)
            .await
            .with_context(|| format!("resolve geography for ZIP {zip}"))
    }

    async fn fetch_ballot(&self, _zip: &str, geo: &GeoResolution) -> Result<BallotSnapshot> {
        let state_l = geo.state.to_ascii_lowercase();
        let state_ocd = format!("ocd-division/country:us/state:{state_l}");
        let cd_num = parse_cd_number(&geo.congressional_district);
        let cd_ocd = geo
            .jurisdictions
            .iter()
            .find(|j| j.level == "congressional")
            .map(|j| j.ocd_id.clone())
            .unwrap_or_else(|| {
                if cd_num == 0 {
                    format!("ocd-division/country:us/state:{state_l}/cd:0")
                } else {
                    format!("ocd-division/country:us/state:{state_l}/cd:{cd_num}")
                }
            });

        let house_office = if cd_num == 0 {
            format!("U.S. House ({}) At-Large", geo.state)
        } else {
            format!("U.S. House ({})", geo.congressional_district)
        };

        let (house_rows, house_url) = self
            .fec_candidates(&geo.state, "H", Some(cd_num))
            .await
            .context("fetch House candidates from FEC")?;
        let (senate_rows, senate_url) = self
            .fec_candidates(&geo.state, "S", None)
            .await
            .context("fetch Senate candidates from FEC")?;

        let mut candidates = map_fec_candidates(
            &house_rows,
            &house_office,
            "house",
            &cd_ocd,
            &house_url,
        );
        candidates.extend(map_fec_candidates(
            &senate_rows,
            "U.S. Senate",
            "senate",
            &state_ocd,
            &senate_url,
        ));

        let federal_count = candidates.len();
        if federal_count == 0 {
            bail!(
                "FEC returned no active federal candidates for {} / {} cycle {}",
                geo.state,
                geo.congressional_district,
                self.cycle
            );
        }

        // State-level extras (FL/AZ scrapes; else Open States people.geo when keyed)
        let mut state_extras = match states::enrich_state_ballot(
            &self.client,
            &self.http_cache,
            geo,
            self.cycle,
        )
        .await
        {
            Ok(x) => x,
            Err(err) => {
                tracing::warn!(state = %geo.state, error = %err, "state ballot enrichment failed");
                states::empty_for(&geo.state)
            }
        };
        if state_extras.candidates.is_empty() {
            match self.openstates.people_for_geo(geo).await {
                Ok(Some(json)) if !json.trim().is_empty() => {
                    let os = openstates_extras_from_people_geo(geo, &json);
                    if !os.candidates.is_empty() {
                        state_extras = os;
                    }
                }
                Ok(_) => {}
                Err(err) => {
                    tracing::warn!(error = %err, "Open States legislature lookup failed");
                }
            }
        }
        candidates.extend(state_extras.candidates);
        let measures = state_extras.measures;
        let extra_jurisdictions = state_extras.extra_jurisdictions;

        let has_state = state_extras.coverage_label.is_some();
        if !state_extras.notes.is_empty() {
            tracing::debug!(notes = ?state_extras.notes, "state ballot notes");
        }
        let mut coverage_parts = vec!["Federal (live via FEC)".to_string()];
        if let Some(label) = state_extras.coverage_label {
            coverage_parts.push(label);
        } else {
            coverage_parts.push("State/local pending".into());
        }
        if measures.is_empty() {
            coverage_parts.push("Measures pending".into());
        } else {
            let real = measures.iter().any(|m| {
                m.measure_code.as_deref().is_some_and(|c| {
                    let u = c.to_ascii_uppercase();
                    u.starts_with("AMENDMENT") || u.starts_with("PROP") || u.starts_with("MEASURE")
                })
            });
            let st = geo.state.to_ascii_uppercase();
            if real {
                coverage_parts.push(format!("{st} measures"));
            } else {
                coverage_parts.push(format!("{st} measures (link)"));
            }
        }

        let election_date = federal_election_date(self.cycle);
        let scope = if has_state {
            "federal+state"
        } else {
            "federal"
        };
        Ok(BallotSnapshot {
            election_name: format!("{} General Election", self.cycle),
            election_date,
            election_scope: scope.into(),
            candidates,
            measures,
            source_url: "https://api.open.fec.gov/v1/candidates/".into(),
            source_publisher: "Federal Election Commission OpenFEC API (+ state sources when available)"
                .into(),
            coverage_note: Some(coverage_parts.join(" · ")),
            extra_jurisdictions,
        })
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

#[cfg(test)]
mod tests {
    use electionizer_core::{format_person_name, parse_cd_number};

    #[test]
    fn formats_fec_names() {
        assert_eq!(format_person_name("SHERMAN, BRAD"), "Brad Sherman");
        assert_eq!(format_person_name("DOE, JANE A"), "Jane A Doe");
    }

    #[test]
    fn parses_cd() {
        assert_eq!(parse_cd_number("CA-36"), 36);
        assert_eq!(parse_cd_number("NY-12"), 12);
        assert_eq!(parse_cd_number("AK-AL"), 0);
    }
}
