//! Candidate detail enrichment with staged progress events (SSE).

use crate::fec::FecClient;
use crate::govtrack::{ballot_affiliations, LegislatorClient};
use crate::models::{
    AffiliationSpan, CandidateDetail, CandidateFinance, CommitteeLink, ContributorRow,
    OutsideSpendRow, SizeBucketRow, VoteRecord,
};
use crate::openstates::{
    district_from_office, is_rate_limit_error, looks_like_fec_id, state_code_from_jurisdiction,
    OpenStatesClient,
};
use crate::redact::redact_secrets;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct StageDef {
    pub id: &'static str,
    pub label: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct StageUpdate {
    pub id: String,
    pub label: String,
    pub status: String,
    pub detail: Option<String>,
    pub index: usize,
    pub total: usize,
    pub pct: u8,
    pub completed: usize,
}

#[derive(Debug, Default)]
pub struct Enrichment {
    pub finance: Option<CandidateFinance>,
    pub finance_error: Option<String>,
    pub finance_unavailable: bool,
    pub outside_spending: Vec<OutsideSpendRow>,
    pub top_individuals: Vec<ContributorRow>,
    pub top_committees: Vec<ContributorRow>,
    pub size_buckets: Vec<SizeBucketRow>,
    pub principal_committee: Option<CommitteeLink>,
    pub votes: Vec<VoteRecord>,
    pub votes_url: Option<String>,
    pub votes_source: Option<String>,
    pub votes_rate_limited: bool,
    pub affiliations: Vec<AffiliationSpan>,
    pub affiliations_source: Option<String>,
}

pub fn plan_stages(detail: &CandidateDetail, openstates_keyed: bool) -> Vec<StageDef> {
    let fec = detail
        .external_id
        .as_deref()
        .is_some_and(looks_like_fec_id);
    if fec {
        return vec![
            StageDef {
                id: "totals",
                label: "FEC cycle totals",
            },
            StageDef {
                id: "outside",
                label: "Outside spending (Schedule E)",
            },
            StageDef {
                id: "indiv",
                label: "Top individual contributors",
            },
            StageDef {
                id: "cmte",
                label: "Top committee contributors",
            },
            StageDef {
                id: "size",
                label: "Contribution size buckets",
            },
            StageDef {
                id: "principal",
                label: "Principal campaign committee",
            },
            StageDef {
                id: "member",
                label: "Match member of Congress",
            },
            StageDef {
                id: "votes",
                label: "Roll-call votes (GovTrack)",
            },
        ];
    }

    let chamber = detail.chamber.as_deref().unwrap_or("");
    if matches!(chamber, "state_senate" | "state_house") && openstates_keyed {
        return vec![
            StageDef {
                id: "os_resolve",
                label: "Match state legislator (Open States)",
            },
            StageDef {
                id: "os_votes",
                label: "State roll-call votes",
            },
        ];
    }

    vec![StageDef {
        id: "profile",
        label: "Profile (no live enrichments)",
    }]
}

fn pct(completed: usize, total: usize) -> u8 {
    if total == 0 {
        return 100;
    }
    ((completed * 100) / total).min(100) as u8
}

fn update(
    def: &StageDef,
    status: &str,
    detail: Option<String>,
    index: usize,
    total: usize,
    completed: usize,
) -> StageUpdate {
    StageUpdate {
        id: def.id.to_string(),
        label: def.label.to_string(),
        status: status.into(),
        detail,
        index,
        total,
        pct: pct(completed, total),
        completed,
    }
}

pub async fn enrich_candidate<F>(
    detail: &CandidateDetail,
    fec: &FecClient,
    legislators: &LegislatorClient,
    openstates: &OpenStatesClient,
    cycle: i32,
    mut on_stage: F,
) -> Enrichment
where
    F: FnMut(StageUpdate),
{
    let stages = plan_stages(detail, openstates.has_key());
    let total = stages.len();
    let mut completed = 0usize;
    let filing_src = detail
        .sources
        .first()
        .and_then(|s| s.publisher.as_deref())
        .or(Some("Candidate filing / roster"));
    let filing_url = detail.sources.first().map(|s| s.url.as_str());
    let mut out = Enrichment {
        affiliations: ballot_affiliations(
            &detail.party,
            &detail.office,
            detail.is_incumbent,
            detail.is_judge,
            filing_src,
            filing_url,
        ),
        ..Default::default()
    };

    let fec_id = detail
        .external_id
        .as_deref()
        .filter(|id| looks_like_fec_id(id));

    if let Some(fec_id) = fec_id {
        for (i, st) in stages.iter().enumerate() {
            on_stage(update(st, "running", None, i + 1, total, completed));
            let detail_msg = match st.id {
                "totals" => {
                    match fec.candidate_totals(fec_id).await {
                        Ok(Some(f)) => {
                            out.finance = Some(f);
                            Some("ok".into())
                        }
                        Ok(None) => {
                            out.finance_error = Some(format!(
                                "No FEC finance totals for cycle {} yet.",
                                fec.cycle()
                            ));
                            Some("no totals yet".into())
                        }
                        Err(err) => {
                            let msg = redact_secrets(&format!("{err:#}"));
                            tracing::warn!(
                                candidate_id = detail.id,
                                fec_id = %fec_id,
                                error = %msg,
                                "FEC totals failed"
                            );
                            out.finance_error = Some(msg.clone());
                            Some(msg)
                        }
                    }
                }
                "outside" => match fec.candidate_outside_spending(fec_id).await {
                    Ok(rows) => {
                        let n = rows.len();
                        out.outside_spending = rows;
                        Some(format!("{n} committee(s)"))
                    }
                    Err(err) => {
                        let msg = redact_secrets(&format!("{err:#}"));
                        tracing::warn!(
                            candidate_id = detail.id,
                            fec_id = %fec_id,
                            error = %msg,
                            "FEC outside spending failed"
                        );
                        Some(msg)
                    }
                },
                "indiv" => match fec.candidate_top_individual_contributors(fec_id, 10).await {
                    Ok(rows) => {
                        let n = rows.len();
                        out.top_individuals = rows;
                        Some(format!("{n} line(s)"))
                    }
                    Err(err) => {
                        let msg = redact_secrets(&format!("{err:#}"));
                        tracing::warn!(
                            candidate_id = detail.id,
                            error = %msg,
                            "FEC top individuals failed"
                        );
                        Some(msg)
                    }
                },
                "cmte" => match fec.candidate_top_committee_contributors(fec_id, 10).await {
                    Ok(rows) => {
                        let n = rows.len();
                        out.top_committees = rows;
                        Some(format!("{n} line(s)"))
                    }
                    Err(err) => {
                        let msg = redact_secrets(&format!("{err:#}"));
                        tracing::warn!(
                            candidate_id = detail.id,
                            error = %msg,
                            "FEC top committees failed"
                        );
                        Some(msg)
                    }
                },
                "size" => match fec.candidate_contribution_sizes(fec_id).await {
                    Ok(rows) => {
                        let n = rows.len();
                        out.size_buckets = rows;
                        Some(format!("{n} bucket(s)"))
                    }
                    Err(err) => {
                        let msg = redact_secrets(&format!("{err:#}"));
                        tracing::warn!(
                            candidate_id = detail.id,
                            error = %msg,
                            "FEC size buckets failed"
                        );
                        Some(msg)
                    }
                },
                "principal" => match fec.candidate_principal_committee(fec_id).await {
                    Ok(c) => {
                        let msg = c
                            .as_ref()
                            .map(|x| x.name.clone())
                            .unwrap_or_else(|| "none".into());
                        // B6: soft CF context — committee ≠ voter affiliation
                        if let Some(ref pc) = c {
                            if let Some(span) =
                                electionizer_core::govtrack::campaign_committee_affiliation(
                                    &pc.name,
                                    &pc.designation,
                                    Some("OpenFEC"),
                                    Some(pc.url.as_str()),
                                )
                            {
                                out.affiliations =
                                    electionizer_core::openstates::merge_affiliation_spans(
                                        &out.affiliations,
                                        &[span],
                                    );
                            }
                        }
                        out.principal_committee = c;
                        Some(msg)
                    }
                    Err(err) => {
                        let msg = redact_secrets(&format!("{err:#}"));
                        tracing::warn!(
                            candidate_id = detail.id,
                            error = %msg,
                            "FEC committees failed"
                        );
                        Some(msg)
                    }
                },
                "member" => match legislators.resolve_by_fec(fec_id).await {
                    Ok(Some(leg)) => {
                        // B4: merge CL terms with ballot filing rows; never drop ballot.
                        if !leg.affiliations.is_empty() {
                            out.affiliations =
                                electionizer_core::openstates::merge_affiliation_spans(
                                    &leg.affiliations,
                                    &out.affiliations,
                                );
                            out.affiliations_source =
                                Some("unitedstates/congress-legislators (terms)".into());
                        }
                        out.votes_url = Some(leg.profile_url.clone());
                        out.votes_source = Some("GovTrack".into());
                        Some(format!("govtrack {}", leg.govtrack_id))
                    }
                    Ok(None) => {
                        // Challenger: keep ballot_affiliations only.
                        Some("no match (challenger / non-member)".into())
                    }
                    Err(err) => {
                        let msg = redact_secrets(&format!("{err:#}"));
                        tracing::warn!(fec_id = %fec_id, error = %msg, "legislator resolve failed");
                        Some(msg)
                    }
                },
                "votes" => {
                    // Re-resolve quickly from cache path inside legislators
                    match legislators.resolve_by_fec(fec_id).await {
                        Ok(Some(leg)) => {
                            out.votes_url = Some(leg.profile_url.clone());
                            out.votes_source = Some("GovTrack".into());
                            match legislators.recent_votes(leg.govtrack_id, 12).await {
                                Ok(rows) => {
                                    let n = rows.len();
                                    out.votes = rows;
                                    Some(format!("{n} vote(s)"))
                                }
                                Err(err) => {
                                    let msg = redact_secrets(&format!("{err:#}"));
                                    tracing::warn!(
                                        candidate_id = detail.id,
                                        govtrack_id = leg.govtrack_id,
                                        error = %msg,
                                        "GovTrack votes failed"
                                    );
                                    Some(msg)
                                }
                            }
                        }
                        _ => Some("skipped (no member match)".into()),
                    }
                }
                _ => Some("ok".into()),
            };
            let status = if detail_msg
                .as_deref()
                .is_some_and(|m| m.starts_with("skipped"))
            {
                "skip"
            } else if st.id == "totals" && out.finance_error.is_some() && out.finance.is_none() {
                // soft error still "done"
                "done"
            } else {
                "done"
            };
            completed += 1;
            on_stage(update(
                st,
                status,
                detail_msg,
                i + 1,
                total,
                completed,
            ));
        }
        return out;
    }

    // Non-federal path
    out.finance_unavailable = true;
    if !out.affiliations.is_empty() {
        out.affiliations_source = Some("Candidate filing / roster".into());
    }

    let chamber = detail.chamber.as_deref().unwrap_or("");
    let wants_os = matches!(chamber, "state_senate" | "state_house") && openstates.has_key();

    if !wants_os {
        for (i, st) in stages.iter().enumerate() {
            on_stage(update(st, "running", None, i + 1, total, completed));
            completed += 1;
            on_stage(update(
                st,
                "done",
                Some("no live data sources for this office".into()),
                i + 1,
                total,
                completed,
            ));
        }
        return out;
    }

    let st_code = detail
        .state_code
        .clone()
        .or_else(|| state_code_from_jurisdiction(&detail.jurisdiction, &detail.office))
        .unwrap_or_default();
    let district = district_from_office(&detail.office);
    let mut person_id = None::<String>;
    let mut jurisdiction = None::<String>;

    for (i, st) in stages.iter().enumerate() {
        on_stage(update(st, "running", None, i + 1, total, completed));
        let detail_msg = match st.id {
            "os_resolve" => {
                if st_code.len() != 2 {
                    Some("skipped (no state code)".into())
                } else {
                    match openstates
                        .resolve_legislator(&detail.name, &st_code, chamber, district)
                        .await
                    {
                        Ok(Some(leg)) => {
                            out.votes_url = Some(leg.profile_url.clone());
                            out.votes_source = Some("Open States".into());
                            if !leg.affiliations.is_empty() {
                                out.affiliations = electionizer_core::openstates::merge_affiliation_spans(
                                    &out.affiliations,
                                    &leg.affiliations,
                                );
                                if out.affiliations_source.is_none() {
                                    out.affiliations_source = Some("Open States".into());
                                }
                            }
                            person_id = Some(leg.person_id.clone());
                            jurisdiction = Some(leg.jurisdiction.clone());
                            Some(leg.person_id)
                        }
                        Ok(None) => Some("no match".into()),
                        Err(err) => {
                            let msg = redact_secrets(&format!("{err:#}"));
                            if is_rate_limit_error(&err) {
                                out.votes_rate_limited = true;
                            }
                            tracing::warn!(
                                candidate_id = detail.id,
                                error = %msg,
                                "OpenStates resolve failed"
                            );
                            Some(msg)
                        }
                    }
                }
            }
            "os_votes" => match (&person_id, &jurisdiction) {
                (Some(pid), Some(jur)) => {
                    match openstates.recent_votes(pid, jur, cycle, 12).await {
                        Ok(rows) => {
                            let n = rows.len();
                            out.votes = rows;
                            Some(format!("{n} vote(s)"))
                        }
                        Err(err) => {
                            let msg = redact_secrets(&format!("{err:#}"));
                            if is_rate_limit_error(&err) {
                                out.votes_rate_limited = true;
                            }
                            tracing::warn!(
                                candidate_id = detail.id,
                                person_id = %pid,
                                error = %msg,
                                "OpenStates votes failed"
                            );
                            Some(msg)
                        }
                    }
                }
                _ => Some("skipped (no legislator match)".into()),
            },
            _ => Some("ok".into()),
        };
        let status = if detail_msg
            .as_deref()
            .is_some_and(|m| m.starts_with("skipped") || m == "no match")
        {
            "skip"
        } else {
            "done"
        };
        completed += 1;
        on_stage(update(st, status, detail_msg, i + 1, total, completed));
    }

    out
}
