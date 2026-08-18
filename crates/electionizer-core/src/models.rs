use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct CandidateSummary {
    pub id: i64,
    pub name: String,
    pub party: String,
    /// CSS chip class: `d`, `r`, or `o`.
    pub party_class: String,
    pub office: String,
    pub is_incumbent: bool,
    pub is_judge: bool,
    pub jurisdiction: String,
    /// FEC id or namespaced state id when known.
    pub external_id: Option<String>,
    pub chamber: Option<String>,
    pub summary: Option<String>,
    /// Filing/roster profile URL when known (non-FEC detail).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    /// Filing/roster publisher when known (for affiliation cites).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_publisher: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OfficeGroup {
    pub office: String,
    pub chamber: Option<String>,
    pub jurisdiction: String,
    pub candidates: Vec<CandidateSummary>,
    pub empty_message: Option<String>,
    /// Judicial / court seats (circuit, appellate, supreme, etc.).
    pub is_judicial: bool,
    /// When true, UI may render the seat collapsed (unopposed / single candidate).
    pub default_open: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct MeasureSummary {
    pub id: i64,
    pub title: String,
    pub measure_code: Option<String>,
    pub summary: Option<String>,
    pub jurisdiction: String,
    /// Detail page when known (e.g. FL DOS InitDetail).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SourceInfo {
    pub url: String,
    pub publisher: Option<String>,
    pub retrieved_at: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CandidateDetail {
    pub id: i64,
    pub name: String,
    pub party: String,
    pub office: String,
    pub chamber: Option<String>,
    pub is_incumbent: bool,
    pub is_judge: bool,
    pub summary: Option<String>,
    pub jurisdiction: String,
    /// USPS state when known from jurisdiction row.
    pub state_code: Option<String>,
    pub election_name: String,
    pub election_date: String,
    /// FEC candidate id when known (e.g. `H8FL08042`), or namespaced state id (`azleg:2371`).
    pub external_id: Option<String>,
    /// A ZIP linked to this candidate’s jurisdiction (for back-nav), when known.
    pub zip: Option<String>,
    pub sources: Vec<SourceInfo>,
}

/// OpenFEC cycle finance totals for a federal candidate (display model).
#[derive(Debug, Clone, Serialize)]
pub struct CandidateFinance {
    pub cycle: i32,
    pub receipts: Option<f64>,
    pub disbursements: Option<f64>,
    pub cash_on_hand: Option<f64>,
    pub debts: Option<f64>,
    pub coverage_end_date: Option<String>,
    pub receipts_display: String,
    pub disbursements_display: String,
    pub cash_on_hand_display: String,
    pub debts_display: Option<String>,
    /// Itemized + unitemized individual contributions when available.
    pub individual_display: Option<String>,
    pub pac_display: Option<String>,
    pub party_display: Option<String>,
    pub profile_url: String,
    pub source_label: String,
}

/// Top independent expenditure row (OpenFEC Schedule E by candidate).
#[derive(Debug, Clone, Serialize)]
pub struct OutsideSpendRow {
    pub committee: String,
    pub amount_display: String,
    pub support_oppose: String,
    pub url: String,
}

/// Schedule A contributor (single line or aggregated gifts).
#[derive(Debug, Clone, Serialize)]
pub struct ContributorRow {
    pub name: String,
    pub amount_display: String,
    pub location: Option<String>,
    pub date: Option<String>,
    pub url: String,
    /// When set, `amount_display` is the sum of this many itemized gifts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gift_count: Option<u32>,
}

/// OpenFEC contribution size bucket for a candidate.
#[derive(Debug, Clone, Serialize)]
pub struct SizeBucketRow {
    pub label: String,
    pub total_display: String,
    pub count_display: String,
}

/// Principal (or primary) campaign committee link.
#[derive(Debug, Clone, Serialize)]
pub struct CommitteeLink {
    pub name: String,
    pub committee_id: String,
    pub designation: String,
    pub url: String,
}

/// Recent roll-call vote for a federal legislator (GovTrack).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteRecord {
    pub date: String,
    pub question: String,
    pub position: String,
    pub result: Option<String>,
    pub url: String,
}

/// Party / chamber service span (from congress-legislators terms or ballot filing).
/// Cite every row: `source` (+ `source_url` when known). Omit rather than invent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffiliationSpan {
    pub party: String,
    #[serde(default)]
    pub start: Option<String>,
    #[serde(default)]
    pub end: Option<String>,
    pub role: String,
    /// Human-readable source name (e.g. DOS publisher, congress-legislators).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// URL for the cited source when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
}

/// Official voter registration check portal for a state.
#[derive(Debug, Clone, Serialize)]
pub struct VoterPortal {
    pub label: String,
    pub url: String,
}

/// Best-effort official portal for checking / updating voter registration.
pub fn voter_portal_for_state(state: &str) -> VoterPortal {
    match state.to_ascii_uppercase().as_str() {
        "AZ" => VoterPortal {
            label: "Arizona voter registration (ServiceArizona)".into(),
            url: "https://servicearizona.com/VoterRegistration/checkStatus".into(),
        },
        "FL" => VoterPortal {
            label: "Florida voter registration (DOS)".into(),
            url: "https://registration.elections.myflorida.com/CheckVoterStatus".into(),
        },
        "CA" => VoterPortal {
            label: "California voter registration".into(),
            url: "https://voterstatus.sos.ca.gov/".into(),
        },
        "NY" => VoterPortal {
            label: "New York voter registration".into(),
            url: "https://voterlookup.elections.ny.gov/".into(),
        },
        _ => VoterPortal {
            label: "Register to vote / check status (Vote.gov)".into(),
            url: "https://vote.gov/".into(),
        },
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MeasureDetail {
    pub id: i64,
    pub title: String,
    pub measure_code: Option<String>,
    pub summary: Option<String>,
    pub jurisdiction: String,
    pub election_name: String,
    pub election_date: String,
    pub sources: Vec<SourceInfo>,
}

/// One visual block on the ballot: a normal office card, or a collapsible judicial block.
#[derive(Debug, Clone, Serialize)]
pub struct BallotSection {
    /// `office` or `judicial`.
    pub kind: String,
    /// Single office when `kind == "office"`.
    pub group: Option<OfficeGroup>,
    /// Judicial seats when `kind == "judicial"` (preserves sort order as one block).
    pub seats: Vec<OfficeGroup>,
    pub explainer: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BallotReport {
    pub zip: String,
    pub status: String,
    pub last_built_at: Option<String>,
    pub coverage_note: Option<String>,
    pub election_name: Option<String>,
    pub election_date: Option<String>,
    pub geo_summary: Option<String>,
    /// USPS state code when known from jurisdictions.
    pub state_code: Option<String>,
    pub voter_portal: Option<VoterPortal>,
    pub office_groups: Vec<OfficeGroup>,
    /// Render order for the ballot UI (judicial seats coalesced into one block).
    pub ballot_sections: Vec<BallotSection>,
    /// Shown once above judicial seats when any exist (circuit-wide explainer).
    pub judicial_explainer: Option<String>,
    pub measures: Vec<MeasureSummary>,
}

/// Raw candidate row used to build office-grouped ballot sections.
#[derive(Debug, Clone)]
pub struct BallotCandidateRow {
    pub id: i64,
    pub name: String,
    pub party: String,
    pub is_incumbent: bool,
    pub is_judge: bool,
    pub office: String,
    pub chamber: Option<String>,
    pub jurisdiction: String,
    pub external_id: Option<String>,
    pub summary: Option<String>,
    pub source_url: Option<String>,
    pub source_publisher: Option<String>,
}

/// Group candidates by (office, jurisdiction), sort within/across groups, inject empty U.S. Senate.
pub fn build_office_groups(
    rows: Vec<BallotCandidateRow>,
    has_state_jurisdiction: bool,
    election_year: Option<i32>,
    state_label: Option<&str>,
) -> Vec<OfficeGroup> {
    use std::collections::BTreeMap;

    let mut map: BTreeMap<(String, String), OfficeGroup> = BTreeMap::new();
    for r in rows {
        let key = (r.office.clone(), r.jurisdiction.clone());
        let row_judicial = r.is_judge
            || r.chamber
                .as_deref()
                .is_some_and(|c| c.eq_ignore_ascii_case("judicial"));
        let entry = map.entry(key).or_insert_with(|| OfficeGroup {
            office: r.office.clone(),
            chamber: r.chamber.clone(),
            jurisdiction: r.jurisdiction.clone(),
            candidates: Vec::new(),
            empty_message: None,
            is_judicial: row_judicial,
            default_open: true,
        });
        // Prefer non-None chamber if we see one later
        if entry.chamber.is_none() && r.chamber.is_some() {
            entry.chamber = r.chamber.clone();
        }
        if row_judicial {
            entry.is_judicial = true;
        }
        entry.candidates.push(CandidateSummary {
            id: r.id,
            name: r.name,
            party: r.party.clone(),
            party_class: party_class(&r.party).into(),
            office: r.office,
            is_incumbent: r.is_incumbent,
            is_judge: r.is_judge,
            jurisdiction: r.jurisdiction,
            external_id: r.external_id,
            chamber: r.chamber,
            summary: r.summary,
            source_url: r.source_url,
            source_publisher: r.source_publisher,
        });
    }

    let mut groups: Vec<OfficeGroup> = map.into_values().collect();
    for g in &mut groups {
        g.candidates.sort_by(|a, b| {
            b.is_incumbent
                .cmp(&a.is_incumbent)
                .then_with(|| a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase()))
        });
        // Contested judicial seats stay open; single-name trial seats collapse
        // (unopposed → takes bench). Merit retention (supreme / appellate, one name
        // but yes/no on the ballot) stays open.
        if g.is_judicial {
            let o = g.office.to_ascii_lowercase();
            let retention = o.contains("supreme")
                || o.contains("appeal")
                || o.contains("retention");
            g.default_open = g.candidates.len() > 1 || retention;
        }
    }
    sort_office_groups(&mut groups);

    let has_us_senate = groups.iter().any(|g| {
        g.chamber
            .as_deref()
            .is_some_and(|c| c.eq_ignore_ascii_case("senate"))
    });
    if has_state_jurisdiction && !has_us_senate {
        let year = election_year.unwrap_or_else(|| Utc::now().year());
        let state = state_label.unwrap_or("this state");
        groups.insert(
            0,
            OfficeGroup {
                office: "U.S. Senate".into(),
                chamber: Some("senate".into()),
                jurisdiction: state.to_string(),
                candidates: Vec::new(),
                empty_message: Some(format!(
                    "No U.S. Senate race in {state} for {year}."
                )),
                is_judicial: false,
                default_open: true,
            },
        );
        // Keep senate first via re-sort (empty senate already rank 0)
        sort_office_groups(&mut groups);
    }

    groups
}

fn sort_office_groups(groups: &mut [OfficeGroup]) {
    groups.sort_by(|a, b| {
        office_sort_key(a)
            .cmp(&office_sort_key(b))
            // Contested (open) before unopposed (collapsed) within the same band.
            .then_with(|| b.default_open.cmp(&a.default_open))
            .then_with(|| a.office.cmp(&b.office))
            .then_with(|| a.jurisdiction.cmp(&b.jurisdiction))
    });
}

fn party_class(party: &str) -> &'static str {
    match party_bucket(party) {
        "democrat" => "d",
        "republican" => "r",
        _ => "o",
    }
}

/// Ballot copy when judicial seats are present (FL circuit-wide design, etc.).
pub fn judicial_explainer_for(groups: &[OfficeGroup]) -> Option<String> {
    if !groups.iter().any(|g| g.is_judicial) {
        return None;
    }
    Some(
        "Judicial seats below are for the whole circuit, appellate district, county, or state — not one precinct. \
         In Florida, a circuit or county judge “Group” is a seat number, not a neighborhood. \
         Contested seats are open; unopposed seats (usually not printed on the sample ballot) are one collapsed table — expand once to see who takes the bench."
            .into(),
    )
}

/// Coalesce consecutive judicial office groups into one collapsible UI block.
pub fn build_ballot_sections(groups: &[OfficeGroup]) -> Vec<BallotSection> {
    let explainer = judicial_explainer_for(groups);
    let mut out = Vec::new();
    let mut i = 0;
    while i < groups.len() {
        if groups[i].is_judicial {
            let start = i;
            while i < groups.len() && groups[i].is_judicial {
                i += 1;
            }
            out.push(BallotSection {
                kind: "judicial".into(),
                group: None,
                seats: groups[start..i].to_vec(),
                explainer: explainer.clone(),
            });
        } else {
            out.push(BallotSection {
                kind: "office".into(),
                group: Some(groups[i].clone()),
                seats: Vec::new(),
                explainer: None,
            });
            i += 1;
        }
    }
    out
}

/// Sort rank: federal S → H → statewide → state leg → judicial → county → municipal → special → other.
fn office_sort_key(g: &OfficeGroup) -> u8 {
    match g.chamber.as_deref().map(|c| c.to_ascii_lowercase()).as_deref() {
        Some("senate") => 0,
        Some("house") => 1,
        Some("statewide") => 2,
        Some("state_senate") => 3,
        Some("state_house") => 4,
        Some("judicial") => 5,
        Some("county") => 6,
        Some("municipal") => 7,
        Some("special_district") => 8,
        _ => {
            if g.candidates.iter().any(|c| c.is_judge) {
                5
            } else {
                9
            }
        }
    }
}

/// Best-effort year from election name ("2026 General…") or ISO date ("2026-11-03").
pub fn election_year_from(name: Option<&str>, date: Option<&str>) -> Option<i32> {
    if let Some(d) = date {
        if d.len() >= 4 {
            if let Ok(y) = d[..4].parse::<i32>() {
                if (1990..2100).contains(&y) {
                    return Some(y);
                }
            }
        }
    }
    if let Some(n) = name {
        for tok in n.split(|c: char| !c.is_ascii_digit()) {
            if tok.len() == 4 {
                if let Ok(y) = tok.parse::<i32>() {
                    if (1990..2100).contains(&y) {
                        return Some(y);
                    }
                }
            }
        }
    }
    None
}

#[derive(Debug, Clone, Deserialize)]
pub struct FixtureFile {
    pub zip: String,
    pub geo: FixtureGeo,
    pub election: FixtureElection,
    pub candidates: Vec<FixtureCandidate>,
    pub measures: Vec<FixtureMeasure>,
    pub source: FixtureSource,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FixtureGeo {
    pub state: String,
    pub state_name: String,
    pub county: String,
    pub city: String,
    pub congressional_district: String,
    pub jurisdictions: Vec<FixtureJurisdiction>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FixtureJurisdiction {
    pub ocd_id: String,
    pub name: String,
    pub level: String,
    pub state: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FixtureElection {
    pub name: String,
    pub election_date: String,
    pub scope: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FixtureCandidate {
    pub office: String,
    pub chamber: Option<String>,
    pub jurisdiction_ocd: String,
    pub is_judicial: bool,
    pub name: String,
    pub party: String,
    pub is_incumbent: bool,
    pub is_judge: bool,
    pub summary: Option<String>,
    pub source_url: String,
    pub source_publisher: Option<String>,
    #[serde(default)]
    pub external_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FixtureMeasure {
    pub title: String,
    pub measure_code: Option<String>,
    pub jurisdiction_ocd: String,
    pub summary: Option<String>,
    pub source_url: String,
    pub source_publisher: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FixtureSource {
    pub url: String,
    pub publisher: String,
}

#[derive(Debug, Clone)]
pub struct GeoResolution {
    pub state: String,
    pub state_name: String,
    pub county: String,
    pub city: String,
    pub congressional_district: String,
    /// State upper chamber district number when known (e.g. FL Senate).
    pub state_senate_district: Option<u32>,
    /// State lower chamber district number when known (e.g. FL House).
    pub state_house_district: Option<u32>,
    /// Lower-chamber label including lettered subdistricts (e.g. MD `30A`, `1B`).
    pub state_house_label: Option<String>,
    /// ZIP centroid when known (Open States people.geo, etc.).
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub jurisdictions: Vec<ResolvedJurisdiction>,
    pub source_url: String,
    pub source_publisher: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedJurisdiction {
    pub ocd_id: String,
    pub name: String,
    pub level: String,
    pub state: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BallotSnapshot {
    pub election_name: String,
    pub election_date: String,
    pub election_scope: String,
    pub candidates: Vec<SnapshotCandidate>,
    pub measures: Vec<SnapshotMeasure>,
    pub source_url: String,
    pub source_publisher: String,
    pub coverage_note: Option<String>,
    /// Jurisdictions needed for candidates/measures but not present on geo (e.g. FL circuits).
    pub extra_jurisdictions: Vec<ResolvedJurisdiction>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SnapshotCandidate {
    pub office: String,
    pub chamber: Option<String>,
    pub jurisdiction_ocd: String,
    pub is_judicial: bool,
    pub name: String,
    pub party: String,
    pub is_incumbent: bool,
    pub is_judge: bool,
    pub summary: Option<String>,
    pub source_url: String,
    pub source_publisher: Option<String>,
    /// External system id (FEC candidate_id for federal).
    pub external_id: Option<String>,
}

/// Format a USD amount for UI (whole dollars, with commas).
pub fn format_usd(amount: f64) -> String {
    let neg = amount < 0.0;
    let n = amount.abs().round() as i64;
    let s = n.to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    let digits: String = out.chars().rev().collect();
    if neg {
        format!("-${digits}")
    } else {
        format!("${digits}")
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SnapshotMeasure {
    pub title: String,
    pub measure_code: Option<String>,
    pub jurisdiction_ocd: String,
    pub summary: Option<String>,
    pub source_url: String,
    pub source_publisher: Option<String>,
}

pub fn normalize_zip(raw: &str) -> Option<String> {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 5 {
        Some(digits[..5].to_string())
    } else {
        None
    }
}

/// Build a full ballot report from fixture JSON (no I/O). Optional ZIP overrides the fixture zip.
pub fn ballot_report_from_fixture(
    fixture_json: &str,
    zip_override: Option<&str>,
) -> Result<BallotReport, String> {
    let mut data: FixtureFile =
        serde_json::from_str(fixture_json).map_err(|e| format!("parse fixture: {e}"))?;

    if let Some(raw) = zip_override {
        let z = normalize_zip(raw).ok_or_else(|| format!("invalid zip: {raw}"))?;
        data.zip = z;
    } else if normalize_zip(&data.zip).is_none() {
        return Err(format!("invalid fixture zip: {}", data.zip));
    }

    let jur_name: std::collections::HashMap<&str, &str> = data
        .geo
        .jurisdictions
        .iter()
        .map(|j| (j.ocd_id.as_str(), j.name.as_str()))
        .collect();

    let has_state_jurisdiction = data.geo.jurisdictions.iter().any(|j| j.level == "state");
    let state_label = data
        .geo
        .jurisdictions
        .iter()
        .find(|j| j.level == "state")
        .map(|j| j.name.as_str());
    let state_code = data
        .geo
        .jurisdictions
        .iter()
        .find_map(|j| j.state.as_deref().filter(|s| !s.is_empty()))
        .or(Some(data.geo.state.as_str()))
        .map(|s| s.to_ascii_uppercase());

    let geo_summary = {
        let mut interesting = Vec::new();
        for j in &data.geo.jurisdictions {
            if matches!(
                j.level.as_str(),
                "state"
                    | "congressional"
                    | "state_senate"
                    | "state_house"
                    | "county"
                    | "municipal"
            ) {
                interesting.push(j.name.as_str());
            }
        }
        if interesting.is_empty() {
            None
        } else {
            Some(interesting.join(" · "))
        }
    };

    let year = election_year_from(Some(&data.election.name), Some(&data.election.election_date));

    let rows: Vec<BallotCandidateRow> = data
        .candidates
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let jurisdiction = jur_name
                .get(c.jurisdiction_ocd.as_str())
                .copied()
                .unwrap_or(c.jurisdiction_ocd.as_str())
                .to_string();
            BallotCandidateRow {
                id: (i as i64) + 1,
                name: c.name.clone(),
                party: c.party.clone(),
                is_incumbent: c.is_incumbent,
                is_judge: c.is_judge || c.is_judicial,
                office: c.office.clone(),
                chamber: c.chamber.clone(),
                jurisdiction,
                external_id: c.external_id.clone(),
                summary: c.summary.clone(),
                source_url: if c.source_url.trim().is_empty() {
                    None
                } else {
                    Some(c.source_url.clone())
                },
                source_publisher: c.source_publisher.clone(),
            }
        })
        .collect();

    let office_groups = build_office_groups(rows, has_state_jurisdiction, year, state_label);
    let judicial_explainer = judicial_explainer_for(&office_groups);
    let ballot_sections = build_ballot_sections(&office_groups);

    let measures: Vec<MeasureSummary> = data
        .measures
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let jurisdiction = jur_name
                .get(m.jurisdiction_ocd.as_str())
                .copied()
                .unwrap_or(m.jurisdiction_ocd.as_str())
                .to_string();
            MeasureSummary {
                id: (i as i64) + 1,
                title: m.title.clone(),
                measure_code: m.measure_code.clone(),
                summary: m.summary.clone(),
                jurisdiction,
                source_url: if m.source_url.trim().is_empty() {
                    None
                } else {
                    Some(m.source_url.clone())
                },
            }
        })
        .collect();

    let voter_portal = Some(voter_portal_for_state(
        state_code.as_deref().unwrap_or(""),
    ));

    Ok(BallotReport {
        zip: data.zip,
        status: "ready".into(),
        last_built_at: None,
        coverage_note: Some("Fixture sample data (offline demo)".into()),
        election_name: Some(data.election.name),
        election_date: Some(data.election.election_date),
        geo_summary,
        state_code,
        voter_portal,
        office_groups,
        ballot_sections,
        judicial_explainer,
        measures,
    })
}

pub fn party_bucket(party: &str) -> &'static str {
    let p = party.to_ascii_lowercase();
    if p.contains("democrat") || p == "dem" || p.starts_with("d/") {
        "democrat"
    } else if p.contains("republican") || p == "rep" || p.starts_with("r/") {
        "republican"
    } else {
        "other"
    }
}

pub fn normalize_party_label(party: &str) -> String {
    match party_bucket(party) {
        "democrat" => "Democratic".into(),
        "republican" => "Republican".into(),
        _ => {
            let t = party.trim();
            if t.is_empty() || t.eq_ignore_ascii_case("none") || t.eq_ignore_ascii_case("npa") {
                "Independent / Other".into()
            } else {
                // Title-ish case for FEC "GREEN PARTY" etc.
                t.split_whitespace()
                    .map(|w| {
                        let mut c = w.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => {
                                f.to_uppercase().collect::<String>() + &c.as_str().to_lowercase()
                            }
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            }
        }
    }
}

/// General election date for a federal cycle (Tuesday after the first Monday in November).
pub fn federal_election_date(cycle: i32) -> String {
    use chrono::NaiveDate;
    let nov1 = NaiveDate::from_ymd_opt(cycle, 11, 1).expect("valid cycle year");
    let mon0 = nov1.weekday().num_days_from_monday(); // Mon = 0
    let first_monday = 1 + (7 - mon0) % 7;
    let election_day = first_monday + 1;
    format!("{cycle}-11-{election_day:02}")
}

pub fn now_str() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_zip_accepts_zip_plus_four() {
        assert_eq!(normalize_zip("90210-1234").as_deref(), Some("90210"));
        assert_eq!(normalize_zip(" 10001 ").as_deref(), Some("10001"));
        assert_eq!(normalize_zip("1234"), None);
    }

    #[test]
    fn party_bucket_groups() {
        assert_eq!(party_bucket("Democratic"), "democrat");
        assert_eq!(party_bucket("Republican Party"), "republican");
        assert_eq!(party_bucket("DEM"), "democrat");
        assert_eq!(party_bucket("Nonpartisan"), "other");
    }

    #[test]
    fn federal_election_date_2026() {
        // Nov 1 2026 is Sunday → first Monday Nov 2 → election Nov 3
        assert_eq!(federal_election_date(2026), "2026-11-03");
    }

    fn row(
        id: i64,
        name: &str,
        party: &str,
        office: &str,
        chamber: Option<&str>,
        jurisdiction: &str,
        incumbent: bool,
    ) -> BallotCandidateRow {
        BallotCandidateRow {
            id,
            name: name.into(),
            party: party.into(),
            is_incumbent: incumbent,
            is_judge: false,
            office: office.into(),
            chamber: chamber.map(str::to_string),
            jurisdiction: jurisdiction.into(),
            external_id: None,
            summary: None,
            source_url: None,
            source_publisher: None,
        }
    }

    #[test]
    fn office_groups_incumbent_first_and_order() {
        let rows = vec![
            row(1, "Zed", "Democratic", "Florida House (District 30)", Some("state_house"), "HD-30", false),
            row(2, "Ann", "Republican", "Florida House (District 30)", Some("state_house"), "HD-30", true),
            row(3, "Bob", "Democratic", "U.S. House", Some("house"), "FL-8", true),
            row(4, "Cal", "Republican", "Florida Senate (District 8)", Some("state_senate"), "SD-8", false),
            row(5, "Dee", "Republican", "U.S. Senate", Some("senate"), "Florida", true),
            row(6, "Eve", "Nonpartisan", "Governor", Some("statewide"), "Florida", false),
        ];
        let groups = build_office_groups(rows, true, Some(2026), Some("Florida"));
        let offices: Vec<_> = groups.iter().map(|g| g.office.as_str()).collect();
        assert_eq!(
            offices,
            vec![
                "U.S. Senate",
                "U.S. House",
                "Governor",
                "Florida Senate (District 8)",
                "Florida House (District 30)",
            ]
        );
        let house = groups.iter().find(|g| g.office.contains("House (District 30)")).unwrap();
        assert_eq!(house.candidates[0].name, "Ann"); // incumbent first
        assert_eq!(house.candidates[1].name, "Zed");
        assert!(groups.iter().all(|g| g.empty_message.is_none()));
    }

    #[test]
    fn empty_us_senate_when_missing_not_state_senate() {
        let rows = vec![
            row(1, "Bob", "Democratic", "U.S. House", Some("house"), "FL-8", true),
            row(
                2,
                "Cal",
                "Republican",
                "Florida Senate (District 8)",
                Some("state_senate"),
                "SD-8",
                true,
            ),
        ];
        let groups = build_office_groups(rows, true, Some(2026), Some("Florida"));
        let senate = groups.iter().find(|g| g.office == "U.S. Senate").unwrap();
        assert!(senate.candidates.is_empty());
        assert_eq!(
            senate.empty_message.as_deref(),
            Some("No U.S. Senate race in Florida for 2026.")
        );
        // state_senate still present and not treated as U.S. Senate
        assert!(groups.iter().any(|g| g.chamber.as_deref() == Some("state_senate")));
    }

    #[test]
    fn no_empty_senate_when_candidates_exist() {
        let rows = vec![row(
            1,
            "Dee",
            "Republican",
            "U.S. Senate",
            Some("senate"),
            "California",
            true,
        )];
        let groups = build_office_groups(rows, true, Some(2026), Some("California"));
        assert_eq!(groups.len(), 1);
        assert!(groups[0].empty_message.is_none());
        assert_eq!(groups[0].candidates.len(), 1);
    }

    #[test]
    fn election_year_from_name_and_date() {
        assert_eq!(
            election_year_from(Some("2026 General Election"), Some("2026-11-03")),
            Some(2026)
        );
        assert_eq!(election_year_from(Some("General Election"), Some("2024-11-05")), Some(2024));
        assert_eq!(election_year_from(Some("2028 Primary"), None), Some(2028));
    }

    #[test]
    fn judicial_groups_collapse_and_section_block() {
        let mut rows = vec![
            row(1, "Bob", "Democratic", "U.S. House", Some("house"), "FL-8", true),
            row(
                2,
                "Ann",
                "Nonpartisan",
                "Circuit Judge (Circuit 18, Group 2)",
                Some("judicial"),
                "Circuit 18",
                true,
            ),
            row(
                3,
                "Cal",
                "Nonpartisan",
                "Circuit Judge (Circuit 18, Group 4)",
                Some("judicial"),
                "Circuit 18",
                false,
            ),
            row(
                4,
                "Dee",
                "Nonpartisan",
                "Circuit Judge (Circuit 18, Group 4)",
                Some("judicial"),
                "Circuit 18",
                false,
            ),
            row(
                5,
                "Eve",
                "Republican",
                "County Commissioner",
                Some("county"),
                "Brevard",
                false,
            ),
        ];
        rows[1].is_judge = true;
        rows[2].is_judge = true;
        rows[3].is_judge = true;

        let groups = build_office_groups(rows, true, Some(2026), Some("Florida"));
        let jud: Vec<_> = groups.iter().filter(|g| g.is_judicial).collect();
        assert_eq!(jud.len(), 2);
        let unopposed = jud.iter().find(|g| g.office.contains("Group 2")).unwrap();
        assert!(!unopposed.default_open);
        let contested = jud.iter().find(|g| g.office.contains("Group 4")).unwrap();
        assert!(contested.default_open);
        assert!(judicial_explainer_for(&groups).is_some());

        let sections = build_ballot_sections(&groups);
        let kinds: Vec<_> = sections.iter().map(|s| s.kind.as_str()).collect();
        assert!(kinds.contains(&"judicial"));
        let jblock = sections.iter().find(|s| s.kind == "judicial").unwrap();
        assert_eq!(jblock.seats.len(), 2);
        assert!(jblock.explainer.is_some());
        // Contested seats first, unopposed collapsed after.
        assert!(jblock.seats[0].default_open);
        assert!(!jblock.seats[1].default_open);
        assert!(jblock.seats[0].office.contains("Group 4"));
        assert!(jblock.seats[1].office.contains("Group 2"));
        // Judicial block sits between federal/state and county in sort order.
        let j_idx = kinds.iter().position(|k| *k == "judicial").unwrap();
        let county_idx = sections
            .iter()
            .position(|s| {
                s.group
                    .as_ref()
                    .is_some_and(|g| g.office.contains("County"))
            })
            .unwrap();
        assert!(j_idx < county_idx);
    }

    #[test]
    fn ballot_report_from_minimal_fixture() {
        let json = r#"{
          "zip": "90210",
          "geo": {
            "state": "CA",
            "state_name": "California",
            "county": "Los Angeles County",
            "city": "Beverly Hills",
            "congressional_district": "CA-32",
            "jurisdictions": [
              {"ocd_id": "ocd-division/country:us/state:ca", "name": "California", "level": "state", "state": "CA"},
              {"ocd_id": "ocd-division/country:us/state:ca/cd:32", "name": "CA-32", "level": "congressional", "state": "CA"}
            ]
          },
          "election": {"name": "2026 General Election", "election_date": "2026-11-03", "scope": "general"},
          "candidates": [
            {
              "office": "U.S. House", "chamber": "house",
              "jurisdiction_ocd": "ocd-division/country:us/state:ca/cd:32",
              "is_judicial": false, "name": "Sam Chen", "party": "Democratic",
              "is_incumbent": true, "is_judge": false, "summary": null,
              "source_url": "https://example.com", "source_publisher": "Fixture"
            }
          ],
          "measures": [],
          "source": {"url": "https://example.com", "publisher": "Fixture"}
        }"#;
        let report = ballot_report_from_fixture(json, Some("90210")).unwrap();
        assert_eq!(report.zip, "90210");
        assert_eq!(report.state_code.as_deref(), Some("CA"));
        assert!(!report.office_groups.is_empty());
        assert!(report.voter_portal.is_some());
        assert!(report.coverage_note.as_deref().unwrap().contains("Fixture"));
    }
}
