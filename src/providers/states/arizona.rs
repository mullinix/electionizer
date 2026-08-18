use super::{from_core, StateBallotExtras};
use crate::http_cache::HttpCache;
use crate::models::GeoResolution;
use anyhow::{Context, Result};
use electionizer_core::az_extras_from_rosters;
use electionizer_core::states::arizona::{
    az_measures_list_url, az_officials_list_url, az_officials_packed, pick_az_measures_county_id,
    pick_az_measures_election_id, HOUSE_URL, MEASURES_COUNTIES_BASE, MEASURES_ELECTIONS_URL,
    SENATE_URL,
};
use reqwest::Client;

pub async fn fetch(
    client: &Client,
    cache: &HttpCache,
    geo: &GeoResolution,
    cycle: i32,
) -> Result<StateBallotExtras> {
    let senate_html = match cached_get(client, cache, "az:senate_roster", SENATE_URL).await {
        Ok(b) => b,
        Err(err) => {
            tracing::warn!(error = %err, "AZ senate roster fetch failed");
            String::new()
        }
    };
    let house_html = match cached_get(client, cache, "az:house_roster", HOUSE_URL).await {
        Ok(b) => b,
        Err(err) => {
            tracing::warn!(error = %err, "AZ house roster fetch failed");
            String::new()
        }
    };
    let measures_html = match fetch_measures(client, cache, geo, cycle).await {
        Ok(b) => b,
        Err(err) => {
            tracing::warn!(error = %err, "AZ measures fetch failed");
            String::new()
        }
    };
    let officials_html = match fetch_officials(client, cache, geo).await {
        Ok(b) => b,
        Err(err) => {
            tracing::warn!(error = %err, "AZ OfficialList fetch failed");
            String::new()
        }
    };
    Ok(from_core(az_extras_from_rosters(
        geo,
        &senate_html,
        &house_html,
        &measures_html,
        &officials_html,
    )))
}

async fn fetch_officials(
    client: &Client,
    cache: &HttpCache,
    geo: &GeoResolution,
) -> Result<String> {
    let cd = geo
        .congressional_district
        .rsplit(|c: char| !c.is_ascii_digit())
        .find(|s| !s.is_empty())
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    let leg = geo
        .state_senate_district
        .or(geo.state_house_district)
        .unwrap_or(0);
    let Some(packed) = az_officials_packed(cd, leg, &geo.county) else {
        anyhow::bail!("AZ OfficialList needs CD + LD + county");
    };
    let url = az_officials_list_url(&packed);
    cached_get(client, cache, &format!("az:officials:{packed}"), &url)
        .await
        .context("AZ OfficialList")
}

async fn fetch_measures(
    client: &Client,
    cache: &HttpCache,
    geo: &GeoResolution,
    cycle: i32,
) -> Result<String> {
    let elections = cached_get(
        client,
        cache,
        "az:measures:elections",
        MEASURES_ELECTIONS_URL,
    )
    .await
    .context("AZ ElectionsForBM")?;
    let Some(election_id) = pick_az_measures_election_id(&elections, cycle) else {
        tracing::info!(cycle, "AZ measures: no election for cycle");
        return Ok(String::new());
    };
    let mut county_id = 0u32;
    if !geo.county.trim().is_empty() {
        let counties_url = format!("{MEASURES_COUNTIES_BASE}?election={election_id}");
        let counties_key = format!("az:measures:counties:{election_id}");
        if let Ok(body) = cached_get(client, cache, &counties_key, &counties_url).await {
            if let Some(id) = pick_az_measures_county_id(&body, &geo.county) {
                county_id = id;
            }
        }
    }
    let list_url = az_measures_list_url(election_id, county_id);
    let key = format!("az:measures:{election_id}:{county_id}");
    cached_get(client, cache, &key, &list_url).await
}

async fn cached_get(client: &Client, cache: &HttpCache, key: &str, url: &str) -> Result<String> {
    if let Ok(Some(body)) = cache.get(key).await {
        tracing::info!(%key, "AZ cache hit");
        return Ok(body);
    }
    let resp = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("GET {url}"))?;
    if !resp.status().is_success() {
        anyhow::bail!("HTTP {} fetching {url}", resp.status());
    }
    let body = resp.text().await.context("read body")?;
    let ctype = if url.contains("ElectionsForBM") || url.contains("CountiesForBM") {
        "application/json"
    } else {
        "text/html"
    };
    if let Err(err) = cache.put(key, &body, url, ctype).await {
        tracing::warn!(%key, error = %err, "AZ cache write failed");
    } else {
        tracing::info!(%key, bytes = body.len(), "AZ cache store");
    }
    Ok(body)
}
