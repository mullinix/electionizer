//! Pure state ballot extras (FL/AZ/Open States) — no HTTP.

use crate::models::{
    normalize_party_label, GeoResolution, ResolvedJurisdiction, SnapshotCandidate, SnapshotMeasure,
};
use crate::openstates::{person_display_name, role_district_num};
use crate::states::arizona::{
    az_measures_link, map_az_measures_for_geo, map_az_officials_for_geo, member_to_candidate,
    parse_az_ballot_measures_html, parse_az_officials_html, parse_roster as parse_az_roster,
};
use crate::states::florida::{
    fl_gen_elec_id, fl_measures_fallback, map_filings_for_geo, parse_dos_tsv, parse_fl_measures_table,
    parse_house_roster, parse_senate_roster, roster_fallback, DOS_PUBLISHER, MEASURES_URL,
};
use crate::states::florida_soe::{
    map_soe_hits_for_geo, parse_fl_soe_candidate_list_html, soe_duplicates_existing, VF_PUBLISHER,
};
use crate::states::maryland::{
    ballot_questions_url as md_ballot_questions_url, local_csv_url as md_local_csv_url,
    map_filings_for_geo as map_md_filings, map_md_measures_for_geo,
    parse_candidate_csv as parse_md_csv, parse_md_ballot_questions_html,
    statewide_csv_url as md_statewide_csv_url, SOURCE_PUBLISHER as MD_PUBLISHER,
};
use crate::states::north_carolina::{
    candidate_listing_csv_url, map_filings_for_geo as map_nc_filings, map_nc_measures_for_geo,
    parse_candidate_listing_csv, parse_nc_referendum_list_text, SOURCE_PUBLISHER as NC_PUBLISHER,
};
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// Named body keys for the live state payload JSON map (JS → WASM → core).
pub const BODY_FL_DOS: &str = "fl:dos";
pub const BODY_FL_SENATE: &str = "fl:senate";
pub const BODY_FL_HOUSE: &str = "fl:house";
pub const BODY_FL_MEASURES: &str = "fl:measures";
/// VoterFocus county SOE candidate list HTML (`candidate_pr.php`).
pub const BODY_FL_SOE: &str = "fl:soe";
/// Official sample-ballot PDF text extract (optional precinct filter).
pub const BODY_FL_SAMPLE_BALLOT: &str = "fl:sample_ballot";
pub const BODY_AZ_SENATE: &str = "az:senate";
pub const BODY_AZ_HOUSE: &str = "az:house";
/// Clean Elections BallotMeasures HTML fragment (`mode=all`).
pub const BODY_AZ_MEASURES: &str = "az:measures";
/// Clean Elections OfficialList HTML (statewide + LD incumbents).
pub const BODY_AZ_OFFICIALS: &str = "az:officials";
pub const BODY_NC_CANDIDATES: &str = "nc:candidates";
/// NCSBE referendum list plain text (PDF extract).
pub const BODY_NC_MEASURES: &str = "nc:measures";
/// Source PDF URL for NC referendum cites (optional companion to `nc:measures`).
pub const BODY_NC_MEASURES_URL: &str = "nc:measures_url";
pub const BODY_MD_STATEWIDE: &str = "md:statewide";
pub const BODY_MD_LOCAL: &str = "md:local";
/// `"GG"` (general) or `"GP"` (primary) — selects cited source URL.
pub const BODY_MD_PHASE: &str = "md:phase";
/// SBE `ballot_questions.html` (statewide + county questions).
pub const BODY_MD_MEASURES: &str = "md:measures";
pub const BODY_OS_PEOPLE_GEO: &str = "os:people.geo";
/// Google Civic `voterInfoQuery` JSON (multi-state ballot when VIP feed is live).
pub const BODY_CIVIC_VOTERINFO: &str = "civic:voterinfo";

/// Parse `{ "fl:dos": "...", "os:people.geo": "..." }` (empty string → empty map).
pub fn parse_state_bodies_json(json: &str) -> Result<BTreeMap<String, String>, String> {
    let t = json.trim();
    if t.is_empty() || t == "{}" {
        return Ok(BTreeMap::new());
    }
    serde_json::from_str(t).map_err(|e| format!("state bodies json: {e}"))
}

fn body<'a>(map: &'a BTreeMap<String, String>, key: &str) -> &'a str {
    map.get(key).map(|s| s.as_str()).unwrap_or("")
}

/// Build FL/AZ/OS extras from a named-bodies map (empty values skipped).
pub fn extras_from_state_bodies(
    geo: &GeoResolution,
    cycle: i32,
    bodies: &BTreeMap<String, String>,
) -> Option<StateBallotExtras> {
    let st = geo.state.to_ascii_uppercase();
    let mut extras = match st.as_str() {
        "FL" => {
            let dos = body(bodies, BODY_FL_DOS);
            let senate = body(bodies, BODY_FL_SENATE);
            let house = body(bodies, BODY_FL_HOUSE);
            let measures = body(bodies, BODY_FL_MEASURES);
            let soe = body(bodies, BODY_FL_SOE);
            let sample = body(bodies, BODY_FL_SAMPLE_BALLOT);
            if dos.trim().is_empty()
                && senate.trim().is_empty()
                && house.trim().is_empty()
                && measures.trim().is_empty()
                && soe.trim().is_empty()
            {
                None
            } else {
                Some(fl_extras_from_bodies(
                    geo, cycle, dos, senate, house, measures, soe, sample,
                ))
            }
        }
        "AZ" => {
            let senate = body(bodies, BODY_AZ_SENATE);
            let house = body(bodies, BODY_AZ_HOUSE);
            let measures = body(bodies, BODY_AZ_MEASURES);
            let officials = body(bodies, BODY_AZ_OFFICIALS);
            if senate.trim().is_empty()
                && house.trim().is_empty()
                && measures.trim().is_empty()
                && officials.trim().is_empty()
            {
                None
            } else {
                Some(az_extras_from_rosters(
                    geo, senate, house, measures, officials,
                ))
            }
        }
        "NC" => {
            let csv = body(bodies, BODY_NC_CANDIDATES);
            let measures = body(bodies, BODY_NC_MEASURES);
            let measures_url = body(bodies, BODY_NC_MEASURES_URL);
            if csv.trim().is_empty() && measures.trim().is_empty() {
                None
            } else {
                Some(nc_extras_from_csv(geo, cycle, csv, measures, measures_url))
            }
        }
        "MD" => {
            let statewide = body(bodies, BODY_MD_STATEWIDE);
            let local = body(bodies, BODY_MD_LOCAL);
            let measures = body(bodies, BODY_MD_MEASURES);
            if statewide.trim().is_empty() && local.trim().is_empty() && measures.trim().is_empty()
            {
                None
            } else {
                let phase = body(bodies, BODY_MD_PHASE);
                Some(md_extras_from_csv(
                    geo, cycle, statewide, local, phase, measures,
                ))
            }
        }
        _ => None,
    };

    // Google Civic / VIP — full contests when live; prefer over thin incumbent/OS coverage.
    let civic_raw = body(bodies, BODY_CIVIC_VOTERINFO);
    if !civic_raw.trim().is_empty() {
        let civic_ex = crate::civic::civic_extras_from_voterinfo(geo, civic_raw);
        extras = Some(match extras {
            None => civic_ex,
            Some(mut existing) => {
                crate::civic::merge_civic_into(&mut existing, civic_ex);
                existing
            }
        });
    }

    let need_os = extras
        .as_ref()
        .map(|e| e.candidates.is_empty())
        .unwrap_or(true);
    let os = body(bodies, BODY_OS_PEOPLE_GEO);
    if need_os && !os.trim().is_empty() {
        extras = Some(openstates_extras_from_people_geo(geo, os));
    }
    extras
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct StateBallotExtras {
    pub candidates: Vec<SnapshotCandidate>,
    pub measures: Vec<SnapshotMeasure>,
    pub coverage_label: Option<String>,
    pub notes: Vec<String>,
    pub extra_jurisdictions: Vec<ResolvedJurisdiction>,
}

impl StateBallotExtras {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.candidates.is_empty()
            && self.measures.is_empty()
            && self.coverage_label.is_none()
    }
}

/// Maryland SBE candidate listing CSVs + optional ballot questions HTML.
pub fn md_extras_from_csv(
    geo: &GeoResolution,
    cycle: i32,
    statewide_csv: &str,
    local_csv: &str,
    phase: &str,
    measures_html: &str,
) -> StateBallotExtras {
    let mut filings = parse_md_csv(statewide_csv);
    let n_state = filings.len();
    filings.extend(parse_md_csv(local_csv));
    let n_local = filings.len().saturating_sub(n_state);
    let general = !phase.eq_ignore_ascii_case("GP");
    let source = md_statewide_csv_url(cycle, general);
    let local_src = md_local_csv_url(cycle, general);
    let candidates = if n_state + n_local == 0 {
        vec![]
    } else {
        map_md_filings(&filings, geo, cycle, &source)
    };
    let mut notes = Vec::new();
    if n_state + n_local > 0 {
        notes.push(format!(
            "Parsed {} MD SBE statewide + {} local row(s); mapped {} for this ZIP’s districts/county.",
            n_state,
            n_local,
            candidates.len()
        ));
        if candidates.is_empty() {
            notes.push(
                "No Maryland filings matched this ZIP’s state districts/county yet.".into(),
            );
        }
        notes.push(format!(
            "Source: {source} + {local_src} ({MD_PUBLISHER}). Federal races from FEC. House subdistricts (e.g. 30A) from TIGERweb."
        ));
    }

    let state_ocd = "ocd-division/country:us/state:md".to_string();
    let measures_url = md_ballot_questions_url(cycle);
    let mut extra_jurisdictions = Vec::new();
    let measures = if !measures_html.trim().is_empty() {
        let parsed =
            parse_md_ballot_questions_html(measures_html, Some(geo.county.as_str()));
        if parsed.is_empty() {
            notes.push(
                "MD ballot questions page provided but no measures parsed for this county/cycle."
                    .into(),
            );
            vec![]
        } else {
            notes.push(format!(
                "Parsed {} MD ballot question(s) from SBE list (statewide + county).",
                parsed.len()
            ));
            let (m, extras) =
                map_md_measures_for_geo(&parsed, &state_ocd, &geo.county, &measures_url);
            extra_jurisdictions.extend(extras);
            m
        }
    } else {
        notes.push("MD ballot measures: SBE ballot questions page not loaded.".into());
        vec![]
    };

    let has_leg = candidates.iter().any(|c| {
        matches!(
            c.chamber.as_deref(),
            Some("state_senate" | "state_house")
        )
    });
    let has_local = candidates.iter().any(|c| {
        matches!(
            c.chamber.as_deref(),
            Some("county" | "judicial" | "municipal" | "statewide")
        )
    });
    let has_measures = !measures.is_empty();
    let coverage = match (has_leg, has_local, has_measures) {
        (true, true, true) => "Maryland SBE (leg + local + measures)",
        (true, true, false) => "Maryland SBE (leg + local)",
        (true, false, true) => "Maryland SBE (legislature + measures)",
        (true, false, false) => "Maryland SBE (legislature)",
        (false, _, true) if !candidates.is_empty() => "Maryland SBE (filings + measures)",
        (false, _, true) => "Maryland measures",
        (false, _, false) if !candidates.is_empty() => "Maryland SBE filings",
        _ => "Maryland (no district match)",
    };
    StateBallotExtras {
        candidates,
        measures,
        coverage_label: Some(coverage.into()),
        notes,
        extra_jurisdictions,
    }
}

/// North Carolina NCSBE candidate listing CSV + optional referendum list text.
pub fn nc_extras_from_csv(
    geo: &GeoResolution,
    cycle: i32,
    csv: &str,
    measures_text: &str,
    measures_url: &str,
) -> StateBallotExtras {
    let state_ocd = "ocd-division/country:us/state:nc".to_string();
    let filings = if csv.trim().is_empty() {
        vec![]
    } else {
        parse_candidate_listing_csv(csv)
    };
    let candidates = if filings.is_empty() {
        vec![]
    } else {
        map_nc_filings(&filings, geo, cycle)
    };
    let mut notes = Vec::new();
    let mut extra_jurisdictions = Vec::new();
    if !csv.trim().is_empty() {
        notes.push(format!(
            "Parsed {} NCSBE candidate listing row(s); mapped {} for this ZIP’s districts/county.",
            filings.len(),
            candidates.len()
        ));
        if candidates.is_empty() {
            notes.push(
                "No NC general-election filings matched this ZIP’s state districts/county yet."
                    .into(),
            );
        }
        notes.push(format!(
            "Source: {} ({NC_PUBLISHER}). Federal races from FEC.",
            candidate_listing_csv_url(cycle)
        ));
    } else {
        notes.push("NC candidate listing CSV not provided.".into());
    }

    let measures = if !measures_text.trim().is_empty() {
        let parsed =
            parse_nc_referendum_list_text(measures_text, Some(geo.county.as_str()));
        if parsed.is_empty() {
            notes.push(
                "NC referendum list provided but no measures parsed for this county/cycle.".into(),
            );
            vec![]
        } else {
            notes.push(format!(
                "Parsed {} NC referendum(s) from NCSBE list (statewide + county).",
                parsed.len()
            ));
            let src = if measures_url.trim().is_empty() {
                CANDIDATE_LISTS_PAGE_URL_FALLBACK
            } else {
                measures_url.trim()
            };
            let (m, extras) =
                map_nc_measures_for_geo(&parsed, &state_ocd, &geo.county, src);
            extra_jurisdictions.extend(extras);
            m
        }
    } else {
        notes.push("NC ballot measures: NCSBE referendum list not loaded.".into());
        vec![]
    };

    let has_leg = candidates.iter().any(|c| {
        matches!(
            c.chamber.as_deref(),
            Some("state_senate" | "state_house")
        )
    });
    let has_local = candidates.iter().any(|c| {
        matches!(
            c.chamber.as_deref(),
            Some("county" | "judicial" | "municipal")
        )
    });
    let has_measures = !measures.is_empty();
    let coverage = match (has_leg, has_local, has_measures) {
        (true, true, true) => "North Carolina NCSBE (leg + local + measures)",
        (true, true, false) => "North Carolina NCSBE (leg + local)",
        (true, false, true) => "North Carolina NCSBE (legislature + measures)",
        (true, false, false) => "North Carolina NCSBE (legislature)",
        (false, _, true) if !candidates.is_empty() => "North Carolina NCSBE (filings + measures)",
        (false, _, true) => "North Carolina measures",
        (false, _, false) if !candidates.is_empty() => "North Carolina NCSBE filings",
        _ => "North Carolina (no district match)",
    };
    StateBallotExtras {
        candidates,
        measures,
        coverage_label: Some(coverage.into()),
        notes,
        extra_jurisdictions,
    }
}

const CANDIDATE_LISTS_PAGE_URL_FALLBACK: &str =
    "https://www.ncsbe.gov/results-data/candidate-lists";

/// Arizona legislature + statewide incumbents from roster / Clean Elections OfficialList.
pub fn az_extras_from_rosters(
    geo: &GeoResolution,
    senate_html: &str,
    house_html: &str,
    measures_html: &str,
    officials_html: &str,
) -> StateBallotExtras {
    let state_l = "az";
    let state_ocd = format!("ocd-division/country:us/state:{state_l}");
    let mut notes = Vec::new();
    let mut candidates = Vec::new();
    let mut extra_jurisdictions = Vec::new();

    let has_rosters = !senate_html.trim().is_empty() || !house_html.trim().is_empty();
    let mut leg_from_roster = 0usize;
    if has_rosters {
        let senators = if senate_html.trim().is_empty() {
            Default::default()
        } else {
            parse_az_roster(senate_html, "Senate")
        };
        let representatives = if house_html.trim().is_empty() {
            Default::default()
        } else {
            parse_az_roster(house_html, "House")
        };

        if let Some(sd) = geo.state_senate_district {
            let ocd = format!("ocd-division/country:us/state:{state_l}/sldu:{sd}");
            let members = senators.get(&sd).cloned().unwrap_or_default();
            if members.is_empty() {
                notes.push(format!("No AZ senator roster match for district {sd}."));
            }
            for m in members {
                candidates.push(member_to_candidate(
                    &m,
                    &format!("Arizona Senate (District {sd})"),
                    "state_senate",
                    &ocd,
                ));
                leg_from_roster += 1;
            }
        }
        if let Some(hd) = geo.state_house_district {
            let ocd = format!("ocd-division/country:us/state:{state_l}/sldl:{hd}");
            let members = representatives.get(&hd).cloned().unwrap_or_default();
            if members.is_empty() {
                notes.push(format!(
                    "No AZ representative roster match for district {hd}."
                ));
            }
            for m in members {
                candidates.push(member_to_candidate(
                    &m,
                    &format!("Arizona House (District {hd})"),
                    "state_house",
                    &ocd,
                ));
                leg_from_roster += 1;
            }
        }
    } else {
        notes.push("Arizona Legislature roster HTML not provided.".into());
    }

    // C4: Clean Elections OfficialList — statewide exec incumbents; leg only if roster empty.
    let mut statewide_n = 0usize;
    let mut leg_from_officials = 0usize;
    if !officials_html.trim().is_empty() {
        let parsed = parse_az_officials_html(officials_html);
        let include_leg = leg_from_roster == 0;
        let mapped = map_az_officials_for_geo(
            &parsed,
            geo.state_senate_district,
            geo.state_house_district,
            &state_ocd,
            include_leg,
        );
        for c in mapped {
            if c.chamber.as_deref() == Some("statewide") {
                statewide_n += 1;
            } else {
                leg_from_officials += 1;
            }
            candidates.push(c);
        }
        if statewide_n > 0 {
            notes.push(format!(
                "Parsed {statewide_n} AZ statewide executive incumbent(s) from Clean Elections OfficialList."
            ));
        } else {
            notes.push(
                "AZ OfficialList provided but no statewide executive rows mapped.".into(),
            );
        }
        if include_leg && leg_from_officials > 0 {
            notes.push(format!(
                "AZ legislature from Clean Elections OfficialList fallback ({leg_from_officials} incumbent(s))."
            ));
        }
    } else {
        notes.push("AZ statewide officials: Clean Elections OfficialList not loaded.".into());
    }

    notes.push(
        "AZ coverage is incumbents only (leg roster + statewide OfficialList). Not full filings — SOS challenger/judicial/local lists are bot-blocked; challengers not shown."
            .into(),
    );

    // C1: Clean Elections propositions table (statewide + ZIP county).
    let measures = if !measures_html.trim().is_empty() {
        let parsed = parse_az_ballot_measures_html(measures_html, Some(geo.county.as_str()));
        if parsed.is_empty() {
            notes.push(
                "AZ measures HTML provided but no propositions parsed for this county/cycle."
                    .into(),
            );
            az_measures_link(&state_ocd)
        } else {
            notes.push(format!(
                "Parsed {} AZ proposition(s) from Clean Elections (statewide + county).",
                parsed.len()
            ));
            let (m, extras) = map_az_measures_for_geo(&parsed, &state_ocd, &geo.county);
            extra_jurisdictions.extend(extras);
            m
        }
    } else {
        notes.push(
            "AZ ballot measures: Clean Elections list not loaded — showing source link only."
                .into(),
        );
        az_measures_link(&state_ocd)
    };

    let has_leg = leg_from_roster + leg_from_officials > 0;
    let has_statewide = statewide_n > 0;
    let has_measures = measures.iter().any(|m| m.measure_code.is_some());
    let coverage = match (has_leg, has_statewide, has_measures) {
        (true, true, true) => "Arizona (leg + statewide incumbents + measures)",
        (true, true, false) => "Arizona (leg + statewide incumbents)",
        (true, false, true) => "Arizona (leg incumbents + measures)",
        (true, false, false) => "Arizona Legislature (incumbents)",
        (false, true, true) => "Arizona (statewide incumbents + measures)",
        (false, true, false) => "Arizona statewide incumbents",
        (false, false, true) => "Arizona measures",
        _ => "Arizona (rosters unavailable)",
    };

    StateBallotExtras {
        candidates,
        measures,
        coverage_label: Some(coverage.into()),
        notes,
        extra_jurisdictions,
    }
}

/// State legislature incumbents from Open States `GET /people.geo` JSON.
/// Sitting senators/reps at the ZIP centroid — not a full ballot (no challengers/measures).
pub fn openstates_extras_from_people_geo(
    geo: &GeoResolution,
    people_geo_json: &str,
) -> StateBallotExtras {
    let st = geo.state.to_ascii_uppercase();
    let st_l = st.to_ascii_lowercase();
    let state_name = if geo.state_name.trim().is_empty() {
        st.clone()
    } else {
        geo.state_name.clone()
    };
    let mut notes = Vec::new();
    let mut candidates = Vec::new();
    let mut seen_ids = BTreeSet::new();

    let Ok(root) = serde_json::from_str::<Value>(people_geo_json) else {
        return StateBallotExtras {
            notes: vec!["Open States people.geo JSON parse failed.".into()],
            coverage_label: Some("Open States (unavailable)".into()),
            ..Default::default()
        };
    };
    let results = root
        .get("results")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if results.is_empty() {
        return StateBallotExtras {
            notes: vec!["Open States returned no legislators for this point.".into()],
            coverage_label: Some("Open States (empty)".into()),
            ..Default::default()
        };
    }

    for p in results {
        let role = p
            .get("current_role")
            .cloned()
            .or_else(|| {
                p.get("roles")
                    .and_then(|r| r.as_array())
                    .and_then(|a| a.first())
                    .cloned()
            })
            .unwrap_or(Value::Null);
        let org = role
            .get("org_classification")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let (chamber, chamber_label, ocd_kind) = match org {
            "upper" => ("state_senate", "Senate", "sldu"),
            "lower" => ("state_house", "House", "sldl"),
            _ => continue,
        };
        let district = role_district_num(&role);
        // Prefer people matching TIGERweb districts when known
        if let (Some(want), Some(have)) = (geo.state_senate_district, district) {
            if chamber == "state_senate" && want != have {
                continue;
            }
        }
        if let (Some(want), Some(have)) = (geo.state_house_district, district) {
            if chamber == "state_house" && want != have {
                continue;
            }
        }

        let name = person_display_name(&p);
        if name.is_empty() || name == "Legislator" {
            continue;
        }
        let person_id = p.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let dedupe_key = if person_id.is_empty() {
            format!("{chamber}:{name}")
        } else {
            person_id.to_string()
        };
        if !seen_ids.insert(dedupe_key) {
            continue;
        }
        let party = p
            .get("party")
            .and_then(|v| v.as_str())
            .map(normalize_party_label)
            .unwrap_or_else(|| "Unknown".into());
        let office = match district {
            Some(d) => format!("{state_name} State {chamber_label} · District {d}"),
            None => format!("{state_name} State {chamber_label}"),
        };
        let ocd = if let Some(d) = district {
            format!("ocd-division/country:us/state:{st_l}/{ocd_kind}:{d}")
        } else {
            role.get("division_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        };
        if ocd.is_empty() {
            continue;
        }
        let profile = p
            .get("openstates_url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                if person_id.is_empty() {
                    "https://openstates.org/".into()
                } else {
                    format!("https://openstates.org/person/{person_id}/")
                }
            });
        let external_id = if person_id.is_empty() {
            None
        } else {
            Some(format!("openstates:{person_id}"))
        };
        candidates.push(SnapshotCandidate {
            office,
            chamber: Some(chamber.into()),
            jurisdiction_ocd: ocd,
            is_judicial: false,
            name,
            party,
            is_incumbent: true,
            is_judge: false,
            summary: Some(format!(
                "Sitting state {chamber_label} member via Open States (ZIP centroid). Incumbents only — not a full ballot (no challengers, statewide, judicial, or local)."
            )),
            source_url: profile,
            source_publisher: Some("Open States".into()),
            external_id,
        });
    }

    if candidates.is_empty() {
        notes.push(
            "Open States people.geo had legislators but none matched upper/lower roles for this district."
                .into(),
        );
        return StateBallotExtras {
            notes,
            coverage_label: Some("Open States (no district match)".into()),
            ..Default::default()
        };
    }

    notes.push(format!(
        "State legislature incumbents via Open States ({}). Not a full ballot — no challengers, statewide exec, judicial, or local.",
        candidates.len()
    ));

    StateBallotExtras {
        candidates,
        measures: vec![],
        coverage_label: Some("Open States incumbents (not full ballot)".into()),
        notes,
        extra_jurisdictions: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{GeoResolution, ResolvedJurisdiction};

    fn az_geo(sd: u32, hd: u32) -> GeoResolution {
        GeoResolution {
            state: "AZ".into(),
            state_name: "Arizona".into(),
            county: "Maricopa County".into(),
            city: "Phoenix".into(),
            congressional_district: "AZ-3".into(),
            state_senate_district: Some(sd),
            state_house_district: Some(hd),
            state_house_label: Some(hd.to_string()),
            latitude: Some(33.45),
            longitude: Some(-112.07),
            jurisdictions: vec![ResolvedJurisdiction {
                ocd_id: "ocd-division/country:us/state:az".into(),
                name: "Arizona".into(),
                level: "state".into(),
                state: Some("AZ".into()),
            }],
            source_url: "test".into(),
            source_publisher: "test".into(),
        }
    }

    #[test]
    fn az_roster_maps_district() {
        let senate = r#"
        <tr>
          <td><a href="https://www.azleg.gov/Senate/Senate-member/?legislature=57&session=130&legislator=2371" class="roster-tooltip">Lela Alston</a></td>
          <td><a title="District 5" href="x.pdf">5</a></td>
          <td><span title="Democratic">D</span></td>
        </tr>"#;
        let house = r#"
        <tr>
          <td><a href="https://www.azleg.gov/House/House-member/?legislature=57&session=130&legislator=1001" class="roster-tooltip">Ada One</a></td>
          <td><a title="District 5" href="x.pdf">5</a></td>
          <td><span title="Democratic">D</span></td>
        </tr>"#;
        let ex = az_extras_from_rosters(&az_geo(5, 5), senate, house, "", "");
        assert!(ex.coverage_label.as_deref().unwrap().contains("Arizona"));
        assert!(ex.candidates.iter().any(|c| c.name.contains("Alston")));
        assert!(ex.candidates.iter().any(|c| c.chamber.as_deref() == Some("state_house")));
        assert!(!ex.measures.is_empty());
        // link-only stub when measures HTML absent
        assert!(ex.measures.iter().any(|m| m.measure_code.is_none()));
        assert!(ex.notes.iter().any(|n| n.contains("incumbents only")));
    }

    #[test]
    fn az_measures_body_parses_table() {
        let html = include_str!("../../../testdata/az_ballot_measures_sample.html");
        let mut geo = az_geo(5, 5);
        geo.county = "Maricopa County".into();
        let ex = az_extras_from_rosters(&geo, "", "", html, "");
        assert!(
            ex.coverage_label
                .as_deref()
                .unwrap_or("")
                .contains("measures"),
            "{:?}",
            ex.coverage_label
        );
        assert!(ex.measures.iter().any(|m| m.measure_code.as_deref() == Some("Prop 133")));
        assert!(ex.measures.iter().all(|m| m.measure_code.is_some()));
        assert!(ex.notes.iter().any(|n| n.contains("Clean Elections")));
    }

    #[test]
    fn extras_from_named_bodies_map() {
        let senate = r#"
        <tr>
          <td><a href="https://www.azleg.gov/Senate/Senate-member/?legislature=57&session=130&legislator=2371" class="roster-tooltip">Lela Alston</a></td>
          <td><a title="District 5" href="x.pdf">5</a></td>
          <td><span title="Democratic">D</span></td>
        </tr>"#;
        let mut map = BTreeMap::new();
        map.insert(BODY_AZ_SENATE.into(), senate.into());
        map.insert(BODY_AZ_HOUSE.into(), String::new());
        let ex = extras_from_state_bodies(&az_geo(5, 5), 2026, &map).expect("extras");
        assert!(ex.candidates.iter().any(|c| c.name.contains("Alston")));

        let empty = parse_state_bodies_json("").unwrap();
        assert!(extras_from_state_bodies(&az_geo(5, 5), 2026, &empty).is_none());
    }

    #[test]
    fn civic_body_replaces_az_incumbent_roster() {
        let senate = r#"
        <tr>
          <td><a href="https://www.azleg.gov/Senate/Senate-member/?legislature=57&session=130&legislator=2371" class="roster-tooltip">Lela Alston</a></td>
          <td><a title="District 5" href="x.pdf">5</a></td>
          <td><span title="Democratic">D</span></td>
        </tr>"#;
        let civic = include_str!("../../../testdata/civic_voterinfo_az_sample.json");
        let mut map = BTreeMap::new();
        map.insert(BODY_AZ_SENATE.into(), senate.into());
        map.insert(BODY_CIVIC_VOTERINFO.into(), civic.into());
        let ex = extras_from_state_bodies(&az_geo(5, 5), 2026, &map).expect("extras");
        assert!(
            ex.coverage_label
                .as_deref()
                .unwrap_or("")
                .contains("Google Civic"),
            "{:?}",
            ex.coverage_label
        );
        assert!(
            ex.candidates.iter().any(|c| c.name.contains("Challenger")),
            "civic challengers expected"
        );
        assert!(
            !ex.candidates.iter().any(|c| c.name.contains("Alston")),
            "roster incumbent should yield to civic slate"
        );
        assert!(ex.measures.iter().any(|m| {
            m.measure_code
                .as_deref()
                .unwrap_or("")
                .to_ascii_uppercase()
                .contains("139")
                || m.title.to_ascii_uppercase().contains("139")
        }));
    }

    #[test]
    fn openstates_people_geo_maps_upper_lower() {
        let json = r#"{
          "results": [
            {
              "id": "ocd-person/aaa",
              "name": "Jane Senator",
              "party": "Democratic",
              "openstates_url": "https://openstates.org/person/jane/",
              "current_role": {
                "title": "Senator",
                "org_classification": "upper",
                "district": "5"
              }
            },
            {
              "id": "ocd-person/bbb",
              "name": "John Rep",
              "party": "Republican",
              "current_role": {
                "org_classification": "lower",
                "district": "12"
              }
            },
            {
              "id": "ocd-person/ccc",
              "name": "Wrong District",
              "current_role": {
                "org_classification": "upper",
                "district": "99"
              }
            }
          ]
        }"#;
        let geo = az_geo(5, 12);
        let ex = openstates_extras_from_people_geo(&geo, json);
        assert_eq!(ex.candidates.len(), 2);
        assert!(ex.candidates.iter().any(|c| c.name.contains("Jane")));
        assert!(ex.candidates.iter().any(|c| c.name.contains("John")));
        assert!(!ex.candidates.iter().any(|c| c.name.contains("Wrong")));
        assert!(ex
            .coverage_label
            .as_deref()
            .unwrap()
            .contains("Open States"));
    }
}

/// Florida state extras from optional DOS TSV + chamber roster HTML + measures HTML
/// + county SOE (VoterFocus) list. Empty `dos_tsv` falls back to chamber incumbents
/// when rosters are present. Empty `measures_html` falls back to a link-only measures row.
/// `soe_html` / `sample_ballot_text` may be empty.
pub fn fl_extras_from_bodies(
    geo: &GeoResolution,
    cycle: i32,
    dos_tsv: &str,
    senate_html: &str,
    house_html: &str,
    measures_html: &str,
    soe_html: &str,
    sample_ballot_text: &str,
) -> StateBallotExtras {
    let state_l = "fl";
    let state_ocd = format!("ocd-division/country:us/state:{state_l}");
    let elec_id = fl_gen_elec_id(cycle);
    let mut notes = Vec::new();
    let mut extra_jurisdictions = Vec::new();

    let senators = if senate_html.trim().is_empty() {
        Default::default()
    } else {
        parse_senate_roster(senate_html)
    };
    let representatives = if house_html.trim().is_empty() {
        Default::default()
    } else {
        parse_house_roster(house_html)
    };

    let mut candidates = if !dos_tsv.trim().is_empty() {
        let filings = parse_dos_tsv(dos_tsv);
        notes.push(format!(
            "Parsed {} DOS filing row(s) ({elec_id}).",
            filings.len()
        ));
        let mut cands = map_filings_for_geo(
            &filings,
            geo,
            &state_ocd,
            &senators,
            &representatives,
            &mut extra_jurisdictions,
        );
        if cands.is_empty() {
            notes.push(
                "DOS candidate list had no active filings for this ZIP’s districts yet.".into(),
            );
            cands = roster_fallback(geo, state_l, &senators, &representatives, &mut notes);
        } else {
            let local_n = cands
                .iter()
                .filter(|c| {
                    matches!(
                        c.chamber.as_deref(),
                        Some("county" | "municipal" | "special_district")
                    )
                })
                .count();
            if local_n == 0 {
                notes.push(
                    "No county/municipal/special-district DOS filings matched this ZIP’s county."
                        .into(),
                );
            } else {
                notes.push(format!(
                    "Includes {local_n} county/local/special-district filing(s) matched to {}.",
                    geo.county
                ));
            }
        }
        cands
    } else {
        notes.push(
            "FL DOS candidate extract not provided. Showing chamber incumbents when available."
                .into(),
        );
        roster_fallback(geo, state_l, &senators, &representatives, &mut notes)
    };

    if !soe_html.trim().is_empty() {
        match parse_fl_soe_candidate_list_html(soe_html) {
            Ok(hits) => {
                let soe_cands = map_soe_hits_for_geo(
                    &hits,
                    geo,
                    sample_ballot_text,
                    &mut extra_jurisdictions,
                );
                let before = candidates.len();
                for c in soe_cands {
                    if candidates.iter().any(|e| soe_duplicates_existing(e, &c)) {
                        continue;
                    }
                    candidates.push(c);
                }
                let added = candidates.len().saturating_sub(before);
                if added > 0 {
                    notes.push(format!(
                        "Added {added} county/local candidate(s) from {VF_PUBLISHER}."
                    ));
                } else if hits.iter().any(|h| {
                    crate::states::florida_soe::soe_status_on_ballot(&h.status)
                        && crate::states::florida_soe::soe_office_is_default_local(&h.office)
                }) {
                    notes.push(format!(
                        "{VF_PUBLISHER} list parsed; county locals already present or filtered."
                    ));
                }
            }
            Err(e) => notes.push(format!("FL SOE candidate list parse skipped: {e}")),
        }
    }

    let measures = if !measures_html.trim().is_empty() {
        let parsed = parse_fl_measures_table(measures_html, cycle);
        if parsed.is_empty() {
            notes.push(
                "FL measures HTML provided but no ballot-qualified amendments parsed for this cycle."
                    .into(),
            );
            fl_measures_fallback(&state_ocd)
        } else {
            notes.push(format!(
                "Parsed {} FL constitutional amendment(s) from DOS initiatives.",
                parsed.len()
            ));
            parsed
                .into_iter()
                .map(|m| SnapshotMeasure {
                    title: m.title,
                    measure_code: m.measure_code,
                    jurisdiction_ocd: state_ocd.clone(),
                    summary: m.summary,
                    source_url: m.source_url,
                    source_publisher: Some(DOS_PUBLISHER.into()),
                })
                .collect()
        }
    } else {
        notes.push(format!(
            "FL constitutional amendments: see {MEASURES_URL} (list not loaded)."
        ));
        fl_measures_fallback(&state_ocd)
    };

    let coverage = if notes.iter().any(|n| n.contains("incumbents") || n.contains("not provided"))
        && candidates.iter().all(|c| {
            matches!(c.chamber.as_deref(), Some("state_senate" | "state_house") | None)
                && c.is_incumbent
        })
        && dos_tsv.trim().is_empty()
    {
        "Florida legislature (incumbents fallback)"
    } else if candidates.iter().any(|c| {
        c.source_publisher
            .as_deref()
            .is_some_and(|p| p == VF_PUBLISHER)
    }) {
        "Florida DOS + county SOE"
    } else if candidates.iter().any(|c| {
        matches!(
            c.chamber.as_deref(),
            Some("county" | "municipal" | "special_district")
        )
    }) {
        "Florida DOS (state + local)"
    } else if !dos_tsv.trim().is_empty() {
        "Florida DOS filings (leg + statewide + judicial)"
    } else {
        "Florida legislature (incumbents fallback)"
    };

    StateBallotExtras {
        candidates,
        measures,
        coverage_label: Some(coverage.into()),
        notes,
        extra_jurisdictions,
    }
}

/// Merge state extras into a federal ballot snapshot (mutates).
pub fn apply_state_extras(
    snapshot: &mut crate::models::BallotSnapshot,
    extras: &StateBallotExtras,
    state: &str,
) {
    if extras.is_empty() {
        return;
    }
    snapshot.candidates.extend(extras.candidates.clone());
    if !extras.measures.is_empty() {
        snapshot.measures = extras.measures.clone();
    }
    snapshot
        .extra_jurisdictions
        .extend(extras.extra_jurisdictions.clone());

    let has_state = extras.coverage_label.is_some();
    if has_state {
        snapshot.election_scope = "federal+state".into();
    }

    let mut parts = vec!["Federal (live via FEC)".to_string()];
    if let Some(ref label) = extras.coverage_label {
        parts.push(label.clone());
    } else {
        parts.push("State/local pending".into());
    }
    if snapshot.measures.is_empty() {
        parts.push("Measures pending".into());
    } else {
        let st = state.to_ascii_uppercase();
        let real = snapshot.measures.iter().any(|m| {
            m.measure_code.as_deref().is_some_and(|c| {
                let u = c.to_ascii_uppercase();
                u.starts_with("AMENDMENT") || u.starts_with("PROP") || u.starts_with("MEASURE")
            })
        });
        if real {
            parts.push(format!("{st} measures"));
        } else {
            parts.push(format!("{st} measures (link)"));
        }
    }
    snapshot.coverage_note = Some(parts.join(" · "));
    snapshot.source_publisher =
        "Federal Election Commission OpenFEC API (+ state sources when available)".into();
}
