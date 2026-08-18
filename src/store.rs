use crate::models::{
    build_ballot_sections, build_office_groups, election_year_from, judicial_explainer_for, now_str,
    voter_portal_for_state, BallotCandidateRow,
    BallotReport, BallotSnapshot, BuildJob, CandidateDetail, GeoResolution, MeasureDetail,
    MeasureSummary, SourceInfo, ZipRow,
};
use anyhow::{Context, Result};
use sqlx::{Sqlite, SqlitePool, Transaction};
use uuid::Uuid;

#[derive(Clone)]
pub struct Store {
    pool: SqlitePool,
}

impl Store {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_zip(&self, zip: &str) -> Result<Option<ZipRow>> {
        let row = sqlx::query_as::<_, ZipRow>(
            "SELECT zip, status, last_built_at, error, coverage_note FROM zips WHERE zip = ?",
        )
        .bind(zip)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn ensure_zip(&self, zip: &str) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO zips (zip, status, updated_at)
            VALUES (?, 'pending', datetime('now'))
            ON CONFLICT(zip) DO NOTHING
            "#,
        )
        .bind(zip)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn set_zip_status(
        &self,
        zip: &str,
        status: &str,
        error: Option<&str>,
        mark_built: bool,
    ) -> Result<()> {
        if mark_built {
            sqlx::query(
                r#"
                UPDATE zips
                SET status = ?, error = ?, last_built_at = datetime('now'), updated_at = datetime('now')
                WHERE zip = ?
                "#,
            )
            .bind(status)
            .bind(error)
            .bind(zip)
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query(
                r#"
                UPDATE zips
                SET status = ?, error = ?, updated_at = datetime('now')
                WHERE zip = ?
                "#,
            )
            .bind(status)
            .bind(error)
            .bind(zip)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    pub async fn enqueue_job(&self, zip: &str) -> Result<BuildJob> {
        // Reuse active job if one exists
        if let Some(existing) = sqlx::query_as::<_, BuildJob>(
            r#"
            SELECT id, zip, status, stage, progress_pct, message, error, created_at, updated_at, finished_at
            FROM build_jobs
            WHERE zip = ? AND status IN ('queued', 'running')
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(zip)
        .fetch_optional(&self.pool)
        .await?
        {
            return Ok(existing);
        }

        let id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO build_jobs (id, zip, status, stage, progress_pct, message)
            VALUES (?, ?, 'queued', 'queued', 0, 'Waiting to start')
            "#,
        )
        .bind(&id)
        .bind(zip)
        .execute(&self.pool)
        .await?;

        self.ensure_zip(zip).await?;
        self.set_zip_status(zip, "building", None, false).await?;

        self.get_job(&id)
            .await?
            .context("job missing after insert")
    }

    pub async fn get_job(&self, id: &str) -> Result<Option<BuildJob>> {
        let row = sqlx::query_as::<_, BuildJob>(
            r#"
            SELECT id, zip, status, stage, progress_pct, message, error, created_at, updated_at, finished_at
            FROM build_jobs WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    /// Active queued/running job for a ZIP, if any.
    pub async fn active_job_for_zip(&self, zip: &str) -> Result<Option<BuildJob>> {
        let row = sqlx::query_as::<_, BuildJob>(
            r#"
            SELECT id, zip, status, stage, progress_pct, message, error, created_at, updated_at, finished_at
            FROM build_jobs
            WHERE zip = ? AND status IN ('queued', 'running')
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(zip)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    /// True when this ZIP previously built successfully (last-good report may still be served).
    pub async fn has_last_good(&self, zip: &str) -> Result<bool> {
        let row: Option<(Option<String>,)> = sqlx::query_as(
            "SELECT last_built_at FROM zips WHERE zip = ?",
        )
        .bind(zip)
        .fetch_optional(&self.pool)
        .await?;
        Ok(matches!(row, Some((Some(ref t),)) if !t.is_empty()))
    }

    /// After a failed refresh, keep serving the prior report.
    pub async fn restore_ready_after_refresh_failure(
        &self,
        zip: &str,
        error: &str,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE zips
            SET status = 'ready',
                error = ?,
                updated_at = datetime('now')
            WHERE zip = ?
              AND last_built_at IS NOT NULL
            "#,
        )
        .bind(error)
        .bind(zip)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn claim_next_job(&self) -> Result<Option<BuildJob>> {
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query_as::<_, BuildJob>(
            r#"
            SELECT id, zip, status, stage, progress_pct, message, error, created_at, updated_at, finished_at
            FROM build_jobs
            WHERE status = 'queued'
            ORDER BY created_at ASC
            LIMIT 1
            "#,
        )
        .fetch_optional(&mut *tx)
        .await?;

        let Some(job) = row else {
            tx.commit().await?;
            return Ok(None);
        };

        sqlx::query(
            r#"
            UPDATE build_jobs
            SET status = 'running', stage = 'starting', message = 'Starting build',
                progress_pct = 5, updated_at = datetime('now')
            WHERE id = ? AND status = 'queued'
            "#,
        )
        .bind(&job.id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        self.get_job(&job.id).await
    }

    pub async fn update_job_progress(
        &self,
        id: &str,
        stage: &str,
        progress_pct: i64,
        message: &str,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE build_jobs
            SET stage = ?, progress_pct = ?, message = ?, updated_at = datetime('now')
            WHERE id = ?
            "#,
        )
        .bind(stage)
        .bind(progress_pct)
        .bind(message)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn finish_job_ok(&self, id: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE build_jobs
            SET status = 'ready', stage = 'done', progress_pct = 100,
                message = 'Build complete', finished_at = datetime('now'), updated_at = datetime('now')
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn finish_job_err(&self, id: &str, error: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE build_jobs
            SET status = 'failed', stage = 'failed', message = 'Build failed',
                error = ?, finished_at = datetime('now'), updated_at = datetime('now')
            WHERE id = ?
            "#,
        )
        .bind(error)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_stale_zips(&self, older_than_hours: i64) -> Result<Vec<String>> {
        let modifier = format!("-{older_than_hours} hours");
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT zip FROM zips
            WHERE status = 'ready'
              AND (
                last_built_at IS NULL
                OR datetime(last_built_at) <= datetime('now', ?)
              )
            "#,
        )
        .bind(modifier)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|(z,)| z).collect())
    }

    pub async fn persist_ballot(
        &self,
        zip: &str,
        geo: &GeoResolution,
        ballot: &BallotSnapshot,
        provider: &str,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        let retrieved = now_str();

        // Clear prior links for this zip (keep global entities if shared later; thin slice is per-zip rebuild)
        sqlx::query("DELETE FROM zip_jurisdictions WHERE zip = ?")
            .bind(zip)
            .execute(&mut *tx)
            .await?;

        let mut jur_ids = std::collections::HashMap::<String, i64>::new();
        let all_jurs: Vec<&crate::models::ResolvedJurisdiction> = geo
            .jurisdictions
            .iter()
            .chain(ballot.extra_jurisdictions.iter())
            .collect();
        for j in all_jurs {
            if jur_ids.contains_key(&j.ocd_id) {
                continue;
            }
            let id: i64 = sqlx::query_scalar(
                r#"
                INSERT INTO jurisdictions (ocd_id, name, level, state)
                VALUES (?, ?, ?, ?)
                ON CONFLICT(ocd_id) DO UPDATE SET name = excluded.name, level = excluded.level, state = excluded.state
                RETURNING id
                "#,
            )
            .bind(&j.ocd_id)
            .bind(&j.name)
            .bind(&j.level)
            .bind(&j.state)
            .fetch_one(&mut *tx)
            .await?;

            sqlx::query(
                "INSERT OR IGNORE INTO zip_jurisdictions (zip, jurisdiction_id) VALUES (?, ?)",
            )
            .bind(zip)
            .bind(id)
            .execute(&mut *tx)
            .await?;

            jur_ids.insert(j.ocd_id.clone(), id);
        }

        let election_id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO elections (name, election_date, scope)
            VALUES (?, ?, ?)
            ON CONFLICT(name, election_date) DO UPDATE SET scope = excluded.scope
            RETURNING id
            "#,
        )
        .bind(&ballot.election_name)
        .bind(&ballot.election_date)
        .bind(&ballot.election_scope)
        .fetch_one(&mut *tx)
        .await?;

        let geo_source_id =
            insert_source(&mut tx, &geo.source_url, Some(&geo.source_publisher), &retrieved, None)
                .await?;
        let ballot_source_id = insert_source(
            &mut tx,
            &ballot.source_url,
            Some(&ballot.source_publisher),
            &retrieved,
            Some(&format!("provider={provider}")),
        )
        .await?;

        // Remove prior candidates/measures for races in this election tied to these jurisdictions
        // Thin-slice approach: delete candidates for races under this election that we will rewrite
        for c in &ballot.candidates {
            let Some(&jur_id) = jur_ids.get(&c.jurisdiction_ocd) else {
                tracing::warn!(ocd = %c.jurisdiction_ocd, "unknown jurisdiction for candidate");
                continue;
            };

            let race_id: i64 = sqlx::query_scalar(
                r#"
                INSERT INTO races (election_id, jurisdiction_id, office, chamber, is_judicial)
                VALUES (?, ?, ?, ?, ?)
                ON CONFLICT(election_id, jurisdiction_id, office) DO UPDATE SET
                    chamber = excluded.chamber,
                    is_judicial = excluded.is_judicial
                RETURNING id
                "#,
            )
            .bind(election_id)
            .bind(jur_id)
            .bind(&c.office)
            .bind(&c.chamber)
            .bind(c.is_judicial as i64)
            .fetch_one(&mut *tx)
            .await?;

            let cand_id: i64 = sqlx::query_scalar(
                r#"
                INSERT INTO candidates (race_id, name, party, is_incumbent, is_judge, summary, external_id, updated_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, datetime('now'))
                ON CONFLICT(race_id, name) DO UPDATE SET
                    party = excluded.party,
                    is_incumbent = excluded.is_incumbent,
                    is_judge = excluded.is_judge,
                    summary = excluded.summary,
                    external_id = COALESCE(excluded.external_id, candidates.external_id),
                    updated_at = datetime('now')
                RETURNING id
                "#,
            )
            .bind(race_id)
            .bind(&c.name)
            .bind(&c.party)
            .bind(c.is_incumbent as i64)
            .bind(c.is_judge as i64)
            .bind(&c.summary)
            .bind(&c.external_id)
            .fetch_one(&mut *tx)
            .await?;

            let src = insert_source(
                &mut tx,
                &c.source_url,
                c.source_publisher.as_deref(),
                &retrieved,
                None,
            )
            .await?;
            link_source(&mut tx, "candidate", cand_id, src).await?;
            link_source(&mut tx, "candidate", cand_id, ballot_source_id).await?;
            link_source(&mut tx, "candidate", cand_id, geo_source_id).await?;
        }

        for m in &ballot.measures {
            let Some(&jur_id) = jur_ids.get(&m.jurisdiction_ocd) else {
                tracing::warn!(ocd = %m.jurisdiction_ocd, "unknown jurisdiction for measure");
                continue;
            };

            let measure_id: i64 = sqlx::query_scalar(
                r#"
                INSERT INTO ballot_measures (election_id, jurisdiction_id, title, summary, measure_code, updated_at)
                VALUES (?, ?, ?, ?, ?, datetime('now'))
                ON CONFLICT(election_id, jurisdiction_id, title) DO UPDATE SET
                    summary = excluded.summary,
                    measure_code = excluded.measure_code,
                    updated_at = datetime('now')
                RETURNING id
                "#,
            )
            .bind(election_id)
            .bind(jur_id)
            .bind(&m.title)
            .bind(&m.summary)
            .bind(&m.measure_code)
            .fetch_one(&mut *tx)
            .await?;

            let src = insert_source(
                &mut tx,
                &m.source_url,
                m.source_publisher.as_deref(),
                &retrieved,
                None,
            )
            .await?;
            link_source(&mut tx, "measure", measure_id, src).await?;
            link_source(&mut tx, "measure", measure_id, ballot_source_id).await?;
        }

        sqlx::query(
            r#"
            INSERT INTO scrape_runs (provider, zip, started_at, finished_at, status, stats_json)
            VALUES (?, ?, datetime('now'), datetime('now'), 'ok', ?)
            "#,
        )
        .bind(provider)
        .bind(zip)
        .bind(serde_json::json!({
            "candidates": ballot.candidates.len(),
            "measures": ballot.measures.len(),
            "jurisdictions": geo.jurisdictions.len(),
        }).to_string())
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            UPDATE zips
            SET status = 'ready', error = NULL, last_built_at = datetime('now'),
                updated_at = datetime('now'), coverage_note = ?
            WHERE zip = ?
            "#,
        )
        .bind(&ballot.coverage_note)
        .bind(zip)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn ballot_report(&self, zip: &str) -> Result<Option<BallotReport>> {
        let Some(z) = self.get_zip(zip).await? else {
            return Ok(None);
        };

        let candidates = sqlx::query_as::<_, CandidateQueryRow>(
            r#"
            SELECT c.id, c.name, c.party, c.is_incumbent, c.is_judge, c.summary, c.external_id,
                   r.office, r.chamber, j.name AS jurisdiction
            FROM candidates c
            JOIN races r ON r.id = c.race_id
            JOIN jurisdictions j ON j.id = r.jurisdiction_id
            JOIN zip_jurisdictions zj ON zj.jurisdiction_id = j.id
            WHERE zj.zip = ?
            ORDER BY r.office, c.name
            "#,
        )
        .bind(zip)
        .fetch_all(&self.pool)
        .await?;

        let measures = sqlx::query_as::<_, MeasureQueryRow>(
            r#"
            SELECT m.id, m.title, m.measure_code, m.summary, j.name AS jurisdiction
            FROM ballot_measures m
            JOIN jurisdictions j ON j.id = m.jurisdiction_id
            JOIN zip_jurisdictions zj ON zj.jurisdiction_id = j.id
            WHERE zj.zip = ?
            ORDER BY m.title
            "#,
        )
        .bind(zip)
        .fetch_all(&self.pool)
        .await?;

        let election = sqlx::query_as::<_, (String, String)>(
            r#"
            SELECT e.name, e.election_date
            FROM elections e
            JOIN races r ON r.election_id = e.id
            JOIN jurisdictions j ON j.id = r.jurisdiction_id
            JOIN zip_jurisdictions zj ON zj.jurisdiction_id = j.id
            WHERE zj.zip = ?
            LIMIT 1
            "#,
        )
        .bind(zip)
        .fetch_optional(&self.pool)
        .await?;

        let geo_parts: Vec<(String, String)> = sqlx::query_as(
            r#"
            SELECT j.level, j.name
            FROM jurisdictions j
            JOIN zip_jurisdictions zj ON zj.jurisdiction_id = j.id
            WHERE zj.zip = ?
            ORDER BY
              CASE j.level
                WHEN 'federal' THEN 0
                WHEN 'state' THEN 1
                WHEN 'congressional' THEN 2
                WHEN 'state_senate' THEN 3
                WHEN 'state_house' THEN 4
                WHEN 'county' THEN 5
                WHEN 'municipal' THEN 6
                ELSE 7
              END
            "#,
        )
        .bind(zip)
        .fetch_all(&self.pool)
        .await?;

        let has_state_jurisdiction = geo_parts.iter().any(|(level, _)| level == "state");
        let state_label = geo_parts
            .iter()
            .find(|(level, _)| level == "state")
            .map(|(_, name)| name.as_str());
        let geo_summary = {
            let interesting: Vec<&str> = geo_parts
                .iter()
                .filter(|(level, _)| {
                    matches!(
                        level.as_str(),
                        "state"
                            | "congressional"
                            | "state_senate"
                            | "state_house"
                            | "county"
                            | "municipal"
                    )
                })
                .map(|(_, name)| name.as_str())
                .collect();
            if interesting.is_empty() {
                None
            } else {
                Some(interesting.join(" · "))
            }
        };
        let election_name = election.as_ref().map(|e| e.0.clone());
        let election_date = election.as_ref().map(|e| e.1.clone());
        let year = election_year_from(election_name.as_deref(), election_date.as_deref());

        let rows: Vec<BallotCandidateRow> = candidates
            .into_iter()
            .map(|c| BallotCandidateRow {
                id: c.id,
                name: c.name,
                party: c.party,
                is_incumbent: c.is_incumbent != 0,
                is_judge: c.is_judge != 0,
                office: c.office,
                chamber: c.chamber,
                jurisdiction: c.jurisdiction,
                external_id: c.external_id,
                summary: c.summary,
                source_url: None,
                source_publisher: None,
            })
            .collect();
        let office_groups = build_office_groups(rows, has_state_jurisdiction, year, state_label);
        let judicial_explainer = judicial_explainer_for(&office_groups);
        let ballot_sections = build_ballot_sections(&office_groups);

        let state_code: Option<String> = sqlx::query_scalar(
            r#"
            SELECT j.state
            FROM jurisdictions j
            JOIN zip_jurisdictions zj ON zj.jurisdiction_id = j.id
            WHERE zj.zip = ? AND j.state IS NOT NULL AND j.state != ''
            LIMIT 1
            "#,
        )
        .bind(zip)
        .fetch_optional(&self.pool)
        .await?
        .flatten();
        let voter_portal = state_code
            .as_deref()
            .map(voter_portal_for_state)
            .or_else(|| Some(voter_portal_for_state("")));

        Ok(Some(BallotReport {
            zip: z.zip,
            status: z.status,
            last_built_at: z.last_built_at,
            coverage_note: z.coverage_note,
            election_name,
            election_date,
            geo_summary,
            state_code,
            voter_portal,
            office_groups,
            ballot_sections,
            judicial_explainer,
            measures: measures
                .into_iter()
                .map(|m| MeasureSummary {
                    id: m.id,
                    title: m.title,
                    measure_code: m.measure_code,
                    summary: m.summary,
                    jurisdiction: m.jurisdiction,
                    source_url: None,
                })
                .collect(),
        }))
    }

    pub async fn candidate_detail(&self, id: i64) -> Result<Option<CandidateDetail>> {
        let row = sqlx::query_as::<_, CandidateDetailRow>(
            r#"
            SELECT c.id, c.name, c.party, c.is_incumbent, c.is_judge, c.summary, c.external_id,
                   r.office, r.chamber, j.name AS jurisdiction, j.state AS state_code,
                   e.name AS election_name, e.election_date
            FROM candidates c
            JOIN races r ON r.id = c.race_id
            JOIN jurisdictions j ON j.id = r.jurisdiction_id
            JOIN elections e ON e.id = r.election_id
            WHERE c.id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(c) = row else {
            return Ok(None);
        };

        let sources = entity_sources(&self.pool, "candidate", id).await?;
        let external_id = c.external_id.filter(|s| !s.is_empty()).or_else(|| {
            sources.iter().find_map(|s| extract_fec_id_from_url(&s.url))
        });
        let zip: Option<String> = sqlx::query_scalar(
            r#"
            SELECT zj.zip
            FROM zip_jurisdictions zj
            JOIN races r ON r.jurisdiction_id = zj.jurisdiction_id
            JOIN candidates c ON c.race_id = r.id
            WHERE c.id = ?
            ORDER BY zj.zip
            LIMIT 1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(Some(CandidateDetail {
            id: c.id,
            name: c.name,
            party: c.party,
            office: c.office,
            chamber: c.chamber,
            is_incumbent: c.is_incumbent != 0,
            is_judge: c.is_judge != 0,
            summary: c.summary,
            jurisdiction: c.jurisdiction,
            state_code: c.state_code.filter(|s| !s.is_empty()),
            election_name: c.election_name,
            election_date: c.election_date,
            external_id,
            zip,
            sources,
        }))
    }

    pub async fn measure_detail(&self, id: i64) -> Result<Option<MeasureDetail>> {
        let row = sqlx::query_as::<_, MeasureDetailRow>(
            r#"
            SELECT m.id, m.title, m.measure_code, m.summary,
                   j.name AS jurisdiction, e.name AS election_name, e.election_date
            FROM ballot_measures m
            JOIN jurisdictions j ON j.id = m.jurisdiction_id
            JOIN elections e ON e.id = m.election_id
            WHERE m.id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(m) = row else {
            return Ok(None);
        };

        let sources = entity_sources(&self.pool, "measure", id).await?;
        Ok(Some(MeasureDetail {
            id: m.id,
            title: m.title,
            measure_code: m.measure_code,
            summary: m.summary,
            jurisdiction: m.jurisdiction,
            election_name: m.election_name,
            election_date: m.election_date,
            sources,
        }))
    }
}

#[derive(sqlx::FromRow)]
struct CandidateQueryRow {
    id: i64,
    name: String,
    party: String,
    is_incumbent: i64,
    is_judge: i64,
    summary: Option<String>,
    external_id: Option<String>,
    office: String,
    chamber: Option<String>,
    jurisdiction: String,
}

#[derive(sqlx::FromRow)]
struct MeasureQueryRow {
    id: i64,
    title: String,
    measure_code: Option<String>,
    summary: Option<String>,
    jurisdiction: String,
}

#[derive(sqlx::FromRow)]
struct CandidateDetailRow {
    id: i64,
    name: String,
    party: String,
    is_incumbent: i64,
    is_judge: i64,
    summary: Option<String>,
    external_id: Option<String>,
    office: String,
    chamber: Option<String>,
    jurisdiction: String,
    state_code: Option<String>,
    election_name: String,
    election_date: String,
}

fn extract_fec_id_from_url(url: &str) -> Option<String> {
    // https://www.fec.gov/data/candidate/H8FL08042/
    let marker = "/candidate/";
    let idx = url.find(marker)?;
    let rest = &url[idx + marker.len()..];
    let id: String = rest
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric())
        .collect();
    if id.len() >= 8 {
        Some(id)
    } else {
        None
    }
}

#[derive(sqlx::FromRow)]
struct MeasureDetailRow {
    id: i64,
    title: String,
    measure_code: Option<String>,
    summary: Option<String>,
    jurisdiction: String,
    election_name: String,
    election_date: String,
}

async fn insert_source(
    tx: &mut Transaction<'_, Sqlite>,
    url: &str,
    publisher: Option<&str>,
    retrieved_at: &str,
    note: Option<&str>,
) -> Result<i64> {
    let id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO sources (url, publisher, retrieved_at, note)
        VALUES (?, ?, ?, ?)
        ON CONFLICT(url, retrieved_at) DO UPDATE SET publisher = COALESCE(excluded.publisher, sources.publisher)
        RETURNING id
        "#,
    )
    .bind(url)
    .bind(publisher)
    .bind(retrieved_at)
    .bind(note)
    .fetch_one(&mut **tx)
    .await?;
    Ok(id)
}

async fn link_source(
    tx: &mut Transaction<'_, Sqlite>,
    entity_type: &str,
    entity_id: i64,
    source_id: i64,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT OR IGNORE INTO entity_sources (entity_type, entity_id, source_id)
        VALUES (?, ?, ?)
        "#,
    )
    .bind(entity_type)
    .bind(entity_id)
    .bind(source_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn entity_sources(pool: &SqlitePool, entity_type: &str, entity_id: i64) -> Result<Vec<SourceInfo>> {
    let rows = sqlx::query_as::<_, SourceInfoRow>(
        r#"
        SELECT s.url, s.publisher, s.retrieved_at, s.note
        FROM sources s
        JOIN entity_sources es ON es.source_id = s.id
        WHERE es.entity_type = ? AND es.entity_id = ?
        ORDER BY s.retrieved_at DESC, s.id DESC
        "#,
    )
    .bind(entity_type)
    .bind(entity_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| SourceInfo {
            url: r.url,
            publisher: r.publisher,
            retrieved_at: r.retrieved_at,
            note: r.note,
        })
        .collect())
}

#[derive(sqlx::FromRow)]
struct SourceInfoRow {
    url: String,
    publisher: Option<String>,
    retrieved_at: String,
    note: Option<String>,
}
