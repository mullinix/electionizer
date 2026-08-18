use super::BallotProvider;
use crate::models::{
    BallotSnapshot, FixtureFile, GeoResolution, ResolvedJurisdiction, SnapshotCandidate,
    SnapshotMeasure,
};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs;

pub struct FixtureProvider {
    dir: PathBuf,
}

impl FixtureProvider {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    async fn load(&self, zip: &str) -> Result<FixtureFile> {
        let path = self.dir.join(format!("fixture_{zip}.json"));
        if path.exists() {
            return read_fixture(&path).await;
        }

        // Fall back to 90210 fixture for any zip in fixture mode (thin-slice demo)
        let fallback = self.dir.join("fixture_90210.json");
        if fallback.exists() {
            let mut data = read_fixture(&fallback).await?;
            data.zip = zip.to_string();
            return Ok(data);
        }

        bail!("no fixture for zip {zip} and no fixture_90210.json fallback");
    }
}

async fn read_fixture(path: &Path) -> Result<FixtureFile> {
    let text = fs::read_to_string(path)
        .await
        .with_context(|| format!("read fixture {}", path.display()))?;
    let data: FixtureFile = serde_json::from_str(&text)
        .with_context(|| format!("parse fixture {}", path.display()))?;
    Ok(data)
}

#[async_trait]
impl BallotProvider for FixtureProvider {
    fn name(&self) -> &'static str {
        "fixture"
    }

    async fn resolve_zip(&self, zip: &str) -> Result<GeoResolution> {
        let data = self.load(zip).await?;
        Ok(GeoResolution {
            state: data.geo.state,
            state_name: data.geo.state_name,
            county: data.geo.county,
            city: data.geo.city,
            congressional_district: data.geo.congressional_district,
            state_senate_district: None,
            state_house_district: None,
            state_house_label: None,
            latitude: None,
            longitude: None,
            jurisdictions: data
                .geo
                .jurisdictions
                .into_iter()
                .map(|j| ResolvedJurisdiction {
                    ocd_id: j.ocd_id,
                    name: j.name,
                    level: j.level,
                    state: j.state,
                })
                .collect(),
            source_url: data.source.url.clone(),
            source_publisher: data.source.publisher.clone(),
        })
    }

    async fn fetch_ballot(&self, zip: &str, _geo: &GeoResolution) -> Result<BallotSnapshot> {
        let data = self.load(zip).await?;
        Ok(BallotSnapshot {
            election_name: data.election.name,
            election_date: data.election.election_date,
            election_scope: data.election.scope,
            candidates: data
                .candidates
                .into_iter()
                .map(|c| SnapshotCandidate {
                    office: c.office,
                    chamber: c.chamber,
                    jurisdiction_ocd: c.jurisdiction_ocd,
                    is_judicial: c.is_judicial,
                    name: c.name,
                    party: c.party,
                    is_incumbent: c.is_incumbent,
                    is_judge: c.is_judge,
                    summary: c.summary,
                    source_url: c.source_url,
                    source_publisher: c.source_publisher,
                    external_id: None,
                })
                .collect(),
            measures: data
                .measures
                .into_iter()
                .map(|m| SnapshotMeasure {
                    title: m.title,
                    measure_code: m.measure_code,
                    jurisdiction_ocd: m.jurisdiction_ocd,
                    summary: m.summary,
                    source_url: m.source_url,
                    source_publisher: m.source_publisher,
                })
                .collect(),
            source_url: data.source.url,
            source_publisher: data.source.publisher,
            coverage_note: Some("Fixture sample data (offline demo)".into()),
            extra_jurisdictions: vec![],
        })
    }
}
