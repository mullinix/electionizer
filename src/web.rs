use crate::config::{is_demo_key, mask_api_key, Config};
use crate::detail::{enrich_candidate, plan_stages};
use crate::fec::FecClient;
use crate::govtrack::LegislatorClient;
use crate::models::normalize_zip;
use crate::openstates::OpenStatesClient;
use crate::redact::redact_secrets;
use crate::store::Store;
use axum::extract::{Form, Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::Router;
use futures_util::stream::Stream;
use minijinja::{context, Environment};
use serde::Deserialize;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct AppState {
    pub store: Store,
    pub templates: Environment<'static>,
    pub provider_name: String,
    pub config: Config,
    pub fec: FecClient,
    pub legislators: LegislatorClient,
    pub openstates: OpenStatesClient,
}

/// Fixture-built rows must not be served while running the live provider.
fn needs_rebuild(provider_name: &str, coverage_note: Option<&str>) -> bool {
    if provider_name != "live" {
        return false;
    }
    match coverage_note {
        None => false,
        Some(note) => {
            let n = note.to_ascii_lowercase();
            n.contains("fixture") || n.contains("offline demo")
        }
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(home).post(submit_zip))
        .route("/ballot/:zip", get(ballot_page))
        .route("/candidate/:id", get(candidate_page))
        .route("/candidate/:id/stream", get(candidate_stream))
        .route("/measure/:id", get(measure_page))
        .route("/jobs/:id", get(job_status))
        .route("/settings", get(settings_page).post(settings_save))
        .route("/health", get(|| async { "ok" }))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(Arc::new(state))
}

pub fn template_env() -> Environment<'static> {
    let mut env = Environment::new();
    env.set_loader(minijinja::path_loader("templates"));
    env
}

fn render(state: &AppState, name: &str, ctx: minijinja::value::Value) -> Result<Html<String>, AppError> {
    let tmpl = state
        .templates
        .get_template(name)
        .map_err(|e| AppError::internal(e.to_string()))?;
    let body = tmpl
        .render(ctx)
        .map_err(|e| AppError::internal(e.to_string()))?;
    Ok(Html(body))
}

async fn home(State(state): State<Arc<AppState>>) -> Result<Html<String>, AppError> {
    let key = state.config.fec_api_key_snapshot();
    let demo_warning = is_demo_key(&key) && state.provider_name == "live";
    render(
        &state,
        "home.html",
        context! {
            title => "electionizer",
            error => Option::<String>::None,
            demo_warning => demo_warning,
        },
    )
}

#[derive(Deserialize)]
struct ZipForm {
    zip: String,
}

async fn submit_zip(
    State(state): State<Arc<AppState>>,
    Form(form): Form<ZipForm>,
) -> Result<Response, AppError> {
    let Some(zip) = normalize_zip(&form.zip) else {
        let key = state.config.fec_api_key_snapshot();
        let html = render(
            &state,
            "home.html",
            context! {
                title => "electionizer",
                error => "Enter a valid 5-digit US ZIP code.",
                demo_warning => is_demo_key(&key) && state.provider_name == "live",
            },
        )?;
        return Ok(html.into_response());
    };

    if let Some(z) = state.store.get_zip(&zip).await? {
        if z.status == "ready" && !needs_rebuild(&state.provider_name, z.coverage_note.as_deref()) {
            return Ok(Redirect::to(&format!("/ballot/{zip}")).into_response());
        }
        // Refresh / rebuild with last-good: stay on ballot, not blank progress page.
        if state.store.has_last_good(&zip).await? {
            let _job = state.store.enqueue_job(&zip).await?;
            return Ok(Redirect::to(&format!("/ballot/{zip}")).into_response());
        }
    }

    let job = state.store.enqueue_job(&zip).await?;
    Ok(Redirect::to(&format!("/jobs/{}", job.id)).into_response())
}

async fn ballot_page(
    State(state): State<Arc<AppState>>,
    Path(zip): Path<String>,
) -> Result<Response, AppError> {
    let Some(zip) = normalize_zip(&zip) else {
        return Ok(StatusCode::BAD_REQUEST.into_response());
    };

    let Some(z) = state.store.get_zip(&zip).await? else {
        let job = state.store.enqueue_job(&zip).await?;
        return Ok(Redirect::to(&format!("/jobs/{}", job.id)).into_response());
    };

    let rebuild = needs_rebuild(&state.provider_name, z.coverage_note.as_deref());
    let last_good = z.last_built_at.as_ref().is_some_and(|t| !t.is_empty());

    let mut refresh_job = state.store.active_job_for_zip(&zip).await?;

    if z.status != "ready" || rebuild {
        if z.status == "ready" && rebuild {
            tracing::info!(zip = %zip, "stale fixture cache; rebuilding with live provider");
        }
        if refresh_job.is_none() {
            refresh_job = Some(state.store.enqueue_job(&zip).await?);
        }
        if !last_good {
            let job_id = refresh_job
                .as_ref()
                .map(|j| j.id.clone())
                .unwrap_or_default();
            return Ok(Redirect::to(&format!("/jobs/{job_id}")).into_response());
        }
    }

    let report = state
        .store
        .ballot_report(&zip)
        .await?
        .ok_or_else(|| AppError::internal("expected last-good ballot report"))?;

    let refreshing = refresh_job
        .as_ref()
        .is_some_and(|j| j.status == "queued" || j.status == "running");
    // Don't show stale error while a new refresh is already in flight.
    let show_refresh_error = if refreshing {
        None
    } else {
        z.error.clone()
    };

    let html = render(
        &state,
        "ballot.html",
        context! {
            title => format!("Ballot · {zip}"),
            report => report,
            refreshing => refreshing,
            refresh_job => refresh_job,
            last_refresh_error => show_refresh_error,
        },
    )?;
    Ok(html.into_response())
}

/// Instant shell: profile from DB + staged progress panel (SSE fills the rest).
async fn candidate_page(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    let Some(detail) = state.store.candidate_detail(id).await? else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };
    let stages = plan_stages(&detail, state.openstates.has_key());
    let html = render(
        &state,
        "candidate.html",
        context! {
            title => detail.name.clone(),
            c => detail,
            stages => stages,
            stream_url => format!("/candidate/{id}/stream"),
        },
    )?;
    Ok(html.into_response())
}

/// SSE: real enrichment stages, then full article HTML.
async fn candidate_stream(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Event, Infallible>>(64);
    let state = state.clone();

    tokio::spawn(async move {
        let send = |tx: &tokio::sync::mpsc::Sender<Result<Event, Infallible>>, ev: Event| {
            let tx = tx.clone();
            async move {
                let _ = tx.send(Ok(ev)).await;
            }
        };

        let Some(detail) = (match state.store.candidate_detail(id).await {
            Ok(d) => d,
            Err(err) => {
                let msg = redact_secrets(&format!("{err:#}"));
                send(
                    &tx,
                    Event::default()
                        .event("fail")
                        .data(serde_json::json!({ "message": msg }).to_string()),
                )
                .await;
                return;
            }
        }) else {
            send(
                &tx,
                Event::default()
                    .event("fail")
                    .data(r#"{"message":"candidate not found"}"#),
            )
            .await;
            return;
        };

        let stages = plan_stages(&detail, state.openstates.has_key());
        let plan = serde_json::json!({
            "total": stages.len(),
            "stages": stages.iter().map(|s| serde_json::json!({
                "id": s.id,
                "label": s.label,
            })).collect::<Vec<_>>(),
        });
        send(&tx, Event::default().event("plan").data(plan.to_string())).await;

        let tx_stage = tx.clone();
        let enrich = enrich_candidate(
            &detail,
            &state.fec,
            &state.legislators,
            &state.openstates,
            state.config.cycle,
            move |u| {
                let payload = serde_json::to_string(&u).unwrap_or_else(|_| "{}".into());
                let _ = tx_stage.try_send(Ok(Event::default().event("stage").data(payload)));
            },
        )
        .await;

        let article = match state.templates.get_template("partials/candidate_article.html") {
            Ok(tmpl) => tmpl
                .render(context! {
                    c => &detail,
                    finance => enrich.finance,
                    finance_error => enrich.finance_error,
                    finance_unavailable => enrich.finance_unavailable,
                    finance_cycle => state.fec.cycle(),
                    outside_spending => enrich.outside_spending,
                    top_individuals => enrich.top_individuals,
                    top_committees => enrich.top_committees,
                    size_buckets => enrich.size_buckets,
                    principal_committee => enrich.principal_committee,
                    votes => enrich.votes,
                    votes_url => enrich.votes_url,
                    votes_source => enrich.votes_source,
                    votes_rate_limited => enrich.votes_rate_limited,
                    openstates_configured => state.openstates.has_key(),
                    affiliations => enrich.affiliations,
                    affiliations_source => enrich.affiliations_source,
                })
                .unwrap_or_else(|e| {
                    format!(
                        r#"<p class="error">Render failed: {}</p>"#,
                        redact_secrets(&e.to_string())
                    )
                }),
            Err(e) => format!(
                r#"<p class="error">Template missing: {}</p>"#,
                redact_secrets(&e.to_string())
            ),
        };

        let html_payload = serde_json::json!({ "html": article }).to_string();
        send(
            &tx,
            Event::default().event("html").data(html_payload),
        )
        .await;
        send(&tx, Event::default().event("done").data("{}")).await;
    });

    let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("ping"),
    )
}

async fn measure_page(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    let Some(detail) = state.store.measure_detail(id).await? else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };
    let html = render(
        &state,
        "measure.html",
        context! {
            title => detail.title.clone(),
            m => detail,
        },
    )?;
    Ok(html.into_response())
}

async fn job_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let Some(job) = state.store.get_job(&id).await? else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    let hx = headers
        .get("hx-request")
        .and_then(|v| v.to_str().ok())
        .is_some();

    if job.status == "ready" && !hx {
        return Ok(Redirect::to(&format!("/ballot/{}", job.zip)).into_response());
    }

    let rate_limited = job
        .error
        .as_deref()
        .is_some_and(|e| e.contains("429") || e.to_ascii_lowercase().contains("rate limit"));

    let is_refresh = state.store.has_last_good(&job.zip).await?;

    if hx {
        if job.status == "ready" || (job.status == "failed" && is_refresh) {
            let mut res = render(
                &state,
                "partials/job_status.html",
                context! {
                    job => &job,
                    rate_limited => false,
                    compact => true,
                },
            )?
            .into_response();
            res.headers_mut().insert(
                header::HeaderName::from_static("hx-redirect"),
                header::HeaderValue::from_str(&format!("/ballot/{}", job.zip)).unwrap(),
            );
            return Ok(res);
        }
        let html = render(
            &state,
            "partials/job_status.html",
            context! {
                job => job,
                rate_limited => rate_limited,
                compact => true,
            },
        )?;
        return Ok(html.into_response());
    }

    let html = render(
        &state,
        "progress.html",
        context! {
            title => format!("Building · {}", job.zip),
            job => job,
            rate_limited => rate_limited,
            is_refresh => is_refresh,
        },
    )?;
    Ok(html.into_response())
}

fn settings_context(
    state: &AppState,
    message: Option<String>,
    error: Option<String>,
) -> minijinja::value::Value {
    let fec = state.config.fec_api_key_snapshot();
    let os = state.config.openstates_api_key_snapshot();
    context! {
        title => "Settings",
        config_path => state.config.config_path.display().to_string(),
        masked_key => mask_api_key(&fec),
        is_demo => is_demo_key(&fec),
        openstates_masked => if os.is_empty() { "(not set)".to_string() } else { mask_api_key(&os) },
        openstates_set => !os.is_empty(),
        cycle => state.config.cycle,
        provider => format!("{:?}", state.config.provider).to_ascii_lowercase(),
        message => message,
        error => error,
    }
}

async fn settings_page(State(state): State<Arc<AppState>>) -> Result<Html<String>, AppError> {
    render(
        &state,
        "settings.html",
        settings_context(&state, None, None),
    )
}

#[derive(Deserialize)]
struct SettingsForm {
    fec_api_key: String,
    #[serde(default)]
    openstates_api_key: String,
}

async fn settings_save(
    State(state): State<Arc<AppState>>,
    Form(form): Form<SettingsForm>,
) -> Result<Response, AppError> {
    let fec_incoming = form.fec_api_key.trim().to_string();
    let fec_key = if fec_incoming.is_empty() {
        state.config.fec_api_key_snapshot()
    } else {
        fec_incoming
    };
    let os_incoming = form.openstates_api_key.trim().to_string();
    let os_key = if os_incoming.is_empty() {
        state.config.openstates_api_key_snapshot()
    } else {
        os_incoming
    };

    if fec_key.is_empty() {
        let html = render(
            &state,
            "settings.html",
            settings_context(&state, None, Some("FEC API key cannot be empty.".into())),
        )?;
        return Ok(html.into_response());
    }

    state.config.set_fec_api_key(fec_key.clone());
    state.config.set_openstates_api_key(os_key.clone());
    if let Err(err) = state.config.save_api_keys_to_file() {
        let html = render(
            &state,
            "settings.html",
            settings_context(
                &state,
                None,
                Some(format!(
                    "Saved in memory but failed to write config file: {err:#}"
                )),
            ),
        )?;
        return Ok(html.into_response());
    }

    tracing::info!(
        path = %state.config.config_path.display(),
        fec = %mask_api_key(&fec_key),
        openstates = %if os_key.is_empty() { "(not set)".into() } else { mask_api_key(&os_key) },
        "API keys updated via Settings"
    );

    let html = render(
        &state,
        "settings.html",
        settings_context(
            &state,
            Some(format!(
                "Saved to {}. Live requests will use the new keys immediately.",
                state.config.config_path.display()
            )),
            None,
        ),
    )?;
    Ok(html.into_response())
}

pub struct AppError(anyhow::Error);

impl AppError {
    fn internal(msg: impl Into<String>) -> Self {
        Self(anyhow::anyhow!(msg.into()))
    }
}

impl From<anyhow::Error> for AppError {
    fn from(value: anyhow::Error) -> Self {
        Self(value)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let msg = redact_secrets(&format!("{:#}", self.0));
        tracing::error!(error = %msg, "request error");
        (StatusCode::INTERNAL_SERVER_ERROR, format!("internal error: {msg}")).into_response()
    }
}
