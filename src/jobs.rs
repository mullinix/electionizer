use crate::providers::BallotProvider;
use crate::redact::redact_secrets;
use crate::store::Store;
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

pub struct Worker {
    store: Store,
    provider: Arc<dyn BallotProvider>,
    stage_delay: Duration,
}

impl Worker {
    pub fn new(store: Store, provider: Arc<dyn BallotProvider>, stage_delay_ms: u64) -> Self {
        Self {
            store,
            provider,
            stage_delay: Duration::from_millis(stage_delay_ms),
        }
    }

    pub async fn run_forever(self) {
        info!(provider = self.provider.name(), "job worker started");
        loop {
            match self.tick().await {
                Ok(true) => {}
                Ok(false) => sleep(Duration::from_millis(500)).await,
                Err(err) => {
                    error!(error = %redact_secrets(&format!("{err:#}")), "worker tick failed");
                    sleep(Duration::from_secs(2)).await;
                }
            }
        }
    }

    async fn tick(&self) -> Result<bool> {
        let Some(job) = self.store.claim_next_job().await? else {
            return Ok(false);
        };

        info!(job_id = %job.id, zip = %job.zip, "claimed build job");
        if let Err(err) = self.run_pipeline(&job.id, &job.zip).await {
            let msg = redact_secrets(&format!("{err:#}"));
            error!(job_id = %job.id, zip = %job.zip, error = %msg, "build failed");
            self.store.finish_job_err(&job.id, &msg).await?;
            if self.store.has_last_good(&job.zip).await? {
                // Keep prior ballot visible; record error on zip for banner.
                self.store
                    .restore_ready_after_refresh_failure(&job.zip, &msg)
                    .await?;
            } else {
                self.store
                    .set_zip_status(&job.zip, "failed", Some(&msg), false)
                    .await?;
            }
        }
        Ok(true)
    }

    async fn run_pipeline(&self, job_id: &str, zip: &str) -> Result<()> {
        self.store
            .update_job_progress(job_id, "validate", 10, "Validating ZIP")
            .await?;
        self.delay().await;

        self.store
            .update_job_progress(job_id, "geography", 30, "Resolving jurisdictions")
            .await?;
        self.delay().await;
        let geo = self.provider.resolve_zip(zip).await?;

        self.store
            .update_job_progress(
                job_id,
                "ballot",
                60,
                &format!(
                    "Fetching ballot for {}, {} ({}/{}, CD {})",
                    geo.city, geo.state_name, geo.state, geo.county, geo.congressional_district
                ),
            )
            .await?;
        self.delay().await;
        let ballot = self.provider.fetch_ballot(zip, &geo).await?;

        self.store
            .update_job_progress(job_id, "persist", 85, "Saving candidates, measures, sources")
            .await?;
        self.delay().await;
        self.store
            .persist_ballot(zip, &geo, &ballot, self.provider.name())
            .await?;

        self.store.finish_job_ok(job_id).await?;
        info!(
            job_id,
            zip,
            candidates = ballot.candidates.len(),
            measures = ballot.measures.len(),
            "build complete"
        );
        Ok(())
    }

    async fn delay(&self) {
        if !self.stage_delay.is_zero() {
            sleep(self.stage_delay).await;
        }
    }
}

pub async fn daemon_loop(store: Store, refresh_hours: i64, interval_secs: u64) {
    info!(
        refresh_hours,
        interval_secs, "daemon refresh loop started"
    );
    loop {
        match store.list_stale_zips(refresh_hours).await {
            Ok(zips) => {
                for zip in zips {
                    match store.enqueue_job(&zip).await {
                        Ok(job) => info!(zip = %zip, job_id = %job.id, "enqueued refresh"),
                        Err(err) => warn!(
                            zip = %zip,
                            error = %redact_secrets(&format!("{err:#}")),
                            "failed to enqueue refresh"
                        ),
                    }
                }
            }
            Err(err) => error!(
                error = %redact_secrets(&format!("{err:#}")),
                "list stale zips failed"
            ),
        }
        sleep(Duration::from_secs(interval_secs)).await;
    }
}
