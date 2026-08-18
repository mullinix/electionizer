//! Pure OpenFEC JSON parsers (no HTTP / cache).
use crate::bio::BioFact;
use crate::models::{
    format_usd, CandidateFinance, CommitteeLink, ContributorRow, OutsideSpendRow, SizeBucketRow,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct TotalsResponse {
    results: Option<Vec<TotalsRow>>,
}

#[derive(Debug, Deserialize)]
struct TotalsRow {
    cycle: Option<i32>,
    receipts: Option<f64>,
    disbursements: Option<f64>,
    cash_on_hand_end_period: Option<f64>,
    debts_owed_by_committee: Option<f64>,
    coverage_end_date: Option<String>,
    individual_itemized_contributions: Option<f64>,
    individual_unitemized_contributions: Option<f64>,
    other_political_committee_contributions: Option<f64>,
    political_party_committee_contributions: Option<f64>,
}

pub fn parse_totals_json(json: &str, candidate_id: &str, cycle: i32) -> Option<CandidateFinance> {
    let parsed: TotalsResponse = serde_json::from_str(json).ok()?;
    let rows = parsed.results.unwrap_or_default();
    if rows.is_empty() {
        return None;
    }
    let row = rows
        .iter()
        .find(|r| r.cycle == Some(cycle))
        .or_else(|| rows.first())?;

    let receipts = row.receipts;
    let disbursements = row.disbursements;
    let cash = row.cash_on_hand_end_period;
    let debts = row.debts_owed_by_committee;
    let debts_display = debts.and_then(|d| {
        if d > 0.5 {
            Some(format_usd(d))
        } else {
            None
        }
    });

    let individual = match (
        row.individual_itemized_contributions,
        row.individual_unitemized_contributions,
    ) {
        (Some(a), Some(b)) => Some(a + b),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    };

    Some(CandidateFinance {
        cycle: row.cycle.unwrap_or(cycle),
        receipts,
        disbursements,
        cash_on_hand: cash,
        debts,
        coverage_end_date: row.coverage_end_date.clone(),
        receipts_display: receipts.map(format_usd).unwrap_or_else(|| "—".into()),
        disbursements_display: disbursements.map(format_usd).unwrap_or_else(|| "—".into()),
        cash_on_hand_display: cash.map(format_usd).unwrap_or_else(|| "—".into()),
        debts_display,
        individual_display: individual.map(format_usd),
        pac_display: row.other_political_committee_contributions.map(format_usd),
        party_display: row.political_party_committee_contributions.map(format_usd),
        profile_url: format!("https://www.fec.gov/data/candidate/{candidate_id}/"),
        source_label: "Federal Election Commission OpenFEC".into(),
    })
}

#[derive(Debug, Deserialize)]
struct IeResponse {
    results: Option<Vec<IeRow>>,
}

#[derive(Debug, Deserialize)]
struct IeRow {
    committee_name: Option<String>,
    committee_id: Option<String>,
    total: Option<f64>,
    support_oppose_indicator: Option<String>,
    #[serde(default)]
    support_oppose: Option<String>,
}

pub fn parse_ie_json(json: &str) -> Vec<OutsideSpendRow> {
    let Ok(parsed) = serde_json::from_str::<IeResponse>(json) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for row in parsed.results.unwrap_or_default() {
        let amount = row.total.unwrap_or(0.0);
        if amount < 0.5 {
            continue;
        }
        let committee = row
            .committee_name
            .filter(|s| !s.trim().is_empty())
            .or_else(|| row.committee_id.clone())
            .unwrap_or_else(|| "Unknown committee".into());
        let so_raw = row
            .support_oppose_indicator
            .or(row.support_oppose)
            .unwrap_or_default();
        let support_oppose = match so_raw.trim().to_ascii_uppercase().as_str() {
            "S" | "SUPPORT" => "Support".into(),
            "O" | "OPPOSE" => "Oppose".into(),
            other if !other.is_empty() => other.to_string(),
            _ => "—".into(),
        };
        let url = match row.committee_id.as_deref() {
            Some(id) if !id.is_empty() => {
                format!("https://www.fec.gov/data/committee/{id}/")
            }
            _ => "https://www.fec.gov/data/".into(),
        };
        out.push(OutsideSpendRow {
            committee,
            amount_display: format_usd(amount),
            support_oppose,
            url,
        });
    }
    out
}

#[derive(Debug, Deserialize)]
struct SchedAResponse {
    results: Option<Vec<SchedARow>>,
}

#[derive(Debug, Deserialize)]
struct SchedARow {
    contributor_name: Option<String>,
    contributor_city: Option<String>,
    contributor_state: Option<String>,
    contribution_receipt_amount: Option<f64>,
    contribution_receipt_date: Option<String>,
    committee_id: Option<String>,
    contributor_id: Option<String>,
    pdf_url: Option<String>,
    memo_code: Option<String>,
    contributor_occupation: Option<String>,
    contributor_employer: Option<String>,
}

fn sched_a_location(row: &SchedARow) -> Option<String> {
    match (
        row.contributor_city
            .as_ref()
            .filter(|s| !s.trim().is_empty())
            .cloned(),
        row.contributor_state
            .as_ref()
            .filter(|s| !s.trim().is_empty())
            .cloned(),
    ) {
        (Some(c), Some(st)) => Some(format!("{c}, {st}")),
        (Some(c), None) => Some(c),
        (None, Some(st)) => Some(st),
        (None, None) => None,
    }
}

fn sched_a_url(row: &SchedARow) -> String {
    row.pdf_url
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .cloned()
        .or_else(|| {
            row.contributor_id
                .as_ref()
                .filter(|id| !id.is_empty())
                .map(|id| format!("https://www.fec.gov/data/committee/{id}/"))
        })
        .or_else(|| {
            row.committee_id
                .as_ref()
                .filter(|id| !id.is_empty())
                .map(|id| format!("https://www.fec.gov/data/committee/{id}/"))
        })
        .unwrap_or_else(|| "https://www.fec.gov/data/".into())
}

/// Largest single Schedule A lines (not unique-donor totals).
pub fn parse_sched_a_json(json: &str, limit: usize) -> Vec<ContributorRow> {
    let Ok(parsed) = serde_json::from_str::<SchedAResponse>(json) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for row in parsed.results.unwrap_or_default() {
        if out.len() >= limit {
            break;
        }
        // Skip pure memo lines when flagged.
        if row
            .memo_code
            .as_deref()
            .is_some_and(|m| m.eq_ignore_ascii_case("X"))
        {
            continue;
        }
        let amount = row.contribution_receipt_amount.unwrap_or(0.0);
        if amount < 0.5 {
            continue;
        }
        let location = sched_a_location(&row);
        let url = sched_a_url(&row);
        let date = row
            .contribution_receipt_date
            .as_ref()
            .filter(|s| !s.trim().is_empty())
            .cloned();
        let name = row
            .contributor_name
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .unwrap_or("Unknown contributor")
            .to_string();
        out.push(ContributorRow {
            name,
            amount_display: format_usd(amount),
            location,
            date,
            url,
            gift_count: None,
        });
    }
    out
}

/// Top contributors by summed itemized Schedule A amount (unique name key).
/// Prefers larger `per_page` fetches; memo/zero rows skipped.
pub fn parse_sched_a_aggregated(json: &str, limit: usize) -> Vec<ContributorRow> {
    let Ok(parsed) = serde_json::from_str::<SchedAResponse>(json) else {
        return Vec::new();
    };
    use std::collections::HashMap;
    struct Agg {
        name: String,
        total: f64,
        count: u32,
        location: Option<String>,
        date: Option<String>,
        url: String,
    }
    let mut map: HashMap<String, Agg> = HashMap::new();
    for row in parsed.results.unwrap_or_default() {
        if row
            .memo_code
            .as_deref()
            .is_some_and(|m| m.eq_ignore_ascii_case("X"))
        {
            continue;
        }
        let amount = row.contribution_receipt_amount.unwrap_or(0.0);
        if amount < 0.5 {
            continue;
        }
        let name = row
            .contributor_name
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("Unknown contributor")
            .to_string();
        let key = name.to_ascii_uppercase();
        let loc = sched_a_location(&row);
        let date = row
            .contribution_receipt_date
            .as_ref()
            .filter(|s| !s.trim().is_empty())
            .cloned();
        let url = sched_a_url(&row);
        let entry = map.entry(key).or_insert_with(|| Agg {
            name: name.clone(),
            total: 0.0,
            count: 0,
            location: loc.clone(),
            date: date.clone(),
            url: url.clone(),
        });
        entry.total += amount;
        entry.count += 1;
        if entry.location.is_none() {
            entry.location = loc;
        }
        // Keep most recent date string when both present (lexicographic ISO works).
        if let Some(ref d) = date {
            if entry.date.as_ref().map(|e| d > e).unwrap_or(true) {
                entry.date = Some(d.clone());
            }
        }
        if entry.url.contains("fec.gov/data/") && !url.contains("pdf") {
            /* keep */
        } else if url.contains("pdf") {
            entry.url = url;
        }
    }
    let mut rows: Vec<Agg> = map.into_values().collect();
    rows.sort_by(|a, b| {
        b.total
            .partial_cmp(&a.total)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.name.cmp(&b.name))
    });
    rows.into_iter()
        .take(limit)
        .map(|a| ContributorRow {
            name: a.name,
            amount_display: format_usd(a.total),
            location: a.location,
            date: a.date,
            url: a.url,
            gift_count: if a.count > 1 { Some(a.count) } else { None },
        })
        .collect()
}

/// G4: occupation/employer from Schedule A when the **candidate** appears as contributor
/// (self-gifts and transfers often carry Form 3 occupation lines). Form 2 has no occupation field.
/// Cite or omit — never invent.
pub fn bio_facts_from_sched_a_candidate(json: &str, candidate_name: &str) -> Vec<BioFact> {
    const SRC: &str = "OpenFEC Schedule A";
    let Ok(parsed) = serde_json::from_str::<SchedAResponse>(json) else {
        return Vec::new();
    };
    let want = normalize_person_key(candidate_name);
    if want.0.is_empty() {
        return Vec::new();
    }
    let mut occupation: Option<(String, String)> = None; // text, url
    let mut employer: Option<(String, String)> = None;
    for row in parsed.results.unwrap_or_default() {
        let cname = row
            .contributor_name
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("");
        if cname.is_empty() || !contributor_matches_candidate(cname, &want) {
            continue;
        }
        let url = sched_a_url(&row);
        if let Some(occ) = row
            .contributor_occupation
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .filter(|s| !is_placeholder_occupation(s))
        {
            if occupation.is_none() {
                occupation = Some((occ.to_string(), url.clone()));
            }
        }
        if let Some(emp) = row
            .contributor_employer
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .filter(|s| !is_placeholder_occupation(s))
        {
            if employer.is_none() {
                employer = Some((emp.to_string(), url));
            }
        }
        if occupation.is_some() && employer.is_some() {
            break;
        }
    }
    let mut facts = Vec::new();
    if let Some((occ, url)) = occupation {
        facts.push(BioFact {
            kind: "work".into(),
            text: format!("Occupation (FEC itemized): {occ}"),
            source: SRC.into(),
            source_url: Some(url),
                ..Default::default()
            });
    }
    if let Some((emp, url)) = employer {
        facts.push(BioFact {
            kind: "work".into(),
            text: format!("Employer (FEC itemized): {emp}"),
            source: SRC.into(),
            source_url: Some(url),
                ..Default::default()
            });
    }
    facts
}

fn normalize_person_key(name: &str) -> (String, Vec<String>) {
    // Returns (last, first_tokens). Handles "First Last" and FEC "LAST, FIRST M".
    let clean_tok = |t: &str| -> Option<String> {
        let u = t
            .chars()
            .filter(|c| c.is_ascii_alphabetic())
            .collect::<String>()
            .to_ascii_uppercase();
        if u.len() < 2 {
            return None;
        }
        if matches!(
            u.as_str(),
            "JR" | "SR" | "II" | "III" | "IV" | "MD" | "ESQ"
        ) {
            return None;
        }
        Some(u)
    };

    if let Some((last_raw, rest)) = name.split_once(',') {
        let last = clean_tok(last_raw.trim()).unwrap_or_default();
        let firsts: Vec<String> = rest.split_whitespace().filter_map(clean_tok).collect();
        if !last.is_empty() {
            return (last, firsts);
        }
    }

    let cleaned: Vec<String> = name.split_whitespace().filter_map(clean_tok).collect();
    if cleaned.is_empty() {
        return (String::new(), Vec::new());
    }
    let last = cleaned.last().cloned().unwrap_or_default();
    let firsts = if cleaned.len() > 1 {
        cleaned[..cleaned.len() - 1].to_vec()
    } else {
        Vec::new()
    };
    (last, firsts)
}

fn contributor_matches_candidate(contributor: &str, want: &(String, Vec<String>)) -> bool {
    let (want_last, want_firsts) = want;
    // FEC: "DOE, JANE A" or "JANE DOE"
    let parts: Vec<String> = contributor
        .replace(',', " ")
        .split_whitespace()
        .map(|t| {
            t.chars()
                .filter(|c| c.is_ascii_alphabetic())
                .collect::<String>()
                .to_ascii_uppercase()
        })
        .filter(|t| t.len() >= 2)
        .collect();
    if parts.is_empty() {
        return false;
    }
    let has_last = parts.iter().any(|p| p == want_last);
    if !has_last {
        return false;
    }
    if want_firsts.is_empty() {
        return true;
    }
    // At least one first/middle token must appear.
    want_firsts.iter().any(|f| parts.iter().any(|p| p == f || p.starts_with(f) || f.starts_with(p)))
}

fn is_placeholder_occupation(s: &str) -> bool {
    let u = s.trim().to_ascii_uppercase();
    matches!(
        u.as_str(),
        "" | "NONE"
            | "N/A"
            | "NA"
            | "NOT APPLICABLE"
            | "INFORMATION REQUESTED"
            | "INFO REQUESTED"
            | "INFORMATION REQUESTED PER BEST EFFORTS"
            | "BEST EFFORTS"
    ) || u.contains("INFORMATION REQUESTED")
}

#[derive(Debug, Deserialize)]
struct SizeResponse {
    results: Option<Vec<SizeRow>>,
}

#[derive(Debug, Deserialize)]
struct SizeRow {
    size: Option<i64>,
    total: Option<f64>,
    count: Option<i64>,
}

pub fn parse_size_json(json: &str) -> Vec<SizeBucketRow> {
    let Ok(parsed) = serde_json::from_str::<SizeResponse>(json) else {
        return Vec::new();
    };
    let mut rows: Vec<(i64, f64, i64)> = parsed
        .results
        .unwrap_or_default()
        .into_iter()
        .filter_map(|r| {
            let size = r.size.unwrap_or(0);
            let total = r.total.unwrap_or(0.0);
            let count = r.count.unwrap_or(0);
            if total < 0.5 && count == 0 {
                return None;
            }
            Some((size, total, count))
        })
        .collect();
    rows.sort_by_key(|(size, _, _)| *size);
    rows.into_iter()
        .map(|(size, total, count)| SizeBucketRow {
            label: size_bucket_label(size),
            total_display: format_usd(total),
            count_display: format_count(count),
        })
        .collect()
}

/// OpenFEC size codes: 0 = $200 and under, 200 = $200.01–$499, etc.
pub fn size_bucket_label(size: i64) -> String {
    match size {
        0 => "$200 and under".into(),
        200 => "$200.01 – $499".into(),
        500 => "$500 – $999".into(),
        1000 => "$1,000 – $1,999".into(),
        2000 => "$2,000 and over".into(),
        other => format!("Size code {other}"),
    }
}

pub fn format_count(n: i64) -> String {
    let neg = n < 0;
    let s = n.unsigned_abs().to_string();
    let bytes = s.as_bytes();
    let mut out = String::new();
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(*b as char);
    }
    if neg {
        format!("-{out}")
    } else {
        out
    }
}

#[derive(Debug, Deserialize)]
struct CommitteesResponse {
    results: Option<Vec<CommitteeRow>>,
}

#[derive(Debug, Deserialize)]
struct CommitteeRow {
    committee_id: Option<String>,
    name: Option<String>,
    designation: Option<String>,
    designation_full: Option<String>,
}

pub fn parse_principal_committee(json: &str) -> Option<CommitteeLink> {
    let parsed: CommitteesResponse = serde_json::from_str(json).ok()?;
    let rows = parsed.results.unwrap_or_default();
    let row = rows
        .iter()
        .find(|r| {
            r.designation
                .as_deref()
                .is_some_and(|d| d.eq_ignore_ascii_case("P"))
        })
        .or_else(|| rows.first())?;
    let committee_id = row.committee_id.as_deref().filter(|s| !s.is_empty())?;
    let name = row
        .name
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(committee_id)
        .to_string();
    let designation = row
        .designation_full
        .clone()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            row.designation.as_ref().map(|d| match d.as_str() {
                "P" => "Principal campaign committee".into(),
                "A" => "Authorized committee".into(),
                "J" => "Joint fundraising committee".into(),
                other => other.to_string(),
            })
        })
        .unwrap_or_else(|| "Campaign committee".into());
    Some(CommitteeLink {
        name,
        committee_id: committee_id.to_string(),
        designation,
        url: format!("https://www.fec.gov/data/committee/{committee_id}/"),
    })
}

pub fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max).collect();
        format!("{t}…")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sample_totals() {
        let json = r#"{
          "results": [
            {
              "cycle": 2026,
              "receipts": 1234567.89,
              "disbursements": 500000.0,
              "cash_on_hand_end_period": 700000.5,
              "debts_owed_by_committee": 0.0,
              "coverage_end_date": "2026-06-30",
              "individual_itemized_contributions": 800000.0,
              "individual_unitemized_contributions": 50000.0,
              "other_political_committee_contributions": 120000.0,
              "political_party_committee_contributions": 10000.0
            }
          ]
        }"#;
        let f = parse_totals_json(json, "H8FL08042", 2026).expect("totals");
        assert_eq!(f.cycle, 2026);
        assert_eq!(f.receipts_display, "$1,234,568");
        assert_eq!(f.disbursements_display, "$500,000");
        assert!(f.debts_display.is_none());
        assert_eq!(f.individual_display.as_deref(), Some("$850,000"));
        assert_eq!(f.pac_display.as_deref(), Some("$120,000"));
        assert_eq!(f.party_display.as_deref(), Some("$10,000"));
        assert!(f.profile_url.contains("H8FL08042"));
    }

    #[test]
    fn prefers_matching_cycle() {
        let json = r#"{
          "results": [
            {"cycle": 2024, "receipts": 1.0, "disbursements": 1.0, "cash_on_hand_end_period": 1.0},
            {"cycle": 2026, "receipts": 99.0, "disbursements": 50.0, "cash_on_hand_end_period": 40.0}
          ]
        }"#;
        let f = parse_totals_json(json, "S0FL00338", 2026).unwrap();
        assert_eq!(f.receipts, Some(99.0));
        assert!(f.individual_display.is_none());
    }

    #[test]
    fn parse_ie_rows() {
        let json = r#"{
          "results": [
            {
              "committee_name": "Super PAC One",
              "committee_id": "C00401224",
              "total": 250000.0,
              "support_oppose_indicator": "S"
            },
            {
              "committee_name": "Oppose Group",
              "committee_id": "C00999999",
              "total": 10000.5,
              "support_oppose_indicator": "O"
            },
            {
              "committee_name": "Tiny",
              "total": 0.0,
              "support_oppose_indicator": "S"
            }
          ]
        }"#;
        let rows = parse_ie_json(json);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].committee, "Super PAC One");
        assert_eq!(rows[0].amount_display, "$250,000");
        assert_eq!(rows[0].support_oppose, "Support");
        assert!(rows[0].url.contains("C00401224"));
        assert_eq!(rows[1].support_oppose, "Oppose");
    }

    #[test]
    fn format_usd_commas() {
        assert_eq!(format_usd(0.0), "$0");
        assert_eq!(format_usd(1234.4), "$1,234");
        assert_eq!(format_usd(-50.0), "-$50");
    }

    #[test]
    fn parse_sched_a_individuals() {
        let json = r#"{
          "results": [
            {
              "contributor_name": "DOE, JANE",
              "contributor_city": "ORLANDO",
              "contributor_state": "FL",
              "contribution_receipt_amount": 3300.0,
              "contribution_receipt_date": "2025-06-01",
              "committee_id": "C00123456",
              "memo_code": null
            },
            {
              "contributor_name": "MEMO ONLY",
              "contribution_receipt_amount": 1000.0,
              "memo_code": "X"
            },
            {
              "contributor_name": "TINY",
              "contribution_receipt_amount": 0.0
            }
          ]
        }"#;
        let rows = parse_sched_a_json(json, 10);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "DOE, JANE");
        assert_eq!(rows[0].amount_display, "$3,300");
        assert_eq!(rows[0].location.as_deref(), Some("ORLANDO, FL"));
        assert_eq!(rows[0].date.as_deref(), Some("2025-06-01"));
    }

    #[test]
    fn sched_a_candidate_occupation_facts() {
        let json = r#"{
          "results": [
            {
              "contributor_name": "SMITH, JANE Q",
              "contributor_occupation": "ATTORNEY",
              "contributor_employer": "SMITH LAW PA",
              "contribution_receipt_amount": 2800.0,
              "committee_id": "C00999999",
              "pdf_url": "https://docquery.fec.gov/cgi-bin/fecimg/?202506010000"
            },
            {
              "contributor_name": "OTHER, DONOR",
              "contributor_occupation": "CEO",
              "contributor_employer": "ACME",
              "contribution_receipt_amount": 1000.0
            },
            {
              "contributor_name": "SMITH, JANE",
              "contributor_occupation": "INFORMATION REQUESTED",
              "contributor_employer": "N/A",
              "contribution_receipt_amount": 500.0
            }
          ]
        }"#;
        let facts = bio_facts_from_sched_a_candidate(json, "Jane Q. Smith");
        assert!(facts.iter().any(|f| f.text.contains("ATTORNEY")));
        assert!(facts.iter().any(|f| f.text.contains("SMITH LAW")));
        assert!(!facts.iter().any(|f| f.text.contains("CEO")));
        assert!(!facts.iter().any(|f| f.text.contains("INFORMATION REQUESTED")));
        assert!(bio_facts_from_sched_a_candidate(json, "Totally Different").is_empty());
        // FEC-style ballot name before format_person_name
        let facts2 = bio_facts_from_sched_a_candidate(json, "SMITH, JANE Q");
        assert!(facts2.iter().any(|f| f.text.contains("ATTORNEY")));
    }

    #[test]
    fn parse_sched_a_aggregates_same_donor() {
        let json = r#"{
          "results": [
            {
              "contributor_name": "DOE, JANE",
              "contributor_city": "ORLANDO",
              "contributor_state": "FL",
              "contribution_receipt_amount": 2000.0,
              "contribution_receipt_date": "2025-01-01",
              "committee_id": "C00123456"
            },
            {
              "contributor_name": "doe, jane",
              "contributor_city": "ORLANDO",
              "contributor_state": "FL",
              "contribution_receipt_amount": 1300.0,
              "contribution_receipt_date": "2025-06-01",
              "committee_id": "C00123456"
            },
            {
              "contributor_name": "SMITH, BOB",
              "contribution_receipt_amount": 500.0,
              "contribution_receipt_date": "2025-03-01",
              "committee_id": "C00123456"
            }
          ]
        }"#;
        let rows = parse_sched_a_aggregated(json, 10);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].name, "DOE, JANE");
        assert_eq!(rows[0].amount_display, "$3,300");
        assert_eq!(rows[0].gift_count, Some(2));
        assert_eq!(rows[0].date.as_deref(), Some("2025-06-01"));
        assert_eq!(rows[1].name, "SMITH, BOB");
        assert!(rows[1].gift_count.is_none());
    }

    #[test]
    fn parse_size_buckets() {
        let json = r#"{
          "results": [
            {"size": 2000, "total": 500000.0, "count": 100},
            {"size": 0, "total": 12000.5, "count": 400},
            {"size": 200, "total": 0.0, "count": 0}
          ]
        }"#;
        let rows = parse_size_json(json);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].label, "$200 and under");
        assert_eq!(rows[0].total_display, "$12,001");
        assert_eq!(rows[0].count_display, "400");
        assert_eq!(rows[1].label, "$2,000 and over");
    }

    #[test]
    fn parse_principal_prefers_p() {
        let json = r#"{
          "results": [
            {
              "committee_id": "C00999999",
              "name": "JOINT FUND",
              "designation": "J",
              "designation_full": "Joint fundraising committee"
            },
            {
              "committee_id": "C00111111",
              "name": "FRIENDS OF X",
              "designation": "P",
              "designation_full": "Principal campaign committee"
            }
          ]
        }"#;
        let c = parse_principal_committee(json).expect("committee");
        assert_eq!(c.committee_id, "C00111111");
        assert_eq!(c.name, "FRIENDS OF X");
        assert!(c.url.contains("C00111111"));
        assert!(c.designation.to_ascii_lowercase().contains("principal"));
    }

    #[test]
    fn size_labels() {
        assert_eq!(size_bucket_label(0), "$200 and under");
        assert_eq!(size_bucket_label(500), "$500 – $999");
    }
}
