mod fixture;
mod live;
mod states;

pub use fixture::FixtureProvider;
pub use live::LiveProvider;

use crate::fec_cache::FecCache;
use crate::http_cache::HttpCache;
use crate::models::{BallotSnapshot, GeoResolution};
use anyhow::Result;
use async_trait::async_trait;
use sqlx::SqlitePool;

#[async_trait]
pub trait BallotProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn resolve_zip(&self, zip: &str) -> Result<GeoResolution>;
    async fn fetch_ballot(&self, zip: &str, geo: &GeoResolution) -> Result<BallotSnapshot>;
}

pub fn box_provider(cfg: &crate::config::Config, pool: &SqlitePool) -> Box<dyn BallotProvider> {
    match cfg.provider {
        crate::config::ProviderKind::Fixture => {
            Box::new(FixtureProvider::new(cfg.fixture_dir.clone()))
        }
        crate::config::ProviderKind::Live => {
            let cache = FecCache::new(pool.clone(), cfg.fec_cache_ttl_hours);
            let http_cache = HttpCache::new(pool.clone(), cfg.fec_cache_ttl_hours);
            Box::new(LiveProvider::new(
                cfg.fec_api_key.clone(),
                cfg.openstates_api_key.clone(),
                cfg.cycle,
                cache,
                http_cache,
            ))
        }
    }
}
