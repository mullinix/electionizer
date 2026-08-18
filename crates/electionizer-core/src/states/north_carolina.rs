//! North Carolina NCSBE candidate listing CSV + referendum list + CF portal parsers (pure — no HTTP).

use crate::models::{
    format_usd, normalize_party_label, GeoResolution, SnapshotCandidate, SnapshotMeasure,
};
use crate::states::florida::{first_names_compatible, last_names_match, split_candidate_first_last};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeSet, HashSet};
use std::sync::OnceLock;

pub const CANDIDATE_LIST_BASE: &str =
    "https://s3.amazonaws.com/dl.ncsbe.gov/Elections";

/// Public CSV URL for a general-election cycle year (e.g. 2026).
pub fn candidate_listing_csv_url(cycle: i32) -> String {
    format!(
        "{CANDIDATE_LIST_BASE}/{cycle}/Candidate%20Filing/Candidate_Listing_{cycle}.csv"
    )
}

/// Candidate lists page hosting referendum PDF links (JS discovers cycle URLs).
pub const CANDIDATE_LISTS_PAGE_URL: &str =
    "https://www.ncsbe.gov/results-data/candidate-lists";

pub const SOURCE_PUBLISHER: &str = "N.C. State Board of Elections";

/// Extract plain text from an NCSBE referendum PDF (pure).
pub fn extract_pdf_text(bytes: &[u8]) -> Result<String, String> {
    pdf_extract::extract_text_from_mem(bytes).map_err(|e| e.to_string())
}

/// One NC referendum from NCSBE referendum-list text (PDF extract).
#[derive(Debug, Clone)]
pub struct ParsedNcMeasure {
    pub title: String,
    pub summary: Option<String>,
    /// County heading (`WAKE`, `STATEWIDE` for constitutional amendments).
    pub county: String,
    pub is_statewide: bool,
}

fn normalize_county_token(s: &str) -> String {
    s.trim()
        .to_ascii_lowercase()
        .replace(" county", "")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

fn is_chrome_line(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() {
        return true;
    }
    let u = t.to_ascii_uppercase();
    u.contains("BOARD OF ELECTIONS")
        || u.contains("REFERENDUM CHOICES LIST")
        || u.contains("CRITERIA:")
        || u.starts_with("CHOICE")
        || u.contains("CONT_CAND")
        || u.contains("PAGE ")
        || u.contains(" DATA SOURCE")
        || regex::Regex::new(r"(?i)^\s*(jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec)\w*\s+\d")
            .ok()
            .map(|r| r.is_match(t))
            .unwrap_or(false)
}

fn is_choice_label(s: &str) -> bool {
    matches!(
        s.trim().to_ascii_lowercase().as_str(),
        "for" | "against" | "yes" | "no"
    )
}

fn looks_like_county_header(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() || t.len() > 28 {
        return false;
    }
    if !t.chars().all(|c| c.is_ascii_uppercase() || c == ' ' || c == '-' || c == '\'') {
        return false;
    }
    if is_chrome_line(t) || is_choice_label(t) {
        return false;
    }
    // County names are short tokens; measure titles are long.
    let words = t.split_whitespace().count();
    words >= 1 && words <= 3 && !t.contains("AMENDMENT") && !t.contains("REFERENDUM")
        && !t.contains("BONDS") && !t.contains("ELECTION") && !t.contains("TAX")
}

fn looks_like_measure_title(s: &str) -> bool {
    let t = s.trim();
    if t.len() < 12 || is_chrome_line(t) || is_choice_label(t) || looks_like_county_header(t) {
        return false;
    }
    // Titles are printed in all caps (may include digits / punctuation).
    let letters: String = t.chars().filter(|c| c.is_ascii_alphabetic()).collect();
    if letters.is_empty() {
        return false;
    }
    let upper = letters.chars().filter(|c| c.is_ascii_uppercase()).count();
    upper * 100 / letters.len() >= 85
        && (t.contains("AMENDMENT")
            || t.contains("REFERENDUM")
            || t.contains("BONDS")
            || t.contains("ELECTION")
            || t.contains("TAX")
            || t.contains("QUESTION")
            || t.len() > 24)
}

/// Parse NCSBE referendum-list plain text (from PDF extract).
/// When `county_filter` is set, keep **statewide** constitutional amendments + that county’s locals.
pub fn parse_nc_referendum_list_text(
    text: &str,
    county_filter: Option<&str>,
) -> Vec<ParsedNcMeasure> {
    let want = county_filter
        .map(normalize_county_token)
        .filter(|s| !s.is_empty());
    let mut out = Vec::new();
    let mut seen_statewide = BTreeSet::new();
    let mut current_county = String::new();
    let mut pending_title: Option<String> = None;
    let mut pending_summary = String::new();
    let mut in_first_choice = false;
    let mut saw_first_choice = false;

    let flush = |title: &Option<String>,
                 summary: &str,
                 county: &str,
                 seen_statewide: &mut BTreeSet<String>,
                 want: &Option<String>,
                 out: &mut Vec<ParsedNcMeasure>| {
        let Some(title) = title.clone().filter(|t| !t.is_empty()) else {
            return;
        };
        let is_statewide = title.to_ascii_uppercase().starts_with("CONSTITUTIONAL AMENDMENT");
        let county_key = normalize_county_token(county);
        if let Some(ref w) = want {
            if is_statewide {
                // keep
            } else if county_key != *w {
                return;
            }
        }
        if is_statewide {
            let k = title.to_ascii_uppercase();
            if !seen_statewide.insert(k) {
                return;
            }
        }
        let mut summary = summary.split_whitespace().collect::<Vec<_>>().join(" ");
        if summary.len() > 600 {
            summary = format!("{}…", summary.chars().take(597).collect::<String>());
        }
        out.push(ParsedNcMeasure {
            title: title
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" "),
            summary: if summary.is_empty() {
                None
            } else {
                Some(summary)
            },
            county: if is_statewide {
                "Statewide".into()
            } else if county.is_empty() {
                "Local".into()
            } else {
                // Title-case county token for display
                county
                    .split_whitespace()
                    .map(|w| {
                        let mut c = w.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => {
                                f.to_uppercase().collect::<String>() + &c.as_str().to_ascii_lowercase()
                            }
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            },
            is_statewide,
        });
    };

    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if is_chrome_line(line) {
            continue;
        }

        // Choice row: "For   description..." or bare "For"
        let mut parts = line.split_whitespace();
        let first = parts.next().unwrap_or("");
        if is_choice_label(first) {
            let rest = parts.collect::<Vec<_>>().join(" ");
            if !saw_first_choice {
                saw_first_choice = true;
                in_first_choice = true;
                if !rest.is_empty() {
                    pending_summary.push_str(&rest);
                    pending_summary.push(' ');
                }
            } else {
                // second choice (Against/No) — stop collecting summary
                in_first_choice = false;
            }
            continue;
        }

        if looks_like_county_header(line) {
            flush(
                &pending_title,
                &pending_summary,
                &current_county,
                &mut seen_statewide,
                &want,
                &mut out,
            );
            pending_title = None;
            pending_summary.clear();
            in_first_choice = false;
            saw_first_choice = false;
            current_county = line.to_ascii_uppercase();
            continue;
        }

        if looks_like_measure_title(line) {
            flush(
                &pending_title,
                &pending_summary,
                &current_county,
                &mut seen_statewide,
                &want,
                &mut out,
            );
            pending_title = Some(line.to_string());
            pending_summary.clear();
            in_first_choice = false;
            saw_first_choice = false;
            continue;
        }

        // Continuation of summary description
        if in_first_choice && pending_title.is_some() {
            pending_summary.push_str(line);
            pending_summary.push(' ');
        }
    }
    flush(
        &pending_title,
        &pending_summary,
        &current_county,
        &mut seen_statewide,
        &want,
        &mut out,
    );
    out
}

/// Map parsed NC measures into snapshot rows.
pub fn map_nc_measures_for_geo(
    parsed: &[ParsedNcMeasure],
    state_ocd: &str,
    county_name: &str,
    source_url: &str,
) -> (Vec<SnapshotMeasure>, Vec<crate::models::ResolvedJurisdiction>) {
    let mut measures = Vec::new();
    let mut extras = Vec::new();
    let county_slug = normalize_county_token(county_name);
    let county_ocd = if county_slug.is_empty() {
        None
    } else {
        let ocd = format!("{state_ocd}/county:{county_slug}");
        let label = {
            let raw = county_name.trim();
            if raw.is_empty() {
                format!(
                    "{} County",
                    {
                        let mut c = county_slug.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                        }
                    }
                )
            } else if raw.to_ascii_lowercase().contains("county") {
                raw.to_string()
            } else {
                format!("{raw} County")
            }
        };
        extras.push(crate::models::ResolvedJurisdiction {
            ocd_id: ocd.clone(),
            name: label,
            level: "county".into(),
            state: Some("NC".into()),
        });
        Some(ocd)
    };

    for m in parsed {
        let jurisdiction_ocd = if m.is_statewide {
            state_ocd.to_string()
        } else {
            county_ocd
                .clone()
                .unwrap_or_else(|| state_ocd.to_string())
        };
        let measure_code = if m.is_statewide {
            m.title
                .split_once(" - ")
                .or_else(|| m.title.split_once(" – "))
                .map(|(_, rest)| rest.trim().to_string())
                .filter(|s| !s.is_empty())
        } else {
            None
        };
        measures.push(SnapshotMeasure {
            title: m.title.clone(),
            measure_code,
            jurisdiction_ocd,
            summary: m.summary.clone(),
            source_url: source_url.to_string(),
            source_publisher: Some(SOURCE_PUBLISHER.into()),
        });
    }
    (measures, extras)
}

#[derive(Debug, Clone)]
pub struct NcFiling {
    pub election_dt: String,
    pub county_name: String,
    pub contest_name: String,
    pub name_on_ballot: String,
    pub party_candidate: String,
}

/// Minimal RFC4180-ish CSV line split (quoted fields, commas).
pub fn split_csv_line(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_q = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                if in_q {
                    if chars.peek() == Some(&'"') {
                        chars.next();
                        cur.push('"');
                    } else {
                        in_q = false;
                    }
                } else {
                    in_q = true;
                }
            }
            ',' if !in_q => {
                out.push(std::mem::take(&mut cur));
            }
            _ => cur.push(c),
        }
    }
    out.push(cur);
    out
}

fn header_index(headers: &[String], name: &str) -> Option<usize> {
    let want = name.to_ascii_lowercase();
    headers.iter().position(|h| h.trim().eq_ignore_ascii_case(&want))
}

/// Parse NCSBE Candidate_Listing CSV text into filings.
pub fn parse_candidate_listing_csv(csv: &str) -> Vec<NcFiling> {
    let mut lines = csv.lines().filter(|l| !l.trim().is_empty());
    let Some(header_line) = lines.next() else {
        return Vec::new();
    };
    let headers = split_csv_line(header_line.trim_start_matches('\u{feff}'));
    let (i_elec, i_county, i_contest, i_name) = match (
        header_index(&headers, "election_dt"),
        header_index(&headers, "county_name"),
        header_index(&headers, "contest_name"),
        header_index(&headers, "name_on_ballot"),
    ) {
        (Some(a), Some(b), Some(c), Some(d)) => (a, b, c, d),
        _ => return Vec::new(),
    };
    let i_party = header_index(&headers, "party_candidate").unwrap_or(usize::MAX);

    let mut out = Vec::new();
    for line in lines {
        let cols = split_csv_line(line);
        let get = |i: usize| cols.get(i).map(|s| s.trim().to_string()).unwrap_or_default();
        let election_dt = get(i_elec);
        let county_name = get(i_county);
        let contest_name = get(i_contest);
        let name_on_ballot = get(i_name);
        if contest_name.is_empty() || name_on_ballot.is_empty() {
            continue;
        }
        let party_candidate = if i_party != usize::MAX {
            get(i_party)
        } else {
            String::new()
        };
        out.push(NcFiling {
            election_dt,
            county_name,
            contest_name,
            name_on_ballot,
            party_candidate,
        });
    }
    out
}

fn is_general_election(dt: &str, cycle: i32) -> bool {
    // Prefer November of cycle: 11/03/2026
    let y = cycle.to_string();
    let Some((m, rest)) = dt.split_once('/') else {
        return false;
    };
    m == "11" && rest.ends_with(&y)
}

fn county_key(s: &str) -> String {
    s.trim()
        .to_ascii_uppercase()
        .replace(" COUNTY", "")
        .replace(' ', "")
}

fn party_label(code: &str) -> String {
    match code.trim().to_ascii_uppercase().as_str() {
        "DEM" => "Democratic".into(),
        "REP" => "Republican".into(),
        "LIB" => "Libertarian".into(),
        "GRE" | "GRN" => "Green".into(),
        "UNA" | "UNAF" => "Unaffiliated".into(),
        "" => "Unknown".into(),
        other => normalize_party_label(other),
    }
}

fn senate_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)^NC STATE SENATE DISTRICT\s+0*(\d+)$").unwrap())
}

fn house_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        // Allow zero-padded districts (e.g. DISTRICT 034)
        Regex::new(r"(?i)^NC HOUSE OF REPRESENTATIVES DISTRICT\s+0*(\d+)$").unwrap()
    })
}

fn us_house_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)^US HOUSE OF REPRESENTATIVES DISTRICT\s+(\d+)$").unwrap()
    })
}

fn is_us_federal(contest: &str) -> bool {
    let u = contest.to_ascii_uppercase();
    u.starts_with("US SENATE") || u.starts_with("US HOUSE") || u.starts_with("PRESIDENT")
}

fn is_judicial(contest: &str) -> bool {
    let u = contest.to_ascii_uppercase();
    u.contains("JUDGE") || u.contains("JUSTICE") || u.contains("COURT")
}

/// Map NCSBE filings for this ZIP geo (general election of `cycle`).
/// Skips federal (FEC covers those). Includes leg by district + county judicial/local.
pub fn map_filings_for_geo(
    filings: &[NcFiling],
    geo: &GeoResolution,
    cycle: i32,
) -> Vec<SnapshotCandidate> {
    let county = county_key(&geo.county);
    let state_l = "nc";
    let state_ocd = format!("ocd-division/country:us/state:{state_l}");
    let source = candidate_listing_csv_url(cycle);

    let mut seen = HashSet::new();
    let mut out = Vec::new();

    for f in filings {
        if !is_general_election(&f.election_dt, cycle) {
            continue;
        }
        if is_us_federal(&f.contest_name) {
            continue;
        }
        let dedupe = format!(
            "{}|{}|{}",
            f.contest_name.to_ascii_uppercase(),
            f.name_on_ballot.to_ascii_uppercase(),
            f.party_candidate.to_ascii_uppercase()
        );
        if !seen.insert(dedupe) {
            continue;
        }

        let contest = f.contest_name.trim();
        let party = party_label(&f.party_candidate);

        if let Some(cap) = senate_re().captures(contest) {
            let d: u32 = cap[1].parse().unwrap_or(0);
            if geo.state_senate_district != Some(d) {
                continue;
            }
            let ocd = format!("ocd-division/country:us/state:{state_l}/sldu:{d}");
            out.push(SnapshotCandidate {
                office: format!("North Carolina Senate (District {d})"),
                chamber: Some("state_senate".into()),
                jurisdiction_ocd: ocd,
                is_judicial: false,
                name: f.name_on_ballot.clone(),
                party,
                is_incumbent: false,
                is_judge: false,
                summary: Some("NC State Board of Elections candidate filing (general).".into()),
                source_url: source.clone(),
                source_publisher: Some(SOURCE_PUBLISHER.into()),
                external_id: None,
            });
            continue;
        }

        if let Some(cap) = house_re().captures(contest) {
            let d: u32 = cap[1].parse().unwrap_or(0);
            if geo.state_house_district != Some(d) {
                continue;
            }
            let ocd = format!("ocd-division/country:us/state:{state_l}/sldl:{d}");
            out.push(SnapshotCandidate {
                office: format!("North Carolina House (District {d})"),
                chamber: Some("state_house".into()),
                jurisdiction_ocd: ocd,
                is_judicial: false,
                name: f.name_on_ballot.clone(),
                party,
                is_incumbent: false,
                is_judge: false,
                summary: Some("NC State Board of Elections candidate filing (general).".into()),
                source_url: source.clone(),
                source_publisher: Some(SOURCE_PUBLISHER.into()),
                external_id: None,
            });
            continue;
        }

        // County-scoped judicial / local (not precinct-precise).
        if county_key(&f.county_name) != county {
            continue;
        }
        // Skip multi-county statewide exec unless contest is clearly statewide and county-duplicated
        // Statewide contests appear once per county — take only matching county row once via seen.
        let judicial = is_judicial(contest);
        let chamber = if judicial {
            "judicial"
        } else if contest.to_ascii_uppercase().contains("COUNTY")
            || contest.to_ascii_uppercase().contains("BOARD")
            || contest.to_ascii_uppercase().contains("CITY")
            || contest.to_ascii_uppercase().contains("TOWN")
            || contest.to_ascii_uppercase().contains("SCHOOL")
        {
            "county"
        } else if contest.to_ascii_uppercase().starts_with("NC ")
            && !contest.to_ascii_uppercase().contains("DISTRICT")
        {
            // NC Governor etc. — statewide, county-duplicated; include once (dedupe already)
            "statewide"
        } else {
            // Skip unknown multi-district noise
            continue;
        };

        // Don't pull every county race for huge counties without filter — still FL model.
        if chamber == "county" || judicial || chamber == "statewide" {
            let ocd = if judicial {
                format!("{state_ocd}/county:{}", county.to_ascii_lowercase())
            } else if chamber == "statewide" {
                state_ocd.clone()
            } else {
                format!("{state_ocd}/county:{}", county.to_ascii_lowercase())
            };
            // Title-case contest for office label
            let office = contest.to_string();
            out.push(SnapshotCandidate {
                office,
                chamber: Some(chamber.into()),
                jurisdiction_ocd: ocd,
                is_judicial: judicial,
                name: f.name_on_ballot.clone(),
                party,
                is_incumbent: false,
                is_judge: judicial,
                summary: Some(if judicial {
                    "NC judicial candidate filing (county-scoped; not precinct-filtered).".into()
                } else if chamber == "statewide" {
                    "NC statewide candidate filing (NCSBE general list).".into()
                } else {
                    "NC local candidate filing (county-scoped; not precinct-filtered).".into()
                }),
                source_url: source.clone(),
                source_publisher: Some(SOURCE_PUBLISHER.into()),
                external_id: None,
            });
        }
    }

    // Stable order: senate, house, statewide, judicial, county
    out.sort_by(|a, b| {
        chamber_rank(a.chamber.as_deref())
            .cmp(&chamber_rank(b.chamber.as_deref()))
            .then_with(|| a.office.cmp(&b.office))
            .then_with(|| a.name.cmp(&b.name))
    });
    out
}

fn chamber_rank(c: Option<&str>) -> u8 {
    match c {
        Some("state_senate") => 0,
        Some("state_house") => 1,
        Some("statewide") => 2,
        Some("judicial") => 3,
        Some("county") => 4,
        _ => 9,
    }
}

/// Suppress unused warning for us_house_re until federal cross-check is wanted.
#[allow(dead_code)]
fn _us_house_district(contest: &str) -> Option<u32> {
    us_house_re()
        .captures(contest)
        .and_then(|c| c.get(1)?.as_str().parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ResolvedJurisdiction;

    fn wake_geo(sd: u32, hd: u32) -> GeoResolution {
        GeoResolution {
            state: "NC".into(),
            state_name: "North Carolina".into(),
            county: "Wake County".into(),
            city: "Raleigh".into(),
            congressional_district: "NC-2".into(),
            state_senate_district: Some(sd),
            state_house_district: Some(hd),
            state_house_label: Some(hd.to_string()),
            latitude: Some(35.78),
            longitude: Some(-78.64),
            jurisdictions: vec![ResolvedJurisdiction {
                ocd_id: "ocd-division/country:us/state:nc".into(),
                name: "North Carolina".into(),
                level: "state".into(),
                state: Some("NC".into()),
            }],
            source_url: "test".into(),
            source_publisher: "test".into(),
        }
    }

    const SAMPLE: &str = r#""election_dt","county_name","contest_name","name_on_ballot","first_name","middle_name","last_name","name_suffix_lbl","nick_name","street_address","city","state","zip_code","phone","office_phone","business_phone","email","candidacy_dt","party_contest","party_candidate","is_unexpired","has_primary","is_partisan","vote_for","term"
"11/03/2026","WAKE","NC STATE SENATE DISTRICT 15","Jay J. Chaudhuri","JAY","JYOTI","CHAUDHURI","","","820 GRAHAM ST","RALEIGH","NC","27605","","","9194235281","jay@jayfornc.com","12/12/2025","","DEM","FALSE","FALSE","TRUE","1","2"
"11/03/2026","WAKE","NC STATE SENATE DISTRICT 15","David Bankert","DAVID","PAUL","BANKERT","","","5420 PEAR ORCHARD LN","RALEIGH","NC","27616","","","9192155670","bank4ncsen@gmail.com","12/11/2025","","REP","FALSE","FALSE","TRUE","1","2"
"11/03/2026","DURHAM","NC STATE SENATE DISTRICT 15","Jay J. Chaudhuri","JAY","JYOTI","CHAUDHURI","","","820 GRAHAM ST","RALEIGH","NC","27605","","","9194235281","jay@jayfornc.com","12/12/2025","","DEM","FALSE","FALSE","TRUE","1","2"
"11/03/2026","WAKE","NC HOUSE OF REPRESENTATIVES DISTRICT 034","Tim Longest","TIMOTHY","WORTH","LONGEST","JR","TIM","PO BOX 482","RALEIGH","NC","27602","","","9196007800","TIM@TIMFORNC.COM","12/01/2025","","DEM","FALSE","FALSE","TRUE","1","2"
"11/03/2026","WAKE","NC STATE SENATE DISTRICT 015","Pad Sen","PAD","","SEN","","","1 MAIN","RALEIGH","NC","27601","","","","","12/01/2025","","DEM","FALSE","FALSE","TRUE","1","2"
"11/03/2026","WAKE","US HOUSE OF REPRESENTATIVES DISTRICT 02","Deborah K. Ross","DEBORAH","K","ROSS","","","P.O. BOX 28258","RALEIGH","NC","27611","","","9196061608","INFO@DEBORAHROSS.COM","12/05/2025","","DEM","FALSE","FALSE","TRUE","1","2"
"03/03/2026","WAKE","NC STATE SENATE DISTRICT 15","Primary Only","P","","ONLY","","","","RALEIGH","NC","27601","","","","","12/01/2025","DEM","DEM","FALSE","TRUE","TRUE","1","2"
"11/03/2026","WAKE","NC DISTRICT COURT JUDGE DISTRICT 10A SEAT 01","Rashad A. Hauter","RASHAD","AHMED","HAUTER","","","9929 SAN REMO PL","WAKE FOREST","NC","27587","","","2522266878","RHAUTER@GMAIL.COM","12/01/2025","","REP","FALSE","FALSE","TRUE","1","4"
"#;

    #[test]
    fn parse_nc_referendum_list_wake() {
        let text = include_str!("../../../../testdata/nc_referendums_sample.txt");
        let all = parse_nc_referendum_list_text(text, None);
        assert!(all.len() >= 3, "got {}", all.len());
        let wake = parse_nc_referendum_list_text(text, Some("Wake County"));
        assert!(
            wake.iter().any(|m| m.is_statewide && m.title.contains("PHOTO ID")),
            "{wake:?}"
        );
        assert!(
            wake.iter().any(|m| m.title.contains("SCHOOL BONDS")),
            "expected Wake school bonds in {wake:?}"
        );
        assert!(
            wake.iter().any(|m| m.title.contains("RALEIGH")),
            "expected Raleigh bonds in {wake:?}"
        );
        // Durham-only locals should not appear when filtering Wake
        assert!(!wake.iter().any(|m| {
            !m.is_statewide && m.county.eq_ignore_ascii_case("durham")
        }));
        let photo = wake
            .iter()
            .find(|m| m.title.contains("PHOTO ID"))
            .expect("photo");
        assert!(photo.summary.as_ref().is_some_and(|s| s.len() > 20));

        let (ms, _) = map_nc_measures_for_geo(
            &wake,
            "ocd-division/country:us/state:nc",
            "Wake County",
            "https://example.test/referendums.pdf",
        );
        assert!(!ms.is_empty());
        assert!(ms.iter().all(|m| m.source_publisher.as_deref() == Some(SOURCE_PUBLISHER)));
        assert!(ms.iter().any(|m| m.measure_code.as_deref() == Some("REQUIRE PHOTO ID FOR VOTING")));
    }

    #[test]
    fn parses_and_maps_wake_leg() {
        let filings = parse_candidate_listing_csv(SAMPLE);
        assert!(filings.len() >= 6);
        let cands = map_filings_for_geo(&filings, &wake_geo(15, 34), 2026);
        assert!(cands.iter().any(|c| c.name.contains("Chaudhuri")));
        assert!(cands.iter().any(|c| c.name.contains("Bankert")));
        assert!(cands.iter().any(|c| c.name.contains("Longest"))); // zero-padded HD 034
        assert!(cands.iter().any(|c| c.name.contains("Pad Sen"))); // zero-padded SD 015
        // federal skipped
        assert!(!cands.iter().any(|c| c.name.contains("Ross")));
        // primary skipped
        assert!(!cands.iter().any(|c| c.name.contains("Primary Only")));
        // dedupe county copy of same senate race
        assert_eq!(
            cands
                .iter()
                .filter(|c| c.name.contains("Chaudhuri"))
                .count(),
            1
        );
        // judicial county-scoped
        assert!(cands.iter().any(|c| c.is_judge && c.name.contains("Hauter")));
    }

    #[test]
    fn csv_url() {
        assert!(candidate_listing_csv_url(2026).contains("2026"));
        assert!(candidate_listing_csv_url(2026).contains("Candidate_Listing"));
    }

    #[test]
    fn nc_cf_search_parse_and_match() {
        let html = include_str!("../../../../testdata/nc_cf_search_chaudhuri.html");
        let hits = parse_nc_cf_committee_search_html(html).expect("parse search");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].sboe_id, "STA-4P6QA8-C-001");
        assert_eq!(hits[0].org_group_id, 30330);
        assert!(hits[0].active);

        let q = NcCfMatchQuery {
            name: "Jay J. Chaudhuri".into(),
            office: "NC State Senate (District 15)".into(),
            chamber: "state_senate".into(),
            party: "Democratic".into(),
        };
        match match_nc_cf_committee(&hits, &q) {
            NcCfMatch::Unique { hit } => assert_eq!(hit.sboe_id, "STA-4P6QA8-C-001"),
            other => panic!("expected unique, got {other:?}"),
        }

        let none_q = NcCfMatchQuery {
            name: "Nobody Here".into(),
            office: q.office.clone(),
            chamber: q.chamber.clone(),
            party: String::new(),
        };
        assert!(matches!(match_nc_cf_committee(&hits, &none_q), NcCfMatch::None));

        let amb = vec![
            hits[0].clone(),
            NcCommitteeHit {
                cand_name: "Jay Other".into(),
                org_name: "OTHER FOR NC".into(),
                sboe_id: "STA-OTHER".into(),
                org_group_id: 1,
                status_desc: "ACTIVE".into(),
                active: true,
            },
        ];
        // last name Other vs Chaudhuri — not amb; need same last
        let amb2 = vec![
            hits[0].clone(),
            NcCommitteeHit {
                cand_name: "Jay Chaudhuri".into(),
                org_name: "CHAUDHURI B".into(),
                sboe_id: "STA-B".into(),
                org_group_id: 2,
                status_desc: "ACTIVE".into(),
                active: true,
            },
        ];
        match match_nc_cf_committee(&amb2, &q) {
            NcCfMatch::Ambiguous { count } => assert_eq!(count, 2),
            other => panic!("expected amb, got {other:?}"),
        }
        let _ = amb;
    }

    #[test]
    fn nc_cf_docs_and_summary() {
        let docs_html = include_str!("../../../../testdata/nc_cf_docs_chaudhuri.html");
        let docs = parse_nc_cf_documents_html(docs_html).expect("docs");
        assert!(!docs.is_empty());
        let best = pick_latest_disclosure_report(&docs, 2026).expect("report");
        assert_eq!(best.data_link, "232644");
        assert!(best.report_year >= 2026);

        let csv = include_str!("../../../../testdata/nc_cf_sum_232644.csv");
        let sum = parse_nc_cf_summary_csv(csv).expect("sum");
        assert!((sum.receipts_cycle.unwrap() - 592391.72).abs() < 0.01);
        assert!((sum.expenditures_cycle.unwrap() - 610122.21).abs() < 0.01);
        assert!((sum.cash_on_hand.unwrap() - 118928.05).abs() < 0.01);

        let hit = NcCommitteeHit {
            cand_name: "JAY CHAUDHURI".into(),
            org_name: "CHAUDHURI FOR NEW NC".into(),
            sboe_id: "STA-4P6QA8-C-001".into(),
            org_group_id: 30330,
            status_desc: "ACTIVE".into(),
            active: true,
        };
        let fin = nc_cf_finance_from_hit(&hit, &sum, &best, 2026);
        assert_eq!(fin.source, "nc_cf");
        assert!(fin.receipts_display.contains("592"));
        assert!(fin.profile_url.contains("STA-4P6QA8-C-001"));
        assert_eq!(fin.report_id, "232644");

        let form = nc_cf_committee_search_form("Chaudhuri");
        assert!(form.contains("UseCandName=true"));
        assert!(form.contains("Name=Chaudhuri"));
        assert!(nc_cf_documents_url("STA-4P6QA8-C-001", 30330).contains("OGID=30330"));
        assert!(nc_cf_summary_csv_url("232644").contains("Type=SUM"));
    }
}

// --- NC campaign finance (cf.ncsbe.gov) — A4 ---

pub const NC_CF_BASE: &str = "https://cf.ncsbe.gov";
pub const NC_CF_SEARCH_URL: &str = "https://cf.ncsbe.gov/CFOrgLkup/";
pub const NC_CF_PUBLISHER: &str = "N.C. State Board of Elections campaign finance";

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

/// POST body for committee search by candidate name fragment (usually last name).
pub fn nc_cf_committee_search_form(name: &str) -> String {
    format!(
        "UseOrgName=false&UseCandName=true&UseInHouseName=false&UseAcronym=false&Name={}",
        urlenc(name.trim())
    )
}

pub fn nc_cf_documents_url(sboe_id: &str, org_group_id: u32) -> String {
    format!(
        "{NC_CF_BASE}/CFOrgLkup/DocumentGeneralResult/?SID={}&OGID={org_group_id}",
        urlenc(sboe_id.trim())
    )
}

pub fn nc_cf_summary_csv_url(report_id: &str) -> String {
    format!(
        "{NC_CF_BASE}/CFOrgLkup/ExportDetailResults/?ReportID={}&Type=SUM&Title=x",
        urlenc(report_id.trim())
    )
}

pub fn nc_cf_report_detail_url(report_id: &str) -> String {
    format!(
        "{NC_CF_BASE}/CFOrgLkup/ReportDetail/?RID={}&TP=SUM",
        urlenc(report_id.trim())
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NcCommitteeHit {
    pub org_name: String,
    pub sboe_id: String,
    pub cand_name: String,
    pub status_desc: String,
    pub org_group_id: u32,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NcDisclosureReport {
    pub committee_name: String,
    pub sboe_id: String,
    pub report_year: i32,
    pub report_type: String,
    pub document_type: String,
    pub period_end: String,
    pub data_link: String,
}

#[derive(Debug, Clone, Default)]
pub struct NcCfSummary {
    pub receipts_cycle: Option<f64>,
    pub expenditures_cycle: Option<f64>,
    pub cash_on_hand: Option<f64>,
    pub cash_beginning_cycle: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct NcCfMatchQuery {
    pub name: String,
    pub office: String,
    pub chamber: String,
    pub party: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind")]
pub enum NcCfMatch {
    #[serde(rename = "unique")]
    Unique { hit: NcCommitteeHit },
    #[serde(rename = "none")]
    None,
    #[serde(rename = "ambiguous")]
    Ambiguous { count: usize },
}

#[derive(Debug, Clone, Serialize)]
pub struct NcCfFinance {
    pub source: String,
    pub cycle: String,
    pub account: String,
    pub match_name: String,
    pub match_office: String,
    pub receipts_display: String,
    pub disbursements_display: String,
    pub cash_on_hand_display: String,
    pub source_label: String,
    pub profile_url: String,
    pub report_url: String,
    pub report_id: String,
    pub report_name: String,
    pub note: String,
}

/// Extract first JSON array assigned in a `var name = [...]` style script block.
pub fn extract_js_json_array(html: &str) -> Option<String> {
    // Prefer `var data = [...]`
    let re = Regex::new(r"(?s)var\s+data\s*=\s*(\[.*?\])\s*;").ok()?;
    if let Some(c) = re.captures(html) {
        return Some(c.get(1)?.as_str().to_string());
    }
    // Fallback: first large array containing OrgName or DataLink
    let idx = html
        .find("\"OrgName\"")
        .or_else(|| html.find("\"DataLink\""))
        .or_else(|| html.find("\"CandName\""))?;
    let start = html[..idx].rfind('[')?;
    let mut depth = 0i32;
    let bytes = html.as_bytes();
    let mut end = None;
    for (i, &b) in bytes.iter().enumerate().skip(start) {
        match b {
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(i + 1);
                    break;
                }
            }
            _ => {}
        }
    }
    end.map(|e| html[start..e].to_string())
}

pub fn parse_nc_cf_committee_search_html(html: &str) -> Result<Vec<NcCommitteeHit>, String> {
    let raw = extract_js_json_array(html).ok_or_else(|| "no committee JSON array".to_string())?;
    let rows: Vec<Value> =
        serde_json::from_str(&raw).map_err(|e| format!("committee JSON: {e}"))?;
    let mut out = Vec::new();
    for r in rows {
        let org_name = r
            .get("OrgName")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let sboe_id = r
            .get("SBoEID")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let cand_name = r
            .get("CandName")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let status_desc = r
            .get("StatusDesc")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let org_group_id = r
            .get("OrgGroupID")
            .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|n| n as u64)))
            .unwrap_or(0) as u32;
        if sboe_id.is_empty() && cand_name.is_empty() && org_name.is_empty() {
            continue;
        }
        let active = status_desc.to_ascii_uppercase().contains("ACTIVE");
        out.push(NcCommitteeHit {
            org_name,
            sboe_id,
            cand_name,
            status_desc,
            org_group_id,
            active,
        });
    }
    Ok(out)
}

pub fn parse_nc_cf_documents_html(html: &str) -> Result<Vec<NcDisclosureReport>, String> {
    let raw = extract_js_json_array(html).ok_or_else(|| "no documents JSON array".to_string())?;
    let rows: Vec<Value> = serde_json::from_str(&raw).map_err(|e| format!("docs JSON: {e}"))?;
    let mut out = Vec::new();
    for r in rows {
        let data_link = r
            .get("DataLink")
            .and_then(|v| {
                v.as_str()
                    .map(|s| s.to_string())
                    .or_else(|| v.as_u64().map(|n| n.to_string()))
                    .or_else(|| v.as_i64().map(|n| n.to_string()))
            })
            .unwrap_or_default()
            .trim()
            .to_string();
        if data_link.is_empty() || data_link.eq_ignore_ascii_case("null") {
            continue;
        }
        let document_type = r
            .get("DocumentType")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let report_year = r
            .get("ReportYear")
            .and_then(|v| v.as_i64().or_else(|| v.as_u64().map(|n| n as i64)))
            .unwrap_or(0) as i32;
        out.push(NcDisclosureReport {
            committee_name: r
                .get("CommitteeName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            sboe_id: r
                .get("SBoEID")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            report_year,
            report_type: r
                .get("ReportType")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            document_type,
            period_end: r
                .get("PeriodEndDate")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            data_link,
        });
    }
    Ok(out)
}

/// Prefer Disclosure Report with DataLink for cycle year (or cycle-1), latest period end.
pub fn pick_latest_disclosure_report(
    docs: &[NcDisclosureReport],
    cycle: i32,
) -> Option<&NcDisclosureReport> {
    let mut scored: Vec<(i32, &str, &NcDisclosureReport)> = docs
        .iter()
        .filter(|d| {
            d.document_type
                .to_ascii_lowercase()
                .contains("disclosure")
                && !d.data_link.is_empty()
        })
        .map(|d| {
            let year_score = if d.report_year == cycle {
                2
            } else if d.report_year == cycle - 1 {
                1
            } else if d.report_year > 0 && (d.report_year - cycle).abs() <= 2 {
                0
            } else {
                -1
            };
            (year_score, d.period_end.as_str(), d)
        })
        .filter(|(ys, _, _)| *ys >= 0)
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(a.1)));
    scored.first().map(|(_, _, d)| *d)
}

fn parse_money_cell(s: &str) -> Option<f64> {
    let t = s.trim().replace(',', "").replace('$', "");
    if t.is_empty() {
        return None;
    }
    t.parse::<f64>().ok()
}

/// Parse NCSBE ExportDetailResults Type=SUM CSV.
pub fn parse_nc_cf_summary_csv(csv: &str) -> Result<NcCfSummary, String> {
    let mut sum = NcCfSummary::default();
    let mut lines = csv.lines();
    // Skip title line "SUMMARY" if present
    let first = lines.next().unwrap_or("");
    let header_line = if first.to_ascii_uppercase().contains("SECTION") {
        first
    } else {
        lines.next().unwrap_or("")
    };
    if !header_line.to_ascii_uppercase().contains("SECTION") {
        return Err("NC CF summary CSV missing Section header".into());
    }
    for line in lines {
        let cols = split_csv_line(line);
        if cols.is_empty() {
            continue;
        }
        let section = cols[0].trim();
        let period = cols.get(1).and_then(|s| parse_money_cell(s));
        let cycle = cols.get(2).and_then(|s| parse_money_cell(s));
        let sec_u = section.to_ascii_uppercase();
        if sec_u == "TOTAL RECEIPTS" {
            sum.receipts_cycle = cycle.or(period);
        } else if sec_u == "TOTAL EXPENDITURES" {
            sum.expenditures_cycle = cycle.or(period);
        } else if sec_u.contains("CASH ON HAND AT END") {
            sum.cash_on_hand = cycle.or(period);
        } else if sec_u.contains("CASH ON HAND AT BEGINNING") {
            sum.cash_beginning_cycle = cycle.or(period);
        }
    }
    if sum.receipts_cycle.is_none()
        && sum.expenditures_cycle.is_none()
        && sum.cash_on_hand.is_none()
    {
        return Err("NC CF summary CSV had no totals".into());
    }
    Ok(sum)
}

fn is_active_status(s: &str) -> bool {
    let u = s.to_ascii_uppercase();
    u.contains("ACTIVE") && !u.contains("TERMINATED")
}

/// Match committee hits to ballot name. Prefer ACTIVE; strict name; skip ambiguity.
pub fn match_nc_cf_committee(hits: &[NcCommitteeHit], q: &NcCfMatchQuery) -> NcCfMatch {
    let active: Vec<&NcCommitteeHit> = hits
        .iter()
        .filter(|h| h.active || is_active_status(&h.status_desc))
        .filter(|h| {
            let cand = if h.cand_name.trim().is_empty() {
                // OrgName often "FOO FOR BAR (LAST, FIRST)"
                h.org_name.as_str()
            } else {
                h.cand_name.as_str()
            };
            last_names_match(&q.name, cand) && first_names_compatible(&q.name, cand)
        })
        .collect();

    let pool = if active.is_empty() {
        hits.iter()
            .filter(|h| {
                let cand = if h.cand_name.trim().is_empty() {
                    h.org_name.as_str()
                } else {
                    h.cand_name.as_str()
                };
                last_names_match(&q.name, cand) && first_names_compatible(&q.name, cand)
            })
            .collect::<Vec<_>>()
    } else {
        active
    };

    // Soft office hint from org name when multiple
    let filtered = if pool.len() > 1 {
        let office_l = q.office.to_ascii_lowercase();
        let chamber = q.chamber.to_ascii_lowercase();
        let want_sen = chamber == "state_senate" || office_l.contains("senate");
        let want_house = chamber == "state_house" || office_l.contains("house");
        let narrowed: Vec<_> = pool
            .iter()
            .copied()
            .filter(|h| {
                let o = h.org_name.to_ascii_uppercase();
                if want_sen && (o.contains("SENATE") || o.contains(" SEN ")) {
                    return true;
                }
                if want_house && (o.contains("HOUSE") || o.contains(" REP ")) {
                    return true;
                }
                if !want_sen && !want_house {
                    return true;
                }
                // keep if org has no office keyword either way
                !o.contains("SENATE") && !o.contains("HOUSE")
            })
            .collect();
        if narrowed.len() == 1 {
            narrowed
        } else if !narrowed.is_empty() && narrowed.len() < pool.len() {
            narrowed
        } else {
            pool
        }
    } else {
        pool
    };

    match filtered.len() {
        0 => NcCfMatch::None,
        1 => NcCfMatch::Unique {
            hit: filtered[0].clone(),
        },
        n => NcCfMatch::Ambiguous { count: n },
    }
}

pub fn nc_cf_finance_from_hit(
    hit: &NcCommitteeHit,
    sum: &NcCfSummary,
    report: &NcDisclosureReport,
    cycle: i32,
) -> NcCfFinance {
    let profile = nc_cf_documents_url(&hit.sboe_id, hit.org_group_id);
    let report_url = nc_cf_report_detail_url(&report.data_link);
    let report_name = if report.report_type.is_empty() {
        format!("{} disclosure", report.report_year)
    } else {
        format!("{} {}", report.report_year, report.report_type)
    };
    NcCfFinance {
        source: "nc_cf".into(),
        cycle: cycle.to_string(),
        account: hit.sboe_id.clone(),
        match_name: if hit.cand_name.is_empty() {
            hit.org_name.clone()
        } else {
            hit.cand_name.clone()
        },
        match_office: hit.org_name.clone(),
        receipts_display: sum
            .receipts_cycle
            .map(format_usd)
            .unwrap_or_else(|| "—".into()),
        disbursements_display: sum
            .expenditures_cycle
            .map(format_usd)
            .unwrap_or_else(|| "—".into()),
        cash_on_hand_display: sum
            .cash_on_hand
            .map(format_usd)
            .unwrap_or_else(|| "—".into()),
        source_label: NC_CF_PUBLISHER.into(),
        profile_url: profile,
        report_url,
        report_id: report.data_link.clone(),
        report_name,
        note: "Totals from latest NCSBE disclosure report summary (election-cycle column). Filed amounts — not a bank-certified cash-on-hand audit.".into(),
    }
}

/// Last-name fragment for CF search (ballot display name).
pub fn nc_cf_search_name_fragment(ballot_name: &str) -> String {
    let (first, last) = split_candidate_first_last(ballot_name);
    if !last.is_empty() {
        last
    } else if !first.is_empty() {
        first
    } else {
        ballot_name.trim().to_string()
    }
}
