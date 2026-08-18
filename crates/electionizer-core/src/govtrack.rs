//! Pure congress-legislators helpers (no HTTP).
use crate::models::{AffiliationSpan, VoteRecord};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

const CL_SOURCE: &str = "unitedstates/congress-legislators";
const CL_SOURCE_URL: &str = "https://github.com/unitedstates/congress-legislators";

#[derive(Debug, Clone, Serialize)]
pub struct LegislatorMatch {
    pub govtrack_id: i64,
    pub bioguide: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wikidata: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wikipedia: Option<String>,
    /// congress-legislators `id.ballotpedia` page title (spaces OK).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ballotpedia: Option<String>,
    /// Latest term `url` (member house.gov / senate.gov site), if present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub official_url: Option<String>,
    pub name: String,
    pub profile_url: String,
    pub affiliations: Vec<AffiliationSpan>,
}

/// Lightweight row from vote_voter list (before detail fetch).
#[derive(Debug, Clone, Serialize)]
pub struct GovTrackVoteStub {
    pub vote_id: Option<i64>,
    pub date: String,
    pub position: String,
}

pub fn build_fec_index(json: &str) -> HashMap<String, LegislatorMatch> {
    let Ok(arr) = serde_json::from_str::<Vec<Value>>(json) else {
        return HashMap::new();
    };
    let mut map = HashMap::new();
    for person in arr {
        let ids = person.get("id").cloned().unwrap_or(Value::Null);
        let Some(govtrack) = ids.get("govtrack").and_then(|v| v.as_i64()) else {
            continue;
        };
        let bioguide = ids
            .get("bioguide")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let wikidata = ids
            .get("wikidata")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let wikipedia = ids
            .get("wikipedia")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let ballotpedia = ids
            .get("ballotpedia")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let name = person
            .pointer("/name/official_full")
            .and_then(|v| v.as_str())
            .or_else(|| person.pointer("/name/last").and_then(|v| v.as_str()))
            .unwrap_or("Member")
            .to_string();
        let slug = name
            .to_ascii_lowercase()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect::<String>();
        let profile_url = format!("https://www.govtrack.us/congress/members/{slug}/{govtrack}");

        let mut affiliations = Vec::new();
        let mut official_url: Option<String> = None;
        if let Some(terms) = person.get("terms").and_then(|t| t.as_array()) {
            for t in terms {
                let party = t
                    .get("party")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string();
                let start = t
                    .get("start")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let end = t.get("end").and_then(|v| v.as_str()).map(|s| s.to_string());
                let ttype = t.get("type").and_then(|v| v.as_str()).unwrap_or("member");
                let state = t.get("state").and_then(|v| v.as_str()).unwrap_or("");
                let role = match ttype {
                    "sen" => format!("U.S. Senate ({state})"),
                    "rep" => {
                        let dist = t
                            .get("district")
                            .map(|d| d.to_string())
                            .unwrap_or_else(|| "?".into());
                        format!("U.S. House ({state}-{dist})")
                    }
                    other => other.to_string(),
                };
                affiliations.push(AffiliationSpan {
                    party,
                    start,
                    end,
                    role,
                    source: Some(CL_SOURCE.into()),
                    source_url: Some(CL_SOURCE_URL.into()),
                });
                // Last term with a member site wins (terms chronological).
                if let Some(u) = t.get("url").and_then(|v| v.as_str()).map(str::trim) {
                    if !u.is_empty()
                        && (u.contains("house.gov")
                            || u.contains("senate.gov")
                            || u.contains(".gov"))
                    {
                        official_url = Some(normalize_official_site_url(u));
                    }
                }
            }
        }
        // Newest last in source; show newest first.
        affiliations.reverse();

        let m = LegislatorMatch {
            govtrack_id: govtrack,
            bioguide,
            wikidata,
            wikipedia,
            ballotpedia,
            official_url,
            name,
            profile_url,
            affiliations,
        };

        if let Some(fec_list) = ids.get("fec").and_then(|v| v.as_array()) {
            for f in fec_list {
                if let Some(fid) = f.as_str() {
                    map.insert(fid.to_ascii_uppercase(), m.clone());
                }
            }
        }
    }
    map
}

/// Ballot / filing public signals as affiliation rows (no extra I/O).
/// Each span cites `source` / `source_url` when provided (publisher + profile URL).
///
/// Judges (`is_judge`): seat/group is a **role** label; party column holds ballot
/// designation (merit retention / nonpartisan / filed party) — not “Independent”
/// for NOP judicial codes.
pub fn ballot_affiliations(
    party: &str,
    office: &str,
    is_incumbent: bool,
    is_judge: bool,
    source: Option<&str>,
    source_url: Option<&str>,
) -> Vec<AffiliationSpan> {
    let src = source
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| Some("Filing / ballot data".into()));
    let url = source_url
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let p = party.trim();
    let office_label = {
        let o = office.trim();
        if o.is_empty() {
            "ballot office"
        } else {
            o
        }
    };

    if is_judge {
        return judicial_ballot_affiliations(p, office_label, is_incumbent, src, url);
    }

    let mut out = Vec::new();
    if !p.is_empty() {
        out.push(AffiliationSpan {
            party: p.to_string(),
            start: None,
            end: None,
            role: format!("As filed · {office_label}"),
            source: src.clone(),
            source_url: url.clone(),
        });
    }
    if is_incumbent {
        out.push(AffiliationSpan {
            party: if p.is_empty() {
                "—".into()
            } else {
                p.to_string()
            },
            start: None,
            end: None,
            role: format!("Incumbent · {office_label}"),
            source: src,
            source_url: url,
        });
    }
    out
}

/// Current filing party as a single affiliation row (no incumbent / source).
pub fn current_party_affiliation(party: &str, office: &str) -> Vec<AffiliationSpan> {
    ballot_affiliations(party, office, false, false, None, None)
}

/// True when the office is typically a yes/no retention ballot (not a multi-candidate race).
pub fn is_merit_retention_office(office: &str) -> bool {
    let o = office.to_ascii_lowercase();
    o.contains("retention")
        || o.contains("supreme")
        || o.contains("district court of appeal")
        || o.contains("court of appeal")
        || o.contains("appellate court")
        || o.contains("appellate circuit")
}

/// Filing labels that mean nonpartisan / no party for judicial seats.
pub fn is_nonpartisan_filing_label(party: &str) -> bool {
    let p = party.trim().to_ascii_lowercase();
    if p.is_empty() || p == "—" || p == "-" || p == "n/a" || p == "na" {
        return true;
    }
    p == "nop"
        || p == "npa"
        || p == "np"
        || p == "nonpartisan"
        || p == "non-partisan"
        || p == "non partisan"
        || p.contains("no party")
        || p == "independent / other"
        || p == "independent/other"
        || p == "independent"
        || p == "none"
        || p == "unknown"
}

/// Format judicial office so Group/seat is clearly a seat number, not a party.
pub fn judicial_seat_role(office: &str) -> String {
    let o = office.trim();
    if o.is_empty() {
        return "Judicial seat".into();
    }
    // "Circuit Judge (Circuit 18, Group 13)" → "Circuit Judge · Circuit 18 · Group 13 (seat)"
    // "County Judge (Group 3)" → "County Judge · Group 3 (seat)"
    let mut s = o.to_string();
    if let Some(open) = s.find('(') {
        if s.ends_with(')') {
            let head = s[..open].trim().to_string();
            let inner = s[open + 1..s.len() - 1].trim().to_string();
            if !head.is_empty() && !inner.is_empty() {
                let inner = inner
                    .replace(", Group ", " · Group ")
                    .replace(", group ", " · Group ");
                let inner = if inner.to_ascii_lowercase().contains("group")
                    && !inner.to_ascii_lowercase().contains("(seat)")
                {
                    // Mark Group N as seat number when present.
                    if let Some(gi) = inner.to_ascii_lowercase().find("group ") {
                        let before = inner[..gi].to_string();
                        let after = &inner[gi + "group ".len()..];
                        let (num, rest) = split_leading_token(after);
                        format!("{before}Group {num} (seat){rest}")
                    } else {
                        inner
                    }
                } else {
                    inner
                };
                s = format!("{head} · {inner}");
            }
        }
    }
    s
}

fn split_leading_token(s: &str) -> (String, String) {
    let s = s.trim_start();
    let end = s
        .find(|c: char| c.is_whitespace() || c == ',' || c == '·' || c == ')')
        .unwrap_or(s.len());
    (s[..end].to_string(), s[end..].to_string())
}

fn judicial_ballot_affiliations(
    party: &str,
    office_label: &str,
    is_incumbent: bool,
    src: Option<String>,
    url: Option<String>,
) -> Vec<AffiliationSpan> {
    let seat = judicial_seat_role(office_label);
    let retention = is_merit_retention_office(office_label);
    let nonpartisan = is_nonpartisan_filing_label(party);
    let major = crate::models::party_bucket(party);

    let mut out = Vec::new();

    // Primary ballot designation (status), not voter registration.
    let designation = if retention {
        "Merit retention"
    } else if nonpartisan {
        "Nonpartisan"
    } else if !party.is_empty() {
        party
    } else {
        "Nonpartisan"
    };

    out.push(AffiliationSpan {
        party: designation.to_string(),
        start: None,
        end: None,
        role: format!("Ballot designation · {seat}"),
        source: src.clone(),
        source_url: url.clone(),
    });

    // If the filing also carries a real party (partisan judicial race, or
    // retention with a listed party), show it separately from status.
    if !nonpartisan
        && !party.is_empty()
        && (major == "democrat" || major == "republican" || major == "other")
        && designation != party
    {
        out.push(AffiliationSpan {
            party: party.to_string(),
            start: None,
            end: None,
            role: format!("As filed · {seat}"),
            source: src.clone(),
            source_url: url.clone(),
        });
    }

    if is_incumbent {
        out.push(AffiliationSpan {
            party: if nonpartisan || party.is_empty() {
                designation.to_string()
            } else {
                party.to_string()
            },
            start: None,
            end: None,
            role: format!("Incumbent · {seat}"),
            source: src,
            source_url: url,
        });
    }

    out
}

/// Soft CF context: campaign committee / account is **not** voter-party affiliation.
/// Returns `None` when name is empty (cite or omit).
pub fn campaign_committee_affiliation(
    committee_name: &str,
    designation: &str,
    source: Option<&str>,
    source_url: Option<&str>,
) -> Option<AffiliationSpan> {
    let name = committee_name.trim();
    if name.is_empty() {
        return None;
    }
    let des = designation.trim();
    let des = if des.is_empty() {
        "Campaign committee"
    } else {
        des
    };
    let src = source
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| Some("Campaign finance".into()));
    let url = source_url
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    Some(AffiliationSpan {
        // Explicit label so the Party column is never mistaken for voter party.
        party: "Committee".into(),
        start: None,
        end: None,
        role: format!("{des} (≠ voter affiliation) · {name}"),
        source: src,
        source_url: url,
    })
}

#[derive(Debug, Deserialize)]
pub struct VoteVoterResponse {
    pub meta: Option<VoteVoterMeta>,
    pub objects: Option<Vec<VoteVoterRow>>,
}

#[derive(Debug, Deserialize)]
pub struct VoteVoterMeta {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub total_count: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct VoteVoterRow {
    pub created: Option<String>,
    pub option: Option<VoteOption>,
    pub vote: Option<VoteRef>,
}

#[derive(Debug, Deserialize)]
pub struct VoteOption {
    pub value: Option<String>,
    pub vote: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum VoteRef {
    Id(i64),
    Obj(VoteRefObj),
}

/// Embedded vote object on vote_voter list rows (GovTrack often inlines full vote).
#[derive(Debug, Deserialize)]
pub struct VoteRefObj {
    pub id: Option<i64>,
    pub question: Option<String>,
    pub question_details: Option<String>,
    pub result: Option<String>,
    pub link: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VoteDetail {
    pub question: Option<String>,
    pub question_details: Option<String>,
    pub result: Option<String>,
    pub link: Option<String>,
}

/// `meta.total_count` from a vote_voter list page, when present.
pub fn vote_voter_total_count(json: &str) -> Option<u64> {
    serde_json::from_str::<VoteVoterResponse>(json)
        .ok()
        .and_then(|r| r.meta)
        .and_then(|m| m.total_count)
}

fn vote_id_from_row(row: &VoteVoterRow) -> Option<i64> {
    row.vote
        .as_ref()
        .and_then(|v| match v {
            VoteRef::Id(i) => Some(*i),
            VoteRef::Obj(o) => o.id,
        })
        .or_else(|| row.option.as_ref().and_then(|o| o.vote))
}

/// Parse GovTrack `vote_voter` list JSON into stubs for detail fetching.
pub fn parse_vote_voter_list(json: &str) -> Vec<GovTrackVoteStub> {
    let Ok(parsed) = serde_json::from_str::<VoteVoterResponse>(json) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for row in parsed.objects.unwrap_or_default() {
        let position = row
            .option
            .as_ref()
            .and_then(|o| o.value.clone())
            .unwrap_or_else(|| "—".into());
        let date = row
            .created
            .as_deref()
            .map(|s| s.chars().take(10).collect::<String>())
            .unwrap_or_default();
        out.push(GovTrackVoteStub {
            vote_id: vote_id_from_row(&row),
            date,
            position,
        });
    }
    out
}

/// Parse a single GovTrack vote detail JSON into question/result/url.
pub fn parse_vote_detail_json(json: &str, vote_id: i64) -> (String, Option<String>, String) {
    let fallback_url = format!("https://www.govtrack.us/congress/votes/{vote_id}");
    let Ok(d) = serde_json::from_str::<VoteDetail>(json) else {
        return (format!("Vote #{vote_id}"), None, fallback_url);
    };
    let question = d
        .question
        .filter(|s| !s.is_empty())
        .or(d.question_details)
        .unwrap_or_else(|| format!("Vote #{vote_id}"));
    let result = d.result.filter(|s| !s.is_empty());
    let url = d.link.filter(|s| !s.is_empty()).unwrap_or(fallback_url);
    (question, result, url)
}

fn embedded_vote_fields(row: &VoteVoterRow, vote_id: Option<i64>) -> Option<(String, Option<String>, String)> {
    let VoteRef::Obj(o) = row.vote.as_ref()? else {
        return None;
    };
    let question = o
        .question
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| {
            o.question_details
                .as_ref()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
        })?;
    let result = o
        .result
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let url = o
        .link
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            vote_id
                .map(|id| format!("https://www.govtrack.us/congress/votes/{id}"))
                .unwrap_or_else(|| "https://www.govtrack.us/congress/votes".into())
        });
    Some((question, result, url))
}

/// Combine vote_voter list JSON with optional detail JSON bodies (keyed by vote id string).
/// Prefers fields embedded on list rows (GovTrack default); details map fills id-only rows.
/// `details_map_json` is a JSON object map: `{ "128911": "{...vote detail...}", ... }`.
pub fn assemble_govtrack_votes(list_json: &str, details_map_json: &str) -> Vec<VoteRecord> {
    let Ok(parsed) = serde_json::from_str::<VoteVoterResponse>(list_json) else {
        return Vec::new();
    };
    let details: HashMap<String, String> =
        serde_json::from_str(details_map_json).unwrap_or_default();
    let rows = parsed.objects.unwrap_or_default();
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let position = row
            .option
            .as_ref()
            .and_then(|o| o.value.clone())
            .unwrap_or_else(|| "—".into());
        let date = row
            .created
            .as_deref()
            .map(|s| s.chars().take(10).collect::<String>())
            .unwrap_or_default();
        let vote_id = vote_id_from_row(&row);
        let (question, result, url) = if let Some(emb) = embedded_vote_fields(&row, vote_id) {
            emb
        } else if let Some(vid) = vote_id {
            if let Some(body) = details.get(&vid.to_string()) {
                parse_vote_detail_json(body, vid)
            } else {
                (
                    format!("Vote #{vid}"),
                    None,
                    format!("https://www.govtrack.us/congress/votes/{vid}"),
                )
            }
        } else {
            (
                "Congressional vote".into(),
                None,
                "https://www.govtrack.us/congress/votes".into(),
            )
        };
        out.push(VoteRecord {
            date,
            question,
            position,
            result,
            url,
        });
    }
    out
}

/// Vote ids on the list page that lack an embedded question (need a detail fetch).
pub fn vote_ids_needing_detail(list_json: &str) -> Vec<i64> {
    let Ok(parsed) = serde_json::from_str::<VoteVoterResponse>(list_json) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for row in parsed.objects.unwrap_or_default() {
        let vote_id = vote_id_from_row(&row);
        if embedded_vote_fields(&row, vote_id).is_some() {
            continue;
        }
        if let Some(id) = vote_id {
            out.push(id);
        }
    }
    out
}

/// Lookup a single FEC id in a legislators-current.json body.
pub fn match_legislator_by_fec(legislators_json: &str, fec_id: &str) -> Option<LegislatorMatch> {
    let key = fec_id.trim().to_ascii_uppercase();
    if key.is_empty() {
        return None;
    }
    build_fec_index(legislators_json).remove(&key)
}

/// Bioguide profile URL when an id is known (no invented ids).
pub fn bioguide_profile_url(bioguide: &str) -> Option<String> {
    let id = bioguide.trim();
    if id.is_empty() {
        return None;
    }
    // Bioguide ids look like A000001 / G000565.
    if id.len() < 4 || !id.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    Some(format!("https://bioguide.congress.gov/search/bio/{id}"))
}

/// Normalize a congress-legislators term site URL to https origin (no path junk).
pub fn normalize_official_site_url(url: &str) -> String {
    let u = url.trim();
    let u = u
        .trim_start_matches("http://")
        .trim_start_matches("https://");
    let host_path = u.split('?').next().unwrap_or(u).trim_end_matches('/');
    // Keep host only for root member sites (bilirakis.house.gov).
    let host = host_path
        .split('/')
        .next()
        .unwrap_or(host_path)
        .trim()
        .trim_start_matches("www.");
    if host.is_empty() {
        return format!("https://{}", u.trim_end_matches('/'));
    }
    // Senate sites commonly use www.; house often bare subdomain.
    if host.ends_with(".senate.gov") && !host.starts_with("www.") {
        format!("https://www.{host}")
    } else {
        format!("https://{host}")
    }
}

/// Candidate About/biography paths on a member house.gov / senate.gov site.
pub fn official_about_urls(site_url: &str) -> Vec<String> {
    let base = normalize_official_site_url(site_url);
    if base.len() < 12 {
        return Vec::new();
    }
    let mut out = vec![
        format!("{base}/about"),
        format!("{base}/about/"),
        format!("{base}/biography"),
        format!("{base}/about/biography"),
        format!("{base}/about/bio"),
    ];
    // De-dupe while preserving order.
    let mut seen = std::collections::HashSet::new();
    out.retain(|u| seen.insert(u.clone()));
    out
}

/// Ballotpedia person page URL from congress-legislators `id.ballotpedia` title.
/// Spaces → underscores; already-encoded paths left alone.
pub fn ballotpedia_page_url(title: &str) -> Option<String> {
    let t = title.trim();
    if t.is_empty() {
        return None;
    }
    if t.starts_with("http://") || t.starts_with("https://") {
        return Some(t.to_string());
    }
    let slug = t
        .trim_start_matches('/')
        .replace(' ', "_");
    if slug.is_empty() {
        return None;
    }
    Some(format!("https://ballotpedia.org/{slug}"))
}

/// congress.gov member search by Bioguide id.
pub fn congress_gov_member_url(bioguide: &str) -> Option<String> {
    let id = bioguide.trim();
    if id.is_empty() {
        return None;
    }
    Some(format!(
        "https://www.congress.gov/member/{id}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_LEGISLATORS: &str = r#"[
      {
        "id": {
          "bioguide": "G000565",
          "govtrack": 412397,
          "wikidata": "Q1397095",
          "wikipedia": "Paul Gosar",
          "ballotpedia": "Paul Gosar",
          "fec": ["H0AZ01259", "H8AZ01189"]
        },
        "name": { "official_full": "Paul A. Gosar" },
        "terms": [
          {
            "type": "rep",
            "start": "2011-01-03",
            "end": "2013-01-03",
            "state": "AZ",
            "district": 1,
            "party": "Republican",
            "url": "http://gosar.house.gov"
          },
          {
            "type": "rep",
            "start": "2025-01-03",
            "end": "2027-01-03",
            "state": "AZ",
            "district": 9,
            "party": "Republican",
            "url": "https://gosar.house.gov"
          }
        ]
      }
    ]"#;

    #[test]
    fn fec_index_maps_ids_and_affiliations() {
        let map = build_fec_index(SAMPLE_LEGISLATORS);
        let m = map.get("H0AZ01259").expect("fec map");
        assert_eq!(m.govtrack_id, 412397);
        assert_eq!(m.wikidata.as_deref(), Some("Q1397095"));
        assert_eq!(m.wikipedia.as_deref(), Some("Paul Gosar"));
        assert_eq!(m.ballotpedia.as_deref(), Some("Paul Gosar"));
        assert_eq!(m.official_url.as_deref(), Some("https://gosar.house.gov"));
        assert_eq!(map.get("H8AZ01189").map(|x| x.govtrack_id), Some(412397));
        assert!(!m.affiliations.is_empty());
        // newest first
        assert!(m.affiliations[0].role.contains("AZ-9") || m.affiliations[0].role.contains("9"));
        assert_eq!(m.affiliations[0].party, "Republican");
    }

    #[test]
    fn parse_vote_voter_shapes() {
        let json = r#"{
          "objects": [
            {
              "created": "2026-07-23T10:32:00",
              "option": { "key": "+", "value": "Yea", "vote": 128911 },
              "vote": 128911
            }
          ]
        }"#;
        let p: VoteVoterResponse = serde_json::from_str(json).unwrap();
        let row = &p.objects.unwrap()[0];
        assert_eq!(row.option.as_ref().unwrap().value.as_deref(), Some("Yea"));
        match row.vote.as_ref().unwrap() {
            VoteRef::Id(i) => assert_eq!(*i, 128911),
            _ => panic!("expected id"),
        }
    }

    #[test]
    fn current_party_row() {
        let a = current_party_affiliation("Democratic", "Arizona Senate (District 5)");
        assert_eq!(a.len(), 1);
        assert!(a[0].role.contains("As filed"));
        assert_eq!(a[0].source.as_deref(), Some("Filing / ballot data"));
    }

    #[test]
    fn ballot_affiliations_filing_and_incumbent() {
        let a = ballot_affiliations(
            "Republican",
            "Florida Senate (District 10)",
            true,
            false,
            Some("Florida Division of Elections"),
            Some("https://dos.fl.gov/example"),
        );
        assert_eq!(a.len(), 2);
        assert!(a[0].role.starts_with("As filed"));
        assert!(a[1].role.starts_with("Incumbent"));
        assert_eq!(a[0].source.as_deref(), Some("Florida Division of Elections"));
        assert_eq!(
            a[0].source_url.as_deref(),
            Some("https://dos.fl.gov/example")
        );
        assert_eq!(a[1].party, "Republican");
    }

    #[test]
    fn ballot_affiliations_incumbent_only_no_party() {
        let a = ballot_affiliations("", "Circuit Judge", true, false, Some("FL DOS"), None);
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].party, "—");
        assert!(a[0].role.contains("Incumbent"));
    }

    #[test]
    fn ballot_affiliations_empty() {
        assert!(ballot_affiliations("", "Office", false, false, None, None).is_empty());
    }

    #[test]
    fn judicial_ctj_nonpartisan_seat_role() {
        let a = ballot_affiliations(
            "Independent / Other",
            "Circuit Judge (Circuit 18, Group 13)",
            false,
            true,
            Some("Florida Division of Elections"),
            Some("https://dos.fl.gov/elections/"),
        );
        assert!(!a.is_empty());
        assert_eq!(a[0].party, "Nonpartisan");
        assert!(a[0].role.contains("Ballot designation"));
        assert!(a[0].role.contains("Group 13 (seat)"));
        assert!(a[0].role.contains("Circuit 18"));
        assert_eq!(
            a[0].source.as_deref(),
            Some("Florida Division of Elections")
        );
        // Group is not treated as a party label.
        assert!(!a[0].party.to_ascii_lowercase().contains("group"));
    }

    #[test]
    fn judicial_coj_nonpartisan() {
        let a = ballot_affiliations(
            "Nonpartisan",
            "County Judge (Group 3)",
            false,
            true,
            Some("Florida Division of Elections"),
            None,
        );
        assert_eq!(a[0].party, "Nonpartisan");
        assert!(a[0].role.contains("Group 3 (seat)"));
    }

    #[test]
    fn judicial_dca_merit_retention() {
        let a = ballot_affiliations(
            "NOP",
            "District Court of Appeal (District 5)",
            true,
            true,
            Some("Florida Division of Elections"),
            None,
        );
        assert!(a.iter().any(|s| s.party == "Merit retention"));
        assert!(a.iter().any(|s| s.role.starts_with("Incumbent")));
        assert!(a[0].role.contains("Ballot designation"));
    }

    #[test]
    fn judicial_partisan_shows_party_and_designation() {
        // Some states elect judges with party labels.
        let a = ballot_affiliations(
            "Democratic",
            "Superior Court Judge",
            false,
            true,
            Some("NCSBE"),
            None,
        );
        assert!(a.iter().any(|s| s.party == "Democratic"));
        assert!(a.iter().any(|s| s.role.contains("Ballot designation")));
    }

    #[test]
    fn cl_affiliations_carry_source() {
        let map = build_fec_index(SAMPLE_LEGISLATORS);
        let m = map.get("H0AZ01259").expect("fec map");
        assert_eq!(m.affiliations[0].source.as_deref(), Some(CL_SOURCE));
        assert!(m.affiliations[0]
            .source_url
            .as_deref()
            .unwrap_or("")
            .contains("github.com"));
    }

    #[test]
    fn bioguide_urls() {
        assert!(bioguide_profile_url("G000565")
            .unwrap()
            .contains("G000565"));
        assert!(bioguide_profile_url("").is_none());
        assert!(bioguide_profile_url("  ").is_none());
        assert!(congress_gov_member_url("G000565")
            .unwrap()
            .contains("congress.gov"));
    }

    #[test]
    fn ballotpedia_urls() {
        assert_eq!(
            ballotpedia_page_url("Gus M. Bilirakis").as_deref(),
            Some("https://ballotpedia.org/Gus_M._Bilirakis")
        );
        assert_eq!(
            ballotpedia_page_url("Paul Gosar").as_deref(),
            Some("https://ballotpedia.org/Paul_Gosar")
        );
        assert!(ballotpedia_page_url("").is_none());
        assert_eq!(
            ballotpedia_page_url("https://ballotpedia.org/Foo").as_deref(),
            Some("https://ballotpedia.org/Foo")
        );
    }

    #[test]
    fn official_site_and_about_urls() {
        assert_eq!(
            normalize_official_site_url("http://bilirakis.house.gov"),
            "https://bilirakis.house.gov"
        );
        assert_eq!(
            normalize_official_site_url("http://gillibrand.senate.gov/"),
            "https://www.gillibrand.senate.gov"
        );
        let about = official_about_urls("http://bilirakis.house.gov");
        assert!(about.iter().any(|u| u.ends_with("/about")));
        assert!(about.iter().any(|u| u.contains("biography")));
    }

    #[test]
    fn challenger_no_cl_match_keeps_empty_option() {
        assert!(match_legislator_by_fec(SAMPLE_LEGISLATORS, "H8XX99999").is_none());
    }

    #[test]
    fn campaign_committee_affiliation_labeled() {
        let a = campaign_committee_affiliation(
            "FRIENDS OF EXAMPLE",
            "Principal campaign committee",
            Some("OpenFEC"),
            Some("https://www.fec.gov/data/committee/C001/"),
        )
        .expect("span");
        assert_eq!(a.party, "Committee");
        assert!(a.role.contains("≠ voter affiliation"));
        assert!(a.role.contains("FRIENDS OF EXAMPLE"));
        assert_eq!(a.source.as_deref(), Some("OpenFEC"));
        assert!(campaign_committee_affiliation("", "P", None, None).is_none());
    }

    #[test]
    fn campaign_committee_fl_account() {
        let a = campaign_committee_affiliation(
            "Account 88799",
            "FL DOS campaign account",
            Some("Florida Division of Elections"),
            Some("https://dos.elections.myflorida.com/candidates/CanDetail.asp?account=88799"),
        )
        .unwrap();
        assert_eq!(a.party, "Committee");
        assert!(a.role.contains("≠ voter affiliation"));
    }

    #[test]
    fn assemble_votes_with_details() {
        let list = r#"{
          "objects": [
            {
              "created": "2026-07-23T10:32:00",
              "option": { "value": "Yea", "vote": 128911 },
              "vote": 128911
            }
          ]
        }"#;
        let details = r#"{
          "128911": "{\"question\":\"On Passage\",\"result\":\"Passed\",\"link\":\"https://www.govtrack.us/congress/votes/128911\"}"
        }"#;
        let votes = assemble_govtrack_votes(list, details);
        assert_eq!(votes.len(), 1);
        assert_eq!(votes[0].position, "Yea");
        assert_eq!(votes[0].question, "On Passage");
        assert_eq!(votes[0].result.as_deref(), Some("Passed"));
        assert_eq!(vote_ids_needing_detail(list), vec![128911]);
    }

    #[test]
    fn assemble_votes_embedded_no_detail_fetch() {
        let list = r#"{
          "meta": { "limit": 2, "offset": 0, "total_count": 6351 },
          "objects": [
            {
              "created": "2013-01-01T22:57:00",
              "option": { "key": "-", "value": "No", "vote": 113121 },
              "vote": {
                "question": "H.R. 8 (112th): American Taxpayer Relief Act of 2012",
                "result": "Passed",
                "link": "https://www.govtrack.us/congress/votes/112-2012/h659"
              }
            }
          ]
        }"#;
        assert_eq!(vote_voter_total_count(list), Some(6351));
        assert!(vote_ids_needing_detail(list).is_empty());
        let votes = assemble_govtrack_votes(list, "{}");
        assert_eq!(votes.len(), 1);
        assert_eq!(votes[0].position, "No");
        assert!(votes[0].question.contains("Taxpayer Relief"));
        assert_eq!(votes[0].result.as_deref(), Some("Passed"));
        assert!(votes[0].url.contains("h659"));
    }
}
