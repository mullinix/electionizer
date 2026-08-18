mod config;
mod db;
mod detail;
mod fec;
mod fec_cache;
mod govtrack;
mod http_cache;
mod jobs;
mod models;
mod openstates;
mod providers;
mod redact;
mod store;
mod web;

use crate::config::{Config, Mode};
use crate::fec::FecClient;
use crate::fec_cache::FecCache;
use crate::govtrack::LegislatorClient;
use crate::http_cache::HttpCache;
use crate::jobs::{daemon_loop, Worker};
use crate::openstates::OpenStatesClient;
use crate::providers::box_provider;
use crate::store::Store;
use crate::web::{router, template_env, AppState};
use anyhow::Result;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cfg = Config::load()?;

    if cfg.fresh {
        tracing::warn!(db = %cfg.db.display(), "--fresh: wiping database before start");
        db::wipe_db_files(&cfg.db)?;
    }

    let pool = db::connect(&cfg.db).await?;
    let store = Store::new(pool.clone());
    let provider: Arc<dyn providers::BallotProvider> = Arc::from(box_provider(&cfg, &pool));
    let provider_name = provider.name().to_string();

    let key_preview = crate::config::mask_api_key(&cfg.fec_api_key_snapshot());
    tracing::info!(
        config = %cfg.config_path.display(),
        provider = %provider_name,
        fec_key = %key_preview,
        fec_cache_ttl_hours = cfg.fec_cache_ttl_hours,
        "config loaded"
    );

    let run_worker = matches!(cfg.mode, Mode::Both | Mode::Daemon | Mode::Serve);
    let run_daemon = matches!(cfg.mode, Mode::Both | Mode::Daemon);
    let run_serve = matches!(cfg.mode, Mode::Both | Mode::Serve);

    if run_worker {
        let worker = Worker::new(store.clone(), provider.clone(), cfg.stage_delay_ms);
        tokio::spawn(async move {
            worker.run_forever().await;
        });
    }

    if run_daemon {
        let store_d = store.clone();
        let hours = cfg.refresh_hours;
        let interval = cfg.daemon_interval_secs;
        tokio::spawn(async move {
            daemon_loop(store_d, hours, interval).await;
        });
    }

    if run_serve {
        let fec = FecClient::new(
            cfg.fec_api_key.clone(),
            cfg.cycle,
            FecCache::new(pool.clone(), cfg.fec_cache_ttl_hours),
        );
        let http_cache = HttpCache::new(pool.clone(), cfg.fec_cache_ttl_hours);
        let legislators = LegislatorClient::new(http_cache.clone());
        let openstates = OpenStatesClient::new(cfg.openstates_api_key.clone(), http_cache);
        let state = AppState {
            store,
            templates: template_env(),
            provider_name: provider_name.clone(),
            config: cfg.clone(),
            fec,
            legislators,
            openstates,
        };
        let app = router(state);
        let listener = tokio::net::TcpListener::bind(&cfg.bind).await?;
        tracing::info!(bind = %cfg.bind, provider = %provider_name, "listening");
        axum::serve(listener, app).await?;
    } else {
        tracing::info!("daemon-only mode");
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
        }
    }

    Ok(())
}
