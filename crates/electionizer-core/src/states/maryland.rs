//! Maryland SBE candidate listing CSVs + ballot questions + MDCRIS CF (pure parse/map — no HTTP).

use crate::models::{
    format_usd, normalize_party_label, ContributorRow, GeoResolution, SnapshotCandidate,
    SnapshotMeasure,
};
use crate::ftm::measure_code_key;
use crate::states::florida::{
    district_from_ballot_office, first_names_compatible, html_unescape, last_names_match,
    split_candidate_first_last,
};
use crate::states::north_carolina::split_csv_line;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::sync::OnceLock;

pub const SOURCE_PUBLISHER: &str = "Maryland State Board of Elections";

const BASE: &str = "https://elections.maryland.gov/elections";

/// Prefer general (`GG`) candidate lists; primary (`GP`) is the pre-primary fallback.
pub fn statewide_csv_url(cycle: i32, general: bool) -> String {
    let (folder, code) = if general {
        ("general_candidates", "GG")
    } else {
        ("primary_candidates", "GP")
    };
    format!("{BASE}/{cycle}/{folder}/{cycle}_{code}_statewide_candidatelist.csv")
}

pub fn local_csv_url(cycle: i32, general: bool) -> String {
    let (folder, code) = if general {
        ("general_candidates", "GG")
    } else {
        ("primary_candidates", "GP")
    };
    format!("{BASE}/{cycle}/{folder}/{cycle}_{code}_all_counties_candidatelist.csv")
}

/// SBE public-comment / certified ballot questions page for a cycle.
pub fn ballot_questions_url(cycle: i32) -> String {
    format!("{BASE}/{cycle}/ballot_questions.html")
}

/// One MD ballot question from SBE `ballot_questions.html`.
#[derive(Debug, Clone)]
pub struct ParsedMdMeasure {
    pub code: Option<String>,
    pub title: String,
    pub summary: Option<String>,
    /// Jurisdiction label (`Statewide`, `Anne Arundel County`, `Baltimore City`, …).
    pub jurisdiction: String,
    pub is_statewide: bool,
}

fn strip_tags(s: &str) -> String {
    let re = Regex::new(r"(?is)<[^>]+>").ok();
    let t = match re {
        Some(r) => r.replace_all(s, " ").to_string(),
        None => s.to_string(),
    };
    html_unescape(&t)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_statewide_summary(s: &str) -> bool {
    let u = s.trim().to_ascii_uppercase();
    u == "SBE"
        || u == "STATEWIDE"
        || u.contains("STATE BOARD")
        || u == "MARYLAND"
        || u.starts_with("STATE ")
}

fn jurisdiction_label(summary: &str, statewide: bool) -> String {
    if statewide {
        return "Statewide".into();
    }
    let t = summary.trim();
    if t.is_empty() {
        return "Local".into();
    }
    let lower = t.to_ascii_lowercase();
    if lower.contains("city") || lower.contains("county") {
        t.to_string()
    } else {
        format!("{t} County")
    }
}

fn normalize_question_code(raw: &str) -> Option<String> {
    let t = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    if t.is_empty() {
        return None;
    }
    // "QUESTION A" / "Question 1" → "Question A" / "Question 1"
    let re = Regex::new(r"(?i)^question\s+([0-9]+|[A-Z])\b").ok()?;
    if let Some(c) = re.captures(&t) {
        let id = c.get(1)?.as_str().to_ascii_uppercase();
        return Some(format!("Question {id}"));
    }
    Some(t)
}

fn is_type_or_meta_line(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() {
        return true;
    }
    let u = t.to_ascii_uppercase();
    if u.starts_with("CHARTER AMENDMENT")
        || u.starts_with("BOND ISSUE")
        || u.starts_with("CONSTITUTIONAL AMENDMENT")
        || u.starts_with("REFERENDUM")
        || u.starts_with("ORDINANCE")
        || u.starts_with("(RESOLUTION")
        || u.starts_with("(BILL")
        || u.starts_with("RESOLUTION NO")
        || u.starts_with("BILL ")
        || u == "FOR CHARTER AMENDMENT"
        || u == "AGAINST CHARTER AMENDMENT"
        || u == "FOR BOND ISSUE"
        || u == "AGAINST BOND ISSUE"
        || u.starts_with("FOR THE CONSTITUTIONAL")
        || u.starts_with("AGAINST THE CONSTITUTIONAL")
        || u.starts_with("FOR CONSTITUTIONAL")
        || u.starts_with("AGAINST CONSTITUTIONAL")
        || u.starts_with("(AMENDING ARTICLE")
        || u.starts_with("AMENDING ARTICLE")
    {
        return true;
    }
    // Vote-choice bullets often appear as bare lines after strip.
    if u.starts_with("A VOTE FOR")
        || u.starts_with("A VOTE AGAINST")
        || u.starts_with("FOR ")
            && (u.contains("AMENDMENT") || u.contains("BOND") || u.contains("QUESTION"))
        || u.starts_with("AGAINST ")
            && (u.contains("AMENDMENT") || u.contains("BOND") || u.contains("QUESTION"))
    {
        return true;
    }
    false
}

fn is_descriptive_start(s: &str) -> bool {
    let u = s.trim().to_ascii_uppercase();
    u.starts_with("TO AMEND")
        || u.starts_with("CURRENTLY")
        || u.starts_with("THIS CHARTER")
        || u.starts_with("THIS WOULD AMEND")
        || u.starts_with("THIS AMENDMENT")
        || u.starts_with("AN ORDINANCE")
        || u.starts_with("PROVIDING THAT")
        || u.starts_with("AUTHORIZING")
        || u.starts_with("THE PROPOSED")
        || u.starts_with("QUESTION ")
        || u.starts_with("THE INSPECTOR")
        || u.contains("AUTHORIZES ")
        || u.contains("WOULD REQUIRE")
        || u.contains("WOULD CREATE")
        || u.contains("WOULD AMEND")
}

/// Parse SBE `ballot_questions.html`.
/// When `county_filter` is set, keep **statewide** questions + that county/city only.
pub fn parse_md_ballot_questions_html(
    html: &str,
    county_filter: Option<&str>,
) -> Vec<ParsedMdMeasure> {
    let want = county_filter
        .map(county_key)
        .filter(|s| !s.is_empty());
    let mut out = Vec::new();

    let details_re = match Regex::new(r"(?is)<details[^>]*>\s*<summary[^>]*>\s*([^<]+?)\s*</summary>(.*?)</details>") {
        Ok(r) => r,
        Err(_) => return out,
    };
    let h2_re = match Regex::new(r"(?is)<h2[^>]*>\s*([^<]+?)\s*</h2>") {
        Ok(r) => r,
        Err(_) => return out,
    };
    let es_re = match Regex::new(r#"(?is)<div[^>]*\blang\s*=\s*["']es["'][^>]*>.*?</div>"#) {
        Ok(r) => r,
        Err(_) => return out,
    };
    let p_re = match Regex::new(r"(?is)<p\b[^>]*>(.*?)</p>") {
        Ok(r) => r,
        Err(_) => return out,
    };
    let bold_re = match Regex::new(r"(?is)<b\b[^>]*>(.*?)</b>") {
        Ok(r) => r,
        Err(_) => return out,
    };
    let li_re = match Regex::new(r"(?is)<li\b[^>]*>(.*?)</li>") {
        Ok(r) => r,
        Err(_) => return out,
    };

    for d in details_re.captures_iter(html) {
        let summary = html_unescape(d.get(1).map(|m| m.as_str()).unwrap_or("").trim());
        if summary.is_empty() {
            continue;
        }
        let is_statewide = is_statewide_summary(&summary);
        let juris_key = if is_statewide {
            String::new()
        } else {
            county_key(&summary)
        };
        if let Some(ref w) = want {
            if !is_statewide && &juris_key != w {
                continue;
            }
        }
        let jurisdiction = jurisdiction_label(&summary, is_statewide);
        let mut body = d.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
        body = es_re.replace_all(&body, "").to_string();

        let headers: Vec<_> = h2_re.captures_iter(&body).collect();
        for (i, h) in headers.iter().enumerate() {
            let h2_raw = html_unescape(h.get(1).map(|m| m.as_str()).unwrap_or("").trim());
            let h2_lower = h2_raw.to_ascii_lowercase();
            // Skip Spanish headings and non-question chrome.
            if h2_lower.starts_with("pregunta") {
                continue;
            }
            if !h2_lower.contains("question") {
                continue;
            }
            let code = normalize_question_code(&h2_raw);
            let start = h.get(0).map(|m| m.end()).unwrap_or(0);
            let end = headers
                .get(i + 1)
                .and_then(|n| n.get(0).map(|m| m.start()))
                .unwrap_or(body.len());
            let section = body.get(start..end).unwrap_or("");

            let mut bolds: Vec<String> = bold_re
                .captures_iter(section)
                .filter_map(|c| {
                    let t = strip_tags(c.get(1).map(|m| m.as_str()).unwrap_or(""));
                    if t.is_empty() {
                        None
                    } else {
                        Some(t)
                    }
                })
                .collect();
            // Drop type labels from bold list.
            bolds.retain(|b| {
                let u = b.to_ascii_uppercase();
                !u.starts_with("CONSTITUTIONAL AMENDMENT")
                    && !u.starts_with("CHARTER AMENDMENT")
                    && !u.starts_with("BOND ISSUE")
                    && !u.starts_with("CAP.")
                    && !u.starts_with("CH.")
            });

            let mut paras: Vec<String> = p_re
                .captures_iter(section)
                .filter_map(|c| {
                    let t = strip_tags(c.get(1).map(|m| m.as_str()).unwrap_or(""));
                    if t.is_empty() || is_type_or_meta_line(&t) {
                        None
                    } else {
                        Some(t)
                    }
                })
                .collect();
            // If almost nothing in <p>, try list items (some city pages).
            if paras.is_empty() {
                paras = li_re
                    .captures_iter(section)
                    .filter_map(|c| {
                        let t = strip_tags(c.get(1).map(|m| m.as_str()).unwrap_or(""));
                        if t.is_empty() || is_type_or_meta_line(&t) {
                            None
                        } else {
                            Some(t)
                        }
                    })
                    .collect();
            }

            let title = if let Some(b) = bolds.first() {
                b.clone()
            } else if let Some(p) = paras.iter().find(|p| !is_descriptive_start(p) && p.len() < 160)
            {
                p.clone()
            } else if let Some(p) = paras.first() {
                // Truncate long ordinance text for title.
                if p.len() > 140 {
                    let mut cut = p.chars().take(137).collect::<String>();
                    if let Some(i) = cut.rfind(' ') {
                        cut.truncate(i);
                    }
                    format!("{cut}…")
                } else {
                    p.clone()
                }
            } else {
                code.clone().unwrap_or_else(|| "Ballot question".into())
            };

            let summary_text = paras
                .iter()
                .find(|p| {
                    let same = p.eq_ignore_ascii_case(&title);
                    !same && (is_descriptive_start(p) || p.len() > title.len())
                })
                .cloned()
                .or_else(|| {
                    paras
                        .iter()
                        .find(|p| !p.eq_ignore_ascii_case(&title))
                        .cloned()
                });

            out.push(ParsedMdMeasure {
                code,
                title,
                summary: summary_text,
                jurisdiction: jurisdiction.clone(),
                is_statewide,
            });
        }
    }
    out
}

/// Map parsed MD measures into snapshot rows (statewide + ZIP county).
pub fn map_md_measures_for_geo(
    parsed: &[ParsedMdMeasure],
    state_ocd: &str,
    county_name: &str,
    source_url: &str,
) -> (Vec<SnapshotMeasure>, Vec<crate::models::ResolvedJurisdiction>) {
    let mut measures = Vec::new();
    let mut extras = Vec::new();
    let county_slug = county_key(county_name).to_ascii_lowercase();
    let county_ocd = if county_slug.is_empty() {
        None
    } else {
        let ocd = format!("{state_ocd}/county:{county_slug}");
        let label = {
            let raw = county_name.trim();
            if raw.is_empty() {
                "County".into()
            } else if raw.to_ascii_lowercase().contains("county")
                || raw.to_ascii_lowercase().contains("city")
            {
                raw.to_string()
            } else {
                format!("{raw} County")
            }
        };
        extras.push(crate::models::ResolvedJurisdiction {
            ocd_id: ocd.clone(),
            name: label,
            level: "county".into(),
            state: Some("MD".into()),
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
        let title = match &m.code {
            Some(c) if !m.title.to_ascii_lowercase().contains(&c.to_ascii_lowercase()) => {
                format!("{c}: {}", m.title)
            }
            _ => m.title.clone(),
        };
        measures.push(SnapshotMeasure {
            title,
            measure_code: m.code.clone(),
            jurisdiction_ocd,
            summary: m.summary.clone(),
            source_url: source_url.to_string(),
            source_publisher: Some(SOURCE_PUBLISHER.into()),
        });
    }
    (measures, extras)
}

#[derive(Debug, Clone)]
pub struct MdFiling {
    pub office: String,
    pub contest: String,
    pub last_name: String,
    pub first_name: String,
    pub party: String,
    pub residential: String,
    pub status: String,
}

fn header_index(headers: &[String], name: &str) -> Option<usize> {
    let want = name.to_ascii_lowercase();
    headers
        .iter()
        .position(|h| h.trim().eq_ignore_ascii_case(&want))
}

/// Parse MSBE candidatelist CSV text into filings.
pub fn parse_candidate_csv(csv: &str) -> Vec<MdFiling> {
    let mut lines = csv.lines().filter(|l| !l.trim().is_empty());
    let Some(header_line) = lines.next() else {
        return Vec::new();
    };
    let headers = split_csv_line(header_line.trim_start_matches('\u{feff}'));
    let (i_office, i_contest, i_last, i_first) = match (
        header_index(&headers, "Office Name"),
        header_index(&headers, "Contest Run By District Name and Number"),
        header_index(&headers, "Candidate Ballot Last Name and Suffix"),
        header_index(&headers, "Candidate First Name and Middle Name"),
    ) {
        (Some(a), Some(b), Some(c), Some(d)) => (a, b, c, d),
        _ => return Vec::new(),
    };
    let i_party = header_index(&headers, "Office Political Party").unwrap_or(usize::MAX);
    let i_res =
        header_index(&headers, "Candidate Residential Jurisdiction").unwrap_or(usize::MAX);
    let i_status = header_index(&headers, "Candidate Status").unwrap_or(usize::MAX);

    let mut out = Vec::new();
    for line in lines {
        let cols = split_csv_line(line);
        let get = |i: usize| {
            cols.get(i)
                .map(|s| s.trim().trim_matches('"').to_string())
                .unwrap_or_default()
        };
        let office = get(i_office);
        let last_name = get(i_last);
        let first_name = get(i_first);
        if office.is_empty() || (last_name.is_empty() && first_name.is_empty()) {
            continue;
        }
        out.push(MdFiling {
            office,
            contest: get(i_contest),
            last_name,
            first_name,
            party: if i_party != usize::MAX {
                get(i_party)
            } else {
                String::new()
            },
            residential: if i_res != usize::MAX {
                get(i_res)
            } else {
                String::new()
            },
            status: if i_status != usize::MAX {
                get(i_status)
            } else {
                String::new()
            },
        });
    }
    out
}

fn is_active(status: &str) -> bool {
    let s = status.trim().to_ascii_lowercase();
    s == "active" || s.starts_with("active ")
}

fn display_name(f: &MdFiling) -> String {
    format!("{} {}", f.first_name.trim(), f.last_name.trim())
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn county_key(s: &str) -> String {
    let u = s.trim().to_ascii_uppercase().replace('\'', "");
    // "Baltimore City" must not collapse to the same key as "Baltimore County".
    if u.contains("BALTIMORE") && u.contains("CITY") {
        return "BALTIMORECITY".into();
    }
    u.replace(" COUNTY", "")
        .replace(" CITY", "")
        .replace(' ', "")
}

fn party_label(p: &str) -> String {
    match p.trim() {
        "" => "Unknown".into(),
        "Non-Partisan" => "Nonpartisan".into(),
        other => normalize_party_label(other),
    }
}

fn leg_district_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)^Legislative\s+District\s+(\d+[A-Za-z]?)$").unwrap()
    })
}

fn circuit_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)^Judicial\s+Circuit\s+(\d+)$").unwrap())
}

fn appellate_circuit_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)^Appellate\s+Circuit\s+(\d+)$").unwrap())
}

/// Maryland Circuit Court circuits → counties (mdcourts.gov).
/// Keys are normalized via `county_key` (no "County", no apostrophe).
fn circuit_for_county(county: &str) -> Option<u32> {
    let c = county_key(county);
    let n = match c.as_str() {
        "DORCHESTER" | "SOMERSET" | "WICOMICO" | "WORCESTER" => 1,
        "CAROLINE" | "CECIL" | "KENT" | "QUEENANNES" | "TALBOT" => 2,
        "BALTIMORE" | "HARFORD" => 3, // Baltimore County (not City)
        "ALLEGANY" | "GARRETT" | "WASHINGTON" => 4,
        "ANNEARUNDEL" | "CARROLL" | "HOWARD" => 5,
        "FREDERICK" | "MONTGOMERY" => 6,
        "CALVERT" | "CHARLES" | "PRINCEGEORGES" | "SAINTMARYS" | "STMARYS" => 7,
        "BALTIMORECITY" => 8,
        _ => return None,
    };
    Some(n)
}

/// Supreme Court of Maryland appellate judicial circuits (MD Const. art. IV;
/// one justice per circuit, retention on that circuit’s ballot).
fn appellate_circuit_for_county(county: &str) -> Option<u32> {
    let c = county_key(county);
    let n = match c.as_str() {
        "CAROLINE" | "CECIL" | "DORCHESTER" | "KENT" | "QUEENANNES" | "SOMERSET"
        | "TALBOT" | "WICOMICO" | "WORCESTER" => 1,
        "BALTIMORE" | "HARFORD" => 2,
        "ALLEGANY" | "CARROLL" | "FREDERICK" | "GARRETT" | "HOWARD" | "WASHINGTON" => 3,
        "PRINCEGEORGES" => 4,
        "ANNEARUNDEL" | "CALVERT" | "CHARLES" | "SAINTMARYS" | "STMARYS" => 5,
        "BALTIMORECITY" => 6,
        "MONTGOMERY" => 7,
        _ => return None,
    };
    Some(n)
}

fn is_federal(office: &str) -> bool {
    let u = office.to_ascii_uppercase();
    u.contains("CONGRESS") || u.contains("U.S. SENATE") || u.contains("PRESIDENT")
}

fn is_party_committee(office: &str) -> bool {
    let u = office.to_ascii_uppercase();
    u.contains("CENTRAL COMMITTEE")
}

fn is_judicial(office: &str) -> bool {
    let u = office.to_ascii_uppercase();
    u.contains("JUDGE") || u.contains("JUSTICE")
}

fn house_label_matches(geo: &GeoResolution, code: &str) -> bool {
    let code_u = code.to_ascii_uppercase();
    if let Some(ref label) = geo.state_house_label {
        return label.eq_ignore_ascii_case(&code_u);
    }
    if code_u.chars().all(|c| c.is_ascii_digit()) {
        if let Some(hd) = geo.state_house_district {
            return code_u.parse::<u32>().ok() == Some(hd);
        }
    }
    false
}

fn contest_is_county_name(contest: &str, county: &str) -> bool {
    let c = county_key(contest);
    let g = county_key(county);
    !c.is_empty() && c == g
}

fn local_county_match(f: &MdFiling, geo_county: &str) -> bool {
    if contest_is_county_name(&f.contest, geo_county) {
        return true;
    }
    // Subdistrict locals (Councilmanic / Board of Education District N): use residence county.
    if !f.residential.is_empty() {
        return county_key(&f.residential) == county_key(geo_county);
    }
    false
}

/// Map MSBE filings for this ZIP geo. Skips federal (FEC). Includes leg by district +
/// county locals (not precinct-filtered). Prefer general-election list when available.
pub fn map_filings_for_geo(
    filings: &[MdFiling],
    geo: &GeoResolution,
    cycle: i32,
    source_url: &str,
) -> Vec<SnapshotCandidate> {
    let county = county_key(&geo.county);
    let state_l = "md";
    let state_ocd = format!("ocd-division/country:us/state:{state_l}");
    let mut seen = HashSet::new();
    let mut out = Vec::new();

    for f in filings {
        if !is_active(&f.status) {
            continue;
        }
        if is_federal(&f.office) || is_party_committee(&f.office) {
            continue;
        }
        let name = display_name(f);
        if name.is_empty() {
            continue;
        }
        let dedupe = format!(
            "{}|{}|{}|{}",
            f.office.to_ascii_uppercase(),
            f.contest.to_ascii_uppercase(),
            name.to_ascii_uppercase(),
            f.party.to_ascii_uppercase()
        );
        if !seen.insert(dedupe) {
            continue;
        }
        let party = party_label(&f.party);
        let office_l = f.office.to_ascii_lowercase();

        // Statewide executive / at-large appellate court
        if office_l.contains("governor")
            || office_l.contains("comptroller")
            || office_l.contains("attorney general")
            || (office_l.contains("appellate") && office_l.contains("at large"))
        {
            out.push(SnapshotCandidate {
                office: f.office.clone(),
                chamber: Some(if is_judicial(&f.office) {
                    "judicial".into()
                } else {
                    "statewide".into()
                }),
                jurisdiction_ocd: state_ocd.clone(),
                is_judicial: is_judicial(&f.office),
                name,
                party,
                is_incumbent: false,
                is_judge: is_judicial(&f.office),
                summary: Some("Maryland SBE candidate filing.".into()),
                source_url: source_url.into(),
                source_publisher: Some(SOURCE_PUBLISHER.into()),
                external_id: None,
            });
            continue;
        }

        // Supreme Court of Maryland — one justice per appellate circuit (retention).
        if office_l.contains("supreme court") {
            let Some(cap) = appellate_circuit_re().captures(f.contest.trim()) else {
                // No circuit on contest → treat as statewide (rare).
                out.push(SnapshotCandidate {
                    office: f.office.clone(),
                    chamber: Some("judicial".into()),
                    jurisdiction_ocd: state_ocd.clone(),
                    is_judicial: true,
                    name,
                    party,
                    is_incumbent: false,
                    is_judge: true,
                    summary: Some("Maryland Supreme Court filing.".into()),
                    source_url: source_url.into(),
                    source_publisher: Some(SOURCE_PUBLISHER.into()),
                    external_id: None,
                });
                continue;
            };
            let circ: u32 = cap[1].parse().unwrap_or(0);
            let Some(want) = appellate_circuit_for_county(&geo.county) else {
                continue;
            };
            if circ != want {
                continue;
            }
            out.push(SnapshotCandidate {
                office: format!("Justice, Supreme Court of Maryland (Appellate Circuit {circ})"),
                chamber: Some("judicial".into()),
                jurisdiction_ocd: state_ocd.clone(),
                is_judicial: true,
                name,
                party,
                is_incumbent: false,
                is_judge: true,
                summary: Some(
                    "MD Supreme Court retention (appellate circuit; on ballot in circuit counties)."
                        .into(),
                ),
                source_url: source_url.into(),
                source_publisher: Some(SOURCE_PUBLISHER.into()),
                external_id: None,
            });
            continue;
        }

        // Appellate Court of Maryland seats tied to an appellate circuit (not at-large).
        if office_l.contains("appellate court") && !office_l.contains("at large") {
            let Some(cap) = appellate_circuit_re().captures(f.contest.trim()) else {
                continue;
            };
            let circ: u32 = cap[1].parse().unwrap_or(0);
            let Some(want) = appellate_circuit_for_county(&geo.county) else {
                continue;
            };
            if circ != want {
                continue;
            }
            out.push(SnapshotCandidate {
                office: format!("Judge, Appellate Court of Maryland (Appellate Circuit {circ})"),
                chamber: Some("judicial".into()),
                jurisdiction_ocd: state_ocd.clone(),
                is_judicial: true,
                name,
                party,
                is_incumbent: false,
                is_judge: true,
                summary: Some(
                    "MD Appellate Court (appellate circuit; on ballot in circuit counties)."
                        .into(),
                ),
                source_url: source_url.into(),
                source_publisher: Some(SOURCE_PUBLISHER.into()),
                external_id: None,
            });
            continue;
        }

        if office_l.contains("state senator") {
            let Some(cap) = leg_district_re().captures(f.contest.trim()) else {
                continue;
            };
            let code = &cap[1];
            let d: u32 = code
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .unwrap_or(0);
            if geo.state_senate_district != Some(d) {
                continue;
            }
            out.push(SnapshotCandidate {
                office: format!("Maryland Senate (District {d})"),
                chamber: Some("state_senate".into()),
                jurisdiction_ocd: format!("ocd-division/country:us/state:{state_l}/sldu:{d}"),
                is_judicial: false,
                name,
                party,
                is_incumbent: false,
                is_judge: false,
                summary: Some("Maryland SBE candidate filing.".into()),
                source_url: source_url.into(),
                source_publisher: Some(SOURCE_PUBLISHER.into()),
                external_id: None,
            });
            continue;
        }

        if office_l.contains("house of delegates") {
            let Some(cap) = leg_district_re().captures(f.contest.trim()) else {
                continue;
            };
            let code = cap[1].to_ascii_uppercase();
            if !house_label_matches(geo, &code) {
                continue;
            }
            let ocd = format!(
                "ocd-division/country:us/state:{state_l}/sldl:{}",
                code.to_ascii_lowercase()
            );
            out.push(SnapshotCandidate {
                office: format!("Maryland House of Delegates (District {code})"),
                chamber: Some("state_house".into()),
                jurisdiction_ocd: ocd,
                is_judicial: false,
                name,
                party,
                is_incumbent: false,
                is_judge: false,
                summary: Some("Maryland SBE candidate filing (subdistrict when lettered).".into()),
                source_url: source_url.into(),
                source_publisher: Some(SOURCE_PUBLISHER.into()),
                external_id: None,
            });
            continue;
        }

        // Circuit Court judges appear on every county ballot in the circuit.
        if office_l.contains("judge of the circuit court")
            || (office_l.contains("circuit court") && !office_l.contains("clerk"))
        {
            let Some(cap) = circuit_re().captures(f.contest.trim()) else {
                continue;
            };
            let circ: u32 = cap[1].parse().unwrap_or(0);
            let Some(want) = circuit_for_county(&geo.county) else {
                continue;
            };
            if circ != want {
                continue;
            }
            out.push(SnapshotCandidate {
                office: format!("Judge of the Circuit Court (Circuit {circ})"),
                chamber: Some("judicial".into()),
                jurisdiction_ocd: format!(
                    "{state_ocd}/county:{}",
                    county.to_ascii_lowercase()
                ),
                is_judicial: true,
                name,
                party,
                is_incumbent: false,
                is_judge: true,
                summary: Some(
                    "MD Circuit Court judge (circuit multi-county; shown for this county)."
                        .into(),
                ),
                source_url: source_url.into(),
                source_publisher: Some(SOURCE_PUBLISHER.into()),
                external_id: None,
            });
            continue;
        }

        // County / local (from all-counties list)
        let localish = office_l.contains("county")
            || office_l.contains("sheriff")
            || office_l.contains("register of wills")
            || office_l.contains("clerk of the circuit")
            || office_l.contains("state's attorney")
            || office_l.contains("board of education")
            || office_l.contains("orphans")
            || office_l.contains("mayor")
            || office_l.contains("council")
            || office_l.contains("commissioner")
            || office_l.contains("treasurer");
        if !localish {
            continue;
        }
        if county.is_empty() || !local_county_match(f, &geo.county) {
            continue;
        }

        let judicial = is_judicial(&f.office);
        let chamber = if judicial {
            "judicial"
        } else if office_l.contains("mayor")
            || office_l.contains("city of")
            || office_l.contains("town")
        {
            "municipal"
        } else {
            "county"
        };
        let office_label = if f.contest.is_empty()
            || contest_is_county_name(&f.contest, &geo.county)
        {
            f.office.clone()
        } else {
            format!("{} — {}", f.office, f.contest)
        };
        out.push(SnapshotCandidate {
            office: office_label,
            chamber: Some(chamber.into()),
            jurisdiction_ocd: format!(
                "{state_ocd}/county:{}",
                county.to_ascii_lowercase()
            ),
            is_judicial: judicial,
            name,
            party,
            is_incumbent: false,
            is_judge: judicial,
            summary: Some(if judicial {
                "MD judicial filing (county-scoped; not precinct-filtered).".into()
            } else {
                "MD local filing (county-scoped; not precinct-filtered).".into()
            }),
            source_url: source_url.into(),
            source_publisher: Some(SOURCE_PUBLISHER.into()),
            external_id: None,
        });
        let _ = cycle; // URL chosen by caller
    }

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
        Some("statewide") => 0,
        Some("state_senate") => 1,
        Some("state_house") => 2,
        Some("judicial") => 3,
        Some("county") => 4,
        Some("municipal") => 5,
        _ => 9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ResolvedJurisdiction;

    fn annapolis_geo() -> GeoResolution {
        GeoResolution {
            state: "MD".into(),
            state_name: "Maryland".into(),
            county: "Anne Arundel County".into(),
            city: "Annapolis".into(),
            congressional_district: "MD-3".into(),
            state_senate_district: Some(30),
            state_house_district: Some(30),
            state_house_label: Some("30A".into()),
            latitude: Some(38.978),
            longitude: Some(-76.492),
            jurisdictions: vec![ResolvedJurisdiction {
                ocd_id: "ocd-division/country:us/state:md".into(),
                name: "Maryland".into(),
                level: "state".into(),
                state: Some("MD".into()),
            }],
            source_url: "test".into(),
            source_publisher: "test".into(),
        }
    }

    const SAMPLE_STATE: &str = r#"Office Name,Contest Run By District Name and Number,Candidate Ballot Last Name and Suffix,Candidate First Name and Middle Name,Additional Information,Office Political Party,Candidate Residential Jurisdiction,Candidate Gender,Candidate Status,Filing Type and Date
Governor / Lt. Governor,State Of Maryland,"Moore",Wes,,Democratic,Anne Arundel County,,Active,Regular - 02/22/2026
Governor / Lt. Governor,State Of Maryland,"Cox",Dan,,Republican,Frederick County,,Active,Regular - 01/30/2026
Governor / Lt. Governor,State Of Maryland,"Jaffe",Ralph,,Democratic,Baltimore County,,Deceased - 02/26/2026,Regular - 07/30/2025
Comptroller,State Of Maryland,"Lierman",Brooke Elizabeth,,Democratic,Baltimore City,,Active,Regular - 09/16/2025
State Senator,Legislative District 30,"Elfreth",Sarah,,Democratic,Anne Arundel County,,Active,Regular - 02/01/2026
State Senator,Legislative District 31,"Other",Sam,,Democratic,Anne Arundel County,,Active,Regular - 02/01/2026
House of Delegates,Legislative District 30A,"Alpha",Ann,,Democratic,Anne Arundel County,,Active,Regular - 02/01/2026
House of Delegates,Legislative District 30B,"Beta",Bob,,Democratic,Anne Arundel County,,Active,Regular - 02/01/2026
House of Delegates,Legislative District 30A,"Gamma",Gina,,Republican,Anne Arundel County,,Active,Regular - 02/01/2026
Representative in Congress,Congressional District 3,"Fed",Frank,,Democratic,Anne Arundel County,,Active,Federal - 01/01/2026
Judge of the Circuit Court,Judicial Circuit 5,"Adams",Pat,,Democratic,Anne Arundel County,,Active,Regular - 02/01/2026
Judge of the Circuit Court,Judicial Circuit 5,"Frazier",Kim,,Democratic,Howard County,,Active,Regular - 02/01/2026
Judge of the Circuit Court,Judicial Circuit 3,"Nope",Ned,,Democratic,Baltimore County,,Active,Regular - 02/01/2026
Justice Supreme Court of Maryland,Appellate Circuit 5,"Biran",Jonathan,,Non-Partisan,Anne Arundel County,,Active,Regular - 02/01/2026
Justice Supreme Court of Maryland,Appellate Circuit 4,"Killough",Peter,,Non-Partisan,Prince George's County,,Active,Regular - 02/01/2026
Judge Appellate Court of Maryland At Large,State Of Maryland,"Friedman",Dan,,Non-Partisan,Baltimore County,,Active,Regular - 02/01/2026
"#;

    const SAMPLE_LOCAL: &str = r#"Office Name,Contest Run By District Name and Number,Candidate Ballot Last Name and Suffix,Candidate First Name and Middle Name,Additional Information,Office Political Party,Candidate Residential Jurisdiction,Candidate Gender,Candidate Status,Filing Type and Date
County Executive,Anne Arundel,"Pittman",Steuart,,Democratic,Anne Arundel County,,Active,Regular - 02/01/2026
County Council,Councilmanic District 5,"Ogilvie",Amanda,,Democratic,Anne Arundel County,,Active,Regular - 02/01/2026
County Council,Councilmanic District 5,"Other",X,,Democratic,Baltimore County,,Active,Regular - 02/01/2026
Sheriff,Anne Arundel,"Davis",Jim,,Republican,Anne Arundel County,,Active,Regular - 02/01/2026
Sheriff,Baltimore County,"Nope",Ned,,Republican,Baltimore County,,Active,Regular - 02/01/2026
Democratic Central Committee,Anne Arundel,"Skip",Me,,Democratic,Anne Arundel County,,Active,Regular - 02/01/2026
"#;

    #[test]
    fn parses_and_maps_annapolis() {
        let mut filings = parse_candidate_csv(SAMPLE_STATE);
        filings.extend(parse_candidate_csv(SAMPLE_LOCAL));
        assert!(filings.len() >= 10);
        let cands = map_filings_for_geo(&filings, &annapolis_geo(), 2026, "test://md");
        assert!(cands.iter().any(|c| c.name.contains("Moore")));
        assert!(cands.iter().any(|c| c.name.contains("Cox")));
        assert!(!cands.iter().any(|c| c.name.contains("Jaffe"))); // deceased
        assert!(cands.iter().any(|c| c.name.contains("Lierman")));
        assert!(cands.iter().any(|c| c.name.contains("Elfreth")));
        assert!(!cands.iter().any(|c| c.name.contains("Other") && c.office.contains("Senate")));
        assert!(cands.iter().any(|c| c.name.contains("Alpha")));
        assert!(cands.iter().any(|c| c.name.contains("Gamma")));
        assert!(!cands.iter().any(|c| c.name.contains("Beta"))); // 30B
        assert!(!cands.iter().any(|c| c.name.contains("Fed"))); // federal
        assert!(cands.iter().any(|c| c.name.contains("Pittman")));
        assert!(cands.iter().any(|c| c.name.contains("Ogilvie")));
        assert!(!cands.iter().any(|c| c.name == "X Other" || c.name.contains("Other X")));
        assert!(cands.iter().any(|c| c.name.contains("Davis")));
        assert!(!cands.iter().any(|c| c.name.contains("Nope")));
        assert!(!cands.iter().any(|c| c.name.contains("Skip")));
        // Circuit 5 (Anne Arundel + Howard + Carroll) — both AA and Howard residences
        assert!(cands.iter().any(|c| c.is_judge && c.name.contains("Adams")));
        assert!(cands.iter().any(|c| c.is_judge && c.name.contains("Frazier")));
        assert!(!cands.iter().any(|c| c.name.contains("Nope"))); // circuit 3
        // Appellate circuit 5 (Anne Arundel) — not circuit 4 Killough
        assert!(cands.iter().any(|c| c.name.contains("Biran")));
        assert!(!cands.iter().any(|c| c.name.contains("Killough")));
        // At-large appellate everywhere
        assert!(cands.iter().any(|c| c.name.contains("Friedman")));
    }

    #[test]
    fn circuit_map_covers_counties() {
        assert_eq!(circuit_for_county("Anne Arundel County"), Some(5));
        assert_eq!(circuit_for_county("Baltimore City"), Some(8));
        assert_eq!(circuit_for_county("Baltimore County"), Some(3));
        assert_eq!(circuit_for_county("Prince George's County"), Some(7));
        assert_eq!(circuit_for_county("Queen Anne's County"), Some(2));
        assert_eq!(circuit_for_county("Saint Mary's County"), Some(7));
        assert_ne!(county_key("Baltimore City"), county_key("Baltimore County"));
        assert_eq!(appellate_circuit_for_county("Prince George's County"), Some(4));
        assert_eq!(appellate_circuit_for_county("Anne Arundel County"), Some(5));
        assert_eq!(appellate_circuit_for_county("Montgomery County"), Some(7));
        assert_eq!(appellate_circuit_for_county("Baltimore City"), Some(6));
    }

    #[test]
    fn baltimore_city_locals_not_county() {
        let csv = r#"Office Name,Contest Run By District Name and Number,Candidate Ballot Last Name and Suffix,Candidate First Name and Middle Name,Additional Information,Office Political Party,Candidate Residential Jurisdiction,Candidate Gender,Candidate Status,Filing Type and Date
Sheriff,Baltimore City,"City",Sam,,Democratic,Baltimore City,,Active,Regular - 02/01/2026
Sheriff,Baltimore County,"County",Pat,,Democratic,Baltimore County,,Active,Regular - 02/01/2026
"#;
        let filings = parse_candidate_csv(csv);
        let mut city_geo = annapolis_geo();
        city_geo.county = "Baltimore City".into();
        city_geo.state_senate_district = None;
        city_geo.state_house_district = None;
        city_geo.state_house_label = None;
        let city = map_filings_for_geo(&filings, &city_geo, 2026, "t");
        assert!(city.iter().any(|c| c.name.contains("City")));
        assert!(!city.iter().any(|c| c.name.contains("County")));

        let mut co_geo = city_geo.clone();
        co_geo.county = "Baltimore County".into();
        let co = map_filings_for_geo(&filings, &co_geo, 2026, "t");
        assert!(co.iter().any(|c| c.name.contains("County")));
        assert!(!co.iter().any(|c| c.name.contains("City")));
    }

    #[test]
    fn urls() {
        assert!(statewide_csv_url(2026, true).contains("GG_statewide"));
        assert!(local_csv_url(2026, false).contains("GP_all_counties"));
        assert!(ballot_questions_url(2026).ends_with("/2026/ballot_questions.html"));
    }

    #[test]
    fn parse_md_ballot_questions_filters_county() {
        let html = include_str!("../../../../testdata/md_ballot_questions_sample.html");
        let all = parse_md_ballot_questions_html(html, None);
        assert!(all.len() >= 6, "got {}", all.len());
        assert!(all.iter().any(|m| m.is_statewide));
        assert!(all.iter().any(|m| m.jurisdiction.contains("Anne Arundel")));
        assert!(all.iter().any(|m| m.jurisdiction.contains("Baltimore County")));
        assert!(all.iter().any(|m| m.jurisdiction.contains("Baltimore City")));

        let aa = parse_md_ballot_questions_html(html, Some("Anne Arundel County"));
        assert!(aa.iter().all(|m| m.is_statewide || m.jurisdiction.contains("Anne Arundel")));
        assert!(aa.iter().any(|m| m.is_statewide));
        assert!(aa.iter().any(|m| !m.is_statewide));
        assert!(!aa.iter().any(|m| m.jurisdiction.contains("Baltimore")));

        let city = parse_md_ballot_questions_html(html, Some("Baltimore City"));
        assert!(city.iter().any(|m| m.jurisdiction.contains("Baltimore City")));
        assert!(!city.iter().any(|m| m.jurisdiction.contains("Baltimore County")));

        let q1 = aa
            .iter()
            .find(|m| m.code.as_deref() == Some("Question 1"))
            .expect("Q1");
        assert!(q1.title.to_ascii_lowercase().contains("arbitration"));
        assert!(q1.summary.as_ref().map(|s| s.len() > 40).unwrap_or(false));

        let (ms, extras) = map_md_measures_for_geo(
            &aa,
            "ocd-division/country:us/state:md",
            "Anne Arundel County",
            &ballot_questions_url(2026),
        );
        assert_eq!(ms.len(), aa.len());
        assert!(ms.iter().all(|m| {
            m.source_publisher.as_deref() == Some(SOURCE_PUBLISHER)
                && m.source_url.contains("ballot_questions")
        }));
        assert!(!extras.is_empty());
        assert!(ms.iter().any(|m| m.measure_code.as_deref() == Some("Question A")));
    }

    #[test]
    fn md_cf_parse_match_finance() {
        let body = include_str!("../../../../testdata/md_cf_committee_ferguson.json");
        let hits = parse_md_cf_committee_list_json(body).expect("parse");
        assert_eq!(hits.len(), 3);
        assert_eq!(hits[0].filing_entity_id, 1005740);
        assert!(hits[0].candidate_name.to_ascii_uppercase().contains("FERGUSON"));

        let q = MdCfMatchQuery {
            name: "William C. Ferguson".into(),
            office: "Maryland Senate (District 46)".into(),
            chamber: "state_senate".into(),
            party: "Democratic".into(),
            district: Some(46),
            county: String::new(),
        };
        match match_md_cf_candidate(&hits, &q) {
            MdCfMatch::Unique { hit } => {
                assert_eq!(hit.filing_entity_id, 1005740);
                let fin = md_cf_finance_from_hit(&hit, 2026, None);
                assert_eq!(fin.source, "md_cf");
                assert!(fin.receipts_display.contains("2,193"));
                assert!(fin.profile_url.contains("filerRegistrationGuid="));
                assert!(fin.profile_url.contains("campaignfinance.maryland.gov"));
            }
            other => panic!("expected unique {other:?}"),
        }

        let sum_body = include_str!("../../../../testdata/md_cf_summary_ferguson.json");
        let sum = parse_md_cf_financial_summary_json(sum_body).expect("sum");
        assert!(sum.end_balance.unwrap_or(0.0) > 0.0);
        match match_md_cf_candidate(&hits, &q) {
            MdCfMatch::Unique { hit } => {
                let fin = md_cf_finance_from_hit(&hit, 2026, Some(&sum));
                assert!(fin.cash_on_hand_display.contains("905"));
            }
            other => panic!("expected unique with summary {other:?}"),
        }

        // Orphans court judge — different office family
        let q_judge = MdCfMatchQuery {
            name: "Elizabeth Ferguson".into(),
            office: "Judge of the Orphans Court".into(),
            chamber: "judicial".into(),
            party: "Republican".into(),
            district: None,
            county: "Talbot".into(),
        };
        match match_md_cf_candidate(&hits, &q_judge) {
            MdCfMatch::Unique { hit } => {
                assert!(hit.office_sought.to_ascii_lowercase().contains("orphan"));
            }
            other => panic!("expected judge unique {other:?}"),
        }

        assert!(matches!(
            match_md_cf_candidate(
                &hits,
                &MdCfMatchQuery {
                    name: "Nobody".into(),
                    office: String::new(),
                    chamber: String::new(),
                    party: String::new(),
                    district: None,
                    county: String::new(),
                }
            ),
            MdCfMatch::None
        ));

        let empty = include_str!("../../../../testdata/md_cf_committee_empty.json");
        assert!(parse_md_cf_committee_list_json(empty)
            .expect("empty")
            .is_empty());

        let url = md_cf_committee_list_url();
        assert!(url.contains("GetCommitteeList"));
        let body = md_cf_committee_list_body("Ferguson", 25);
        assert!(body.contains("filerName"));
    }

    #[test]
    fn md_measure_finance_from_ballot_issue_fixture() {
        let body = include_str!("../../../../testdata/md_cf_ballot_issue_question.json");
        let hits = parse_md_cf_committee_list_json(body).expect("parse");
        assert!(hits.len() >= 5);
        assert!(hits.iter().any(is_md_ballot_issue_committee));
        assert_eq!(md_measure_committee_stance("Against Question A"), "oppose");
        assert_eq!(
            md_measure_committee_stance("Marylanders for Question 1"),
            "support"
        );
        let terms = md_measure_search_terms("Question A", "Local charter change", 2024);
        assert!(terms.iter().any(|t| t.contains("Question")));

        let fin = md_measure_finance_from_hits(
            &hits,
            "Question A",
            "Make Your Vote Count",
            2024,
        )
        .expect("finance");
        assert_eq!(fin.source, "md_cf_measure");
        assert!(!fin.oppose.is_empty() || fin.contributions_sum > 0.0);
        assert!(
            fin.committee_url.contains("campaignfinance.maryland.gov")
                || fin
                    .oppose
                    .iter()
                    .any(|o| o.committee_url.contains("campaignfinance"))
        );
    }
}

// --- MDCRIS campaign finance (A4) ---

pub const MD_CF_API_BASE: &str = "https://api-campaignfinance.maryland.gov/api";
pub const MD_CF_SITE_BASE: &str = "https://campaignfinance.maryland.gov";
pub const MD_CF_PUBLISHER: &str = "Maryland State Board of Elections (MDCRIS)";

pub fn md_cf_committee_list_url() -> String {
    format!("{MD_CF_API_BASE}/PublicGrid/GetCommitteeList")
}

pub fn md_cf_financial_summary_url() -> String {
    format!("{MD_CF_API_BASE}/PublicFilerDetails/GetFinancialSummaryDetails")
}

/// JSON body for committee search (`filerName` is the working name filter).
pub fn md_cf_committee_list_body(filer_name: &str, page_size: u32) -> String {
    let size = page_size.clamp(1, 100);
    serde_json::json!({
        "pageNumber": 1,
        "pageSize": size,
        "filerName": filer_name.trim(),
    })
    .to_string()
}

pub fn md_cf_financial_summary_body(filer_registration_guid: &str) -> String {
    serde_json::json!({
        "filerRegistrationGuid": filer_registration_guid.trim(),
    })
    .to_string()
}

pub fn md_cf_profile_url(filer_registration_guid: &str) -> String {
    format!(
        "{MD_CF_SITE_BASE}/public/cf/candidateprofile?filerRegistrationGuid={}",
        filer_registration_guid.trim()
    )
}

pub fn md_cf_search_name_fragment(ballot_name: &str) -> String {
    let (_f, last) = split_candidate_first_last(ballot_name);
    if !last.is_empty() {
        last
    } else {
        ballot_name.trim().to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdCfHit {
    pub filer_registration_guid: String,
    pub filing_entity_id: u32,
    pub committee_name: String,
    pub candidate_name: String,
    pub office_sought: String,
    pub jurisdiction: String,
    pub party_affiliation: String,
    pub account_status: String,
    pub committee_type: String,
    pub total_contributions: Option<f64>,
    pub total_expenditures: Option<f64>,
    pub election_years: String,
}

#[derive(Debug, Clone)]
pub struct MdCfMatchQuery {
    pub name: String,
    pub office: String,
    pub chamber: String,
    pub party: String,
    pub district: Option<u32>,
    pub county: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind")]
pub enum MdCfMatch {
    #[serde(rename = "unique")]
    Unique { hit: MdCfHit },
    #[serde(rename = "none")]
    None,
    #[serde(rename = "ambiguous")]
    Ambiguous { count: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdCfFinancialSummary {
    pub filer_registration_guid: String,
    pub entity_id: Option<u32>,
    pub total_contributions_cumulative: Option<f64>,
    pub total_expenditures_cumulative: Option<f64>,
    pub end_balance: Option<f64>,
    pub election_cycle_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MdCfFinance {
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
    pub note: String,
}

fn json_f64(v: &Value) -> Option<f64> {
    v.as_f64()
        .or_else(|| v.as_i64().map(|n| n as f64))
        .or_else(|| v.as_u64().map(|n| n as f64))
        .or_else(|| v.as_str().and_then(|s| s.replace(',', "").parse().ok()))
}

fn json_u32(v: &Value) -> Option<u32> {
    v.as_u64()
        .map(|n| n as u32)
        .or_else(|| v.as_i64().map(|n| n as u32))
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
}

fn row_str(row: &Value, keys: &[&str]) -> String {
    for k in keys {
        if let Some(s) = row.get(*k).and_then(|v| v.as_str()) {
            let t = s.trim();
            if !t.is_empty() {
                return t.to_string();
            }
        }
    }
    String::new()
}

/// Display name from MDCRIS `LAST, FIRST M.` candidate field (or committee fallback).
pub fn md_cf_row_display_name(candidate_name: &str, committee: &str) -> String {
    let t = candidate_name.trim();
    if t.contains(',') {
        let mut parts = t.splitn(2, ',');
        let last = parts.next().unwrap_or("").trim();
        let first = parts.next().unwrap_or("").trim();
        if !first.is_empty() && !last.is_empty() {
            return format!("{first} {last}");
        }
    }
    if !t.is_empty() {
        return t.to_string();
    }
    committee.trim().to_string()
}

/// Parse `PublicGrid/GetCommitteeList` JSON into hits.
pub fn parse_md_cf_committee_list_json(body: &str) -> Result<Vec<MdCfHit>, String> {
    let t = body.trim();
    if t.is_empty() || t == "[]" || t == "null" {
        return Ok(vec![]);
    }
    let v: Value = serde_json::from_str(t).map_err(|e| format!("MD CF committee JSON: {e}"))?;
    let rows = v
        .pointer("/data/items")
        .and_then(|d| d.as_array())
        .cloned()
        .or_else(|| v.get("items").and_then(|d| d.as_array()).cloned())
        .or_else(|| v.as_array().cloned())
        .unwrap_or_default();
    let mut out = Vec::new();
    for row in rows {
        let guid = row_str(&row, &["filerRegistrationGuid", "FilerRegistrationGuid"]);
        let entity_id = row
            .get("filingEntityId")
            .or_else(|| row.get("FilingEntityId"))
            .and_then(json_u32)
            .unwrap_or(0);
        let committee = row_str(&row, &["committeeName", "CommitteeName"]);
        let cand_raw = row_str(&row, &["candidateName", "CandidateName"]);
        let candidate_name = md_cf_row_display_name(&cand_raw, &committee);
        if guid.is_empty() && entity_id == 0 && candidate_name.is_empty() {
            continue;
        }
        out.push(MdCfHit {
            filer_registration_guid: guid,
            filing_entity_id: entity_id,
            committee_name: committee,
            candidate_name,
            office_sought: row_str(&row, &["officeSought", "OfficeSought", "ballotOfficeSought"]),
            jurisdiction: row_str(&row, &["jurisdiction", "Jurisdiction", "ballotJurisdiction"]),
            party_affiliation: row_str(&row, &["partyAffiliation", "PartyAffiliation"]),
            account_status: row_str(&row, &["accountStatus", "AccountStatus"]),
            committee_type: row_str(&row, &["committeeType", "CommitteeType"]),
            total_contributions: row
                .get("totalContributions")
                .or_else(|| row.get("TotalContributions"))
                .and_then(json_f64),
            total_expenditures: row
                .get("totalExpenditures")
                .or_else(|| row.get("TotalExpenditures"))
                .and_then(json_f64),
            election_years: row_str(&row, &["electionYears", "ElectionYears"]),
        });
    }
    Ok(out)
}

/// Parse `PublicFilerDetails/GetFinancialSummaryDetails` JSON.
pub fn parse_md_cf_financial_summary_json(body: &str) -> Result<MdCfFinancialSummary, String> {
    let t = body.trim();
    if t.is_empty() {
        return Err("empty MD CF summary".into());
    }
    let v: Value = serde_json::from_str(t).map_err(|e| format!("MD CF summary JSON: {e}"))?;
    let data = v.get("data").cloned().unwrap_or(v);
    if data.is_null() {
        return Err("MD CF summary missing data".into());
    }
    Ok(MdCfFinancialSummary {
        filer_registration_guid: row_str(&data, &["filerRegistrationGuid"]),
        entity_id: data.get("entityId").and_then(json_u32),
        total_contributions_cumulative: data
            .get("totalContributionsCumulative")
            .and_then(json_f64),
        total_expenditures_cumulative: data
            .get("totalExpendituresCumulative")
            .and_then(json_f64),
        end_balance: data.get("endBalance").and_then(json_f64),
        election_cycle_id: data.get("electionCycleId").and_then(json_u32),
    })
}

fn parties_conflict(a: &str, b: &str) -> bool {
    let norm = |p: &str| -> Option<&'static str> {
        let u = p.trim().to_ascii_uppercase();
        if u.is_empty() || u == "OTHER" || u == "UNAFFILIATED" || u == "N/A" {
            return None;
        }
        if u.starts_with('D') || u.contains("DEM") {
            return Some("DEM");
        }
        if u.starts_with('R') || u.contains("REP") {
            return Some("REP");
        }
        None
    };
    match (norm(a), norm(b)) {
        (Some(x), Some(y)) => x != y,
        _ => false,
    }
}

fn office_compatible(q: &MdCfMatchQuery, hit: &MdCfHit) -> bool {
    let ho = hit.office_sought.to_ascii_lowercase();
    let qo = q.office.to_ascii_lowercase();
    let ch = q.chamber.to_ascii_lowercase();
    if ho.is_empty() {
        // PAC / exploratory without office — only accept if ballot also lacks office family cues
        return ch.is_empty()
            && !qo.contains("senate")
            && !qo.contains("house")
            && !qo.contains("delegate")
            && !qo.contains("judge");
    }
    if ch == "state_senate" || qo.contains("senate") {
        return ho.contains("senator") || (ho.contains("senate") && !ho.contains("house"));
    }
    if ch == "state_house" || qo.contains("house") || qo.contains("delegate") {
        return ho.contains("delegate") || (ho.contains("house") && !ho.contains("senate"));
    }
    if ch == "judicial" || qo.contains("judge") || qo.contains("court") {
        return ho.contains("judge") || ho.contains("court") || ho.contains("judicial");
    }
    if ch == "statewide" || qo.contains("governor") || qo.contains("attorney general") {
        return ho.contains("governor")
            || ho.contains("attorney general")
            || ho.contains("comptroller")
            || (!ho.contains("district") && !ho.contains("county"));
    }
    // Local / other: soft office token overlap when both non-empty
    if !qo.is_empty() {
        let tokens = ["sheriff", "mayor", "council", "commissioner", "clerk", "attorney", "board"];
        for t in tokens {
            if qo.contains(t) {
                return ho.contains(t);
            }
        }
    }
    true
}

fn county_compatible(q: &MdCfMatchQuery, hit: &MdCfHit) -> bool {
    let qc = q.county.trim().to_ascii_lowercase();
    if qc.is_empty() {
        return true;
    }
    let hj = hit.jurisdiction.trim().to_ascii_lowercase();
    if hj.is_empty() || hj == "maryland state" {
        return true;
    }
    let qc_key = qc
        .replace(" county", "")
        .replace(" city", "")
        .replace('\'', "");
    let hj_key = hj.replace(" county", "").replace(" city", "").replace('\'', "");
    hj_key.contains(&qc_key) || qc_key.contains(&hj_key)
}

fn is_active_status(s: &str) -> bool {
    let u = s.trim().to_ascii_lowercase();
    u.is_empty() || u == "active" || u.starts_with("act")
}

pub fn match_md_cf_candidate(hits: &[MdCfHit], q: &MdCfMatchQuery) -> MdCfMatch {
    let matched: Vec<&MdCfHit> = hits
        .iter()
        .filter(|h| {
            last_names_match(&q.name, &h.candidate_name)
                || last_names_match(&q.name, &h.committee_name)
        })
        .filter(|h| {
            first_names_compatible(&q.name, &h.candidate_name)
                || h.candidate_name.split_whitespace().count() <= 1
        })
        .filter(|h| office_compatible(q, h))
        .filter(|h| county_compatible(q, h))
        .filter(|h| !parties_conflict(&q.party, &h.party_affiliation))
        .collect();

    // Prefer Active accounts when multiple name+office hits (terminated twins).
    let active: Vec<&MdCfHit> = matched
        .iter()
        .copied()
        .filter(|h| is_active_status(&h.account_status))
        .collect();
    let pool = if active.len() == 1 {
        active
    } else if active.len() > 1 {
        active
    } else {
        matched
    };

    // District is not published on MDCRIS committee rows — if still >1 after filters, ambiguous.
    let _ = q.district.or_else(|| district_from_ballot_office(&q.office));

    match pool.len() {
        0 => MdCfMatch::None,
        1 => MdCfMatch::Unique {
            hit: pool[0].clone(),
        },
        n => MdCfMatch::Ambiguous { count: n },
    }
}

pub fn md_cf_finance_from_hit(
    hit: &MdCfHit,
    cycle: i32,
    summary: Option<&MdCfFinancialSummary>,
) -> MdCfFinance {
    let receipts = summary
        .and_then(|s| s.total_contributions_cumulative)
        .or(hit.total_contributions);
    let disbursements = summary
        .and_then(|s| s.total_expenditures_cumulative)
        .or(hit.total_expenditures);
    let cash = summary.and_then(|s| s.end_balance);
    let account = if !hit.filer_registration_guid.is_empty() {
        hit.filer_registration_guid.clone()
    } else {
        hit.filing_entity_id.to_string()
    };
    MdCfFinance {
        source: "md_cf".into(),
        cycle: cycle.to_string(),
        account,
        match_name: hit.candidate_name.clone(),
        match_office: if hit.office_sought.is_empty() {
            hit.committee_name.clone()
        } else {
            hit.office_sought.clone()
        },
        receipts_display: receipts.map(format_usd).unwrap_or_else(|| "—".into()),
        disbursements_display: disbursements.map(format_usd).unwrap_or_else(|| "—".into()),
        cash_on_hand_display: cash.map(format_usd).unwrap_or_else(|| "—".into()),
        source_label: MD_CF_PUBLISHER.into(),
        profile_url: if hit.filer_registration_guid.is_empty() {
            MD_CF_SITE_BASE.into()
        } else {
            md_cf_profile_url(&hit.filer_registration_guid)
        },
        note: "Totals from Maryland MDCRIS public committee / financial summary (election-cycle cumulative where available). Filed portal figures — not a bank audit.".into(),
    }
}

// --- Ballot-issue committee finance (live MDCRIS; cycle-current when filed) ---

/// Suggested `filerName` queries to harvest ballot-issue committees for a ballot.
pub fn md_measure_search_terms(measure_code: &str, title: &str, cycle: i32) -> Vec<String> {
    let mut out = Vec::new();
    let push = |out: &mut Vec<String>, s: &str| {
        let t = s.trim();
        if t.is_empty() {
            return;
        }
        if !out.iter().any(|x| x.eq_ignore_ascii_case(t)) {
            out.push(t.to_string());
        }
    };
    push(&mut out, "Question");
    if let Some(k) = measure_code_key(measure_code).or_else(|| measure_code_key(title)) {
        if let Some(rest) = k.strip_prefix("Q:") {
            push(&mut out, &format!("Question {rest}"));
            push(&mut out, &format!("Question {}", rest));
            // zero-pad common statewide form
            if let Ok(n) = rest.parse::<u32>() {
                push(&mut out, &format!("Question {n:03}"));
            }
        }
    }
    let code = measure_code.trim();
    if !code.is_empty() {
        push(&mut out, code);
    }
    // Title words (skip stopwords)
    for w in title.split_whitespace() {
        let clean = w.trim_matches(|c: char| !c.is_alphanumeric());
        if clean.len() >= 5
            && !["question", "amendment", "ballot", "county", "maryland"].contains(&clean.to_ascii_lowercase().as_str())
        {
            push(&mut out, clean);
            if out.len() >= 8 {
                break;
            }
        }
    }
    let _ = cycle;
    out
}

pub fn is_md_ballot_issue_committee(hit: &MdCfHit) -> bool {
    let t = hit.committee_type.to_ascii_lowercase();
    t.contains("ballot") || t.contains("issue") || t == "bic"
}

/// Stance from committee name: support / oppose / unknown.
pub fn md_measure_committee_stance(name: &str) -> &'static str {
    let u = name.to_ascii_lowercase();
    let oppose = [
        "against", "oppose", "no on", "vote no", "vote against", "defeat", "stop ",
    ];
    let support = [
        "for question", "yes on", "vote yes", "support", "pro-", " in favor",
    ];
    if oppose.iter().any(|p| u.contains(p)) {
        return "oppose";
    }
    if support.iter().any(|p| u.contains(p)) {
        return "support";
    }
    // "Marylanders for Question 1" style
    if u.contains(" for ") && u.contains("question") {
        return "support";
    }
    "unknown"
}

fn hit_year_score(hit: &MdCfHit, cycle: i32) -> i32 {
    let years = hit.election_years.as_str();
    if years.contains(&cycle.to_string()) {
        return 100;
    }
    // Prefer nearby cycles over ancient
    for delta in 0..=4 {
        let y = cycle - delta;
        if years.contains(&y.to_string()) {
            return 80 - delta * 10;
        }
    }
    if years.is_empty() {
        return 10;
    }
    0
}

fn committee_mentions_measure(hit: &MdCfHit, measure_code: &str, title: &str) -> bool {
    let name = hit.committee_name.to_ascii_lowercase();
    let keys = [
        measure_code_key(measure_code),
        measure_code_key(title),
        measure_code_key(&hit.committee_name),
    ];
    let mut measure_keys = Vec::new();
    for k in keys.into_iter().flatten() {
        if !measure_keys.contains(&k) {
            measure_keys.push(k);
        }
    }
    let hit_key = measure_code_key(&hit.committee_name);
    if let Some(ref hk) = hit_key {
        if measure_keys.iter().any(|k| k == hk) {
            return true;
        }
    }
    // Letter question: "Question A" in name
    if let Some(k) = measure_code_key(measure_code).or_else(|| measure_code_key(title)) {
        if let Some(rest) = k.strip_prefix("Q:") {
            let patterns = [
                format!("question {rest}"),
                format!("question {}", rest.to_ascii_lowercase()),
                format!(" q {rest}"),
                format!("q{rest}"),
            ];
            if patterns.iter().any(|p| name.contains(&p.to_ascii_lowercase())) {
                return true;
            }
        }
    }
    // Title word overlap (unique-ish)
    let title_l = title.to_ascii_lowercase();
    let title_n: Vec<&str> = title_l
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 3)
        .filter(|w| !["question", "amendment", "ballot", "county", "maryland", "right"].contains(w))
        .collect();
    if title_n.len() >= 2 {
        let hits = title_n.iter().filter(|w| name.contains(**w)).count();
        if hits >= 2 {
            return true;
        }
    }
    false
}

#[derive(Debug, Clone, Serialize)]
pub struct MdMeasureFinanceSide {
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

#[derive(Debug, Clone, Serialize)]
pub struct MdMeasureFinance {
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
    pub oppose: Vec<MdMeasureFinanceSide>,
    pub source_label: String,
}

fn hit_to_side(hit: &MdCfHit, role: &str) -> MdMeasureFinanceSide {
    let sum = hit.total_contributions.unwrap_or(0.0);
    let url = if hit.filer_registration_guid.is_empty() {
        MD_CF_SITE_BASE.to_string()
    } else {
        md_cf_profile_url(&hit.filer_registration_guid)
    };
    MdMeasureFinanceSide {
        account: hit.filer_registration_guid.clone(),
        contributions_sum: sum,
        contributions_sum_display: format_usd(sum),
        top_contributors: vec![],
        line_count: if sum > 0.0 { 1 } else { 0 },
        committee_url: url,
        trefin_url: String::new(),
        note: String::new(),
        committee_name: hit.committee_name.clone(),
        role: role.into(),
    }
}

/// Attach MDCRIS ballot-issue committee $ to one ballot measure.
pub fn md_measure_finance_from_hits(
    hits: &[MdCfHit],
    measure_code: &str,
    title: &str,
    cycle: i32,
) -> Option<MdMeasureFinance> {
    let mut matched: Vec<&MdCfHit> = hits
        .iter()
        .filter(|h| is_md_ballot_issue_committee(h) || md_measure_committee_stance(&h.committee_name) != "unknown")
        .filter(|h| committee_mentions_measure(h, measure_code, title))
        .collect();
    if matched.is_empty() {
        return None;
    }
    // Prefer cycle-aligned election years; drop ancient when any near-cycle exists.
    let best = matched
        .iter()
        .map(|h| hit_year_score(h, cycle))
        .max()
        .unwrap_or(0);
    if best >= 40 {
        matched.retain(|h| hit_year_score(h, cycle) >= 40);
    }
    if matched.is_empty() {
        return None;
    }

    let mut support: Vec<&MdCfHit> = Vec::new();
    let mut oppose: Vec<&MdCfHit> = Vec::new();
    let mut unknown: Vec<&MdCfHit> = Vec::new();
    for h in matched {
        match md_measure_committee_stance(&h.committee_name) {
            "oppose" => oppose.push(h),
            "support" => support.push(h),
            _ => unknown.push(h),
        }
    }
    // Unknown with money → treat as support only when no explicit support (avoid mislabel oppose).
    if support.is_empty() && !unknown.is_empty() {
        support = unknown;
    }

    support.sort_by(|a, b| {
        b.total_contributions
            .unwrap_or(0.0)
            .partial_cmp(&a.total_contributions.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    oppose.sort_by(|a, b| {
        b.total_contributions
            .unwrap_or(0.0)
            .partial_cmp(&a.total_contributions.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let support_sum: f64 = support.iter().map(|h| h.total_contributions.unwrap_or(0.0)).sum();
    let oppose_sides: Vec<MdMeasureFinanceSide> = oppose.iter().take(4).map(|h| hit_to_side(h, "oppose")).collect();
    let top: Vec<ContributorRow> = support
        .iter()
        .take(5)
        .map(|h| ContributorRow {
            name: h.committee_name.clone(),
            amount_display: format_usd(h.total_contributions.unwrap_or(0.0)),
            location: None,
            date: None,
            url: if h.filer_registration_guid.is_empty() {
                MD_CF_SITE_BASE.to_string()
            } else {
                md_cf_profile_url(&h.filer_registration_guid)
            },
            gift_count: None,
        })
        .collect();

    let primary = support.first().copied().or_else(|| oppose.first().copied())?;
    let line_count = if support_sum > 0.0 || !oppose_sides.is_empty() {
        top.len().max(1)
    } else {
        0
    };

    Some(MdMeasureFinance {
        source: "md_cf_measure".into(),
        account: primary.filer_registration_guid.clone(),
        contributions_sum: support_sum,
        contributions_sum_display: format_usd(support_sum),
        top_contributors: top,
        line_count,
        committee_url: if primary.filer_registration_guid.is_empty() {
            MD_CF_SITE_BASE.to_string()
        } else {
            md_cf_profile_url(&primary.filer_registration_guid)
        },
        trefin_url: String::new(),
        note: "MDCRIS ballot-issue committee totals (public list). Multiple committees may work the same question; not certified cash-on-hand.".into(),
        committee_name: primary.committee_name.clone(),
        role: "sponsor".into(),
        oppose: oppose_sides,
        source_label: format!("{MD_CF_PUBLISHER} · ballot-issue committees"),
    })
}
