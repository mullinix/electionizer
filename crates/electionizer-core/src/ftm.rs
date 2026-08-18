//! FollowTheMoney / OpenSecrets state campaign-finance API (pure parse/match, no HTTP).
//!
//! Docs: https://www.followthemoney.org/our-data/apis/
//! Base: `https://api.followthemoney.org/?APIKey=…&mode=json&…`
//! Entity: `https://api.followthemoney.org/entity.php?eid=…&APIKey=…&mode=json`
//!
//! FTM state data is published through the 2024 cycle (site note); newer cycles fall back to 2024.

use crate::models::{format_usd, ContributorRow};
use crate::states::florida::{first_names_compatible, last_names_match};
use serde_json::Value;

pub const FTM_API_BASE: &str = "https://api.followthemoney.org/";
pub const FTM_ENTITY_BASE: &str = "https://api.followthemoney.org/entity.php";
pub const FTM_SITE: &str = "https://www.followthemoney.org";
/// Latest election year with published FTM state CF (per followthemoney.org banner).
pub const FTM_MAX_YEAR: i32 = 2024;

/// Map ballot chamber/office → FTM `c-r-ot` office-type code when known.
///
/// Common codes: G gubernatorial, S state senate, H state house, J judicial,
/// K other statewide, U US House, L US Senate.
pub fn ftm_office_type_code(chamber: &str, office: &str) -> Option<&'static str> {
    let ch = chamber.trim();
    let o = office.to_ascii_lowercase();
    match ch {
        "state_senate" => Some("S"),
        "state_house" => Some("H"),
        "judicial" => Some("J"),
        "statewide" | "state_exec" => {
            if o.contains("governor") && !o.contains("lieutenant") {
                Some("G")
            } else if o.contains("lieutenant") {
                Some("G")
            } else {
                Some("K")
            }
        }
        "us_senate" | "senate" if o.contains("united states") || o.contains("u.s.") => Some("L"),
        "us_house" | "house" if o.contains("united states") || o.contains("u.s.") => Some("U"),
        _ => {
            if o.contains("state senator") || o.contains("state senate") {
                Some("S")
            } else if o.contains("state representative") || o.contains("state house") {
                Some("H")
            } else if o.contains("governor") {
                Some("G")
            } else if o.contains("judge") || o.contains("justice") || o.contains("judicial") {
                Some("J")
            } else {
                None
            }
        }
    }
}

/// FTM published year for a ballot cycle (cap at [`FTM_MAX_YEAR`]).
pub fn ftm_data_year(cycle: i32) -> i32 {
    if cycle <= 0 {
        FTM_MAX_YEAR
    } else {
        cycle.min(FTM_MAX_YEAR)
    }
}

fn urlenc(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for b in s.as_bytes() {
        match *b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*b as char);
            }
            b' ' => out.push('+'),
            _ => {
                out.push('%');
                out.push_str(&format!("{b:02X}"));
            }
        }
    }
    out
}

/// Candidates grouped by `c-t-id` for a state/year (and optional office type).
pub fn ftm_candidates_url(
    api_key: &str,
    state: &str,
    year: i32,
    office_type: Option<&str>,
) -> String {
    let mut q = format!(
        "APIKey={}&mode=json&s={}&y={}&gro=c-t-id&so=u-tot&sod=0",
        urlenc(api_key.trim()),
        urlenc(state.trim().to_ascii_uppercase().as_str()),
        year
    );
    if let Some(ot) = office_type.map(str::trim).filter(|s| !s.is_empty()) {
        q.push_str(&format!("&c-r-ot={}", urlenc(ot)));
    }
    format!("{FTM_API_BASE}?{q}")
}

/// Top contributors for one FTM candidate entity id.
pub fn ftm_top_donors_url(api_key: &str, candidate_id: &str, state: &str, year: i32) -> String {
    format!(
        "{FTM_API_BASE}?APIKey={}&mode=json&s={}&y={}&c-t-id={}&gro=d-eid&so=u-tot&sod=0",
        urlenc(api_key.trim()),
        urlenc(state.trim().to_ascii_uppercase().as_str()),
        year,
        urlenc(candidate_id.trim())
    )
}

pub fn ftm_entity_url(api_key: &str, eid: &str) -> String {
    format!(
        "{FTM_ENTITY_BASE}?eid={}&APIKey={}&mode=json",
        urlenc(eid.trim()),
        urlenc(api_key.trim())
    )
}

pub fn ftm_profile_url(eid: &str) -> String {
    format!("{FTM_SITE}/entity-details?eid={}", urlenc(eid.trim()))
}

/// Public citation URL (no API key) for the underlying show-me query.
pub fn ftm_show_me_url(state: &str, year: i32, office_type: Option<&str>) -> String {
    let mut q = format!(
        "s={}&y={}&gro=c-t-id",
        urlenc(state.trim().to_ascii_uppercase().as_str()),
        year
    );
    if let Some(ot) = office_type.map(str::trim).filter(|s| !s.is_empty()) {
        q.push_str(&format!("&c-r-ot={}", urlenc(ot)));
    }
    format!("{FTM_SITE}/show-me?{q}")
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FtmCandidateHit {
    pub id: String,
    pub name: String,
    pub party: String,
    pub office: String,
    pub office_sought: String,
    pub election_state: String,
    pub election_year: String,
    pub total_amount: f64,
    pub total_display: String,
    pub record_count: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct FtmMatchQuery {
    pub name: String,
    pub office: String,
    pub chamber: String,
    pub party: String,
    pub district: Option<u32>,
    pub state: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FtmMatch {
    Unique { hit: FtmCandidateHit },
    None,
    Ambiguous { count: usize },
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FtmFinance {
    pub source: String,
    pub candidate_id: String,
    pub candidate_name: String,
    pub cycle: String,
    pub state: String,
    pub receipts_display: String,
    pub total_amount: f64,
    pub office: String,
    pub office_sought: String,
    pub party: String,
    pub profile_url: String,
    pub source_label: String,
    pub note: String,
    pub show_me_url: String,
    pub top_contributors: Vec<ContributorRow>,
}

/// Extract error message from FTM JSON if present.
pub fn ftm_error_message(json: &str) -> Option<String> {
    let v: Value = serde_json::from_str(json).ok()?;
    if let Some(e) = v.get("error").and_then(|x| x.as_str()) {
        return Some(e.to_string());
    }
    // XML-ish error sometimes returned as plain text
    let t = json.trim();
    if t.starts_with("<error>") {
        let inner = t
            .trim_start_matches("<error>")
            .trim_end_matches("</error>")
            .trim();
        if !inner.is_empty() {
            return Some(inner.to_string());
        }
    }
    None
}

fn as_object_map(v: &Value) -> Option<&serde_json::Map<String, Value>> {
    v.as_object()
}

/// Read a nested FTM field: object with same-named display key, or plain string/number.
fn field_display(rec: &Value, names: &[&str]) -> Option<String> {
    let obj = as_object_map(rec)?;
    for name in names {
        if let Some(v) = obj.get(*name) {
            if let Some(s) = v.as_str() {
                let t = s.trim();
                if !t.is_empty() {
                    return Some(t.to_string());
                }
            }
            if let Some(n) = v.as_f64() {
                return Some(n.to_string());
            }
            if let Some(m) = v.as_object() {
                // Prefer display key matching field name, else first string value that isn't token/id
                if let Some(s) = m.get(*name).and_then(|x| x.as_str()) {
                    let t = s.trim();
                    if !t.is_empty() {
                        return Some(t.to_string());
                    }
                }
                for (k, val) in m {
                    if k == "token" || k == "id" {
                        continue;
                    }
                    if let Some(s) = val.as_str() {
                        let t = s.trim();
                        if !t.is_empty() {
                            return Some(t.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

fn field_id(rec: &Value, names: &[&str]) -> Option<String> {
    let obj = as_object_map(rec)?;
    for name in names {
        if let Some(v) = obj.get(*name) {
            if let Some(m) = v.as_object() {
                if let Some(id) = m.get("id").and_then(|x| x.as_str()) {
                    let t = id.trim();
                    if !t.is_empty() && t != "token value" {
                        return Some(t.to_string());
                    }
                }
            }
            if let Some(s) = v.as_str() {
                let t = s.trim();
                if !t.is_empty() && t.chars().all(|c| c.is_ascii_digit()) {
                    return Some(t.to_string());
                }
            }
        }
    }
    None
}

fn parse_amount(raw: &str) -> f64 {
    raw.replace(['$', ',', ' '], "")
        .trim()
        .parse()
        .unwrap_or(0.0)
}

/// Convert FTM "LAST, FIRST M" → ballot-style "FIRST M LAST".
pub fn ftm_name_to_ballot(raw: &str) -> String {
    let s = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    if let Some((last, rest)) = s.split_once(',') {
        let last = last.trim();
        let rest = rest.trim();
        if !last.is_empty() && !rest.is_empty() {
            return format!("{rest} {last}");
        }
    }
    s
}

fn records_array(root: &Value) -> Vec<&Value> {
    match root.get("records") {
        Some(Value::Array(a)) => a.iter().collect(),
        Some(Value::Object(m)) => {
            // Sometimes a single record object
            if m.contains_key("record") {
                match &m["record"] {
                    Value::Array(a) => a.iter().collect(),
                    v => vec![v],
                }
            } else {
                vec![]
            }
        }
        _ => vec![],
    }
}

/// Parse candidate-grouped FTM JSON (`gro=c-t-id`).
pub fn parse_ftm_candidate_records(json: &str) -> Result<Vec<FtmCandidateHit>, String> {
    if let Some(err) = ftm_error_message(json) {
        return Err(err);
    }
    let root: Value =
        serde_json::from_str(json).map_err(|e| format!("FTM JSON parse error: {e}"))?;
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for rec in records_array(&root) {
        let id = field_id(rec, &["Candidate", "c-t-id"]).unwrap_or_default();
        let name_raw = field_display(rec, &["Candidate", "c-t-id"]).unwrap_or_default();
        if id.is_empty() || name_raw.is_empty() {
            continue;
        }
        if !seen.insert(id.clone()) {
            continue;
        }
        let party = field_display(rec, &["Political_Party", "Party_Details", "c-t-p", "c-t-pt"])
            .unwrap_or_default();
        let office = field_display(rec, &["Office", "Type_of_Office", "c-r-oc", "c-r-ot"])
            .unwrap_or_default();
        let office_sought =
            field_display(rec, &["Office_Sought", "c-r-osid"]).unwrap_or_default();
        // Prefer token id (AZ) over display label (Arizona) for matching.
        let election_state = field_id(rec, &["Election_State", "s"])
            .or_else(|| field_display(rec, &["Election_State", "s"]))
            .unwrap_or_default();
        let election_year = field_id(rec, &["Election_Year", "y"])
            .or_else(|| field_display(rec, &["Election_Year", "y"]))
            .unwrap_or_default();
        let total_s = field_display(rec, &["Total_$", "Total_Dollars", "u-tot"]).unwrap_or_default();
        let total_amount = parse_amount(&total_s);
        let record_count = field_display(rec, &["#_of_Records", "Num_of_Records", "u-rec"])
            .and_then(|s| s.replace(',', "").parse().ok());
        out.push(FtmCandidateHit {
            id,
            name: ftm_name_to_ballot(&name_raw),
            party,
            office,
            office_sought,
            election_state,
            election_year,
            total_amount,
            total_display: format_usd(total_amount),
            record_count,
        });
    }
    Ok(out)
}

/// Parse contributor-grouped FTM JSON (`gro=d-eid`) → top donor rows.
pub fn parse_ftm_donor_records(json: &str, limit: usize, profile_url: &str) -> Result<Vec<ContributorRow>, String> {
    if let Some(err) = ftm_error_message(json) {
        return Err(err);
    }
    let root: Value =
        serde_json::from_str(json).map_err(|e| format!("FTM JSON parse error: {e}"))?;
    let mut rows = Vec::new();
    for rec in records_array(&root) {
        let name = field_display(rec, &["Contributor", "d-eid"]).unwrap_or_default();
        if name.is_empty() {
            continue;
        }
        let total_s = field_display(rec, &["Total_$", "Total_Dollars", "u-tot"]).unwrap_or_default();
        let total = parse_amount(&total_s);
        let gift_count = field_display(rec, &["#_of_Records", "Num_of_Records", "u-rec"])
            .and_then(|s| s.replace(',', "").parse().ok());
        let loc = field_display(rec, &["Type_of_Contributor", "d-et"]);
        rows.push((total, ContributorRow {
            name,
            amount_display: format_usd(total),
            location: loc,
            date: None,
            url: profile_url.to_string(),
            gift_count,
        }));
    }
    rows.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.1.name.cmp(&b.1.name))
    });
    Ok(rows
        .into_iter()
        .take(limit.max(1))
        .map(|(_, r)| r)
        .collect())
}

fn district_from_text(s: &str) -> Option<u32> {
    let re = regex::Regex::new(r"(?i)\b(?:district|dist\.?|hd|sd)\s*#?(\d+)\b").ok()?;
    if let Some(c) = re.captures(s) {
        return c.get(1).and_then(|m| m.as_str().parse().ok());
    }
    // trailing number on codes like "HOUSE DISTRICT 008"
    let re2 = regex::Regex::new(r"(?i)\b(\d{1,3})\s*$").ok()?;
    re2.captures(s)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

fn office_family_compatible(query_chamber: &str, query_office: &str, hit: &FtmCandidateHit) -> bool {
    let want = ftm_office_type_code(query_chamber, query_office);
    let blob = format!(
        "{} {} {}",
        hit.office, hit.office_sought, hit.name
    )
    .to_ascii_uppercase();
    let q_office = query_office.to_ascii_uppercase();

    if let Some(code) = want {
        // Prefer explicit office type codes in Office / Type_of_Office when present.
        let hit_ot = hit.office.to_ascii_uppercase();
        match code {
            "S" => {
                return blob.contains("SENATE")
                    || hit_ot.contains("SENATE")
                    || hit_ot == "S"
                    || q_office.contains("SENATE");
            }
            "H" => {
                return (blob.contains("HOUSE") || blob.contains("REPRESENTATIVE") || hit_ot == "H")
                    && !blob.contains("UNITED STATES");
            }
            "G" => {
                return blob.contains("GOVERNOR") || hit_ot.starts_with('G') || hit_ot == "G";
            }
            "J" => {
                return blob.contains("JUDGE")
                    || blob.contains("JUSTICE")
                    || blob.contains("JUDICIAL")
                    || blob.contains("COURT")
                    || hit_ot == "J";
            }
            "K" => {
                return !blob.contains("SENATE")
                    && !blob.contains("HOUSE")
                    && !blob.contains("JUDGE");
            }
            _ => {}
        }
    }
    // No chamber filter — name+district must carry match.
    true
}

fn state_name_matches(code: &str, label: &str) -> bool {
    // Loose: "AZ" matches "ARIZONA"; full names match when equal after normalize.
    let code = code.trim().to_ascii_uppercase();
    let label = label.trim().to_ascii_uppercase();
    if code.is_empty() || label.is_empty() {
        return false;
    }
    if code == label {
        return true;
    }
    const PAIRS: &[(&str, &str)] = &[
        ("AZ", "ARIZONA"),
        ("CA", "CALIFORNIA"),
        ("NY", "NEW YORK"),
        ("TX", "TEXAS"),
        ("NC", "NORTH CAROLINA"),
        ("MD", "MARYLAND"),
        ("FL", "FLORIDA"),
        ("WA", "WASHINGTON"),
        ("OR", "OREGON"),
        ("NV", "NEVADA"),
        ("CO", "COLORADO"),
        ("GA", "GEORGIA"),
        ("PA", "PENNSYLVANIA"),
        ("OH", "OHIO"),
        ("IL", "ILLINOIS"),
        ("MI", "MICHIGAN"),
        ("VA", "VIRGINIA"),
        ("MA", "MASSACHUSETTS"),
        ("NJ", "NEW JERSEY"),
        ("MN", "MINNESOTA"),
        ("WI", "WISCONSIN"),
        ("MO", "MISSOURI"),
        ("TN", "TENNESSEE"),
        ("IN", "INDIANA"),
        ("SC", "SOUTH CAROLINA"),
        ("AL", "ALABAMA"),
        ("LA", "LOUISIANA"),
        ("KY", "KENTUCKY"),
        ("OK", "OKLAHOMA"),
        ("CT", "CONNECTICUT"),
        ("IA", "IOWA"),
        ("MS", "MISSISSIPPI"),
        ("AR", "ARKANSAS"),
        ("KS", "KANSAS"),
        ("UT", "UTAH"),
        ("NV", "NEVADA"),
        ("NM", "NEW MEXICO"),
        ("NE", "NEBRASKA"),
        ("WV", "WEST VIRGINIA"),
        ("ID", "IDAHO"),
        ("HI", "HAWAII"),
        ("NH", "NEW HAMPSHIRE"),
        ("ME", "MAINE"),
        ("MT", "MONTANA"),
        ("RI", "RHODE ISLAND"),
        ("DE", "DELAWARE"),
        ("SD", "SOUTH DAKOTA"),
        ("ND", "NORTH DAKOTA"),
        ("AK", "ALASKA"),
        ("VT", "VERMONT"),
        ("WY", "WYOMING"),
        ("DC", "DISTRICT OF COLUMBIA"),
    ];
    PAIRS.iter().any(|(c, n)| {
        (*c == code && *n == label.as_str()) || (*n == code && *c == label.as_str())
    })
}

fn parties_conflict(query_party: &str, hit_party: &str) -> bool {
    let norm = |p: &str| -> Option<&'static str> {
        let u = p.trim().to_ascii_uppercase();
        if u.is_empty() {
            return None;
        }
        if u.starts_with("DEM") || u.contains("DEMOCRATIC") {
            return Some("DEM");
        }
        if u.starts_with("REP") || u.contains("REPUBLICAN") {
            return Some("REP");
        }
        None
    };
    match (norm(query_party), norm(hit_party)) {
        (Some(a), Some(b)) => a != b,
        _ => false,
    }
}

/// Strict match: last name, first compatible, office family, district when both known; skip ambiguity.
pub fn match_ftm_candidate(hits: &[FtmCandidateHit], q: &FtmMatchQuery) -> FtmMatch {
    let q_dist = q.district.or_else(|| {
        district_from_text(&q.office)
    });
    let st = q.state.trim().to_ascii_uppercase();

    let matched: Vec<&FtmCandidateHit> = hits
        .iter()
        .filter(|h| {
            if st.is_empty() {
                true
            } else {
                let hs = h.election_state.trim().to_ascii_uppercase();
                hs.is_empty()
                    || hs == st
                    || hs == "US"
                    || hs.starts_with(&st)
                    || state_name_matches(&st, &hs)
            }
        })
        .filter(|h| last_names_match(&q.name, &h.name))
        .filter(|h| first_names_compatible(&q.name, &h.name))
        .filter(|h| office_family_compatible(&q.chamber, &q.office, h))
        .filter(|h| {
            let hit_dist = district_from_text(&h.office_sought)
                .or_else(|| district_from_text(&h.office));
            match (q_dist, hit_dist) {
                (Some(want), Some(got)) => want == got,
                _ => true,
            }
        })
        .filter(|h| !parties_conflict(&q.party, &h.party))
        .collect();

    match matched.len() {
        0 => FtmMatch::None,
        1 => FtmMatch::Unique {
            hit: matched[0].clone(),
        },
        n => FtmMatch::Ambiguous { count: n },
    }
}

/// Build finance block from a unique hit (+ optional top donors).
pub fn ftm_finance_from_hit(
    hit: &FtmCandidateHit,
    state: &str,
    year: i32,
    office_type: Option<&str>,
    top: Vec<ContributorRow>,
) -> FtmFinance {
    FtmFinance {
        source: "ftm".into(),
        candidate_id: hit.id.clone(),
        candidate_name: hit.name.clone(),
        cycle: year.to_string(),
        state: state.to_ascii_uppercase(),
        receipts_display: hit.total_display.clone(),
        total_amount: hit.total_amount,
        office: hit.office.clone(),
        office_sought: hit.office_sought.clone(),
        party: hit.party.clone(),
        profile_url: ftm_profile_url(&hit.id),
        source_label: "FollowTheMoney / OpenSecrets (state campaign finance)".into(),
        note: format!(
            "State campaign-finance totals from FollowTheMoney.org (data through {FTM_MAX_YEAR}). Not a certified cash-on-hand figure."
        ),
        show_me_url: ftm_show_me_url(state, year, office_type),
        top_contributors: top,
    }
}

/// Redact API key from URL for logging/display.
pub fn ftm_redact_url(url: &str) -> String {
    let re = regex::Regex::new(r"(?i)(APIKey=)[^&]+").ok();
    if let Some(re) = re {
        re.replace_all(url, "${1}REDACTED").to_string()
    } else {
        url.to_string()
    }
}

// --- Ballot measure finance (public HTML via aafetch / edetails; no API key) ---

/// List measures for a state/year (`aafetch.php?y=&s=&gro=m-id,s,y`).
pub fn ftm_measures_list_url(state: &str, year: i32) -> String {
    format!(
        "{FTM_SITE}/aaengine/aafetch.php?y={year}&s={}&gro=m-id,s,y",
        urlenc(state.trim().to_ascii_uppercase().as_str())
    )
}

/// Overview pane for one measure entity (`edetails.php?eid=&mod=ballot`).
pub fn ftm_measure_overview_url(eid: &str) -> String {
    format!(
        "{FTM_SITE}/details/modules/edetails.php?eid={}&mod=ballot",
        urlenc(eid.trim())
    )
}

/// Supporting (`m-t-rt=3`) or opposing (`m-t-rt=4`) committees for a measure eid.
pub fn ftm_measure_committees_url(eid: &str, support: bool) -> String {
    let rt = if support { 3 } else { 4 };
    format!(
        "{FTM_SITE}/aaengine/aafetch.php?m-eid={}&m-t-rt={rt}&gro=m-t-eid",
        urlenc(eid.trim())
    )
}

/// Top donors supporting (`true`) or opposing (`false`) a measure.
pub fn ftm_measure_donors_url(eid: &str, support: bool) -> String {
    let rt = if support { 3 } else { 4 };
    format!(
        "{FTM_SITE}/aaengine/aafetch.php?m-eid={}&m-t-rt={rt}&gro=d-eid",
        urlenc(eid.trim())
    )
}

/// Public citation URL for a measure entity.
pub fn ftm_measure_show_me_url(eid: &str) -> String {
    format!("{FTM_SITE}/show-me?m-eid={}&m-exi=1", urlenc(eid.trim()))
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FtmMeasureHit {
    pub m_id: String,
    pub eid: String,
    pub name: String,
    pub status: String,
    pub description: String,
    pub state: String,
    pub year: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FtmMeasureCommittee {
    pub eid: String,
    pub name: String,
    pub total_amount: f64,
    pub total_display: String,
    pub record_count: Option<u32>,
    pub profile_url: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FtmMeasureOverview {
    pub title: String,
    pub support_total: f64,
    pub support_display: String,
    pub oppose_total: f64,
    pub oppose_display: String,
    pub status: String,
    pub note: String,
}

/// Measure finance block attached to ballot `m.finance` (same shape as FL TreFin).
#[derive(Debug, Clone, serde::Serialize)]
pub struct FtmMeasureFinance {
    pub source: String,
    pub account: String,
    pub contributions_sum: f64,
    pub contributions_sum_display: String,
    pub top_contributors: Vec<ContributorRow>,
    pub line_count: usize,
    pub committee_url: String,
    pub trefin_url: String,
    pub note: String,
    pub committee_name: String,
    pub role: String,
    pub oppose: Vec<FtmMeasureFinanceSide>,
    pub source_label: String,
    pub profile_url: String,
    pub show_me_url: String,
    pub support_total: f64,
    pub oppose_total: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FtmMeasureFinanceSide {
    pub account: String,
    pub contributions_sum: f64,
    pub contributions_sum_display: String,
    pub top_contributors: Vec<ContributorRow>,
    pub line_count: usize,
    pub committee_url: String,
    pub trefin_url: String,
    pub note: String,
    pub committee_name: String,
    pub role: String,
}

fn strip_tags(html: &str) -> String {
    let re = regex::Regex::new(r"(?is)<[^>]+>").ok();
    let s = if let Some(re) = re {
        re.replace_all(html, " ").to_string()
    } else {
        html.to_string()
    };
    html_unescape_basic(&s)
}

fn html_unescape_basic(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

fn parse_money_cell(raw: &str) -> Option<f64> {
    let t = raw.trim().replace(['$', ','], "");
    if t.is_empty() {
        return None;
    }
    t.parse::<f64>().ok()
}

fn normalize_measure_num(raw: &str) -> String {
    let u = raw.to_ascii_uppercase();
    // Keep trailing letter (MD-style) but strip leading zeros from digit run.
    let re = regex::Regex::new(r"^0*(\d+)([A-Z]?)$").ok();
    if let Some(c) = re.and_then(|r| r.captures(&u)) {
        let n = c.get(1).map(|m| m.as_str()).unwrap_or("0");
        let n = if n.is_empty() { "0" } else { n };
        let suf = c.get(2).map(|m| m.as_str()).unwrap_or("");
        return format!("{n}{suf}");
    }
    u
}

/// Extract a ballot-measure code key for matching (e.g. `PROP:139`, `AMD:4`, `Q:1`, `HB:1074`).
pub fn measure_code_key(raw: &str) -> Option<String> {
    let s = raw.trim();
    if s.is_empty() {
        return None;
    }
    let upper = s.to_ascii_uppercase();
    // Proposition / Prop N
    if let Some(c) = regex::Regex::new(r"(?i)\bPROP(?:OSITION)?\.?\s*([A-Z]?\d+[A-Z]?)\b")
        .ok()
        .and_then(|re| re.captures(s))
    {
        let n = normalize_measure_num(&c[1]);
        return Some(format!("PROP:{n}"));
    }
    // Amendment N
    if let Some(c) = regex::Regex::new(r"(?i)\bAMENDMENT\s*([A-Z]?\d+[A-Z]?)\b")
        .ok()
        .and_then(|re| re.captures(s))
    {
        let n = normalize_measure_num(&c[1]);
        return Some(format!("AMD:{n}"));
    }
    // Question N / Q N / Question A (letter-only local codes)
    if let Some(c) = regex::Regex::new(r"(?i)\b(?:QUESTION|Q)\.?\s*([A-Z]?\d+[A-Z]?|[A-Z])\b")
        .ok()
        .and_then(|re| re.captures(s))
    {
        let n = normalize_measure_num(&c[1]);
        return Some(format!("Q:{n}"));
    }
    // HB / SB / HJR / etc.
    if let Some(c) = regex::Regex::new(r"(?i)\b((?:H|S)(?:B|JR|CR|R))\s*(\d+)\b")
        .ok()
        .and_then(|re| re.captures(s))
    {
        return Some(format!(
            "{}:{}",
            c[1].to_ascii_uppercase(),
            c[2].trim_start_matches('0')
        ));
    }
    // Bare leading number only when whole string is short code-like
    if let Some(c) = regex::Regex::new(r"(?i)^\s*(?:NO\.?\s*)?(\d{1,4}[A-Z]?)\s*$")
        .ok()
        .and_then(|re| re.captures(&upper))
    {
        let n = c[1].trim_start_matches('0');
        let n = if n.is_empty() { "0" } else { n };
        return Some(format!("N:{n}"));
    }
    None
}

fn measure_keys_for_ballot(code: &str, title: &str) -> Vec<String> {
    let mut keys = Vec::new();
    for s in [code, title] {
        if let Some(k) = measure_code_key(s) {
            if !keys.contains(&k) {
                keys.push(k);
            }
        }
    }
    // Combined "code: title"
    let combo = format!("{code} {title}");
    if let Some(k) = measure_code_key(&combo) {
        if !keys.contains(&k) {
            keys.push(k);
        }
    }
    keys
}

fn normalize_title_words(s: &str) -> String {
    let lower = s.to_ascii_lowercase();
    let re = regex::Regex::new(r"[^a-z0-9]+").unwrap_or_else(|_| regex::Regex::new(".").unwrap());
    re.split(&lower)
        .filter(|w| w.len() > 2)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Parse FTM measure list HTML (`aafetch` table with m-id / m-eid-group).
pub fn parse_ftm_measures_list_html(html: &str) -> Vec<FtmMeasureHit> {
    let mut out = Vec::new();
    // No backrefs (regex crate): capture eid once, then entity-details link + name.
    let re = regex::Regex::new(
        r#"(?is)m-id=(\d+)".*?token="s"\s+tokenvalue="([A-Z]{2})".*?token="y"\s+tokenvalue="(\d{4})".*?token="m-eid-group"\s+tokenvalue="(\d+)".*?<a\s+href="/entity-details\?eid=\d+"[^>]*>([^<]+)</a>.*?token="m-sts"[^>]*>([^<]+)</td>.*?<small>([^<]*)</small>"#,
    )
    .ok();
    let Some(re) = re else {
        return out;
    };
    for cap in re.captures_iter(html) {
        out.push(FtmMeasureHit {
            m_id: cap[1].to_string(),
            state: cap[2].to_string(),
            year: cap[3].to_string(),
            eid: cap[4].to_string(),
            name: html_unescape_basic(cap[5].trim()),
            status: cap[6].trim().to_string(),
            description: html_unescape_basic(cap[7].trim()),
        });
    }
    out
}

/// Parse overview totals from `edetails.php?mod=ballot` HTML.
pub fn parse_ftm_measure_overview_html(html: &str) -> Option<FtmMeasureOverview> {
    let text = strip_tags(html);
    let title_re = regex::Regex::new(
        r"(?is)RecordsLink[^>]*>([^<]+)</a>\s*was\s*on the ballot",
    )
    .ok()
    .or_else(|| regex::Regex::new(r"(?is)>([^<]{3,120})</a>\s*was\s*on the ballot").ok());
    let title = title_re
        .as_ref()
        .and_then(|re| re.captures(html))
        .map(|c| html_unescape_basic(c[1].trim()))
        .unwrap_or_default();

    // "$X was raised in support of the measure, while $Y was raised in opposition."
    let money_re = regex::Regex::new(
        r"(?is)\$([0-9,]+(?:\.\d+)?)\s+was raised in support of the measure,\s*while\s*\$([0-9,]+(?:\.\d+)?)\s+was raised in opposition",
    )
    .ok()?;
    let (support_total, oppose_total) = if let Some(c) = money_re.captures(&text) {
        (
            parse_money_cell(&c[1]).unwrap_or(0.0),
            parse_money_cell(&c[2]).unwrap_or(0.0),
        )
    } else {
        // spans in HTML
        let span_re = regex::Regex::new(
            r"(?is)<span>\$([0-9,]+(?:\.\d+)?)</span>\s*was raised in support.*?<span>\$([0-9,]+(?:\.\d+)?)</span>\s*was raised in opposition",
        )
        .ok()?;
        let c = span_re.captures(html)?;
        (
            parse_money_cell(&c[1]).unwrap_or(0.0),
            parse_money_cell(&c[2]).unwrap_or(0.0),
        )
    };

    let status_re = regex::Regex::new(r"(?is)The measure\s*<span>([^<]+)</span>").ok();
    let status = status_re
        .as_ref()
        .and_then(|re| re.captures(html))
        .map(|c| c[1].trim().to_string())
        .unwrap_or_default();

    Some(FtmMeasureOverview {
        title,
        support_total,
        support_display: format_usd(support_total),
        oppose_total,
        oppose_display: format_usd(oppose_total),
        status,
        note: format!(
            "FollowTheMoney ballot-measure committee totals (data through {FTM_MAX_YEAR}). Committees may work multiple measures; amounts are not certified cash-on-hand."
        ),
    })
}

/// Parse committee or donor aafetch table (entity link + amount columns).
pub fn parse_ftm_measure_entity_table_html(html: &str, limit: usize) -> Vec<FtmMeasureCommittee> {
    let mut out = Vec::new();
    if html.contains("No results found") || html.contains("table-no-data") {
        return out;
    }
    // No backrefs: tokenvalue + following entity-details href eid + name + $amount
    let re = regex::Regex::new(
        r#"(?is)token="(?:m-t-eid|d-eid)"\s+tokenvalue="(\d+)"[^>]*>.*?<a\s+href="/entity-details\?eid=(\d+)[^"]*"[^>]*>([^<]+)</a>.*?\$([0-9,\.]+)"#,
    )
    .ok();
    let Some(re) = re else {
        return out;
    };
    for cap in re.captures_iter(html) {
        if limit > 0 && out.len() >= limit {
            break;
        }
        let token_eid = &cap[1];
        let href_eid = &cap[2];
        // Prefer href eid when both present; they should match.
        let eid = if href_eid == token_eid {
            token_eid.to_string()
        } else {
            href_eid.to_string()
        };
        let name = html_unescape_basic(cap[3].trim());
        let total = parse_money_cell(&cap[4]).unwrap_or(0.0);
        out.push(FtmMeasureCommittee {
            eid: eid.clone(),
            name,
            total_amount: total,
            total_display: format_usd(total),
            record_count: None,
            profile_url: ftm_profile_url(&eid),
        });
    }
    out
}

fn committees_to_contributors(rows: &[FtmMeasureCommittee], limit: usize) -> Vec<ContributorRow> {
    rows.iter()
        .take(if limit == 0 { rows.len() } else { limit })
        .map(|r| ContributorRow {
            name: r.name.clone(),
            amount_display: r.total_display.clone(),
            location: None,
            date: None,
            url: r.profile_url.clone(),
            gift_count: r.record_count,
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FtmMeasureMatch {
    Unique { hit: FtmMeasureHit },
    None,
    Ambiguous { count: usize },
}

/// Match a ballot measure (code + title) to FTM list hits.
pub fn match_ftm_measure(
    hits: &[FtmMeasureHit],
    measure_code: &str,
    title: &str,
) -> FtmMeasureMatch {
    if hits.is_empty() {
        return FtmMeasureMatch::None;
    }
    let keys = measure_keys_for_ballot(measure_code, title);
    if !keys.is_empty() {
        let mut matched: Vec<&FtmMeasureHit> = hits
            .iter()
            .filter(|h| {
                let hk = measure_keys_for_ballot(&h.name, &h.description);
                keys.iter().any(|k| hk.iter().any(|x| x == k))
            })
            .collect();
        // Prefer exact name key match over description-only
        if matched.len() > 1 {
            let tight: Vec<&FtmMeasureHit> = matched
                .iter()
                .copied()
                .filter(|h| {
                    let hk = measure_keys_for_ballot(&h.name, "");
                    keys.iter().any(|k| hk.iter().any(|x| x == k))
                })
                .collect();
            if !tight.is_empty() {
                matched = tight;
            }
        }
        match matched.len() {
            0 => {}
            1 => {
                return FtmMeasureMatch::Unique {
                    hit: matched[0].clone(),
                };
            }
            n => return FtmMeasureMatch::Ambiguous { count: n },
        }
    }

    // Exact description / title containment (NC statewide codes are short titles)
    let code_n = normalize_title_words(measure_code);
    let title_n = normalize_title_words(title);
    if !code_n.is_empty() || !title_n.is_empty() {
        let mut exact: Vec<&FtmMeasureHit> = hits
            .iter()
            .filter(|h| {
                let desc = normalize_title_words(&h.description);
                let name = normalize_title_words(&h.name);
                (!code_n.is_empty()
                    && (desc == code_n || name == code_n || desc.contains(&code_n) || code_n.contains(&desc)))
                    || (!title_n.is_empty()
                        && (desc == title_n
                            || title_n.contains(&desc) && desc.len() >= 12
                            || desc.contains(&title_n) && title_n.len() >= 12))
            })
            .collect();
        exact.sort_by_key(|h| h.eid.as_str());
        exact.dedup_by_key(|h| h.eid.as_str());
        match exact.len() {
            0 => {}
            1 => {
                return FtmMeasureMatch::Unique {
                    hit: exact[0].clone(),
                };
            }
            n => return FtmMeasureMatch::Ambiguous { count: n },
        }
    }

    // Title word overlap fallback (unique only)
    let want = if title_n.len() >= 8 {
        title_n.clone()
    } else if code_n.len() >= 8 {
        code_n.clone()
    } else {
        String::new()
    };
    if want.is_empty() {
        return FtmMeasureMatch::None;
    }
    let want_set: std::collections::HashSet<&str> = want.split_whitespace().collect();
    let mut best: Vec<(&FtmMeasureHit, usize)> = hits
        .iter()
        .filter_map(|h| {
            let hay = normalize_title_words(&format!("{} {}", h.name, h.description));
            let set: std::collections::HashSet<&str> = hay.split_whitespace().collect();
            let score = want_set.intersection(&set).count();
            if score >= 3 {
                Some((h, score))
            } else {
                None
            }
        })
        .collect();
    best.sort_by(|a, b| b.1.cmp(&a.1));
    if best.is_empty() {
        return FtmMeasureMatch::None;
    }
    if best.len() == 1 || best[0].1 > best.get(1).map(|x| x.1).unwrap_or(0) {
        return FtmMeasureMatch::Unique {
            hit: best[0].0.clone(),
        };
    }
    FtmMeasureMatch::Ambiguous { count: best.len() }
}

/// Build measure finance from overview + optional support/oppose tables.
pub fn ftm_measure_finance_from_parts(
    hit: &FtmMeasureHit,
    overview: &FtmMeasureOverview,
    support_donors: &[FtmMeasureCommittee],
    oppose_committees: &[FtmMeasureCommittee],
    top_limit: usize,
) -> FtmMeasureFinance {
    let top = committees_to_contributors(support_donors, top_limit);
    let line_count = if overview.support_total > 0.0 {
        top.len().max(1)
    } else {
        top.len()
    };
    let sponsor_name = support_donors
        .first()
        .map(|c| c.name.clone())
        .unwrap_or_else(|| hit.name.clone());
    let oppose: Vec<FtmMeasureFinanceSide> = if overview.oppose_total > 0.0
        || !oppose_committees.is_empty()
    {
        if oppose_committees.is_empty() {
            vec![FtmMeasureFinanceSide {
                account: hit.eid.clone(),
                contributions_sum: overview.oppose_total,
                contributions_sum_display: overview.oppose_display.clone(),
                top_contributors: vec![],
                line_count: if overview.oppose_total > 0.0 { 1 } else { 0 },
                committee_url: ftm_measure_show_me_url(&hit.eid),
                trefin_url: String::new(),
                note: String::new(),
                committee_name: "Oppose (FTM total)".into(),
                role: "oppose".into(),
            }]
        } else {
            oppose_committees
                .iter()
                .take(3)
                .map(|c| FtmMeasureFinanceSide {
                    account: c.eid.clone(),
                    contributions_sum: c.total_amount,
                    contributions_sum_display: c.total_display.clone(),
                    top_contributors: vec![],
                    line_count: if c.total_amount > 0.0 { 1 } else { 0 },
                    committee_url: c.profile_url.clone(),
                    trefin_url: String::new(),
                    note: String::new(),
                    committee_name: c.name.clone(),
                    role: "oppose".into(),
                })
                .collect()
        }
    } else {
        vec![]
    };

    FtmMeasureFinance {
        source: "ftm_measure".into(),
        account: hit.eid.clone(),
        contributions_sum: overview.support_total,
        contributions_sum_display: overview.support_display.clone(),
        top_contributors: top,
        line_count,
        committee_url: ftm_measure_show_me_url(&hit.eid),
        trefin_url: String::new(),
        note: overview.note.clone(),
        committee_name: sponsor_name,
        role: "sponsor".into(),
        oppose,
        source_label: "FollowTheMoney / OpenSecrets (ballot measure committees)".into(),
        profile_url: ftm_profile_url(&hit.eid),
        show_me_url: ftm_measure_show_me_url(&hit.eid),
        support_total: overview.support_total,
        oppose_total: overview.oppose_total,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_CANDIDATES: &str = r##"{
      "metaInfo": {"format": "json"},
      "records": [
        {
          "record_id": 1,
          "Candidate": {"token": "c-t-id", "id": "1001", "Candidate": "SMITH, DAVID A"},
          "Political_Party": {"token": "c-t-p", "id": "2", "Political_Party": "Republican"},
          "Election_State": {"token": "s", "id": "AZ", "Election_State": "Arizona"},
          "Election_Year": {"token": "y", "id": "2022", "Election_Year": "2022"},
          "Office_Sought": {"token": "c-r-osid", "id": "9", "Office_Sought": "HOUSE DISTRICT 008"},
          "Office": {"token": "c-r-oc", "id": "H", "Office": "STATE HOUSE"},
          "Total_$": {"Total_$": "125000.50"},
          "#_of_Records": {"#_of_Records": "42"}
        },
        {
          "record_id": 2,
          "Candidate": {"token": "c-t-id", "id": "1002", "Candidate": "SMITH, DAVID"},
          "Political_Party": {"token": "c-t-p", "id": "1", "Political_Party": "Democratic"},
          "Election_State": {"token": "s", "id": "AZ", "Election_State": "Arizona"},
          "Election_Year": {"token": "y", "id": "2022", "Election_Year": "2022"},
          "Office_Sought": {"token": "c-r-osid", "id": "10", "Office_Sought": "HOUSE DISTRICT 012"},
          "Office": {"token": "c-r-oc", "id": "H", "Office": "STATE HOUSE"},
          "Total_$": {"Total_$": "80000.00"}
        },
        {
          "record_id": 3,
          "Candidate": {"token": "c-t-id", "id": "2001", "Candidate": "JONES, ALICE"},
          "Political_Party": {"token": "c-t-p", "id": "1", "Political_Party": "Democratic"},
          "Election_State": {"token": "s", "id": "AZ", "Election_State": "Arizona"},
          "Office_Sought": {"token": "c-r-osid", "id": "20", "Office_Sought": "SENATE DISTRICT 008"},
          "Office": {"token": "c-r-oc", "id": "S", "Office": "STATE SENATE"},
          "Total_$": {"Total_$": "50000.00"}
        }
      ]
    }"##;

    const SAMPLE_DONORS: &str = r##"{
      "records": [
        {
          "Contributor": {"token": "d-eid", "id": "9", "Contributor": "ACME PAC"},
          "Type_of_Contributor": {"token": "d-et", "id": "3", "Type_of_Contributor": "Non-Individual"},
          "Total_$": {"Total_$": "10000.00"},
          "#_of_Records": {"#_of_Records": "2"}
        },
        {
          "Contributor": {"token": "d-eid", "id": "8", "Contributor": "DOE, JANE"},
          "Type_of_Contributor": {"token": "d-et", "id": "2", "Type_of_Contributor": "Individual"},
          "Total_$": {"Total_$": "5000.00"},
          "#_of_Records": {"#_of_Records": "1"}
        }
      ]
    }"##;

    #[test]
    fn parse_candidates_and_unique_match() {
        let hits = parse_ftm_candidate_records(SAMPLE_CANDIDATES).expect("parse");
        assert_eq!(hits.len(), 3);
        assert_eq!(hits[0].name, "DAVID A SMITH");
        assert!((hits[0].total_amount - 125000.50).abs() < 0.01);

        let q = FtmMatchQuery {
            name: "David A. Smith".into(),
            office: "Arizona House (District 8)".into(),
            chamber: "state_house".into(),
            party: "Republican".into(),
            district: Some(8),
            state: "AZ".into(),
        };
        match match_ftm_candidate(&hits, &q) {
            FtmMatch::Unique { hit } => assert_eq!(hit.id, "1001"),
            other => panic!("expected unique, got {other:?}"),
        }
    }

    #[test]
    fn match_skips_ambiguous_without_district() {
        let hits = parse_ftm_candidate_records(SAMPLE_CANDIDATES).unwrap();
        let q = FtmMatchQuery {
            name: "David Smith".into(),
            office: "State House".into(),
            chamber: "state_house".into(),
            party: String::new(),
            district: None,
            state: "AZ".into(),
        };
        match match_ftm_candidate(&hits, &q) {
            FtmMatch::Ambiguous { count } => assert_eq!(count, 2),
            other => panic!("expected ambiguous, got {other:?}"),
        }
    }

    #[test]
    fn match_rejects_wrong_chamber() {
        let hits = parse_ftm_candidate_records(SAMPLE_CANDIDATES).unwrap();
        let q = FtmMatchQuery {
            name: "Alice Jones".into(),
            office: "Arizona House (District 8)".into(),
            chamber: "state_house".into(),
            party: "Democratic".into(),
            district: Some(8),
            state: "AZ".into(),
        };
        assert!(matches!(match_ftm_candidate(&hits, &q), FtmMatch::None));
    }

    #[test]
    fn parse_donors() {
        let rows = parse_ftm_donor_records(SAMPLE_DONORS, 5, "https://example").unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].name, "ACME PAC");
        assert_eq!(rows[0].gift_count, Some(2));
    }

    #[test]
    fn error_message_and_urls() {
        assert_eq!(
            ftm_error_message(r#"{"error":"Invalid API Key"}"#).as_deref(),
            Some("Invalid API Key")
        );
        assert_eq!(ftm_data_year(2026), 2024);
        assert_eq!(ftm_data_year(2022), 2022);
        let u = ftm_candidates_url("secret", "az", 2022, Some("H"));
        assert!(u.contains("APIKey=secret"));
        assert!(u.contains("s=AZ"));
        assert!(u.contains("c-r-ot=H"));
        assert!(u.contains("gro=c-t-id"));
        assert!(ftm_redact_url(&u).contains("APIKey=REDACTED"));
        assert!(!ftm_redact_url(&u).contains("secret"));
        assert_eq!(ftm_office_type_code("state_senate", ""), Some("S"));
        assert_eq!(ftm_office_type_code("state_house", ""), Some("H"));
        assert_eq!(
            ftm_office_type_code("statewide", "Governor"),
            Some("G")
        );
    }

    #[test]
    fn name_convert() {
        assert_eq!(ftm_name_to_ballot("INSLEE, JAY ROBERT"), "JAY ROBERT INSLEE");
        assert!(last_names_match(
            &ftm_name_to_ballot("SMITH, DAVID"),
            "David Smith"
        ));
        assert!(last_names_match("David A. Smith", "DAVID A SMITH"));
    }

    #[test]
    fn finance_block() {
        let hits = parse_ftm_candidate_records(SAMPLE_CANDIDATES).unwrap();
        let fin = ftm_finance_from_hit(&hits[0], "AZ", 2022, Some("H"), vec![]);
        assert_eq!(fin.source, "ftm");
        assert!(fin.profile_url.contains("1001"));
        assert!(fin.note.contains("FollowTheMoney"));
    }

    #[test]
    fn measure_code_keys() {
        assert_eq!(measure_code_key("PROPOSITION 139").as_deref(), Some("PROP:139"));
        assert_eq!(measure_code_key("Prop. 139").as_deref(), Some("PROP:139"));
        assert_eq!(measure_code_key("AMENDMENT 004").as_deref(), Some("AMD:4"));
        assert_eq!(measure_code_key("Amendment 4").as_deref(), Some("AMD:4"));
        assert_eq!(measure_code_key("QUESTION 001").as_deref(), Some("Q:1"));
        assert_eq!(measure_code_key("Question 1").as_deref(), Some("Q:1"));
        assert_eq!(measure_code_key("Question A").as_deref(), Some("Q:A"));
        assert_eq!(measure_code_key("HB 1074").as_deref(), Some("HB:1074"));
    }

    #[test]
    fn match_by_description_title() {
        let hits = vec![FtmMeasureHit {
            m_id: "1".into(),
            eid: "60299708".into(),
            name: "HB 1074".into(),
            status: "PASSED".into(),
            description: "Citizenship Requirement for Voting".into(),
            state: "NC".into(),
            year: "2024".into(),
        }];
        match match_ftm_measure(
            &hits,
            "Citizenship Requirement for Voting",
            "Constitutional amendment - Citizenship Requirement for Voting",
        ) {
            FtmMeasureMatch::Unique { hit } => assert_eq!(hit.eid, "60299708"),
            other => panic!("expected unique, got {other:?}"),
        }
    }

    #[test]
    fn parse_az_measures_list_fixture() {
        let html = include_str!("../../../testdata/ftm_measures_az_2024.html");
        let hits = parse_ftm_measures_list_html(html);
        assert!(hits.len() >= 10, "got {}", hits.len());
        let p139 = hits.iter().find(|h| h.name.contains("139")).expect("prop 139");
        assert_eq!(p139.eid, "60371730");
        assert_eq!(p139.state, "AZ");
    }

    #[test]
    fn parse_overview_and_sides_fixture() {
        let ov = parse_ftm_measure_overview_html(include_str!(
            "../../../testdata/ftm_measure_overview_az_prop139.html"
        ))
        .expect("overview");
        assert!((ov.support_total - 36_786_150.0).abs() < 1.0);
        assert!((ov.oppose_total - 1_396_537.0).abs() < 1.0);
        assert!(ov.title.to_ascii_uppercase().contains("139"));

        let support = parse_ftm_measure_entity_table_html(
            include_str!("../../../testdata/ftm_measure_support_cmtes_az_prop139.html"),
            5,
        );
        assert_eq!(support.len(), 2);
        assert!(support[0].name.contains("ABORTION"));

        let oppose = parse_ftm_measure_entity_table_html(
            include_str!("../../../testdata/ftm_measure_oppose_cmtes_az_prop139.html"),
            5,
        );
        assert_eq!(oppose.len(), 1);

        let donors = parse_ftm_measure_entity_table_html(
            include_str!("../../../testdata/ftm_measure_support_donors_az_prop139.html"),
            5,
        );
        assert_eq!(donors.len(), 5);
        assert!(donors[0].total_amount > 1_000_000.0);
    }

    #[test]
    fn match_prop_and_build_finance() {
        let hits = parse_ftm_measures_list_html(include_str!(
            "../../../testdata/ftm_measures_az_2024.html"
        ));
        match match_ftm_measure(&hits, "Prop 139", "Fundamental Right to Abortion") {
            FtmMeasureMatch::Unique { hit } => {
                assert_eq!(hit.eid, "60371730");
                let ov = parse_ftm_measure_overview_html(include_str!(
                    "../../../testdata/ftm_measure_overview_az_prop139.html"
                ))
                .unwrap();
                let donors = parse_ftm_measure_entity_table_html(
                    include_str!("../../../testdata/ftm_measure_support_donors_az_prop139.html"),
                    3,
                );
                let oppose = parse_ftm_measure_entity_table_html(
                    include_str!("../../../testdata/ftm_measure_oppose_cmtes_az_prop139.html"),
                    3,
                );
                let fin = ftm_measure_finance_from_parts(&hit, &ov, &donors, &oppose, 3);
                assert_eq!(fin.source, "ftm_measure");
                assert!(fin.contributions_sum > 30_000_000.0);
                assert_eq!(fin.top_contributors.len(), 3);
                assert!(!fin.oppose.is_empty());
                assert!(fin.show_me_url.contains("60371730"));
            }
            other => panic!("expected unique, got {other:?}"),
        }
    }
}
