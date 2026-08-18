//! Arizona pure mappers: roster + Clean Elections officials/measures + SeeTheMoney CF (no HTTP).
use crate::models::{format_usd, normalize_party_label, SnapshotCandidate, SnapshotMeasure};
use crate::states::florida::{
    district_from_ballot_office, first_names_compatible, last_names_match, split_candidate_first_last,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub const SENATE_URL: &str = "https://www.azleg.gov/MemberRoster/?body=S";
pub const HOUSE_URL: &str = "https://www.azleg.gov/MemberRoster/?body=H";
pub const MEASURES_INFO_URL: &str =
    "https://www.azcleanelections.gov/arizona-elections/propositions";
pub const MEASURES_ELECTIONS_URL: &str =
    "https://www.azcleanelections.gov/Custom/ElectionsForBM";
pub const MEASURES_LIST_BASE: &str =
    "https://www.azcleanelections.gov/Custom/BallotMeasures";
pub const MEASURES_COUNTIES_BASE: &str =
    "https://www.azcleanelections.gov/Custom/CountiesForBM";
pub const MEASURE_DETAIL_BASE: &str =
    "https://www.azcleanelections.gov/Custom/BallotMeasure/?id=";
/// Clean Elections “Find my Elected Officials” list fragment (incumbents by Packed location).
pub const OFFICIALS_LIST_BASE: &str =
    "https://www.azcleanelections.gov/Custom/OfficialList";
pub const OFFICIALS_PAGE_URL: &str =
    "https://www.azcleanelections.gov/elected-officials";
pub const AZ_PUBLISHER: &str = "Arizona Legislature";
pub const AZ_ELECTIONS_PUBLISHER: &str = "Arizona Citizens Clean Elections Commission";

/// Build Clean Elections `OfficialList` Packed location from geo (CD + LD + county).
/// County district / city slots are zeroed — still filters statewide + LD correctly.
pub fn az_officials_packed(cd: u32, leg_dist: u32, county: &str) -> Option<String> {
    if cd == 0 || leg_dist == 0 {
        return None;
    }
    let county_name = county
        .trim()
        .trim_end_matches(" County")
        .trim_end_matches(" county")
        .trim();
    if county_name.is_empty() {
        return None;
    }
    // Sample: AZ-3-11-Maricopa~County-0--
    Some(format!(
        "AZ-{cd}-{leg_dist}-{county_name}~County-0--"
    ))
}

pub fn az_officials_list_url(packed: &str) -> String {
    format!(
        "{OFFICIALS_LIST_BASE}?location={}",
        urlenc_path(packed)
    )
}

fn urlenc_path(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// One row from Clean Elections OfficialList HTML.
#[derive(Debug, Clone)]
pub struct ParsedAzOfficial {
    pub name: String,
    pub office: String,
    pub party: String,
    pub official_id: Option<u32>,
}

/// Parse Clean Elections `Custom/OfficialList` HTML fragment.
pub fn parse_az_officials_html(html: &str) -> Vec<ParsedAzOfficial> {
    let mut out = Vec::new();
    let li_re = match Regex::new(r"(?is)<li\b[^>]*>(.*?)</li>") {
        Ok(r) => r,
        Err(_) => return out,
    };
    let name_re = match Regex::new(r"(?is)<b\b[^>]*>(.*?)</b>") {
        Ok(r) => r,
        Err(_) => return out,
    };
    let office_re = match Regex::new(r#"(?is)<span\s+class=["']office["'][^>]*>(.*?)</span>"#) {
        Ok(r) => r,
        Err(_) => return out,
    };
    let party_re = match Regex::new(r#"(?is)<span\s+class=["']party["'][^>]*>(.*?)</span>"#) {
        Ok(r) => r,
        Err(_) => return out,
    };
    let id_re = match Regex::new(r"(?i)ViewOfficial\((\d+)\)") {
        Ok(r) => r,
        Err(_) => return out,
    };
    let strip = match Regex::new(r"(?is)<[^>]+>") {
        Ok(r) => r,
        Err(_) => return out,
    };

    for li in li_re.captures_iter(html) {
        let block = li.get(1).map(|m| m.as_str()).unwrap_or("");
        let name = name_re
            .captures(block)
            .and_then(|c| c.get(1).map(|m| html_unescape(&strip.replace_all(m.as_str(), " "))))
            .map(|s| s.split_whitespace().collect::<Vec<_>>().join(" "))
            .unwrap_or_default();
        let office = office_re
            .captures(block)
            .and_then(|c| c.get(1).map(|m| html_unescape(&strip.replace_all(m.as_str(), " "))))
            .map(|s| s.split_whitespace().collect::<Vec<_>>().join(" "))
            .unwrap_or_default();
        if name.is_empty() || office.is_empty() {
            continue;
        }
        // Skip federal — OpenFEC owns those races.
        let ol = office.to_ascii_lowercase();
        if ol.contains("u.s.") || ol.contains("united states") || ol.starts_with("us ") {
            continue;
        }
        let party = party_re
            .captures(block)
            .and_then(|c| c.get(1).map(|m| html_unescape(&strip.replace_all(m.as_str(), " "))))
            .map(|s| s.split_whitespace().collect::<Vec<_>>().join(" "))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "Unknown".into());
        let official_id = id_re
            .captures(block)
            .and_then(|c| c.get(1).and_then(|m| m.as_str().parse().ok()));
        out.push(ParsedAzOfficial {
            name,
            office,
            party: normalize_party_label(&party),
            official_id,
        });
    }
    out
}

fn parse_leg_district_from_office(office: &str) -> Option<(/*chamber*/ &'static str, u32)> {
    let re = Regex::new(r"(?i)State\s+Senat\w*.*District\s+(\d+)").ok()?;
    if let Some(c) = re.captures(office) {
        let d = c.get(1)?.as_str().parse().ok()?;
        return Some(("state_senate", d));
    }
    let re = Regex::new(r"(?i)State\s+Represent\w*.*District\s+(\d+)").ok()?;
    if let Some(c) = re.captures(office) {
        let d = c.get(1)?.as_str().parse().ok()?;
        return Some(("state_house", d));
    }
    None
}

fn is_statewide_office(office: &str) -> bool {
    let u = office.to_ascii_lowercase();
    u.contains("governor")
        || u.contains("secretary of state")
        || u.contains("attorney general")
        || u.contains("treasurer")
        || u.contains("superintendent")
        || u.contains("mine inspector")
        || u.contains("corporation commissioner")
        || u.contains("lieutenant governor")
}

/// Map OfficialList rows: statewide exec always; leg only for matching geo districts
/// (used as roster fallback). Federal already stripped in parse.
pub fn map_az_officials_for_geo(
    officials: &[ParsedAzOfficial],
    geo_senate: Option<u32>,
    geo_house: Option<u32>,
    state_ocd: &str,
    include_leg: bool,
) -> Vec<SnapshotCandidate> {
    let mut out = Vec::new();
    for o in officials {
        if let Some((chamber, dist)) = parse_leg_district_from_office(&o.office) {
            if !include_leg {
                continue;
            }
            let want = if chamber == "state_senate" {
                geo_senate
            } else {
                geo_house.or(geo_senate) // AZ LD is shared; house geo may equal senate
            };
            if want != Some(dist) {
                continue;
            }
            let ocd = if chamber == "state_senate" {
                format!("{state_ocd}/sldu:{dist}")
            } else {
                format!("{state_ocd}/sldl:{dist}")
            };
            let office_label = if chamber == "state_senate" {
                format!("Arizona Senate (District {dist})")
            } else {
                format!("Arizona House (District {dist})")
            };
            out.push(SnapshotCandidate {
                office: office_label,
                chamber: Some(chamber.into()),
                jurisdiction_ocd: ocd,
                is_judicial: false,
                name: o.name.clone(),
                party: o.party.clone(),
                is_incumbent: true,
                is_judge: false,
                summary: Some(format!(
                    "Incumbent {} ({}). Source: {AZ_ELECTIONS_PUBLISHER} elected officials.",
                    o.office, o.party
                )),
                source_url: OFFICIALS_PAGE_URL.into(),
                source_publisher: Some(AZ_ELECTIONS_PUBLISHER.into()),
                external_id: o.official_id.map(|id| format!("azce:{id}")),
            });
            continue;
        }
        if !is_statewide_office(&o.office) {
            continue;
        }
        out.push(SnapshotCandidate {
            office: format!("Arizona {}", o.office),
            chamber: Some("statewide".into()),
            jurisdiction_ocd: state_ocd.to_string(),
            is_judicial: false,
            name: o.name.clone(),
            party: o.party.clone(),
            is_incumbent: true,
            is_judge: false,
            summary: Some(format!(
                "Incumbent {} ({}). Source: {AZ_ELECTIONS_PUBLISHER} elected officials — not a full candidate filing list.",
                o.office, o.party
            )),
            source_url: OFFICIALS_PAGE_URL.into(),
            source_publisher: Some(AZ_ELECTIONS_PUBLISHER.into()),
            external_id: o.official_id.map(|id| format!("azce:{id}")),
        });
    }
    out
}

#[derive(Debug, Clone)]
pub struct Member {
    pub name: String,
    pub party: String,
    pub profile_url: String,
    pub legislator_id: Option<u32>,
}




/// Parse AZ Leg MemberRoster HTML into district → members (House has up to 2 per LD).
pub fn parse_roster(html: &str, chamber: &str) -> HashMap<u32, Vec<Member>> {
    let mut out: HashMap<u32, Vec<Member>> = HashMap::new();
    let row_re = match regex::Regex::new(r"(?is)<tr[^>]*>(.*?)</tr>") {
        Ok(r) => r,
        Err(_) => return out,
    };
    let link_re = match regex::Regex::new(
        r#"(?is)href="(https://www\.azleg\.gov/(?:Senate|House)/[^"]*?legislator=(\d+)[^"]*)"[^>]*class="roster-tooltip">\s*([^<]+)"#,
    ) {
        Ok(r) => r,
        Err(_) => return out,
    };
    let dist_re = match regex::Regex::new(r#"(?i)title="District\s+(\d+)""#) {
        Ok(r) => r,
        Err(_) => return out,
    };
    let party_re = match regex::Regex::new(r#"(?is)<span[^>]*title="([^"]+)"[^>]*>\s*([RDI])\s*</span>"#) {
        Ok(r) => r,
        Err(_) => return out,
    };

    for row in row_re.captures_iter(html) {
        let row_html = row.get(1).map(|m| m.as_str()).unwrap_or("");
        if !row_html.contains("roster-tooltip") {
            continue;
        }
        let Some(cap) = link_re.captures(row_html) else {
            continue;
        };
        let profile_url = cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
        let mut name = html_unescape(cap.get(3).map(|m| m.as_str()).unwrap_or("").trim());
        // Strip role suffixes already outside the first text node; trim leftover.
        name = name
            .split("--")
            .next()
            .unwrap_or(&name)
            .trim()
            .to_string();
        if name.is_empty() {
            continue;
        }
        let district = dist_re
            .captures(row_html)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0);
        if district == 0 {
            continue;
        }
        let party = party_re
            .captures(row_html)
            .map(|c| {
                let title = c.get(1).map(|m| m.as_str()).unwrap_or("");
                if !title.is_empty() {
                    normalize_party_label(title)
                } else {
                    match c.get(2).map(|m| m.as_str()).unwrap_or("") {
                        "R" => "Republican".into(),
                        "D" => "Democratic".into(),
                        _ => "Independent / Other".into(),
                    }
                }
            })
            .unwrap_or_else(|| "Unknown".into());

        let legislator_id = cap
            .get(2)
            .and_then(|m| m.as_str().parse::<u32>().ok());
        out.entry(district).or_default().push(Member {
            name,
            party,
            profile_url,
            legislator_id,
        });
    }

    let _ = chamber;
    out
}

pub fn member_to_candidate(
    m: &Member,
    office: &str,
    chamber: &str,
    jurisdiction_ocd: &str,
) -> SnapshotCandidate {
    let party = m.party.clone();
    SnapshotCandidate {
        office: office.to_string(),
        chamber: Some(chamber.into()),
        jurisdiction_ocd: jurisdiction_ocd.to_string(),
        is_judicial: false,
        name: m.name.clone(),
        party: party.clone(),
        is_incumbent: true,
        is_judge: false,
        summary: Some(format!(
            "Incumbent {office} ({party}). Source: {AZ_PUBLISHER} member roster."
        )),
        source_url: m.profile_url.clone(),
        source_publisher: Some(AZ_PUBLISHER.into()),
        external_id: m.legislator_id.map(|id| format!("azleg:{id}")),
    }
}

pub fn az_measures_link(state_ocd: &str) -> Vec<SnapshotMeasure> {
    vec![SnapshotMeasure {
        title: "Arizona ballot measures (see official list)".into(),
        measure_code: None,
        jurisdiction_ocd: state_ocd.to_string(),
        summary: Some(
            "Could not load the Clean Elections propositions list. Open the source for the current ballot measures."
                .into(),
        ),
        source_url: MEASURES_INFO_URL.into(),
        source_publisher: Some(AZ_ELECTIONS_PUBLISHER.into()),
    }]
}

/// One parsed AZ proposition from Clean Elections HTML fragment.
#[derive(Debug, Clone)]
pub struct ParsedAzMeasure {
    pub measure_code: Option<String>,
    pub title: String,
    pub summary: Option<String>,
    /// County heading from the feed (`Statewide`, `Maricopa`, …).
    pub county: String,
    /// Local place/district label when present (`Eagar`, school district, …).
    pub place: Option<String>,
    pub detail_id: Option<u32>,
}

/// Pick Clean Elections election key: prefer `{cycle} - General`, else `{cycle} - Primary`,
/// else first entry whose label contains the cycle year.
pub fn pick_az_measures_election_id(elections_json: &str, cycle: i32) -> Option<u32> {
    let Ok(v) = serde_json::from_str::<Value>(elections_json) else {
        return None;
    };
    let arr = v.as_array()?;
    let year = cycle.to_string();
    let mut general = None;
    let mut primary = None;
    let mut any_year = None;
    for item in arr {
        let Some(key) = item.get("key").and_then(|k| {
            k.as_u64()
                .or_else(|| k.as_i64().map(|i| i as u64))
                .or_else(|| k.as_str().and_then(|s| s.parse().ok()))
        }) else {
            continue;
        };
        let label = item
            .get("value")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !label.contains(&year) {
            continue;
        }
        let Ok(id) = u32::try_from(key) else {
            continue;
        };
        if label.contains("general") {
            general = Some(id);
        } else if label.contains("primary") {
            primary = Some(id);
        } else if any_year.is_none() {
            any_year = Some(id);
        }
    }
    general.or(primary).or(any_year)
}

/// County id for Clean Elections `CountiesForBM` JSON (`[{key,value},…]`).
pub fn pick_az_measures_county_id(counties_json: &str, county_name: &str) -> Option<u32> {
    let want = normalize_county_token(county_name);
    if want.is_empty() {
        return None;
    }
    let Ok(v) = serde_json::from_str::<Value>(counties_json) else {
        return None;
    };
    let arr = v.as_array()?;
    for item in arr {
        let label = item
            .get("value")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if normalize_county_token(label) == want {
            let key = item.get("key").and_then(|k| {
                k.as_u64()
                    .or_else(|| k.as_i64().map(|i| i as u64))
                    .or_else(|| k.as_str()?.parse().ok())
            })?;
            return u32::try_from(key).ok();
        }
    }
    None
}

/// Build list URL for Clean Elections BallotMeasures fragment (`mode=all` expands truncated lists).
pub fn az_measures_list_url(election_id: u32, county_id: u32) -> String {
    format!(
        "{MEASURES_LIST_BASE}?election={election_id}&county={county_id}&lang=en&mode=all"
    )
}

fn normalize_county_token(s: &str) -> String {
    s.trim()
        .to_ascii_lowercase()
        .replace(" county", "")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

/// Parse Clean Elections `Custom/BallotMeasures` HTML fragment.
/// When `county_filter` is set, keep **Statewide** + that county only.
pub fn parse_az_ballot_measures_html(
    html: &str,
    county_filter: Option<&str>,
) -> Vec<ParsedAzMeasure> {
    let want = county_filter
        .map(normalize_county_token)
        .filter(|s| !s.is_empty());
    let mut out = Vec::new();
    // No look-around (unsupported by `regex` crate): split on <h3> headers.
    let h3_re = match regex::Regex::new(r"(?i)<h3>\s*([^<]+?)\s*</h3>") {
        Ok(r) => r,
        Err(_) => return out,
    };
    let detail_re = match regex::Regex::new(
        r"(?is)<details[^>]*>\s*<summary>\s*([^<]+?)\s*</summary>\s*<div>([\s\S]*?)</div>\s*</details>",
    ) {
        Ok(r) => r,
        Err(_) => return out,
    };
    let bold_re = match regex::Regex::new(r"(?is)<b>\s*([^<]+?)\s*</b>") {
        Ok(r) => r,
        Err(_) => return out,
    };
    let italic_re = match regex::Regex::new(r"(?is)<i>\s*([^<]+?)\s*</i>") {
        Ok(r) => r,
        Err(_) => return out,
    };
    let id_re = match regex::Regex::new(r"(?i)ViewBallot\((\d+)\)") {
        Ok(r) => r,
        Err(_) => return out,
    };
    let strip_tags = match regex::Regex::new(r"(?is)<[^>]+>") {
        Ok(r) => r,
        Err(_) => return out,
    };
    let strip_see = match regex::Regex::new(r"(?i)\s*See the Proposition\s*") {
        Ok(r) => r,
        Err(_) => return out,
    };

    let headers: Vec<_> = h3_re.captures_iter(html).collect();
    for (i, block) in headers.iter().enumerate() {
        let county = html_unescape(block.get(1).map(|m| m.as_str()).unwrap_or("").trim());
        let county_key = normalize_county_token(&county);
        let is_statewide = county_key == "statewide" || county_key.is_empty();
        if let Some(ref want) = want {
            if !is_statewide && &county_key != want {
                continue;
            }
        }
        let body_start = block.get(0).map(|m| m.end()).unwrap_or(0);
        let body_end = headers
            .get(i + 1)
            .and_then(|n| n.get(0).map(|m| m.start()))
            .unwrap_or(html.len());
        let body = html.get(body_start..body_end).unwrap_or("");
        for d in detail_re.captures_iter(body) {
            let code_raw = html_unescape(d.get(1).map(|m| m.as_str()).unwrap_or("").trim());
            let inner = d.get(2).map(|m| m.as_str()).unwrap_or("");
            let bolds: Vec<String> = bold_re
                .captures_iter(inner)
                .filter_map(|c| c.get(1).map(|m| html_unescape(m.as_str().trim())))
                .filter(|s| !s.is_empty())
                .collect();
            let is_jurisdiction_label = |s: &str| {
                let l = s.to_ascii_lowercase();
                l == "statewide"
                    || l == "countywide"
                    || l.starts_with("proposition")
                    || l.starts_with("official title")
                    || l.starts_with("descriptive title")
            };
            let is_meta_label = |s: &str| {
                let l = s.to_ascii_lowercase();
                l.starts_with("official") || l.starts_with("descriptive")
            };
            // First bold is usually place (city / Countywide / Statewide).
            let place = bolds.first().cloned().filter(|s| !is_jurisdiction_label(s));
            let title_from_bold = bolds.iter().find(|s| {
                let l = s.to_ascii_lowercase();
                if is_meta_label(s) {
                    return false;
                }
                if l == "statewide" || l == "countywide" {
                    return false;
                }
                if place.as_ref().map(|p| p.as_str()) == Some(s.as_str()) {
                    return false;
                }
                l.starts_with("proposition") || s.len() > 3
            });
            let italic = italic_re
                .captures_iter(inner)
                .filter_map(|c| c.get(1).map(|m| html_unescape(m.as_str().trim())))
                .find(|s| !s.is_empty() && !s.to_ascii_lowercase().contains("fa-"));

            let mut text = strip_tags.replace_all(inner, " ").to_string();
            text = html_unescape(&text);
            text = strip_see.replace_all(&text, " ").to_string();
            text = text.split_whitespace().collect::<Vec<_>>().join(" ");

            let code = {
                let c = code_raw.trim();
                if c.is_empty() {
                    None
                } else if c.chars().all(|ch| ch.is_ascii_digit()) {
                    Some(format!("Prop {c}"))
                } else {
                    Some(c.to_string())
                }
            };

            let title = if let Some(t) = title_from_bold {
                t.clone()
            } else if let Some(ref it) = italic {
                it.clone()
            } else if let Some(ref c) = code {
                c.clone()
            } else {
                "Ballot measure".into()
            };

            let title = if title.to_ascii_lowercase().starts_with("proposition") {
                if let Some(ref it) = italic {
                    if !it.is_empty() {
                        it.clone()
                    } else {
                        title
                    }
                } else {
                    title
                }
            } else {
                title
            };

            let mut summary = text;
            for prefix in bolds.iter().chain(italic.iter()) {
                if let Some(rest) = summary.strip_prefix(prefix.as_str()) {
                    summary = rest.trim().to_string();
                }
            }
            if let Some(ref c) = code {
                if let Some(rest) = summary.strip_prefix(c.as_str()) {
                    summary = rest.trim().to_string();
                }
                let bare = c.trim_start_matches("Prop ").trim();
                if let Some(rest) = summary.strip_prefix(&format!("PROPOSITION {bare}")) {
                    summary = rest.trim().to_string();
                }
            }
            if summary.len() > 600 {
                summary = format!("{}…", summary.chars().take(597).collect::<String>());
            }

            let detail_id = id_re
                .captures(inner)
                .and_then(|c| c.get(1))
                .and_then(|m| m.as_str().parse().ok());

            out.push(ParsedAzMeasure {
                measure_code: code,
                title,
                summary: if summary.is_empty() {
                    None
                } else {
                    Some(summary)
                },
                county: if is_statewide {
                    "Statewide".into()
                } else {
                    county.clone()
                },
                place,
                detail_id,
            });
        }
    }
    out
}

/// Map parsed AZ measures into snapshot rows for the ZIP’s county (+ statewide).
pub fn map_az_measures_for_geo(
    parsed: &[ParsedAzMeasure],
    state_ocd: &str,
    county_name: &str,
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
                format!("{} County", title_case_token(&county_slug))
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
            state: Some("AZ".into()),
        });
        Some(ocd)
    };

    for m in parsed {
        let is_state = m.county.eq_ignore_ascii_case("statewide");
        let jurisdiction_ocd = if is_state {
            state_ocd.to_string()
        } else {
            county_ocd
                .clone()
                .unwrap_or_else(|| state_ocd.to_string())
        };
        let source_url = m
            .detail_id
            .map(|id| format!("{MEASURE_DETAIL_BASE}{id}"))
            .unwrap_or_else(|| MEASURES_INFO_URL.into());
        let title = match (&m.place, is_state) {
            (Some(place), false) if !m.title.to_ascii_lowercase().contains(&place.to_ascii_lowercase()) => {
                format!("{} — {place}", m.title)
            }
            _ => m.title.clone(),
        };
        measures.push(SnapshotMeasure {
            title,
            measure_code: m.measure_code.clone(),
            jurisdiction_ocd,
            summary: m.summary.clone(),
            source_url,
            source_publisher: Some(AZ_ELECTIONS_PUBLISHER.into()),
        });
    }
    (measures, extras)
}

fn title_case_token(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

pub fn html_unescape(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
        .replace("&mdash;", "—")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::GeoResolution;

    const SAMPLE_SENATE: &str = r#"
    <table><thead><tr><th>Name</th><th>District</th><th>Party</th></tr></thead>
    <tr class="">
      <td><a href="https://www.azleg.gov/Senate/Senate-member/?legislature=57&session=130&legislator=2371" class="roster-tooltip">Lela Alston<strong> -- Minority Caucus Chair</strong></a></td>
      <td><a href="https://www.azleg.gov/images/LegislativeDistrictMaps/LegislativeDistrict05.pdf" class="district-tooltip" title="District 5" target="_blank">5</a></td>
      <td><span title="Democratic">D</span></td>
    </tr>
    <tr class="odd">
      <td><a href="https://www.azleg.gov/Senate/Senate-member/?legislature=57&session=130&legislator=2400" class="roster-tooltip">Shawnna Bolick<strong></strong></a></td>
      <td><a title="District 2" href="x.pdf">2</a></td>
      <td><span title="Republican">R</span></td>
    </tr>
    </table>
    "#;

    const SAMPLE_HOUSE: &str = r#"
    <table>
    <tr>
      <td><a href="https://www.azleg.gov/House/House-member/?legislature=57&session=130&legislator=1001" class="roster-tooltip">Ada One</a></td>
      <td><a title="District 8" href="x.pdf">8</a></td>
      <td><span title="Democratic">D</span></td>
    </tr>
    <tr>
      <td><a href="https://www.azleg.gov/House/House-member/?legislature=57&session=130&legislator=1002" class="roster-tooltip">Bea Two</a></td>
      <td><a title="District 8" href="x.pdf">8</a></td>
      <td><span title="Republican">R</span></td>
    </tr>
    <tr>
      <td><a href="https://www.azleg.gov/House/House-member/?legislature=57&session=130&legislator=1003" class="roster-tooltip">Cee Three</a></td>
      <td><a title="District 3" href="x.pdf">3</a></td>
      <td><span title="Republican">R</span></td>
    </tr>
    </table>
    "#;

    #[test]
    fn parse_senate_sample() {
        let map = parse_roster(SAMPLE_SENATE, "Senate");
        assert_eq!(map.len(), 2);
        let d5 = map.get(&5).expect("district 5");
        assert_eq!(d5.len(), 1);
        assert!(d5[0].name.contains("Alston"));
        assert_eq!(d5[0].party, "Democratic");
        assert!(d5[0].profile_url.contains("legislator=2371"));
        assert_eq!(d5[0].legislator_id, Some(2371));
        let d2 = map.get(&2).unwrap();
        assert_eq!(d2[0].party, "Republican");
    }

    #[test]
    fn parse_house_two_per_district() {
        let map = parse_roster(SAMPLE_HOUSE, "House");
        let d8 = map.get(&8).expect("district 8");
        assert_eq!(d8.len(), 2);
        assert!(d8.iter().any(|m| m.name.contains("Ada")));
        assert!(d8.iter().any(|m| m.name.contains("Bea")));
        assert_eq!(map.get(&3).map(|v| v.len()), Some(1));
    }

    #[test]
    fn map_roster_for_geo() {
        let senators = parse_roster(SAMPLE_SENATE, "Senate");
        let house = parse_roster(SAMPLE_HOUSE, "House");
        let geo = GeoResolution {
            state: "AZ".into(),
            state_name: "Arizona".into(),
            county: "Maricopa County".into(),
            city: "Phoenix".into(),
            congressional_district: "AZ-3".into(),
            state_senate_district: Some(5),
            state_house_district: Some(8),
            state_house_label: Some("8".into()),
            latitude: None,
            longitude: None,
            jurisdictions: vec![],
            source_url: String::new(),
            source_publisher: String::new(),
        };
        let mut cands = Vec::new();
        if let Some(sd) = geo.state_senate_district {
            for m in senators.get(&sd).into_iter().flatten() {
                cands.push(member_to_candidate(
                    m,
                    &format!("Arizona Senate (District {sd})"),
                    "state_senate",
                    &format!("ocd-division/country:us/state:az/sldu:{sd}"),
                ));
            }
        }
        if let Some(hd) = geo.state_house_district {
            for m in house.get(&hd).into_iter().flatten() {
                cands.push(member_to_candidate(
                    m,
                    &format!("Arizona House (District {hd})"),
                    "state_house",
                    &format!("ocd-division/country:us/state:az/sldl:{hd}"),
                ));
            }
        }
        assert_eq!(cands.len(), 3); // 1 senate + 2 house
        assert!(cands.iter().all(|c| c.is_incumbent));
        assert!(cands.iter().any(|c| c.office.contains("Senate") && c.name.contains("Alston")));
    }

    #[test]
    fn pick_az_measures_election_prefers_general() {
        let json = r#"[
          {"key":62,"value":"2026 - Primary"},
          {"key":61,"value":"2024 - General"},
          {"key":52,"value":"2022 - General"}
        ]"#;
        assert_eq!(pick_az_measures_election_id(json, 2024), Some(61));
        assert_eq!(pick_az_measures_election_id(json, 2026), Some(62));
        assert_eq!(pick_az_measures_election_id(json, 2099), None);
    }

    #[test]
    fn pick_az_measures_county_id_normalizes() {
        let json = r#"[
          {"key":1,"value":"Maricopa"},
          {"key":2,"value":"Pima"}
        ]"#;
        assert_eq!(pick_az_measures_county_id(json, "Maricopa County"), Some(1));
        assert_eq!(pick_az_measures_county_id(json, "pima"), Some(2));
        assert_eq!(pick_az_measures_county_id(json, "Coconino"), None);
    }

    #[test]
    fn parse_az_ballot_measures_inline_minimal() {
        let html = r#"
        <h3>Statewide</h3>
        <details open>
          <summary>133</summary>
          <div>
            <b>Statewide</b><br />
            <b>Direct Primary Elections</b><br /><br />REQUIRES DIRECT PRIMARY.
            <span class="link" onclick="myPopup.ViewBallot(254);">See the Proposition</span>
          </div>
        </details>
        <h3>Maricopa</h3>
        <details>
          <summary>479</summary>
          <div>
            <b>Countywide</b><br />
            <b>Transport tax</b><br /><br />A measure continuing tax.
            <span onclick="myPopup.ViewBallot(275);">See</span>
          </div>
        </details>
        <h3>Pima</h3>
        <details>
          <summary>999</summary>
          <div><b>Tucson</b><br /><b>Other</b><br />drop me</div>
        </details>
        "#;
        let all = parse_az_ballot_measures_html(html, None);
        assert_eq!(all.len(), 3, "all={all:?}");
        let m = parse_az_ballot_measures_html(html, Some("Maricopa County"));
        assert_eq!(m.len(), 2, "maricopa={m:?}");
        assert_eq!(m[0].measure_code.as_deref(), Some("Prop 133"));
        assert!(m[0].title.contains("Primary"));
        assert_eq!(m[0].detail_id, Some(254));
        assert_eq!(m[1].measure_code.as_deref(), Some("Prop 479"));
        assert!(m[1].place.as_deref() == Some("Countywide") || m[1].title.contains("Transport"));
        let pima = parse_az_ballot_measures_html(html, Some("Pima"));
        assert_eq!(pima.len(), 2); // statewide + Pima local
        assert!(pima.iter().any(|m| m.county.eq_ignore_ascii_case("statewide")));
        assert!(pima.iter().any(|m| m.county.eq_ignore_ascii_case("pima")));
        assert!(!pima.iter().any(|m| m.county.eq_ignore_ascii_case("maricopa")));
    }

    #[test]
    fn parse_az_ballot_measures_sample_filters_county() {
        let html = include_str!("../../../../testdata/az_ballot_measures_sample.html");
        assert!(html.contains("<details"), "fixture missing details, len={}", html.len());
        let all = parse_az_ballot_measures_html(html, None);
        assert!(all.len() >= 5, "got {} html_len={}", all.len(), html.len());
        let maricopa = parse_az_ballot_measures_html(html, Some("Maricopa County"));
        assert_eq!(maricopa.len(), all.len()); // fixture is statewide + maricopa only
        assert!(maricopa.iter().any(|m| m.measure_code.as_deref() == Some("Prop 133")));
        let p133 = maricopa
            .iter()
            .find(|m| m.measure_code.as_deref() == Some("Prop 133"))
            .expect("133");
        assert!(p133.title.to_ascii_lowercase().contains("primary"));
        assert!(p133.county.eq_ignore_ascii_case("statewide"));
        assert!(p133.detail_id == Some(254));
        assert!(p133.summary.as_ref().is_some_and(|s| s.len() > 20));

        let pima = parse_az_ballot_measures_html(html, Some("Pima"));
        assert!(
            pima.iter().all(|m| m.county.eq_ignore_ascii_case("statewide")),
            "Pima filter should drop Maricopa locals"
        );
        assert!(pima.iter().any(|m| m.measure_code.as_deref() == Some("Prop 133")));
    }

    #[test]
    fn az_officials_packed_and_parse() {
        let p = az_officials_packed(3, 11, "Maricopa County").expect("packed");
        assert_eq!(p, "AZ-3-11-Maricopa~County-0--");
        assert!(az_officials_list_url(&p).contains("OfficialList"));
        assert!(az_officials_packed(0, 11, "Maricopa").is_none());

        let html = include_str!("../../../../testdata/az_officials_sample.html");
        let all = parse_az_officials_html(html);
        assert!(all.len() >= 12, "got {}", all.len());
        assert!(all.iter().any(|o| o.office.eq_ignore_ascii_case("Governor")));
        assert!(!all.iter().any(|o| o.office.to_ascii_lowercase().contains("u.s.")));

        let statewide = map_az_officials_for_geo(
            &all,
            Some(11),
            Some(11),
            "ocd-division/country:us/state:az",
            false,
        );
        assert!(statewide.iter().all(|c| c.chamber.as_deref() == Some("statewide")));
        assert!(statewide.iter().any(|c| c.name.contains("Hobbs")));
        assert!(statewide.iter().all(|c| c.is_incumbent));

        let with_leg = map_az_officials_for_geo(
            &all,
            Some(11),
            Some(11),
            "ocd-division/country:us/state:az",
            true,
        );
        assert!(with_leg.iter().any(|c| c.chamber.as_deref() == Some("state_senate")));
        assert!(with_leg.iter().any(|c| c.chamber.as_deref() == Some("state_house")));
        assert!(with_leg.iter().all(|c| {
            !matches!(c.chamber.as_deref(), Some("state_senate" | "state_house"))
                || c.office.contains("District 11")
        }));
    }

    #[test]
    fn map_az_measures_for_geo_cites_clean_elections() {
        let html = include_str!("../../../../testdata/az_ballot_measures_sample.html");
        let parsed = parse_az_ballot_measures_html(html, Some("Maricopa"));
        let (ms, extras) =
            map_az_measures_for_geo(&parsed, "ocd-division/country:us/state:az", "Maricopa County");
        assert!(!ms.is_empty());
        assert!(ms.iter().all(|m| {
            m.source_publisher.as_deref() == Some(AZ_ELECTIONS_PUBLISHER)
                && !m.source_url.is_empty()
        }));
        assert!(ms.iter().any(|m| m.measure_code.as_deref() == Some("Prop 133")));
        assert!(extras.iter().any(|j| j.level == "county" && j.ocd_id.contains("maricopa")));
    }

    #[test]
    fn az_cf_parse_match_finance() {
        let body = include_str!("../../../../testdata/az_cf_candidates_sample.json");
        let hits = parse_az_cf_table_json(body).expect("parse");
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].entity_id, 10001);

        let q = AzCfMatchQuery {
            name: "Katie Hobbs".into(),
            office: "Governor".into(),
            chamber: "statewide".into(),
            party: "Democratic".into(),
            district: None,
        };
        match match_az_cf_candidate(&hits, &q) {
            AzCfMatch::Unique { hit } => {
                assert_eq!(hit.entity_id, 10001);
                let fin = az_cf_finance_from_hit(&hit, 2026);
                assert_eq!(fin.source, "az_cf");
                assert!(fin.receipts_display.contains("1,250"));
                assert!(fin.profile_url.contains("seethemoney.az.gov"));
            }
            other => panic!("expected unique {other:?}"),
        }

        let q2 = AzCfMatchQuery {
            name: "Lela Alston".into(),
            office: "Arizona Senate (District 5)".into(),
            chamber: "state_senate".into(),
            party: "Democratic".into(),
            district: Some(5),
        };
        match match_az_cf_candidate(&hits, &q2) {
            AzCfMatch::Unique { hit } => assert_eq!(hit.entity_id, 10002),
            other => panic!("expected senate unique {other:?}"),
        }

        assert!(matches!(
            match_az_cf_candidate(
                &hits,
                &AzCfMatchQuery {
                    name: "Nobody".into(),
                    office: String::new(),
                    chamber: String::new(),
                    party: String::new(),
                    district: None,
                }
            ),
            AzCfMatch::None
        ));

        let url = az_cf_candidates_url(2024, 2026, "Hobbs", None);
        assert!(url.contains("GetNEWTableData"));
        assert!(url.contains("Page=1"));
        assert!(url.contains("Name=Hobbs"));
        assert_eq!(az_cf_office_id("state_senate", Some(5)), Some(3105));
        assert_eq!(az_cf_office_id("state_house", Some(12)), Some(3212));
        assert_eq!(az_cf_office_id("statewide", None), Some(2000)); // governor heuristic needs office text
    }
}

// --- AZ SeeTheMoney campaign finance (A4) ---

pub const AZ_CF_BASE: &str = "https://seethemoney.az.gov";
pub const AZ_CF_PUBLISHER: &str = "Arizona Secretary of State (SeeTheMoney)";

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

/// SeeTheMoney candidate-tab office IDs (from site filters / JS).
/// Senate LD N → 3100+N; House LD N → 3200+N; Governor → 2000.
pub fn az_cf_office_id(chamber: &str, district: Option<u32>) -> Option<u32> {
    match chamber.trim() {
        "state_senate" => district.map(|d| 3100 + d),
        "state_house" => district.map(|d| 3200 + d),
        "statewide" | "state_exec" => Some(2000),
        _ => None,
    }
}

/// DataTables POST URL for candidate overview rows (Page=1 = Candidates tab).
pub fn az_cf_candidates_url(
    start_year: i32,
    end_year: i32,
    name: &str,
    office_id: Option<u32>,
) -> String {
    let mut q = format!(
        "Page=1&startYear={start_year}&endYear={end_year}&JurisdictionId=0&TablePage=0&TableLength=25&IsLessActive=false&ShowOfficeHolder=true"
    );
    let n = name.trim();
    if !n.is_empty() {
        q.push_str(&format!("&Name={}", urlenc(n)));
    }
    if let Some(oid) = office_id {
        if oid > 0 {
            q.push_str(&format!("&OfficeID={oid}"));
        }
    }
    // Leading ? is required: ASP.NET treats bare & in the path as dangerous.
    format!("{AZ_CF_BASE}/Reporting/GetNEWTableData/?{q}")
}

/// Minimal DataTables serverSide body (site accepts this shape).
pub fn az_cf_datatables_body() -> String {
    "draw=1&start=0&length=25&search%5Bvalue%5D=&search%5Bregex%5D=false&order%5B0%5D%5Bcolumn%5D=0&order%5B0%5D%5Bdir%5D=asc".into()
}

pub fn az_cf_explore_url(entity_id: u32, start_year: i32, end_year: i32) -> String {
    // Explore hash mirrors BuildHash("11", "1~{entityId}")
    format!(
        "{AZ_CF_BASE}/Reporting/Explore#JurisdictionId=0|Page=11|startYear={start_year}|endYear={end_year}|entityId={entity_id}|TablePage=1|TableLength=10"
    )
}

pub fn az_cf_search_name_fragment(ballot_name: &str) -> String {
    let (_f, last) = split_candidate_first_last(ballot_name);
    if !last.is_empty() {
        last
    } else {
        ballot_name.trim().to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzCfHit {
    pub entity_id: u32,
    pub name: String,
    pub committee_name: String,
    pub office_name: String,
    pub party_name: String,
    pub income: Option<f64>,
    pub expense: Option<f64>,
    pub cash_balance: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct AzCfMatchQuery {
    pub name: String,
    pub office: String,
    pub chamber: String,
    pub party: String,
    pub district: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind")]
pub enum AzCfMatch {
    #[serde(rename = "unique")]
    Unique { hit: AzCfHit },
    #[serde(rename = "none")]
    None,
    #[serde(rename = "ambiguous")]
    Ambiguous { count: usize },
}

#[derive(Debug, Clone, Serialize)]
pub struct AzCfFinance {
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

/// Display name from SeeTheMoney row (EntityLastName often "LAST, FIRST").
pub fn az_cf_row_display_name(last_field: &str, committee: &str) -> String {
    let t = last_field.trim();
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

/// Parse DataTables JSON from GetNEWTableData (candidates tab).
pub fn parse_az_cf_table_json(body: &str) -> Result<Vec<AzCfHit>, String> {
    let t = body.trim();
    if t.is_empty() || t == "\"\"" || t == "[]" {
        return Ok(vec![]);
    }
    let v: Value = serde_json::from_str(t).map_err(|e| format!("AZ CF JSON: {e}"))?;
    let rows = v
        .get("data")
        .and_then(|d| d.as_array())
        .cloned()
        .or_else(|| v.as_array().cloned())
        .unwrap_or_default();
    let mut out = Vec::new();
    for row in rows {
        let entity_id = row
            .get("EntityID")
            .or_else(|| row.get("EntityId"))
            .and_then(json_u32)
            .unwrap_or(0);
        let last_field = row_str(&row, &["EntityLastName", "EntityName", "Name"]);
        let committee = row_str(&row, &["CommitteeName"]);
        let name = az_cf_row_display_name(&last_field, &committee);
        if entity_id == 0 && name.is_empty() {
            continue;
        }
        out.push(AzCfHit {
            entity_id,
            name,
            committee_name: committee,
            office_name: row_str(&row, &["OfficeName", "Office"]),
            party_name: row_str(&row, &["PartyName", "Party"]),
            income: row.get("Income").and_then(json_f64),
            expense: row.get("Expense").and_then(json_f64),
            cash_balance: row
                .get("CashBalance")
                .or_else(|| row.get("CashOnHand"))
                .and_then(json_f64),
        });
    }
    Ok(out)
}

fn parties_conflict(a: &str, b: &str) -> bool {
    let norm = |p: &str| -> Option<&'static str> {
        let u = p.trim().to_ascii_uppercase();
        if u.is_empty() {
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

fn office_compatible(q: &AzCfMatchQuery, hit: &AzCfHit) -> bool {
    let ho = hit.office_name.to_ascii_lowercase();
    let qo = q.office.to_ascii_lowercase();
    let ch = q.chamber.to_ascii_lowercase();
    if ho.is_empty() {
        return true;
    }
    if ch == "state_senate" || qo.contains("senate") {
        if ho.contains("house") && !ho.contains("senate") {
            return false;
        }
        return ho.contains("senate") || ho.contains("legislat") || !ho.contains("house");
    }
    if ch == "state_house" || qo.contains("house") {
        if ho.contains("senate") {
            return false;
        }
        return ho.contains("house") || ho.contains("legislat") || !ho.contains("senate");
    }
    if ch == "statewide" || qo.contains("governor") {
        return ho.contains("governor") || !ho.contains("district");
    }
    true
}

fn district_compatible(q: &AzCfMatchQuery, hit: &AzCfHit) -> bool {
    let q_dist = q.district.or_else(|| district_from_ballot_office(&q.office));
    let h_dist = district_from_ballot_office(&hit.office_name);
    match (q_dist, h_dist) {
        (Some(a), Some(b)) => a == b,
        _ => true,
    }
}

pub fn match_az_cf_candidate(hits: &[AzCfHit], q: &AzCfMatchQuery) -> AzCfMatch {
    let matched: Vec<&AzCfHit> = hits
        .iter()
        .filter(|h| last_names_match(&q.name, &h.name) || last_names_match(&q.name, &h.committee_name))
        .filter(|h| {
            first_names_compatible(&q.name, &h.name)
                || h.name.split_whitespace().count() <= 1 // last-only field
        })
        .filter(|h| office_compatible(q, h))
        .filter(|h| district_compatible(q, h))
        .filter(|h| !parties_conflict(&q.party, &h.party_name))
        .collect();
    match matched.len() {
        0 => AzCfMatch::None,
        1 => AzCfMatch::Unique {
            hit: matched[0].clone(),
        },
        n => AzCfMatch::Ambiguous { count: n },
    }
}

pub fn az_cf_finance_from_hit(hit: &AzCfHit, cycle: i32) -> AzCfFinance {
    let end_year = cycle;
    let start_year = cycle.saturating_sub(2).max(2000);
    AzCfFinance {
        source: "az_cf".into(),
        cycle: cycle.to_string(),
        account: hit.entity_id.to_string(),
        match_name: hit.name.clone(),
        match_office: if hit.office_name.is_empty() {
            hit.committee_name.clone()
        } else {
            hit.office_name.clone()
        },
        receipts_display: hit.income.map(format_usd).unwrap_or_else(|| "—".into()),
        disbursements_display: hit.expense.map(format_usd).unwrap_or_else(|| "—".into()),
        cash_on_hand_display: hit
            .cash_balance
            .map(format_usd)
            .unwrap_or_else(|| "—".into()),
        source_label: AZ_CF_PUBLISHER.into(),
        profile_url: az_cf_explore_url(hit.entity_id, start_year, end_year),
        note: "Income / expense / cash balance from Arizona SeeTheMoney candidate overview for the selected year window. Filed portal totals — not a bank audit.".into(),
    }
}
