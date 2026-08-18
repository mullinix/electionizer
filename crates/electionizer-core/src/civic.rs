//! Google Civic Information API — pure parse/map of `voterInfoQuery` JSON (no HTTP).
//!
//! Docs: https://developers.google.com/civic-information/docs/v2/elections/voterInfoQuery
//! Data: Voting Information Project feeds; seasonal (often ~2–4 weeks pre-election).

use crate::models::{
    normalize_party_label, GeoResolution, ResolvedJurisdiction, SnapshotCandidate, SnapshotMeasure,
};
use crate::state_ballot::StateBallotExtras;
use serde_json::Value;
use std::collections::BTreeSet;

pub const CIVIC_PUBLISHER: &str = "Google Civic Information API (Voting Information Project)";
pub const CIVIC_DOCS_URL: &str = "https://developers.google.com/civic-information";
pub const CIVIC_API_ROOT: &str = "https://www.googleapis.com/civicinfo/v2";

/// Build `StateBallotExtras` from a voterinfo JSON body. Federal contests skipped (FEC owns those).
pub fn civic_extras_from_voterinfo(geo: &GeoResolution, json: &str) -> StateBallotExtras {
    let Ok(root) = serde_json::from_str::<Value>(json) else {
        return StateBallotExtras {
            notes: vec!["Google Civic voterinfo JSON parse failed.".into()],
            coverage_label: Some("Google Civic (unavailable)".into()),
            ..Default::default()
        };
    };

    if let Some(err) = root.get("error") {
        let msg = err
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("API error");
        return StateBallotExtras {
            notes: vec![format!("Google Civic error: {msg}")],
            coverage_label: Some("Google Civic (error)".into()),
            ..Default::default()
        };
    }

    let status = root
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("success");
    let mut notes = Vec::new();
    if status != "success" {
        notes.push(format!(
            "Google Civic address status: {status} (partial data may still appear)."
        ));
    }

    let election_name = root
        .pointer("/election/name")
        .and_then(|v| v.as_str())
        .unwrap_or("election")
        .to_string();
    let election_day = root
        .pointer("/election/electionDay")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let election_id = root
        .pointer("/election/id")
        .map(|v| match v {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            _ => String::new(),
        })
        .unwrap_or_default();

    let source_url = ballot_info_url(&root).unwrap_or_else(|| CIVIC_DOCS_URL.to_string());

    notes.push(format!(
        "Google Civic / VIP: {election_name}{} (id {election_id}). Address matched via ZIP centroid — precinct may differ.",
        if election_day.is_empty() {
            String::new()
        } else {
            format!(" · {election_day}")
        }
    ));

    let contests = root
        .get("contests")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut candidates = Vec::new();
    let mut measures = Vec::new();
    let mut extra_jurisdictions = Vec::new();
    let mut seen_jur = BTreeSet::new();
    let mut skipped_federal = 0usize;

    for c in &contests {
        if contest_is_federal(c) {
            skipped_federal += 1;
            continue;
        }
        let ctype = c
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if ctype.contains("referendum") || ctype.contains("ballot-measure") || ctype == "ballot measure"
        {
            if let Some(m) = map_measure(c, geo, &source_url) {
                push_jur(&mut extra_jurisdictions, &mut seen_jur, &m.jurisdiction_ocd, c);
                measures.push(m);
            }
            continue;
        }
        // Candidate contest (General / Primary / Run-off / …)
        let office = contest_office_label(c);
        if office.is_empty() {
            continue;
        }
        let (chamber, is_judicial) = map_chamber(c, &office);
        let ocd = jurisdiction_ocd(c, geo, chamber.as_deref());
        if ocd.is_empty() {
            continue;
        }
        push_jur(&mut extra_jurisdictions, &mut seen_jur, &ocd, c);

        let rows = c
            .get("candidates")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        if rows.is_empty() {
            continue;
        }
        for cand in rows {
            let name = cand
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            if name.is_empty() {
                continue;
            }
            let party = cand
                .get("party")
                .and_then(|v| v.as_str())
                .map(|p| {
                    let cleaned = p
                        .trim()
                        .trim_start_matches("Party Preference:")
                        .trim()
                        .to_string();
                    if cleaned.is_empty() || cleaned.eq_ignore_ascii_case("none") {
                        "Unknown".into()
                    } else {
                        normalize_party_label(&cleaned)
                    }
                })
                .unwrap_or_else(|| "Unknown".into());
            let cand_url = cand
                .get("candidateUrl")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .unwrap_or(source_url.as_str())
                .to_string();
            candidates.push(SnapshotCandidate {
                office: office.clone(),
                chamber: chamber.clone(),
                jurisdiction_ocd: ocd.clone(),
                is_judicial,
                name,
                party,
                is_incumbent: false,
                is_judge: is_judicial,
                summary: None,
                source_url: cand_url,
                source_publisher: Some(CIVIC_PUBLISHER.into()),
                external_id: None,
            });
        }
    }

    if skipped_federal > 0 {
        notes.push(format!(
            "Skipped {skipped_federal} federal contest(s) from Civic (OpenFEC owns federal)."
        ));
    }
    if candidates.is_empty() && measures.is_empty() {
        notes.push(
            "Google Civic returned no state/local contests for this address/election (VIP coverage is seasonal — often only near election day)."
                .into(),
        );
        return StateBallotExtras {
            notes,
            coverage_label: Some("Google Civic (empty)".into()),
            ..Default::default()
        };
    }

    notes.push(format!(
        "Mapped {} Civic candidate(s) + {} measure(s) for this address.",
        candidates.len(),
        measures.len()
    ));
    notes.push(format!("Source: {CIVIC_PUBLISHER} · {source_url}"));

    StateBallotExtras {
        candidates,
        measures,
        coverage_label: Some(format!("Google Civic / VIP ({election_name})")),
        notes,
        extra_jurisdictions,
    }
}

/// Prefer Civic full slate over thin incumbent/OS coverage; keep real official measures when present.
pub fn merge_civic_into(existing: &mut StateBallotExtras, civic: StateBallotExtras) {
    if civic.is_empty() && civic.notes.is_empty() {
        return;
    }
    existing.notes.extend(civic.notes);
    existing
        .extra_jurisdictions
        .extend(civic.extra_jurisdictions);

    if !civic.candidates.is_empty() {
        let thin = existing.candidates.is_empty()
            || existing.coverage_label.as_deref().is_some_and(|l| {
                let u = l.to_ascii_lowercase();
                u.contains("incumbent")
                    || u.contains("open states")
                    || u.contains("roster")
                    || u.contains("officiallist")
            })
            || civic.candidates.len() >= existing.candidates.len();
        if thin {
            existing.candidates = civic.candidates;
            if civic.coverage_label.is_some() {
                existing.coverage_label = civic.coverage_label;
            }
        }
    } else if existing.coverage_label.is_none() {
        existing.coverage_label = civic.coverage_label;
    }

    if !civic.measures.is_empty() {
        if !measures_look_real(&existing.measures) {
            existing.measures = civic.measures;
        }
    }
}

fn measures_look_real(m: &[SnapshotMeasure]) -> bool {
    m.iter().any(|x| {
        let title = x.title.to_ascii_lowercase();
        if title.contains("see official") || title.contains("ballot measures") && x.summary.is_none()
        {
            return false;
        }
        x.measure_code.as_ref().is_some_and(|c| !c.is_empty())
            || x.summary.as_ref().is_some_and(|s| s.len() > 40)
            || (!title.is_empty() && !title.contains("link"))
                && m.len() > 1
    })
}

fn ballot_info_url(root: &Value) -> Option<String> {
    root.pointer("/state/0/electionAdministrationBody/ballotInfoUrl")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| {
            root.pointer("/state/0/electionAdministrationBody/electionInfoUrl")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
        })
}

fn contest_is_federal(c: &Value) -> bool {
    let levels = c
        .get("level")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str())
                .map(|s| s.to_ascii_lowercase())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if levels.iter().any(|l| l == "country" || l == "international") {
        return true;
    }
    let scope = c
        .pointer("/district/scope")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if scope == "congressional" || scope == "national" {
        return true;
    }
    let roles = c
        .get("roles")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str())
                .map(|s| s.to_ascii_lowercase())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if roles.iter().any(|r| {
        matches!(
            r.as_str(),
            "headofstate" | "deputyheadofgovernment" | "headofgovernment"
        ) && levels.iter().any(|l| l == "country")
    }) {
        return true;
    }
    // US Senate is headOfGovernment-ish at country; also catch by office text
    let office = contest_office_label(c).to_ascii_lowercase();
    if office.contains("united states")
        || office.contains("u.s. house")
        || office.contains("u.s. senate")
        || office.contains("us house")
        || office.contains("us senate")
        || office.contains("u.s. representative")
        || office.contains("u.s. senator")
        || office.contains("president of the united states")
    {
        return true;
    }
    false
}

fn contest_office_label(c: &Value) -> String {
    c.get("ballotTitle")
        .and_then(|v| v.as_str())
        .or_else(|| c.get("office").and_then(|v| v.as_str()))
        .unwrap_or("")
        .trim()
        .to_string()
}

fn map_chamber(c: &Value, office: &str) -> (Option<String>, bool) {
    let roles = c
        .get("roles")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str())
                .map(|s| s.to_ascii_lowercase())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let scope = c
        .pointer("/district/scope")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let office_l = office.to_ascii_lowercase();

    if roles.iter().any(|r| r == "legislatorupperbody") || scope == "stateupper" {
        return (Some("state_senate".into()), false);
    }
    if roles.iter().any(|r| r == "legislatorlowerbody") || scope == "statelower" {
        return (Some("state_house".into()), false);
    }
    if roles
        .iter()
        .any(|r| r == "highestcourtjudge" || r == "judge")
        || scope == "judicial"
        || office_l.contains("judge")
        || office_l.contains("justice")
    {
        return (Some("judicial".into()), true);
    }
    if roles.iter().any(|r| {
        matches!(
            r.as_str(),
            "headofgovernment" | "deputyheadofgovernment" | "governmentofficer" | "executivecouncil"
        )
    }) || scope == "statewide"
        || office_l.contains("governor")
        || office_l.contains("attorney general")
        || office_l.contains("secretary of state")
        || office_l.contains("state treasurer")
        || office_l.contains("superintendent")
        || office_l.contains("corporation commission")
        || office_l.contains("mine inspector")
    {
        return (Some("statewide".into()), false);
    }
    if scope == "schoolboard" || roles.iter().any(|r| r == "schoolboard") {
        return (Some("local".into()), false);
    }
    if matches!(
        scope.as_str(),
        "countywide"
            | "county council"
            | "countycouncil"
            | "citywide"
            | "citycouncil"
            | "ward"
            | "township"
            | "special"
    ) {
        return (Some("local".into()), false);
    }
    // Fallbacks from office text
    if office_l.contains("state senate") || office_l.contains("state senator") {
        return (Some("state_senate".into()), false);
    }
    if office_l.contains("state house")
        || office_l.contains("state representative")
        || office_l.contains("house of representatives") && office_l.contains("district")
    {
        // avoid US House: already filtered as federal
        return (Some("state_house".into()), false);
    }
    if office_l.contains("county")
        || office_l.contains("mayor")
        || office_l.contains("city council")
        || office_l.contains("school")
        || office_l.contains("sheriff")
    {
        return (Some("local".into()), false);
    }
    (Some("local".into()), false)
}

fn jurisdiction_ocd(c: &Value, geo: &GeoResolution, chamber: Option<&str>) -> String {
    if let Some(id) = c
        .pointer("/district/id")
        .and_then(|v| v.as_str())
        .filter(|s| s.starts_with("ocd-division/"))
    {
        return id.to_string();
    }
    let st = geo.state.to_ascii_lowercase();
    let scope = c
        .pointer("/district/scope")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let dist_name = c
        .pointer("/district/name")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let dist_num = extract_district_num(c, dist_name);

    match chamber {
        Some("state_senate") => {
            let d = dist_num
                .or(geo.state_senate_district)
                .unwrap_or(0);
            if d > 0 {
                return format!("ocd-division/country:us/state:{st}/sldu:{d}");
            }
        }
        Some("state_house") => {
            let d = dist_num.or(geo.state_house_district).unwrap_or(0);
            if d > 0 {
                return format!("ocd-division/country:us/state:{st}/sldl:{d}");
            }
        }
        Some("statewide") | Some("judicial") => {
            return format!("ocd-division/country:us/state:{st}");
        }
        _ => {}
    }
    if scope == "statewide" || scope == "judicial" {
        return format!("ocd-division/country:us/state:{st}");
    }
    if !geo.county.trim().is_empty()
        && (scope.contains("county")
            || chamber == Some("local")
            || dist_name.to_ascii_lowercase().contains("county"))
    {
        let slug = slugify(&geo.county.replace(" County", ""));
        return format!("ocd-division/country:us/state:{st}/county:{slug}");
    }
    format!("ocd-division/country:us/state:{st}")
}

fn extract_district_num(c: &Value, dist_name: &str) -> Option<u32> {
    if let Some(id) = c.pointer("/district/id").and_then(|v| v.as_str()) {
        if let Ok(n) = id.parse::<u32>() {
            return Some(n);
        }
        // trailing digits
        let digits: String = id
            .chars()
            .rev()
            .take_while(|ch| ch.is_ascii_digit())
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        if let Ok(n) = digits.parse::<u32>() {
            if n > 0 {
                return Some(n);
            }
        }
    }
    let re_digits: String = dist_name
        .chars()
        .filter(|ch| ch.is_ascii_digit())
        .collect();
    // Prefer last number group in name e.g. "Legislative District 8"
    let mut last = None;
    let mut cur = String::new();
    for ch in dist_name.chars() {
        if ch.is_ascii_digit() {
            cur.push(ch);
        } else if !cur.is_empty() {
            if let Ok(n) = cur.parse::<u32>() {
                last = Some(n);
            }
            cur.clear();
        }
    }
    if !cur.is_empty() {
        if let Ok(n) = cur.parse::<u32>() {
            last = Some(n);
        }
    }
    last.or_else(|| re_digits.parse().ok())
}

fn map_measure(c: &Value, geo: &GeoResolution, source_url: &str) -> Option<SnapshotMeasure> {
    let title = c
        .get("referendumTitle")
        .and_then(|v| v.as_str())
        .or_else(|| c.get("ballotTitle").and_then(|v| v.as_str()))
        .or_else(|| c.get("office").and_then(|v| v.as_str()))
        .unwrap_or("")
        .trim()
        .to_string();
    if title.is_empty() {
        return None;
    }
    let summary = c
        .get("referendumBrief")
        .and_then(|v| v.as_str())
        .or_else(|| c.get("referendumSubtitle").and_then(|v| v.as_str()))
        .or_else(|| c.get("referendumText").and_then(|v| v.as_str()))
        .map(|s| {
            let t = s.trim();
            if t.len() > 600 {
                format!("{}…", &t[..600])
            } else {
                t.to_string()
            }
        })
        .filter(|s| !s.is_empty());
    let code = measure_code_from_title(&title);
    let url = c
        .get("referendumUrl")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(source_url)
        .to_string();
    let ocd = jurisdiction_ocd(c, geo, Some("local"));
    Some(SnapshotMeasure {
        title,
        measure_code: code,
        jurisdiction_ocd: ocd,
        summary,
        source_url: url,
        source_publisher: Some(CIVIC_PUBLISHER.into()),
    })
}

fn measure_code_from_title(title: &str) -> Option<String> {
    let u = title.to_ascii_uppercase();
    for prefix in ["PROPOSITION ", "PROP. ", "PROP ", "MEASURE ", "QUESTION ", "AMENDMENT "] {
        if let Some(rest) = u.strip_prefix(prefix) {
            let code: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '.')
                .collect();
            if !code.is_empty() {
                let label = prefix.trim();
                return Some(format!("{label} {code}"));
            }
        }
    }
    // "PROPOSITION 139" mid-string
    if let Some(i) = u.find("PROPOSITION ") {
        let rest = &u[i + "PROPOSITION ".len()..];
        let code: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .collect();
        if !code.is_empty() {
            return Some(format!("Proposition {code}"));
        }
    }
    None
}

fn push_jur(
    extra: &mut Vec<ResolvedJurisdiction>,
    seen: &mut BTreeSet<String>,
    ocd: &str,
    c: &Value,
) {
    if ocd.is_empty() || !seen.insert(ocd.to_string()) {
        return;
    }
    let name = c
        .pointer("/district/name")
        .and_then(|v| v.as_str())
        .or_else(|| c.get("ballotTitle").and_then(|v| v.as_str()))
        .or_else(|| c.get("office").and_then(|v| v.as_str()))
        .unwrap_or(ocd)
        .to_string();
    let level = c
        .pointer("/district/scope")
        .and_then(|v| v.as_str())
        .unwrap_or("state")
        .to_string();
    extra.push(ResolvedJurisdiction {
        ocd_id: ocd.to_string(),
        name,
        level,
        state: None,
    });
}

fn slugify(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if ch == ' ' || ch == '-' || ch == '_' {
            if !out.ends_with('_') {
                out.push('_');
            }
        }
    }
    out.trim_matches('_').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::GeoResolution;

    fn az_geo() -> GeoResolution {
        GeoResolution {
            state: "AZ".into(),
            state_name: "Arizona".into(),
            county: "Maricopa".into(),
            city: "Phoenix".into(),
            congressional_district: "3".into(),
            state_senate_district: Some(5),
            state_house_district: Some(5),
            state_house_label: None,
            latitude: Some(33.4),
            longitude: Some(-112.0),
            jurisdictions: vec![],
            source_url: String::new(),
            source_publisher: String::new(),
        }
    }

    #[test]
    fn parses_state_and_measure_skips_federal() {
        let json = include_str!("../../../testdata/civic_voterinfo_az_sample.json");
        let ex = civic_extras_from_voterinfo(&az_geo(), json);
        assert!(
            ex.candidates.iter().all(|c| {
                let o = c.office.to_ascii_lowercase();
                !o.contains("united states representative")
            }),
            "federal should be skipped"
        );
        assert!(
            ex.candidates
                .iter()
                .any(|c| c.chamber.as_deref() == Some("statewide")),
            "governor-like statewide expected"
        );
        assert!(
            ex.candidates
                .iter()
                .any(|c| c.chamber.as_deref() == Some("state_senate")),
            "state senate expected"
        );
        assert!(
            ex.candidates
                .iter()
                .any(|c| c.name.contains("Hobbs") || c.party == "Democratic"),
            "named candidates"
        );
        assert!(!ex.measures.is_empty(), "referendum expected");
        assert!(ex
            .coverage_label
            .as_deref()
            .unwrap_or("")
            .contains("Google Civic"));
        assert!(ex
            .candidates
            .iter()
            .all(|c| c.source_publisher.as_deref() == Some(CIVIC_PUBLISHER)));
    }

    #[test]
    fn merge_prefers_civic_over_thin_incumbents() {
        let mut existing = StateBallotExtras {
            candidates: vec![SnapshotCandidate {
                office: "Arizona Senate (District 5)".into(),
                chamber: Some("state_senate".into()),
                jurisdiction_ocd: "ocd-division/country:us/state:az/sldu:5".into(),
                is_judicial: false,
                name: "Incumbent Only".into(),
                party: "Republican".into(),
                is_incumbent: true,
                is_judge: false,
                summary: None,
                source_url: "https://example.com".into(),
                source_publisher: Some("roster".into()),
                external_id: None,
            }],
            coverage_label: Some("AZ legislature incumbents (roster)".into()),
            ..Default::default()
        };
        let json = include_str!("../../../testdata/civic_voterinfo_az_sample.json");
        let civic = civic_extras_from_voterinfo(&az_geo(), json);
        let n = civic.candidates.len();
        assert!(n > 1);
        merge_civic_into(&mut existing, civic);
        assert_eq!(existing.candidates.len(), n);
        assert!(!existing.candidates.iter().any(|c| c.name == "Incumbent Only"));
    }

    #[test]
    fn empty_error_json() {
        let ex = civic_extras_from_voterinfo(
            &az_geo(),
            r#"{"error":{"message":"API key not valid."}}"#,
        );
        assert!(ex.candidates.is_empty());
        assert!(ex.notes.iter().any(|n| n.contains("API key")));
    }
}
