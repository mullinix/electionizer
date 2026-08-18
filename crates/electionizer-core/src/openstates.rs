//! Pure Open States JSON helpers (no HTTP).
use crate::bio::{
    facts_from_openstates_person, openstates_external_ids, spans_from_openstates_person, BioFact,
    CareerSpan,
};
use crate::models::{normalize_party_label, AffiliationSpan, VoteRecord};
use serde::Serialize;
use serde_json::Value;

const OS_SOURCE: &str = "Open States";
const OS_SOURCE_HOME: &str = "https://openstates.org/";

#[derive(Debug, Clone, Serialize)]
pub struct StateLegislatorMatch {
    pub person_id: String,
    pub name: String,
    pub profile_url: String,
    pub jurisdiction: String,
    /// Party / role spans from the matched person payload (cited Open States).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affiliations: Vec<AffiliationSpan>,
    /// Public headshot when Open States provides `image`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// Birth year when present on the person payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub birth_year: Option<i32>,
    /// Political (and other) career spans for dossier assessment.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub career_spans: Vec<CareerSpan>,
    /// Occupation / education / family when present in OS extras (cited).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bio_facts: Vec<BioFact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wikidata: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wikipedia: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OsErrorKind {
    RateLimit,
    Server,
    Client,
    Network,
}

use std::time::Duration;
use anyhow::Error as AnyhowError;

/// Sessions to probe for votes given an election cycle year.
pub fn vote_sessions(cycle: i32) -> Vec<i32> {
    // Prefer prior legislative year (often has floor votes), then cycle, then cycle-2.
    vec![cycle - 1, cycle, cycle - 2]
}

pub fn backoff_delay(attempt: u32, rate_limit: bool) -> Duration {
    // ~6–12s for 429; shorter for 5xx.
    if rate_limit {
        Duration::from_secs(6 + u64::from(attempt.saturating_sub(1)) * 3)
    } else {
        Duration::from_secs(2 * u64::from(attempt))
    }
}

pub fn classify_os_error(err: &AnyhowError) -> OsErrorKind {
    let s = format!("{err:#}").to_ascii_lowercase();
    if s.contains("http 429") || s.contains("rate limit") || s.contains("exceeded limit") {
        OsErrorKind::RateLimit
    } else if s.contains("http 5") {
        OsErrorKind::Server
    } else if s.contains("network error") || s.contains("timed out") || s.contains("timeout") {
        OsErrorKind::Network
    } else {
        OsErrorKind::Client
    }
}

pub fn is_soft_fail(err: &AnyhowError) -> bool {
    matches!(
        classify_os_error(err),
        OsErrorKind::RateLimit | OsErrorKind::Server | OsErrorKind::Network
    )
}

/// True when the error message indicates OpenStates rate limiting (for UI copy).
pub fn is_rate_limit_error(err: &AnyhowError) -> bool {
    classify_os_error(err) == OsErrorKind::RateLimit
}

pub fn pick_person(
    json: &str,
    name: &str,
    want_org: &str,
    district: Option<u32>,
    state: &str,
) -> Option<StateLegislatorMatch> {
    let root: Value = serde_json::from_str(json).ok()?;
    let results = root
        .get("results")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if results.is_empty() {
        return None;
    }

    let want_last = last_name(name).to_ascii_lowercase();
    let mut best: Option<(i32, StateLegislatorMatch)> = None;

    for p in results {
        let pname = person_display_name(&p);
        let plast = last_name(&pname).to_ascii_lowercase();
        if plast != want_last && !pname.to_ascii_lowercase().contains(&want_last) {
            continue;
        }
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
        let role_district = role_district_num(&role);
        let mut score = 0;
        if plast == want_last {
            score += 10;
        }
        if org == want_org {
            score += 20;
        } else if !org.is_empty() {
            score -= 5;
        }
        if let (Some(want), Some(have)) = (district, role_district) {
            if want == have {
                score += 30;
            } else {
                score -= 10;
            }
        }
        if score < 10 {
            continue;
        }

        let person_id = p.get("id").and_then(|v| v.as_str())?.to_string();
        let profile_url = p
            .get("openstates_url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("https://openstates.org/person/{person_id}/"));
        let affiliations = affiliations_from_openstates_person(&p);
        let (birth_year, image_url, career_spans) = spans_from_openstates_person(&p);
        let bio_facts = facts_from_openstates_person(&p);
        let (wikidata, wikipedia) = openstates_external_ids(&p);
        let m = StateLegislatorMatch {
            person_id,
            name: pname,
            profile_url,
            jurisdiction: state.to_string(),
            affiliations,
            image_url,
            birth_year,
            career_spans,
            bio_facts,
            wikidata,
            wikipedia,
        };
        if best.as_ref().map(|(s, _)| *s).unwrap_or(i32::MIN) < score {
            best = Some((score, m));
        }
    }
    best.map(|(_, m)| m)
}

/// Rebuild match fields from a single Open States person JSON object (detail fetch).
pub fn state_legislator_from_person_json(person_json: &str, state: &str) -> Option<StateLegislatorMatch> {
    let p: Value = serde_json::from_str(person_json).ok()?;
    let person_id = p.get("id").and_then(|v| v.as_str())?.to_string();
    let name = person_display_name(&p);
    let profile_url = p
        .get("openstates_url")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("https://openstates.org/person/{person_id}/"));
    let affiliations = affiliations_from_openstates_person(&p);
    let (birth_year, image_url, career_spans) = spans_from_openstates_person(&p);
    let bio_facts = facts_from_openstates_person(&p);
    let (wikidata, wikipedia) = openstates_external_ids(&p);
    Some(StateLegislatorMatch {
        person_id,
        name,
        profile_url,
        jurisdiction: state.to_string(),
        affiliations,
        image_url,
        birth_year,
        career_spans,
        bio_facts,
        wikidata,
        wikipedia,
    })
}

/// Map an Open States person object to cited affiliation spans.
/// Uses `roles[]` when present; else `current_role` + top-level `party`.
/// Cite or omit — no span without a source name.
pub fn affiliations_from_openstates_person(person: &Value) -> Vec<AffiliationSpan> {
    let source_url = person
        .get("openstates_url")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            person
                .get("id")
                .and_then(|v| v.as_str())
                .map(|id| format!("https://openstates.org/person/{id}/"))
        })
        .unwrap_or_else(|| OS_SOURCE_HOME.into());

    let party_fallback = person_party_label(person);

    let mut roles: Vec<Value> = person
        .get("roles")
        .and_then(|r| r.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|r| r.is_object())
        .collect();

    if roles.is_empty() {
        if let Some(cr) = person.get("current_role").filter(|v| v.is_object()) {
            roles.push(cr.clone());
        }
    }

    let mut out = Vec::new();
    if !roles.is_empty() {
        // Prefer newest first (open end / latest start).
        roles.sort_by(|a, b| {
            let ae = role_end_key(a);
            let be = role_end_key(b);
            be.cmp(&ae).then_with(|| {
                let as_ = role_start_key(a);
                let bs = role_start_key(b);
                bs.cmp(&as_)
            })
        });
        for role in &roles {
            let party = role_party_label(role)
                .or_else(|| party_fallback.clone())
                .unwrap_or_else(|| "—".into());
            let label = os_role_label(role);
            if label.is_empty() && party == "—" {
                continue;
            }
            out.push(AffiliationSpan {
                party,
                start: role_date(role, "start_date").or_else(|| role_date(role, "start")),
                end: role_date(role, "end_date").or_else(|| role_date(role, "end")),
                role: if label.is_empty() {
                    "Legislative service".into()
                } else {
                    label
                },
                source: Some(OS_SOURCE.into()),
                source_url: Some(source_url.clone()),
            });
        }
    } else if let Some(party) = party_fallback {
        out.push(AffiliationSpan {
            party,
            start: None,
            end: None,
            role: "Open States party record".into(),
            source: Some(OS_SOURCE.into()),
            source_url: Some(source_url),
        });
    }
    out
}

/// Find person by id in a people list/geo payload and map affiliations.
pub fn affiliations_from_openstates_people_json(
    people_json: &str,
    person_id: &str,
) -> Vec<AffiliationSpan> {
    let want = person_id.trim();
    if want.is_empty() {
        return Vec::new();
    }
    let Ok(root) = serde_json::from_str::<Value>(people_json) else {
        return Vec::new();
    };
    // Accept list envelope or a bare person object.
    if root.get("id").and_then(|v| v.as_str()) == Some(want) {
        return affiliations_from_openstates_person(&root);
    }
    let results = root
        .get("results")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    for p in results {
        if p.get("id").and_then(|v| v.as_str()) == Some(want) {
            return affiliations_from_openstates_person(&p);
        }
    }
    Vec::new()
}

/// Append `extra` spans after `base` (no silent drop; both sources stay visible).
pub fn merge_affiliation_spans(
    base: &[AffiliationSpan],
    extra: &[AffiliationSpan],
) -> Vec<AffiliationSpan> {
    if extra.is_empty() {
        return base.to_vec();
    }
    if base.is_empty() {
        return extra.to_vec();
    }
    let mut out = base.to_vec();
    for e in extra {
        let dup = out.iter().any(|b| {
            b.party == e.party
                && b.role == e.role
                && b.start == e.start
                && b.end == e.end
                && b.source == e.source
        });
        if !dup {
            out.push(e.clone());
        }
    }
    out
}

fn person_party_label(person: &Value) -> Option<String> {
    // API: string. Bulk YAML-style JSON may use [{name, start_date, end_date}, ...].
    if let Some(s) = person.get("party").and_then(|v| v.as_str()) {
        let t = s.trim();
        if t.is_empty() {
            return None;
        }
        return Some(normalize_party_label(t));
    }
    let arr = person.get("party").and_then(|v| v.as_array())?;
    // Prefer current (no end_date), else last entry.
    let current = arr.iter().find(|p| {
        p.get("end_date")
            .or_else(|| p.get("end"))
            .and_then(|v| v.as_str())
            .map(|s| s.trim().is_empty())
            .unwrap_or(true)
    });
    let entry = current.or_else(|| arr.last())?;
    let name = entry
        .get("name")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())?;
    Some(normalize_party_label(name))
}

fn role_party_label(role: &Value) -> Option<String> {
    role.get("party")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(normalize_party_label)
}

fn role_date(role: &Value, key: &str) -> Option<String> {
    role.get(key)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

fn role_start_key(role: &Value) -> String {
    role_date(role, "start_date")
        .or_else(|| role_date(role, "start"))
        .unwrap_or_default()
}

fn role_end_key(role: &Value) -> String {
    // Open-ended current roles sort first when reversing end key.
    role_date(role, "end_date")
        .or_else(|| role_date(role, "end"))
        .unwrap_or_else(|| "9999-12-31".into())
}

fn os_role_label(role: &Value) -> String {
    let org = role
        .get("org_classification")
        .or_else(|| role.get("type"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    let title = role
        .get("title")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let district = role
        .get("district")
        .and_then(|d| {
            if let Some(s) = d.as_str() {
                let t = s.trim();
                if t.is_empty() {
                    None
                } else {
                    Some(t.to_string())
                }
            } else if let Some(n) = d.as_u64() {
                Some(n.to_string())
            } else {
                None
            }
        });

    let chamber = match org {
        "upper" => Some("State Senate"),
        "lower" => Some("State House"),
        "legislature" => Some("Legislature"),
        "executive" => Some("Executive"),
        "government" => Some("Government"),
        "governor" => Some("Governor"),
        "lt_governor" => Some("Lieutenant Governor"),
        "mayor" => Some("Mayor"),
        _ => None,
    };

    if let Some(ch) = chamber {
        return match (&title, &district) {
            (Some(t), Some(d)) if !t.eq_ignore_ascii_case(ch) => {
                format!("{ch} · {t} · District {d}")
            }
            (_, Some(d)) => format!("{ch} · District {d}"),
            (Some(t), None) => format!("{ch} · {t}"),
            _ => ch.to_string(),
        };
    }

    if let Some(t) = title {
        return match &district {
            Some(d) => format!("{t} · District {d}"),
            None => t.to_string(),
        };
    }

    if !org.is_empty() {
        return match &district {
            Some(d) => format!("{org} · District {d}"),
            None => org.to_string(),
        };
    }

    String::new()
}

pub fn extract_votes_for_person(json: &str, person_id: &str, limit: usize) -> Vec<VoteRecord> {
    let Ok(root) = serde_json::from_str::<Value>(json) else {
        return Vec::new();
    };
    let bills = root
        .get("results")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut out = Vec::new();
    for bill in bills {
        let bill_id = bill
            .get("identifier")
            .and_then(|v| v.as_str())
            .unwrap_or("Bill");
        let title = bill
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        let bill_url = bill
            .get("openstates_url")
            .and_then(|v| v.as_str())
            .unwrap_or("https://openstates.org/");
        let votes = bill
            .get("votes")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        for vote in votes {
            let voters = vote
                .get("votes")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let mut position = None;
            for v in &voters {
                let vid = v
                    .get("voter_id")
                    .or_else(|| v.get("voter").and_then(|x| x.get("id")))
                    .and_then(|x| x.as_str())
                    .unwrap_or("");
                if vid == person_id {
                    position = v
                        .get("option")
                        .and_then(|x| x.as_str())
                        .map(normalize_position);
                    break;
                }
            }
            let Some(position) = position else {
                continue;
            };
            let motion = vote
                .get("motion_text")
                .or_else(|| vote.get("motion"))
                .and_then(|v| v.as_str())
                .unwrap_or("Floor vote");
            let date = vote
                .get("start_date")
                .or_else(|| vote.get("date"))
                .and_then(|v| v.as_str())
                .map(|s| s.chars().take(10).collect::<String>())
                .unwrap_or_default();
            let result = vote
                .get("result")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let question = if title.is_empty() {
                format!("{bill_id}: {motion}")
            } else {
                format!("{bill_id} — {title} ({motion})")
            };
            let url = vote
                .get("openstates_url")
                .and_then(|v| v.as_str())
                .unwrap_or(bill_url)
                .to_string();
            out.push(VoteRecord {
                date,
                question,
                position,
                result,
                url,
            });
            if out.len() >= limit {
                return out;
            }
        }
    }
    out
}

pub fn person_display_name(p: &Value) -> String {
    p.get("name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            let family = p.pointer("/family_name").and_then(|v| v.as_str())?;
            let given = p.pointer("/given_name").and_then(|v| v.as_str()).unwrap_or("");
            if given.is_empty() {
                Some(family.to_string())
            } else {
                Some(format!("{given} {family}"))
            }
        })
        .unwrap_or_else(|| "Legislator".into())
}

pub fn role_district_num(role: &Value) -> Option<u32> {
    let d = role.get("district")?;
    if let Some(n) = d.as_u64() {
        return u32::try_from(n).ok();
    }
    if let Some(s) = d.as_str() {
        return s
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse()
            .ok();
    }
    None
}

pub fn last_name(name: &str) -> String {
    let cleaned = name
        .split("--")
        .next()
        .unwrap_or(name)
        .trim()
        .trim_end_matches(',');
    // "Last, First" or "First Last"
    if cleaned.contains(',') {
        cleaned
            .split(',')
            .next()
            .unwrap_or(cleaned)
            .trim()
            .to_string()
    } else {
        cleaned
            .split_whitespace()
            .last()
            .unwrap_or(cleaned)
            .to_string()
    }
}

pub fn normalize_name_key(name: &str) -> String {
    last_name(name)
        .to_ascii_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

pub fn normalize_position(raw: &str) -> String {
    match raw.trim().to_ascii_lowercase().as_str() {
        "yes" | "yea" | "aye" | "y" => "Yea".into(),
        "no" | "nay" | "n" => "Nay".into(),
        "abstain" | "present" | "not voting" | "absent" | "excused" => {
            let mut c = raw.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => raw.to_string(),
            }
        }
        _ => {
            let t = raw.trim();
            if t.is_empty() {
                "—".into()
            } else {
                t.to_string()
            }
        }
    }
}

pub fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max).collect();
        format!("{t}…")
    }
}

/// Parse `District N` from office labels like `Arizona Senate (District 5)`.
pub fn district_from_office(office: &str) -> Option<u32> {
    let re = regex::Regex::new(r"(?i)district\s+(\d+)").ok()?;
    re.captures(office)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

/// USPS state from jurisdiction label ("Arizona") or chamber OCD-ish text.
pub fn state_code_from_jurisdiction(jurisdiction: &str, office: &str) -> Option<String> {
    let j = jurisdiction.to_ascii_lowercase();
    let o = office.to_ascii_lowercase();
    let table = [
        ("arizona", "AZ"),
        ("florida", "FL"),
        ("california", "CA"),
        ("new york", "NY"),
        ("texas", "TX"),
    ];
    for (name, code) in table {
        if j.contains(name) || o.contains(name) {
            return Some(code.into());
        }
    }
    // "AZ-…" style
    None
}

/// FEC-shaped ids vs namespaced state ids (`azleg:2371`).
pub fn looks_like_fec_id(id: &str) -> bool {
    let id = id.trim();
    if id.is_empty() || id.contains(':') {
        return false;
    }
    let bytes = id.as_bytes();
    if bytes.len() < 8 {
        return false;
    }
    bytes[0].is_ascii_alphabetic() && id.chars().all(|c| c.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PEOPLE_JSON: &str = r#"{
      "results": [
        {
          "id": "ocd-person/aaa",
          "name": "Lela Alston",
          "party": "Democratic",
          "openstates_url": "https://openstates.org/person/lela-alston/",
          "image": "https://example.com/lela.jpg",
          "birth_date": "1942-06-26",
          "extras": { "occupation": "Educator", "spouse": "Alex Alston" },
          "identifiers": [
            { "scheme": "wikidata", "identifier": "Q16730315" },
            { "scheme": "wikipedia", "identifier": "Lela_Alston" }
          ],
          "current_role": {
            "title": "Senator",
            "org_classification": "upper",
            "district": "5"
          }
        },
        {
          "id": "ocd-person/bbb",
          "name": "Someone Else",
          "current_role": {
            "org_classification": "lower",
            "district": "5"
          }
        }
      ]
    }"#;

    const PEOPLE_ROLES_JSON: &str = r#"{
      "results": [
        {
          "id": "ocd-person/ccc",
          "name": "Alma Hernandez",
          "party": "Democratic",
          "openstates_url": "https://openstates.org/person/alma-hernandez/",
          "roles": [
            {
              "type": "lower",
              "district": "3",
              "start_date": "2019-01-14",
              "end_date": "2023-01-09"
            },
            {
              "type": "lower",
              "district": "20",
              "start_date": "2023-01-09"
            }
          ]
        }
      ]
    }"#;

    const BILLS_JSON: &str = r#"{
      "results": [
        {
          "identifier": "HB 2001",
          "title": "elections early voting",
          "openstates_url": "https://openstates.org/bills/az/2026/HB2001/",
          "votes": [
            {
              "motion_text": "Third Reading",
              "start_date": "2026-02-10",
              "result": "pass",
              "openstates_url": "https://openstates.org/bills/az/2026/HB2001/#vote-1",
              "votes": [
                { "voter_id": "ocd-person/aaa", "option": "yes" },
                { "voter_id": "ocd-person/zzz", "option": "no" }
              ]
            }
          ]
        },
        {
          "identifier": "SB 10",
          "title": "other bill",
          "votes": [
            {
              "motion_text": "Final Passage",
              "start_date": "2026-01-05",
              "result": "fail",
              "votes": [
                { "voter_id": "ocd-person/aaa", "option": "no" }
              ]
            }
          ]
        }
      ]
    }"#;

    #[test]
    fn pick_person_by_district_and_chamber() {
        let m = pick_person(PEOPLE_JSON, "Lela Alston", "upper", Some(5), "az").unwrap();
        assert_eq!(m.person_id, "ocd-person/aaa");
        assert!(m.profile_url.contains("openstates.org"));
        assert_eq!(m.affiliations.len(), 1);
        assert_eq!(m.affiliations[0].party, "Democratic");
        assert!(m.affiliations[0].role.contains("Senate"));
        assert_eq!(m.birth_year, Some(1942));
        assert!(m.image_url.as_deref().unwrap_or("").contains("lela.jpg"));
        assert_eq!(m.wikidata.as_deref(), Some("Q16730315"));
        assert_eq!(m.wikipedia.as_deref(), Some("Lela Alston"));
        assert!(m.bio_facts.iter().any(|f| f.kind == "work" && f.text.contains("Educator")));
        assert!(m.bio_facts.iter().any(|f| f.kind == "family" && f.text.contains("Alex")));
        let detail = state_legislator_from_person_json(
            r#"{"id":"ocd-person/aaa","name":"Lela Alston","extras":{"education":"ASU"},"links":[{"url":"https://www.wikidata.org/wiki/Q16730315"}]}"#,
            "az",
        )
        .unwrap();
        assert!(detail.bio_facts.iter().any(|f| f.kind == "education"));
        assert_eq!(detail.wikidata.as_deref(), Some("Q16730315"));
        assert_eq!(
            m.affiliations[0].source.as_deref(),
            Some("Open States")
        );
        assert!(m.affiliations[0]
            .source_url
            .as_deref()
            .unwrap_or("")
            .contains("openstates.org"));
    }

    #[test]
    fn openstates_affiliations_from_roles_history() {
        let spans =
            affiliations_from_openstates_people_json(PEOPLE_ROLES_JSON, "ocd-person/ccc");
        assert_eq!(spans.len(), 2);
        // Newest first (open-ended district 20 before ended district 3).
        assert!(spans[0].role.contains("District 20"));
        assert_eq!(spans[0].end, None);
        assert!(spans[1].role.contains("District 3"));
        assert_eq!(spans[1].end.as_deref(), Some("2023-01-09"));
        assert_eq!(spans[0].party, "Democratic");
        assert_eq!(spans[0].source.as_deref(), Some("Open States"));
    }

    #[test]
    fn openstates_affiliations_party_only() {
        let person: Value = serde_json::from_str(
            r#"{"id":"ocd-person/x","party":"Republican","openstates_url":"https://openstates.org/person/x/"}"#,
        )
        .unwrap();
        let spans = affiliations_from_openstates_person(&person);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].party, "Republican");
        assert!(spans[0].role.contains("party"));
    }

    #[test]
    fn merge_affiliation_keeps_ballot_and_os() {
        use crate::govtrack::ballot_affiliations;
        let base = ballot_affiliations(
            "Democratic",
            "State Senate · District 5",
            true,
            false,
            Some("AZ Legislature"),
            Some("https://www.azleg.gov/"),
        );
        let os = affiliations_from_openstates_people_json(PEOPLE_JSON, "ocd-person/aaa");
        let merged = merge_affiliation_spans(&base, &os);
        assert!(merged.len() >= 3); // filing + incumbent + OS role
        assert!(merged.iter().any(|s| s.source.as_deref() == Some("AZ Legislature")));
        assert!(merged.iter().any(|s| s.source.as_deref() == Some("Open States")));
    }

    #[test]
    fn extract_votes_filters_person() {
        let rows = extract_votes_for_person(BILLS_JSON, "ocd-person/aaa", 12);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].position, "Yea");
        assert!(rows[0].question.contains("HB 2001"));
        assert_eq!(rows[1].position, "Nay");
    }

    #[test]
    fn district_and_fec_helpers() {
        assert_eq!(
            district_from_office("Arizona Senate (District 5)"),
            Some(5)
        );
        assert_eq!(
            state_code_from_jurisdiction("Arizona", "Arizona House (District 8)")
                .as_deref(),
            Some("AZ")
        );
        assert!(looks_like_fec_id("H8FL08042"));
        assert!(!looks_like_fec_id("azleg:2371"));
        assert!(!looks_like_fec_id(""));
    }

    #[test]
    fn last_name_parsing() {
        assert_eq!(last_name("Lela Alston"), "Alston");
        assert_eq!(last_name("Alston, Lela"), "Alston");
    }

    #[test]
    fn rate_limit_body_is_retryable() {
        let err = anyhow::anyhow!(
            "OpenStates HTTP 429: {{\"detail\":\"Request was throttled. Expected available in 48 seconds. exceeded limit of 10/min\"}}"
        );
        assert_eq!(classify_os_error(&err), OsErrorKind::RateLimit);
        assert!(is_soft_fail(&err));
        assert!(is_rate_limit_error(&err));
    }

    #[test]
    fn server_error_is_soft_fail() {
        let err = anyhow::anyhow!("OpenStates HTTP 503: unavailable");
        assert_eq!(classify_os_error(&err), OsErrorKind::Server);
        assert!(is_soft_fail(&err));
    }

    #[test]
    fn client_error_is_hard() {
        let err = anyhow::anyhow!("OpenStates HTTP 401: invalid key");
        assert_eq!(classify_os_error(&err), OsErrorKind::Client);
        assert!(!is_soft_fail(&err));
    }

    #[test]
    fn vote_sessions_order() {
        assert_eq!(vote_sessions(2026), vec![2025, 2026, 2024]);
    }

    #[test]
    fn empty_bills_votes_yields_empty() {
        let json = r#"{"results":[{"identifier":"HB1","title":"x","votes":[]}]}"#;
        let rows = extract_votes_for_person(json, "ocd-person/aaa", 12);
        assert!(rows.is_empty());
    }
}
