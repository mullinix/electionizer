use super::{from_core, StateBallotExtras};
use crate::http_cache::HttpCache;
use crate::models::GeoResolution;
use anyhow::{Context, Result};
use electionizer_core::fl_extras_from_bodies;
use electionizer_core::states::florida::{
    extract_verification_token, fl_gen_elec_id, fl_measure_detail_cache_key,
    parse_fl_measure_summary_html, DOS_CANTYPES, DOS_CTS_INDEX, DOS_EXTRACT_URL, HOUSE_URL,
    MEASURES_URL, SENATE_URL,
};
use reqwest::Client;

pub async fn fetch(
    client: &Client,
    cache: &HttpCache,
    geo: &GeoResolution,
    cycle: i32,
) -> Result<StateBallotExtras> {
    let senate_html = match cached_get(client, cache, "fl:senate_roster", SENATE_URL).await {
        Ok(b) => b,
        Err(err) => {
            tracing::warn!(error = %err, "FL senate roster fetch failed");
            String::new()
        }
    };
    let house_html = match cached_get(client, cache, "fl:house_roster", HOUSE_URL).await {
        Ok(b) => b,
        Err(err) => {
            tracing::warn!(error = %err, "FL house roster fetch failed");
            String::new()
        }
    };

    let elec_id = fl_gen_elec_id(cycle);
    let dos_tsv = match fetch_dos_lists(client, cache, &elec_id).await {
        Ok(t) => t,
        Err(err) => {
            tracing::warn!(error = %err, elec_id = %elec_id, "FL DOS CTS fetch failed; using rosters");
            String::new()
        }
    };

    let measures_html = match fetch_fl_measures_html_cached(client, cache, cycle).await {
        Ok(h) => h,
        Err(err) => {
            tracing::warn!(error = %err, cycle, "FL measures scrape failed");
            String::new()
        }
    };

    let mut extras = from_core(fl_extras_from_bodies(
        geo,
        cycle,
        &dos_tsv,
        &senate_html,
        &house_html,
        &measures_html,
        "",
        "",
    ));

    // Native-only: InitDetail summaries (best-effort).
    for m in &mut extras.measures {
        if m.summary.is_some() {
            continue;
        }
        if let Some(summary) = fetch_fl_measure_summary(client, cache, &m.source_url).await {
            m.summary = Some(summary);
        }
    }

    Ok(extras)
}

async fn fetch_dos_lists(client: &Client, cache: &HttpCache, elec_id: &str) -> Result<String> {
    let merge_key = format!("fl:dos_cts:{elec_id}:STA+MUL+LOC");
    if let Ok(Some(body)) = cache.get(&merge_key).await {
        tracing::info!(%merge_key, "FL DOS CTS merged cache hit");
        return Ok(body);
    }

    let mut header: Option<String> = None;
    let mut data_lines: Vec<String> = Vec::new();
    let mut seen_acct = std::collections::HashSet::new();
    let mut any_ok = false;
    let mut last_err = None::<String>;

    for cantype in DOS_CANTYPES {
        match fetch_dos_list_cantype(client, cache, elec_id, cantype).await {
            Ok(body) => {
                any_ok = true;
                let mut lines = body.lines();
                let Some(h) = lines.next() else { continue };
                if header.is_none() {
                    header = Some(h.to_string());
                }
                for line in lines {
                    if line.trim().is_empty() {
                        continue;
                    }
                    let acct = line.split('\t').next().unwrap_or("").trim();
                    if !acct.is_empty() && !seen_acct.insert(acct.to_string()) {
                        continue;
                    }
                    data_lines.push(line.to_string());
                }
            }
            Err(err) => {
                tracing::warn!(%elec_id, %cantype, error = %err, "FL DOS CTS cantype fetch failed");
                last_err = Some(err.to_string());
            }
        }
    }

    if !any_ok {
        anyhow::bail!(
            "DOS extract failed for all cantypes: {}",
            last_err.unwrap_or_else(|| "unknown".into())
        );
    }
    let header = header.context("DOS extract returned no header")?;
    let body = std::iter::once(header)
        .chain(data_lines)
        .collect::<Vec<_>>()
        .join("\n");
    if !body.contains("AcctNum") || !body.contains('\t') {
        anyhow::bail!("DOS extract response does not look like CandidateList TSV");
    }
    if let Err(err) = cache
        .put(
            &merge_key,
            &body,
            DOS_EXTRACT_URL,
            "text/tab-separated-values",
        )
        .await
    {
        tracing::warn!(%merge_key, error = %err, "FL DOS CTS merged cache write failed");
    } else {
        tracing::info!(%merge_key, bytes = body.len(), "FL DOS CTS merged cache store");
    }
    Ok(body)
}

async fn fetch_dos_list_cantype(
    client: &Client,
    cache: &HttpCache,
    elec_id: &str,
    cantype: &str,
) -> Result<String> {
    let key = format!("fl:dos_cts:{elec_id}:{cantype}");
    if let Ok(Some(body)) = cache.get(&key).await {
        tracing::info!(%key, "FL DOS CTS cache hit");
        return Ok(body);
    }

    let resp = client
        .post(DOS_EXTRACT_URL)
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .header(reqwest::header::REFERER, DOS_CTS_INDEX)
        .body(format!(
            "elecID={elec_id}&office=All&status=All&cantype={cantype}&FormSubmit=Download+Candidate+List"
        ))
        .send()
        .await
        .with_context(|| format!("POST {DOS_EXTRACT_URL} cantype={cantype}"))?;

    if !resp.status().is_success() {
        anyhow::bail!(
            "HTTP {} from DOS extractCanList cantype={cantype}",
            resp.status()
        );
    }
    let body = resp.text().await.context("read DOS TSV body")?;
    if !body.contains("AcctNum") || !body.contains('\t') {
        anyhow::bail!("DOS extract cantype={cantype} does not look like CandidateList TSV");
    }
    if let Err(err) = cache
        .put(&key, &body, DOS_EXTRACT_URL, "text/tab-separated-values")
        .await
    {
        tracing::warn!(%key, error = %err, "FL DOS CTS cache write failed");
    } else {
        tracing::info!(%key, bytes = body.len(), "FL DOS CTS cache store");
    }
    Ok(body)
}

async fn fetch_fl_measures_html_cached(
    client: &Client,
    cache: &HttpCache,
    cycle: i32,
) -> Result<String> {
    let key = format!("fl:measures:{cycle}:ballot");
    if let Ok(Some(body)) = cache.get(&key).await {
        tracing::info!(%key, "FL measures cache hit");
        return Ok(body);
    }
    let body = fetch_fl_measures_html(client, cycle).await?;
    if let Err(err) = cache.put(&key, &body, MEASURES_URL, "text/html").await {
        tracing::warn!(%key, error = %err, "FL measures cache write failed");
    } else {
        tracing::info!(%key, bytes = body.len(), "FL measures cache store");
    }
    Ok(body)
}

async fn fetch_fl_measures_html(client: &Client, cycle: i32) -> Result<String> {
    let get = client
        .get(MEASURES_URL)
        .send()
        .await
        .with_context(|| format!("GET {MEASURES_URL}"))?;
    if !get.status().is_success() {
        anyhow::bail!("HTTP {} fetching measures index", get.status());
    }
    let index = get.text().await.context("read measures index")?;
    let token = extract_verification_token(&index)
        .context("missing __RequestVerificationToken on measures index")?;

    let form = [
        ("__RequestVerificationToken", token.as_str()),
        ("Year", &cycle.to_string()),
        ("Status", "ACT"),
        ("MadeBallot", "Y"),
        ("Sponsor", "ALL"),
    ];
    let post = client
        .post(MEASURES_URL)
        .header(reqwest::header::REFERER, MEASURES_URL)
        .form(&form)
        .send()
        .await
        .with_context(|| format!("POST {MEASURES_URL}"))?;
    if !post.status().is_success() {
        anyhow::bail!("HTTP {} filtering measures", post.status());
    }
    post.text().await.context("read measures filter body")
}

async fn fetch_fl_measure_summary(
    client: &Client,
    cache: &HttpCache,
    detail_url: &str,
) -> Option<String> {
    if detail_url.trim().is_empty() || !detail_url.contains("InitDetail") {
        return None;
    }
    let key = fl_measure_detail_cache_key(detail_url);
    let html = if let Ok(Some(body)) = cache.get(&key).await {
        body
    } else {
        let resp = client.get(detail_url).send().await.ok()?;
        if !resp.status().is_success() {
            return None;
        }
        let body = resp.text().await.ok()?;
        let _ = cache.put(&key, &body, detail_url, "text/html").await;
        body
    };
    parse_fl_measure_summary_html(&html)
}

async fn cached_get(client: &Client, cache: &HttpCache, key: &str, url: &str) -> Result<String> {
    if let Ok(Some(body)) = cache.get(key).await {
        tracing::info!(%key, "FL roster cache hit");
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
    if let Err(err) = cache.put(key, &body, url, "text/html").await {
        tracing::warn!(%key, error = %err, "FL roster cache write failed");
    } else {
        tracing::info!(%key, bytes = body.len(), "FL roster cache store");
    }
    Ok(body)
}
