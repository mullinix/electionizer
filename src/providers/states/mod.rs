mod arizona;
mod florida;

use crate::http_cache::HttpCache;
use crate::models::GeoResolution;
use anyhow::Result;
use reqwest::Client;

/// Optional state-level ballot enrichment keyed by USPS state code.
pub async fn enrich_state_ballot(
    client: &Client,
    cache: &HttpCache,
    geo: &GeoResolution,
    cycle: i32,
) -> Result<StateBallotExtras> {
    match geo.state.to_ascii_uppercase().as_str() {
        "FL" => florida::fetch(client, cache, geo, cycle).await,
        "AZ" => arizona::fetch(client, cache, geo, cycle).await,
        _ => Ok(empty_for(&geo.state)),
    }
}

/// Same shape as core; native keeps a local alias for call sites.
pub type StateBallotExtras = electionizer_core::StateBallotExtras;

pub fn empty_for(state: &str) -> StateBallotExtras {
    StateBallotExtras {
        candidates: vec![],
        measures: vec![],
        coverage_label: None,
        notes: vec![format!(
            "{state} state/local races not yet collected (FL, AZ, NC available in static app)."
        )],
        extra_jurisdictions: vec![],
    }
}

pub(crate) fn from_core(ex: electionizer_core::StateBallotExtras) -> StateBallotExtras {
    ex
}
