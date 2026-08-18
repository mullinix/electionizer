//! Person dossier / career assessment (pure — no I/O).
//! Cite or omit. Bench time counts as political. Career-politician label is blunt.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

/// Life / career category for adult-life fractions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifeCategory {
    Political,
    Education,
    Work,
    Business,
    Legal,
}

impl LifeCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Political => "political",
            Self::Education => "education",
            Self::Work => "work",
            Self::Business => "business",
            Self::Legal => "legal",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Political => "Political / bench",
            Self::Education => "Education",
            Self::Work => "Work / employment",
            Self::Business => "Business",
            Self::Legal => "Legal practice",
        }
    }
}

/// One dated (or undated) career/life span with a cite.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CareerSpan {
    pub category: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_year: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_year: Option<i32>,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
}

impl CareerSpan {
    pub fn new(
        category: LifeCategory,
        label: impl Into<String>,
        start_year: Option<i32>,
        end_year: Option<i32>,
        source: impl Into<String>,
        source_url: Option<String>,
    ) -> Self {
        Self {
            category: category.as_str().into(),
            label: label.into(),
            start_year,
            end_year,
            source: source.into(),
            source_url,
        }
    }
}

/// Approximate adult-life share for one category.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LifeFraction {
    pub category: String,
    pub category_label: String,
    /// Distinct calendar years covered (union of spans).
    pub years: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adult_years: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fraction: Option<f64>,
    pub display: String,
}

/// One cite on a bio fact (multi-source after coalesce).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BioFactSource {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Single cited bio fact (family, education line, etc.).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BioFact {
    /// `family` | `education` | `work` | `business` | `legal` | `other`
    pub kind: String,
    pub text: String,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    /// Full multi-cite list after coalesce; empty → use `source` / `source_url` only.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<BioFactSource>,
}

impl BioFact {
    pub fn new(
        kind: impl Into<String>,
        text: impl Into<String>,
        source: impl Into<String>,
        source_url: Option<String>,
    ) -> Self {
        let source = source.into();
        Self {
            kind: kind.into(),
            text: text.into(),
            source: source.clone(),
            source_url: source_url.clone(),
            sources: vec![BioFactSource {
                name: source,
                url: source_url,
            }],
        }
    }

    /// All cites: `sources` if non-empty, else primary `source`/`source_url`.
    pub fn all_sources(&self) -> Vec<BioFactSource> {
        if !self.sources.is_empty() {
            return self.sources.clone();
        }
        if self.source.is_empty() {
            return Vec::new();
        }
        vec![BioFactSource {
            name: self.source.clone(),
            url: self.source_url.clone(),
        }]
    }
}

/// Org endorsement or opposition (people or measures).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Endorsement {
    pub org: String,
    /// `support` | `oppose` | `endorse`
    pub stance: String,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    /// `ie` | `committee` | `org` | `person` | `party` | `union` | `newspaper` | `bar_poll` | `self_reported`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// `filing` | `official` | `reference` | `campaign` | `news` | `opinion` | `inference`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trust: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
}

/// Cited personal economic tie when a public disclosure parses.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PersonalHolding {
    /// `property` | `business` | `stock` | `other`
    pub kind: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount_display: Option<String>,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
}

/// One official portal for personal financial disclosure search (not campaign finance).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DisclosurePortal {
    pub label: String,
    pub url: String,
    pub note: String,
}

/// Citizenship / allegiance — only when explicitly disclosed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CitizenshipRecord {
    pub countries: Vec<String>,
    pub disclosed: bool,
    pub note: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
}

impl Default for CitizenshipRecord {
    fn default() -> Self {
        Self {
            countries: Vec::new(),
            disclosed: false,
            note: "Not disclosed in the public sources we check. Dual citizenship is almost never on candidate filings — we do not guess.".into(),
            source: None,
            source_url: None,
        }
    }
}

/// Family snapshot for voters — cite or omit; never invent unmarried/childless.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FamilySummary {
    pub disclosed: bool,
    /// e.g. `Married to Eva Lialios · 4 children`
    pub display: String,
    pub note: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spouse: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub children_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub children_detail: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<BioFactSource>,
}

impl Default for FamilySummary {
    fn default() -> Self {
        Self {
            disclosed: false,
            display: String::new(),
            note: "Not disclosed in sources we check.".into(),
            spouse: None,
            children_count: None,
            children_detail: None,
            sources: Vec::new(),
        }
    }
}

/// Sexual orientation — only when a public source **explicitly** states it. Never inferred.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrientationRecord {
    pub disclosed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub note: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<BioFactSource>,
}

impl Default for OrientationRecord {
    fn default() -> Self {
        Self {
            disclosed: false,
            label: None,
            note: "Not disclosed in sources we check.".into(),
            sources: Vec::new(),
        }
    }
}

/// Career-politician assessment + political fraction math.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CareerAssessment {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub birth_year: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adult_years: Option<f64>,
    pub political_years: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub political_fraction: Option<f64>,
    pub is_career_politician: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banner: Option<String>,
    pub blurb: String,
    pub fractions: Vec<LifeFraction>,
    pub spans: Vec<CareerSpan>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

/// Full person dossier for the detail card (may be sparse).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PersonDossier {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo_source_url: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub facts: Vec<BioFact>,
    pub career: CareerAssessment,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub endorsements: Vec<Endorsement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub holdings: Vec<PersonalHolding>,
    pub citizenship: CitizenshipRecord,
    /// Married + kids snapshot (I3). Never invents unmarried/childless.
    #[serde(default)]
    pub family_summary: FamilySummary,
    /// Explicit public orientation only (I3). Default not disclosed.
    #[serde(default)]
    pub orientation: OrientationRecord,
    /// Bio hosts actually consulted this enrich (I4 empty-state: “Checked X / Y — not found.”).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources_checked: Vec<String>,
    /// Official personal financial disclosure search portals (not holdings).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub disclosure_portals: Vec<DisclosurePortal>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub empty_notes: Vec<String>,
}

/// Majority of adult life in politics/bench → career politician.
pub const CAREER_POLITICIAN_FRACTION: f64 = 0.5;
/// Without birth year: long continuous political/bench service still earns the label.
pub const CAREER_POLITICIAN_YEARS_WITHOUT_BIRTH: f64 = 18.0;

/// Parse `YYYY` or `YYYY-MM-DD` → year. Also accepts Wikidata `+1971-05-28T00:00:00Z`.
pub fn year_from_date(s: &str) -> Option<i32> {
    let t = s.trim().trim_start_matches('+');
    if t.starts_with('-') {
        return None; // BCE — out of adult-life range
    }
    if t.len() < 4 {
        return None;
    }
    t[..4].parse().ok().filter(|y| (1800..=2100).contains(y))
}

/// Distinct calendar years covered by spans of one category (union; ongoing → `as_of_year`).
pub fn years_covered(spans: &[CareerSpan], category: &str, as_of_year: i32) -> f64 {
    let mut set = BTreeSet::new();
    for s in spans {
        if s.category != category {
            continue;
        }
        let Some(start) = s.start_year else {
            continue;
        };
        let end = s.end_year.unwrap_or(as_of_year).max(start);
        let end = end.min(as_of_year);
        if end < start {
            continue;
        }
        for y in start..=end {
            set.insert(y);
        }
    }
    set.len() as f64
}

fn fraction_row(category: LifeCategory, years: f64, adult_years: Option<f64>) -> LifeFraction {
    let (fraction, display) = match adult_years {
        Some(a) if a > 0.0 => {
            let f = (years / a).clamp(0.0, 1.0);
            let pct = (f * 100.0).round() as i32;
            (
                Some(f),
                format!(
                    "{y:.0} / {a:.0} adult yr ({pct}%)",
                    y = years,
                    a = a,
                    pct = pct
                ),
            )
        }
        _ => (
            None,
            if years > 0.0 {
                format!("{years:.0} yr covered (adult life unknown)")
            } else {
                "—".into()
            },
        ),
    };
    LifeFraction {
        category: category.as_str().into(),
        category_label: category.label().into(),
        years,
        adult_years,
        fraction,
        display,
    }
}

/// Assess career politician status and per-category fractions from cited spans.
pub fn assess_career(
    spans: &[CareerSpan],
    birth_year: Option<i32>,
    as_of_year: i32,
) -> CareerAssessment {
    let adult_years = birth_year.and_then(|b| {
        let a = as_of_year - b - 18;
        if a > 0 {
            Some(a as f64)
        } else {
            None
        }
    });

    let political_years = years_covered(spans, LifeCategory::Political.as_str(), as_of_year);
    let political_fraction = adult_years.map(|a| (political_years / a).clamp(0.0, 1.0));

    let mut notes = Vec::new();
    if birth_year.is_none() {
        notes.push(
            "Birth year not in public timeline sources we checked — adult-life fraction may be incomplete."
                .into(),
        );
    }
    if political_years <= 0.0 {
        notes.push("No dated political or bench service spans found yet.".into());
    }

    let is_career_politician = match political_fraction {
        Some(f) if f >= CAREER_POLITICIAN_FRACTION => true,
        None if political_years >= CAREER_POLITICIAN_YEARS_WITHOUT_BIRTH => true,
        _ => false,
    };

    let (banner, blurb) = if is_career_politician {
        let banner = Some("CAREER POLITICIAN".into());
        let blurb = match political_fraction {
            Some(f) => {
                let pct = (f * 100.0).round() as i32;
                let a = adult_years.unwrap_or(0.0);
                format!(
                    "About {pct}% of adult life in elected office, political employment, or on the bench ({p:.0} of {a:.0} years since age 18). This is not a side job.",
                    pct = pct,
                    p = political_years,
                    a = a
                )
            }
            None => format!(
                "At least {p:.0} years in elected office, political employment, or on the bench (birth date not public — labeled by ≥{th:.0} years of political/bench time). This is not a side job.",
                p = political_years,
                th = CAREER_POLITICIAN_YEARS_WITHOUT_BIRTH
            ),
        };
        (banner, blurb)
    } else if political_years > 0.0 {
        let blurb = match political_fraction {
            Some(f) => {
                let pct = (f * 100.0).round() as i32;
                let a = adult_years.unwrap_or(0.0);
                format!(
                    "About {pct}% of adult life in elected office, political employment, or on the bench ({p:.0} of {a:.0} years). Below the career-politician threshold (≥50%).",
                    pct = pct,
                    p = political_years,
                    a = a
                )
            }
            None => format!(
                "At least {p:.0} years of dated political or bench service found. Not enough public timeline to score career-politician status.",
                p = political_years
            ),
        };
        (None, blurb)
    } else {
        (
            None,
            "Not enough public timeline to score career-politician status.".into(),
        )
    };

    let cats = [
        LifeCategory::Political,
        LifeCategory::Education,
        LifeCategory::Work,
        LifeCategory::Business,
        LifeCategory::Legal,
    ];
    let fractions: Vec<LifeFraction> = cats
        .into_iter()
        .map(|c| {
            let y = years_covered(spans, c.as_str(), as_of_year);
            fraction_row(c, y, adult_years)
        })
        .collect();

    CareerAssessment {
        birth_year,
        adult_years,
        political_years,
        political_fraction,
        is_career_politician,
        banner,
        blurb,
        fractions,
        spans: spans.to_vec(),
        notes,
    }
}

/// Empty dossier shell with honest empty-states (F0).
pub fn empty_dossier(as_of_year: i32) -> PersonDossier {
    let career = assess_career(&[], None, as_of_year);
    PersonDossier {
        photo_url: None,
        photo_source: None,
        photo_source_url: None,
        facts: Vec::new(),
        career,
        endorsements: Vec::new(),
        holdings: Vec::new(),
        citizenship: CitizenshipRecord::default(),
        family_summary: FamilySummary::default(),
        orientation: OrientationRecord::default(),
        sources_checked: Vec::new(),
        disclosure_portals: Vec::new(),
        empty_notes: vec![
            "Photo: not found in sources we check yet.".into(),
            "Family: not disclosed in sources we check.".into(),
            "Education / work / business / legal: no cited timeline yet.".into(),
            "Endorsements: none loaded yet.".into(),
            "Personal holdings: no public disclosure parsed yet.".into(),
        ],
    }
}

/// Political spans from congress-legislators person object (terms + optional bio.birthday).
pub fn spans_from_congress_legislator(person: &Value) -> (Option<i32>, Vec<CareerSpan>) {
    const SRC: &str = "unitedstates/congress-legislators";
    const SRC_URL: &str = "https://github.com/unitedstates/congress-legislators";

    let birth_year = person
        .pointer("/bio/birthday")
        .and_then(|v| v.as_str())
        .and_then(year_from_date);

    let mut spans = Vec::new();
    if let Some(terms) = person.get("terms").and_then(|t| t.as_array()) {
        for t in terms {
            let start = t
                .get("start")
                .and_then(|v| v.as_str())
                .and_then(year_from_date);
            let end = t
                .get("end")
                .and_then(|v| v.as_str())
                .and_then(year_from_date);
            let ttype = t.get("type").and_then(|v| v.as_str()).unwrap_or("member");
            let state = t.get("state").and_then(|v| v.as_str()).unwrap_or("");
            let label = match ttype {
                "sen" => format!("U.S. Senate ({state})"),
                "rep" => {
                    let dist = t
                        .get("district")
                        .map(|d| d.to_string())
                        .unwrap_or_else(|| "?".into());
                    format!("U.S. House ({state}-{dist})")
                }
                other => format!("Federal office ({other}, {state})"),
            };
            spans.push(CareerSpan::new(
                LifeCategory::Political,
                label,
                start,
                end,
                SRC,
                Some(SRC_URL.into()),
            ));
        }
    }
    (birth_year, spans)
}

/// Extract person object matching FEC id from legislators-current/historical JSON array.
pub fn congress_person_by_fec(legislators_json: &str, fec_id: &str) -> Option<Value> {
    let want = fec_id.trim().to_ascii_uppercase();
    if want.is_empty() {
        return None;
    }
    let arr: Vec<Value> = serde_json::from_str(legislators_json).ok()?;
    for person in arr {
        let ids = person.get("id")?;
        if let Some(list) = ids.get("fec").and_then(|v| v.as_array()) {
            for f in list {
                if f.as_str().map(|s| s.to_ascii_uppercase()) == Some(want.clone()) {
                    return Some(person);
                }
            }
        }
    }
    None
}

/// Career assessment from a legislators JSON body + FEC id.
pub fn assess_from_congress_legislators(
    legislators_json: &str,
    fec_id: &str,
    as_of_year: i32,
) -> Option<CareerAssessment> {
    let person = congress_person_by_fec(legislators_json, fec_id)?;
    let (birth, spans) = spans_from_congress_legislator(&person);
    Some(assess_career(&spans, birth, as_of_year))
}

/// Political (+ optional) spans from an Open States person object.
pub fn spans_from_openstates_person(
    person: &Value,
) -> (Option<i32>, Option<String>, Vec<CareerSpan>) {
    const SRC: &str = "Open States";
    let profile = person
        .get("openstates_url")
        .and_then(|v| v.as_str())
        .or_else(|| person.get("id").and_then(|v| v.as_str()).map(|_| ""))
        .filter(|s| !s.is_empty())
        .map(|s| {
            if s.starts_with("http") {
                s.to_string()
            } else {
                format!(
                    "https://openstates.org/person/{}/",
                    s.trim_start_matches("ocd-person/")
                )
            }
        })
        .or_else(|| {
            person
                .get("id")
                .and_then(|v| v.as_str())
                .map(|id| format!("https://openstates.org/person/{id}/"))
        });

    let birth_year = person
        .get("birth_date")
        .and_then(|v| v.as_str())
        .and_then(year_from_date)
        .or_else(|| {
            person
                .pointer("/extras/birthday")
                .and_then(|v| v.as_str())
                .and_then(year_from_date)
        });

    let photo = person
        .get("image")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty() && (*s).starts_with("http"))
        .map(|s| s.to_string());

    let mut spans = Vec::new();
    let roles = person
        .get("roles")
        .and_then(|r| r.as_array())
        .cloned()
        .unwrap_or_default();
    for role in roles {
        let start = role
            .get("start_date")
            .or_else(|| role.get("start"))
            .and_then(|v| v.as_str())
            .and_then(year_from_date);
        let end = role
            .get("end_date")
            .or_else(|| role.get("end"))
            .and_then(|v| v.as_str())
            .and_then(year_from_date);
        let title = role.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let org = role
            .get("org_classification")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let district = role
            .get("district")
            .map(|d| match d {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            })
            .unwrap_or_default();
        let label = if !title.is_empty() {
            if district.is_empty() {
                title.to_string()
            } else {
                format!("{title} (District {district})")
            }
        } else if !org.is_empty() {
            if district.is_empty() {
                org.to_string()
            } else {
                format!("{org} District {district}")
            }
        } else {
            "Legislative service".into()
        };
        // Elected chamber / executive roles are political; staff titles too when present.
        spans.push(CareerSpan::new(
            LifeCategory::Political,
            label,
            start,
            end,
            SRC,
            profile.clone(),
        ));
    }

    // current_role only when roles empty
    if spans.is_empty() {
        if let Some(role) = person.get("current_role") {
            let start = role
                .get("start_date")
                .and_then(|v| v.as_str())
                .and_then(year_from_date);
            let title = role
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Legislative service");
            let district = role
                .get("district")
                .map(|d| match d {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .unwrap_or_default();
            let label = if district.is_empty() {
                title.to_string()
            } else {
                format!("{title} (District {district})")
            };
            spans.push(CareerSpan::new(
                LifeCategory::Political,
                label,
                start,
                None,
                SRC,
                profile,
            ));
        }
    }

    (birth_year, photo, spans)
}

/// Wikidata / Wikipedia ids from an Open States person (`identifiers`, `other_identifiers`, links).
pub fn openstates_external_ids(person: &Value) -> (Option<String>, Option<String>) {
    let mut wikidata: Option<String> = None;
    let mut wikipedia: Option<String> = None;

    let mut consider = |scheme: &str, ident: &str| {
        let s = scheme.trim().to_ascii_lowercase();
        let id = ident.trim();
        if id.is_empty() {
            return;
        }
        if (s == "wikidata" || s == "wd") && wikidata.is_none() {
            let q = if looks_like_wikidata_id(id) {
                id.to_string()
            } else if id.chars().all(|c| c.is_ascii_digit()) {
                format!("Q{id}")
            } else {
                return;
            };
            if looks_like_wikidata_id(&q) {
                wikidata = Some(q);
            }
        } else if (s == "wikipedia" || s == "enwiki" || s.contains("wikipedia"))
            && wikipedia.is_none()
        {
            // Accept "Title", "en:Title", or full wiki URL path.
            let t = id
                .trim_start_matches("https://en.wikipedia.org/wiki/")
                .trim_start_matches("http://en.wikipedia.org/wiki/")
                .trim_start_matches("enwiki:")
                .trim_start_matches("en:");
            let t = t.replace('_', " ");
            if !t.is_empty() {
                wikipedia = Some(t);
            }
        }
    };

    for key in ["identifiers", "other_identifiers"] {
        if let Some(arr) = person.get(key).and_then(|v| v.as_array()) {
            for row in arr {
                let scheme = row
                    .get("scheme")
                    .or_else(|| row.get("identifier_scheme"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let ident = row
                    .get("identifier")
                    .or_else(|| row.get("id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                consider(scheme, ident);
            }
        }
    }

    if wikipedia.is_none() {
        if let Some(arr) = person.get("links").and_then(|v| v.as_array()) {
            for link in arr {
                let url = link.get("url").and_then(|v| v.as_str()).unwrap_or("");
                let note = link
                    .get("note")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                if url.contains("wikipedia.org/wiki/")
                    || note.contains("wikipedia")
                    || note.contains("wiki")
                {
                    if let Some(rest) = url
                        .split("wikipedia.org/wiki/")
                        .nth(1)
                        .map(|s| s.split('?').next().unwrap_or(s))
                    {
                        let t = rest.replace('_', " ");
                        if !t.is_empty() {
                            wikipedia = Some(t);
                        }
                    }
                }
                if wikidata.is_none() && url.contains("wikidata.org/wiki/") {
                    if let Some(q) = url
                        .split("wikidata.org/wiki/")
                        .nth(1)
                        .map(|s| s.split('?').next().unwrap_or(s).trim())
                    {
                        if looks_like_wikidata_id(q) {
                            wikidata = Some(q.to_string());
                        }
                    }
                }
            }
        }
    }

    (wikidata, wikipedia)
}

/// Cited bio facts from Open States person extras / occupation fields (cite or omit).
pub fn facts_from_openstates_person(person: &Value) -> Vec<BioFact> {
    const SRC: &str = "Open States";
    let src_url = person
        .get("openstates_url")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| {
            person
                .get("id")
                .and_then(|v| v.as_str())
                .map(|id| format!("https://openstates.org/person/{id}/"))
        });

    let mut facts = Vec::new();
    let mut push = |kind: &str, text: String| {
        let t = text.split_whitespace().collect::<Vec<_>>().join(" ");
        if t.is_empty() {
            return;
        }
        let dup = facts
            .iter()
            .any(|f: &BioFact| f.kind == kind && f.text == t);
        if !dup {
            facts.push(BioFact {
                kind: kind.into(),
                text: t,
                source: SRC.into(),
                source_url: src_url.clone(),
                ..Default::default()
            });
        }
    };

    // Top-level occupation when present (rare).
    if let Some(occ) = person
        .get("occupation")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        push("work", format!("Occupation: {occ}"));
    }

    if let Some(extras) = person.get("extras").and_then(|v| v.as_object()) {
        for (key, val) in extras {
            let k = key.trim().to_ascii_lowercase().replace('-', "_");
            let text = match val {
                Value::String(s) => s.trim().to_string(),
                Value::Number(n) => n.to_string(),
                Value::Bool(_) | Value::Null | Value::Array(_) | Value::Object(_) => continue,
            };
            if text.is_empty() || text.len() > 400 {
                continue;
            }
            // Skip contact / private-ish noise.
            if matches!(
                k.as_str(),
                "email"
                    | "phone"
                    | "fax"
                    | "address"
                    | "capitol_email"
                    | "district_email"
                    | "gender"
                    | "religion"
            ) {
                continue;
            }
            match k.as_str() {
                "occupation" | "profession" | "job" | "employer" => {
                    let label = if k == "employer" {
                        "Employer"
                    } else {
                        "Occupation"
                    };
                    push("work", format!("{label}: {text}"));
                }
                "education" | "alma_mater" | "school" | "schools" => {
                    push("education", format!("Education: {text}"));
                }
                "spouse" | "spouse_name" | "partner" => {
                    push("family", format!("Spouse: {text}"));
                }
                "children" | "family" => {
                    push("family", format!("Family: {text}"));
                }
                "biography" | "bio" | "notes" => {
                    // Only short public blurbs — not free-form invention.
                    if text.len() <= 280 {
                        push("other", text);
                    }
                }
                _ => {}
            }
        }
    }

    facts
}

/// Build assessment + optional photo from Open States person JSON object string.
pub fn assess_from_openstates_person_json(
    person_json: &str,
    as_of_year: i32,
) -> Option<(CareerAssessment, Option<String>, Option<String>)> {
    let person: Value = serde_json::from_str(person_json).ok()?;
    let (birth, photo, spans) = spans_from_openstates_person(&person);
    let career = assess_career(&spans, birth, as_of_year);
    let photo_source = photo.as_ref().map(|_| "Open States".to_string());
    Some((career, photo, photo_source))
}

/// Judicial incumbent with no dated spans: still mark political category note.
pub fn judicial_placeholder_span(
    office: &str,
    source: Option<&str>,
    source_url: Option<&str>,
) -> CareerSpan {
    CareerSpan::new(
        LifeCategory::Political,
        format!("Judicial / bench — {office}"),
        None,
        None,
        source.unwrap_or("Ballot filing"),
        source_url.map(|s| s.to_string()),
    )
}

/// Merge span lists (dedupe by category+label+years+source).
pub fn merge_career_spans(base: &[CareerSpan], extra: &[CareerSpan]) -> Vec<CareerSpan> {
    let mut out = base.to_vec();
    for e in extra {
        let dup = out.iter().any(|b| {
            b.category == e.category
                && b.label == e.label
                && b.start_year == e.start_year
                && b.end_year == e.end_year
                && b.source == e.source
        });
        if !dup {
            out.push(e.clone());
        }
    }
    out
}

/// Build / refresh a dossier from career pieces (photo + facts filled later).
pub fn dossier_from_career(
    career: CareerAssessment,
    photo_url: Option<String>,
    photo_source: Option<String>,
    photo_source_url: Option<String>,
) -> PersonDossier {
    let mut empty_notes = Vec::new();
    if photo_url.is_none() {
        empty_notes.push("Photo: not found in sources we check yet.".into());
    }
    empty_notes.push("Family: not disclosed in sources we check.".into());
    if !career
        .fractions
        .iter()
        .any(|f| f.category != "political" && f.years > 0.0)
    {
        empty_notes.push("Education / work / business / legal: no cited timeline yet.".into());
    }
    empty_notes.push("Endorsements: none loaded yet.".into());
    empty_notes.push("Personal holdings: no public disclosure parsed yet.".into());

    PersonDossier {
        photo_url,
        photo_source,
        photo_source_url,
        facts: Vec::new(),
        career,
        endorsements: Vec::new(),
        holdings: Vec::new(),
        citizenship: CitizenshipRecord::default(),
        family_summary: FamilySummary::default(),
        orientation: OrientationRecord::default(),
        sources_checked: Vec::new(),
        disclosure_portals: Vec::new(),
        empty_notes,
    }
}

/// FEC outside-spend support rows → endorsement stubs (org backers).
pub fn endorsements_from_ie_support(
    committee: &str,
    support_oppose: &str,
    url: &str,
) -> Option<Endorsement> {
    let stance = support_oppose.trim().to_ascii_lowercase();
    let stance = if stance.contains("support") {
        "support"
    } else if stance.contains("oppose") {
        "oppose"
    } else {
        return None;
    };
    let org = committee.trim();
    if org.is_empty() {
        return None;
    }
    Some(Endorsement {
        org: org.into(),
        stance: stance.into(),
        source: "OpenFEC independent expenditures".into(),
        source_url: if url.trim().is_empty() {
            None
        } else {
            Some(url.trim().into())
        },
        kind: Some("ie".into()),
        trust: Some("filing".into()),
        date: None,
    })
}

/// Public congressional headshot via unitedstates/images (GitHub Pages; CORS OK).
/// Bioguide id e.g. `R000609`. No fetch — URL only; missing ids 404 at img load.
pub fn unitedstates_congress_photo_url(bioguide: &str) -> Option<String> {
    let id = bioguide.trim();
    if id.len() < 3 || id.len() > 12 {
        return None;
    }
    if !id.chars().all(|c| c.is_ascii_alphanumeric()) {
        return None;
    }
    // Normalize to Bioguide shape: letter + digits (preserve case used by CDN — upper).
    let id = id.to_ascii_uppercase();
    Some(format!(
        "https://unitedstates.github.io/images/congress/450x550/{id}.jpg"
    ))
}

/// Parsed chamber / bio page payload for dossier merge.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MemberBioParse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo_source_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub birth_year: Option<i32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub facts: Vec<BioFact>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub spans: Vec<CareerSpan>,
    /// Only when Wikidata/public text explicitly multi-citizenship (never invent).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub citizenship: Option<CitizenshipRecord>,
}

fn strip_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    let mut tag_buf = String::new();
    for c in s.chars() {
        match c {
            '<' => {
                in_tag = true;
                tag_buf.clear();
            }
            '>' => {
                in_tag = false;
                let t = tag_buf.to_ascii_lowercase();
                let name = t
                    .trim_start_matches('/')
                    .split(|ch: char| ch.is_whitespace() || ch == '>')
                    .next()
                    .unwrap_or("");
                // Block boundaries must not glue sentences: `</p><p>` → space
                let block = matches!(
                    name,
                    "p" | "div"
                        | "br"
                        | "li"
                        | "tr"
                        | "td"
                        | "th"
                        | "h1"
                        | "h2"
                        | "h3"
                        | "h4"
                        | "blockquote"
                        | "section"
                        | "article"
                        | "ul"
                        | "ol"
                );
                if block && !out.ends_with(char::is_whitespace) {
                    out.push(' ');
                }
            }
            _ if in_tag => tag_buf.push(c),
            _ => out.push(c),
        }
    }
    let mut text = html_unescape_basic(&out);
    // Soft-fix sentence glue only: lowercase/digit end → `.Next` (not `U.S.` / `A.B.`)
    if let Ok(re) = regex::Regex::new(r"([a-z0-9\)])\.([A-ZÁÉÍÓÚÑÜ])") {
        text = re.replace_all(&text, "$1. $2").into_owned();
    }
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn html_unescape_basic(s: &str) -> String {
    let mut t = s
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
        .replace("&ndash;", "–")
        .replace("&mdash;", "—")
        .replace("&ntilde;", "ñ")
        .replace("&Ntilde;", "Ñ")
        .replace("&eacute;", "é")
        .replace("&Eacute;", "É")
        .replace("&aacute;", "á")
        .replace("&iacute;", "í")
        .replace("&oacute;", "ó")
        .replace("&uacute;", "ú")
        .replace("&uuml;", "ü")
        .replace("&hellip;", "…")
        .replace("&#x27;", "'")
        .replace("&#241;", "ñ")
        .replace("&#209;", "Ñ")
        .replace("\\/", "/")
        .replace("\\\"", "\"");
    // Numeric &#NNN; / &#xHH; (common on court CMS + double-encoded embeds)
    if let Ok(re) = regex::Regex::new(r"&#(\d+);") {
        t = re
            .replace_all(&t, |caps: &regex::Captures| {
                caps.get(1)
                    .and_then(|m| m.as_str().parse::<u32>().ok())
                    .and_then(char::from_u32)
                    .map(|c| c.to_string())
                    .unwrap_or_default()
            })
            .into_owned();
    }
    if let Ok(re) = regex::Regex::new(r"&#x([0-9a-fA-F]+);") {
        t = re
            .replace_all(&t, |caps: &regex::Captures| {
                caps.get(1)
                    .and_then(|m| u32::from_str_radix(m.as_str(), 16).ok())
                    .and_then(char::from_u32)
                    .map(|c| c.to_string())
                    .unwrap_or_default()
            })
            .into_owned();
    }
    t
}

/// Decode HTML entities deeply enough to recover flcourts-media URLs inside CMS embeds.
fn html_unescape_deep(s: &str) -> String {
    let mut t = html_unescape_basic(s);
    // Second pass — embeds often double-encode `&` as `&amp;` then entities.
    t = html_unescape_basic(&t);
    t
}

fn abs_fl_url(path: &str) -> String {
    let p = path.trim();
    if p.starts_with("http://") || p.starts_with("https://") {
        p.to_string()
    } else if p.starts_with("//") {
        format!("https:{p}")
    } else if p.starts_with('/') {
        format!("https://www.flsenate.gov{p}")
    } else {
        format!("https://www.flsenate.gov/{p}")
    }
}

/// Parse year range at end of a line: `2016-2022`, `2016–2022`, `2016-current`, `in 2024`.
fn years_from_label(text: &str) -> (Option<i32>, Option<i32>) {
    let t = text.trim();
    // "… in 2024" / "Elected … 2024"
    if let Some(re) = regex_lite_year_in(t) {
        return (Some(re), None);
    }
    // trailing 2016-2022 or 2016–present
    let re = match regex::Regex::new(r"(?i)(\d{4})\s*[-–—]\s*(present|current|now|\d{4})\s*$") {
        Ok(r) => r,
        Err(_) => return (None, None),
    };
    if let Some(c) = re.captures(t) {
        let start = c.get(1).and_then(|m| m.as_str().parse().ok());
        let end_s = c.get(2).map(|m| m.as_str()).unwrap_or("");
        let end = if end_s.eq_ignore_ascii_case("present")
            || end_s.eq_ignore_ascii_case("current")
            || end_s.eq_ignore_ascii_case("now")
        {
            None
        } else {
            end_s.parse().ok()
        };
        return (start, end);
    }
    (None, None)
}

fn regex_lite_year_in(t: &str) -> Option<i32> {
    let re = regex::Regex::new(r"(?i)\bin\s+(19|20)\d{2}\b").ok()?;
    let c = re.find(t)?;
    let y: i32 = c
        .as_str()
        .chars()
        .filter(|ch| ch.is_ascii_digit())
        .collect::<String>()
        .parse()
        .ok()?;
    if (1900..=2100).contains(&y) {
        Some(y)
    } else {
        None
    }
}

fn category_from_affiliation_line(line: &str) -> LifeCategory {
    let l = line.to_ascii_lowercase();
    if l.contains("legislative assistant")
        || l.contains("chief of staff")
        || l.contains("campaign")
        || l.contains("elected")
        || l.contains("senate")
        || l.contains("house of representatives")
        || l.contains("representative")
        || l.contains("senator")
        || l.contains("legislature")
        || l.contains("committee chair")
        || l.contains("caucus")
        || l.contains("city council")
        || l.contains("commissioner")
        || l.contains("judge")
        || l.contains("court")
        || l.contains("government affairs")
        || l.contains("public policy")
        || l.contains("democratic executive")
        || l.contains("republican executive")
    {
        LifeCategory::Political
    } else if l.contains("attorney")
        || l.contains("lawyer")
        || l.contains("law firm")
        || l.contains("esq")
        || l.contains("legal")
    {
        LifeCategory::Legal
    } else if l.contains("llc")
        || l.contains("inc.")
        || l.contains("owner")
        || l.contains("founder")
        || l.contains("ceo")
        || l.contains("business")
        || l.contains("store manager")
    {
        LifeCategory::Business
    } else if l.contains("university")
        || l.contains("college")
        || l.contains("school")
        || l.contains("b.s.")
        || l.contains("b.a.")
        || l.contains("m.a.")
        || l.contains("ph.d")
        || l.contains("degree")
    {
        LifeCategory::Education
    } else {
        LifeCategory::Work
    }
}

/// Route FL chamber member HTML by page URL (Senate vs House).
pub fn parse_fl_chamber_member_html(html: &str, page_url: &str) -> MemberBioParse {
    let u = page_url.to_ascii_lowercase();
    if u.contains("flhouse") || u.contains("myfloridahouse") || u.contains("details.aspx") {
        parse_fl_house_member_html(html, page_url)
    } else {
        parse_fl_senate_member_html(html, page_url)
    }
}

/// Parse FL House member details page (best-effort; host often bot-blocked).
pub fn parse_fl_house_member_html(html: &str, page_url: &str) -> MemberBioParse {
    const SRC: &str = "Florida House of Representatives";
    let page = page_url.trim();
    let src_url = if page.is_empty() {
        None
    } else {
        Some(page.to_string())
    };
    let mut out = MemberBioParse {
        photo_source: Some(SRC.into()),
        photo_source_url: src_url.clone(),
        ..Default::default()
    };
    if html.len() < 80 {
        return out;
    }

    // Common photo patterns on house member pages / cards.
    if let Ok(re) = regex::Regex::new(r#"(?is)<img\b([^>]+)>"#) {
        for cap in re.captures_iter(html) {
            let attrs = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let src = attr_value(attrs, "src").unwrap_or_default();
            if src.is_empty() {
                continue;
            }
            let low = src.to_ascii_lowercase();
            if !(low.ends_with(".jpg")
                || low.ends_with(".jpeg")
                || low.ends_with(".png")
                || low.ends_with(".webp"))
            {
                continue;
            }
            if low.contains("logo")
                || low.contains("seal")
                || low.contains("icon")
                || low.contains("sprite")
            {
                continue;
            }
            let cls = attr_value(attrs, "class")
                .unwrap_or_default()
                .to_ascii_lowercase();
            let alt = attr_value(attrs, "alt")
                .unwrap_or_default()
                .to_ascii_lowercase();
            let hint = format!("{low} {cls} {alt}");
            if hint.contains("member")
                || hint.contains("rep")
                || hint.contains("portrait")
                || hint.contains("photo")
                || hint.contains("headshot")
                || alt.contains("representative")
            {
                out.photo_url = Some(abs_fl_house_url(&src, page));
                break;
            }
        }
    }

    // Label: value rows (table or definition lists)
    let row_re = match regex::Regex::new(
        r#"(?is)(?:Occupation|Spouse|Children|Education|Born|Birthday|Residence)\s*:?\s*</t[dh]>\s*<t[dh][^>]*>([\s\S]{0,600}?)</t[dh]>"#,
    ) {
        Ok(r) => r,
        Err(_) => return out,
    };
    // Also match "Occupation:</td><td>..." with label captured via lookbehind-ish split
    let row_re2 = regex::Regex::new(
        r#"(?is)(Occupation|Spouse|Children|Education|Born|Birthday|Residence)\s*:?\s*</t[dh]>\s*<t[dh][^>]*>([\s\S]{0,600}?)</t[dh]>"#,
    )
    .ok();
    let row_re = row_re2.unwrap_or(row_re);
    for cap in row_re.captures_iter(html) {
        let label = cap
            .get(1)
            .map(|m| m.as_str().trim().to_ascii_lowercase())
            .unwrap_or_default();
        let raw = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        let text = strip_tags(raw);
        if text.is_empty() || text.len() > 500 {
            continue;
        }
        let kind = match label.as_str() {
            "spouse" | "children" => "family",
            "education" => "education",
            "occupation" => "work",
            "born" | "birthday" => "other",
            "biography" | "biographical" => "other",
            _ => "other",
        };
        if kind == "other" && (label == "born" || label == "birthday") {
            if let Some(y) = year_from_born_text(&text) {
                out.birth_year = Some(y);
            }
            out.facts.push(BioFact {
                kind: "other".into(),
                text: format!("Born: {text}"),
                source: SRC.into(),
                source_url: src_url.clone(),
                ..Default::default()
            });
            continue;
        }
        if label.starts_with("biograph") && text.len() > 40 {
            // Leave full bio as other; light education/work heuristics later.
            out.facts.push(BioFact {
                kind: "other".into(),
                text: if text.len() > 400 {
                    format!("{}…", &text[..397])
                } else {
                    text
                },
                source: SRC.into(),
                source_url: src_url.clone(),
                ..Default::default()
            });
            continue;
        }
        out.facts.push(BioFact {
            kind: kind.into(),
            text,
            source: SRC.into(),
            source_url: src_url.clone(),
            ..Default::default()
        });
    }
    out
}

fn attr_value(attrs: &str, name: &str) -> Option<String> {
    let re = regex::Regex::new(&format!(
        r#"(?i)\b{}\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))"#,
        regex::escape(name)
    ))
    .ok()?;
    let c = re.captures(attrs)?;
    c.get(1)
        .or_else(|| c.get(2))
        .or_else(|| c.get(3))
        .map(|m| m.as_str().to_string())
}

fn abs_fl_house_url(path: &str, page_url: &str) -> String {
    let p = path.trim();
    if p.starts_with("http://") || p.starts_with("https://") {
        return p.to_string();
    }
    if p.starts_with("//") {
        return format!("https:{p}");
    }
    let base = if page_url.contains("myfloridahouse") {
        "https://www.myfloridahouse.gov"
    } else {
        "https://www.flhouse.gov"
    };
    if p.starts_with('/') {
        format!("{base}{p}")
    } else {
        format!("{base}/{p}")
    }
}

/// Parse FL Senate member page HTML → photo, family/education facts, career spans.
/// Cite Florida Senate; never invent missing fields.
pub fn parse_fl_senate_member_html(html: &str, page_url: &str) -> MemberBioParse {
    const SRC: &str = "Florida Senate";
    let page = page_url.trim();
    let src_url = if page.is_empty() {
        None
    } else {
        Some(page.to_string())
    };

    let mut out = MemberBioParse {
        photo_source: Some(SRC.into()),
        photo_source_url: src_url.clone(),
        ..Default::default()
    };

    // Photo: /PublishedContent/Senators/.../Photos/sNN_....jpg
    if let Ok(re) = regex::Regex::new(
        r#"(?is)<img[^>]+src="(/PublishedContent/Senators/[^"]+/Photos/[^"]+\.(?:jpg|jpeg|png|webp))"[^>]*>"#,
    ) {
        if let Some(c) = re.captures(html) {
            if let Some(m) = c.get(1) {
                out.photo_url = Some(abs_fl_url(m.as_str()));
            }
        }
    }
    if out.photo_url.is_none() {
        if let Ok(re) = regex::Regex::new(
            r#"(?is)<img[^>]+src="([^"]*Senator[^"]*\.(?:jpg|jpeg|png))"[^>]*alt="[^"]*Senator"#,
        ) {
            if let Some(c) = re.captures(html) {
                if let Some(m) = c.get(1) {
                    out.photo_url = Some(abs_fl_url(m.as_str()));
                }
            }
        }
    }

    // Biographical table rows: Occupation, Spouse, Education, Born
    let row_re = match regex::Regex::new(
        r#"(?is)<td[^>]*class="bold"[^>]*>\s*(Occupation|Spouse|Children|Child|Family|Education|Born|Birth)\s*:?\s*</td>\s*<td[^>]*>([\s\S]*?)</td>"#,
    ) {
        Ok(r) => r,
        Err(_) => return out,
    };

    for cap in row_re.captures_iter(html) {
        let label = cap
            .get(1)
            .map(|m| m.as_str().trim().to_ascii_lowercase())
            .unwrap_or_default();
        let raw = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        // Education may be a list of <li>
        let texts: Vec<String> = if label == "education" {
            let li_re = regex::Regex::new(r"(?is)<li[^>]*>([\s\S]*?)</li>").ok();
            if let Some(re) = li_re {
                let items: Vec<String> = re
                    .captures_iter(raw)
                    .filter_map(|c| c.get(1).map(|m| strip_tags(m.as_str())))
                    .filter(|s| !s.is_empty())
                    .collect();
                if !items.is_empty() {
                    items
                } else {
                    let t = strip_tags(raw);
                    if t.is_empty() {
                        vec![]
                    } else {
                        vec![t]
                    }
                }
            } else {
                vec![strip_tags(raw)]
            }
        } else {
            let t = strip_tags(raw);
            if t.is_empty() {
                vec![]
            } else {
                vec![t]
            }
        };

        for text in texts {
            if text.is_empty() || text == "—" || text.eq_ignore_ascii_case("n/a") {
                continue;
            }
            let kind = match label.as_str() {
                "spouse" | "children" | "child" | "family" => "family",
                "education" => "education",
                "occupation" => "work",
                "born" | "birth" => "other",
                _ => "other",
            };
            if kind == "other" && (label == "born" || label == "birth") {
                // Try birth year from "Month DD, YYYY" or leading year
                if let Some(y) = year_from_born_text(&text) {
                    out.birth_year = Some(y);
                }
                out.facts.push(BioFact {
                    kind: "other".into(),
                    text: format!("Born: {text}"),
                    source: SRC.into(),
                    source_url: src_url.clone(),
                    ..Default::default()
                });
                continue;
            }
            out.facts.push(BioFact {
                kind: kind.into(),
                text,
                source: SRC.into(),
                source_url: src_url.clone(),
                ..Default::default()
            });
        }
    }

    // Legislative Service list items
    if let Ok(sec) =
        regex::Regex::new(r#"(?is)<h4>\s*Legislative Service\s*</h4>\s*<ul[^>]*>([\s\S]*?)</ul>"#)
    {
        if let Some(c) = sec.captures(html) {
            let block = c.get(1).map(|m| m.as_str()).unwrap_or("");
            push_li_spans(
                block,
                LifeCategory::Political,
                SRC,
                src_url.as_deref(),
                &mut out.spans,
            );
        }
    }

    // Affiliations — mix of political staff and work
    if let Ok(sec) = regex::Regex::new(
        r#"(?is)<h4>\s*Affiliations\s*</h4>\s*<ul[^>]*>([\s\S]*?)</ul>\s*(?:<h4>|</div>)"#,
    ) {
        if let Some(c) = sec.captures(html) {
            let block = c.get(1).map(|m| m.as_str()).unwrap_or("");
            let li_re = regex::Regex::new(r"(?is)<li[^>]*>([\s\S]*?)</li>").ok();
            if let Some(re) = li_re {
                for li in re.captures_iter(block) {
                    let raw = li.get(1).map(|m| m.as_str()).unwrap_or("");
                    // Flatten nested ul text
                    let line = strip_tags(raw);
                    if line.is_empty() {
                        continue;
                    }
                    let cat = category_from_affiliation_line(&line);
                    let (start, end) = years_from_label(&line);
                    out.spans
                        .push(CareerSpan::new(cat, line, start, end, SRC, src_url.clone()));
                }
            }
        }
    }

    // Education facts → education spans when years present
    for f in out.facts.iter().filter(|f| f.kind == "education") {
        let (start, end) = years_from_label(&f.text);
        let y = start.or_else(|| {
            // trailing , 2003
            let re = regex::Regex::new(r",\s*((?:19|20)\d{2})\s*$").ok()?;
            re.captures(&f.text)
                .and_then(|c| c.get(1))
                .and_then(|m| m.as_str().parse().ok())
        });
        if let Some(y) = y {
            out.spans.push(CareerSpan::new(
                LifeCategory::Education,
                f.text.clone(),
                Some(y),
                end.or(Some(y)),
                SRC,
                src_url.clone(),
            ));
        }
    }

    out
}

fn push_li_spans(
    block: &str,
    default_cat: LifeCategory,
    source: &str,
    source_url: Option<&str>,
    out: &mut Vec<CareerSpan>,
) {
    let Ok(re) = regex::Regex::new(r"(?is)<li[^>]*>([\s\S]*?)</li>") else {
        return;
    };
    for li in re.captures_iter(block) {
        let line = strip_tags(li.get(1).map(|m| m.as_str()).unwrap_or(""));
        if line.is_empty() {
            continue;
        }
        let (start, end) = years_from_label(&line);
        out.push(CareerSpan::new(
            default_cat,
            line,
            start,
            end,
            source,
            source_url.map(|s| s.to_string()),
        ));
    }
}

fn year_from_born_text(text: &str) -> Option<i32> {
    // "January 1, 1970" or "1970 in Miami"
    let re = regex::Regex::new(r"(?i)(?:\b(?:19|20)\d{2}\b)|(?:,\s*((?:19|20)\d{2})\b)").ok()?;
    for c in re.captures_iter(text) {
        let s = c
            .get(1)
            .map(|m| m.as_str())
            .unwrap_or_else(|| c.get(0).map(|m| m.as_str()).unwrap_or(""));
        let digits: String = s.chars().filter(|ch| ch.is_ascii_digit()).collect();
        if let Ok(y) = digits.parse::<i32>() {
            if (1900..=2100).contains(&y) {
                return Some(y);
            }
        }
    }
    None
}

// --- Florida courts official bios (judge pages on flcourts / circuit sites) ---

/// One judge/justice link from a court index (Next.js or WP directory).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FlCourtsJudgeLink {
    pub name: String,
    /// Absolute page URL.
    pub url: String,
    /// `judge` | `folder` | `wp_bio`
    pub kind: String,
}

/// Official FL courts judges index URL for this ballot office, if known.
pub fn fl_courts_index_url(office: &str) -> Option<String> {
    let o = office.trim().to_ascii_lowercase();
    if o.is_empty() {
        return None;
    }
    if o.contains("supreme") {
        return Some("https://supremecourt.flcourts.gov/Justices".into());
    }
    // "District Court of Appeal (District 5)" / "5th DCA"
    if o.contains("district court of appeal") || o.contains(" dca") || o.contains("dca ") {
        if let Ok(re) = regex::Regex::new(r"(?i)(?:district|dca)\s*(\d{1,2})") {
            if let Some(c) = re.captures(&o) {
                let n: u32 = c.get(1)?.as_str().parse().ok()?;
                if (1..=6).contains(&n) {
                    return Some(format!("https://{n}dca.flcourts.gov/Judges"));
                }
            }
        }
    }
    // "Circuit Judge (Circuit 18, Group 13)"
    if o.contains("circuit") {
        if let Ok(re) = regex::Regex::new(r"(?i)circuit\s*(\d{1,2})") {
            if let Some(c) = re.captures(&o) {
                let n: u32 = c.get(1)?.as_str().parse().ok()?;
                return fl_circuit_directory_url(n);
            }
        }
    }
    // County judge — same circuit directory when circuit known from jurisdiction text
    None
}

/// Known circuit public sites with judge directories (expand as hosts verified).
/// Prefer hosts with per-judge bio pages when possible; roster-only URLs still help as portals.
pub fn fl_circuit_directory_url(circuit: u32) -> Option<String> {
    match circuit {
        2 => Some("https://2ndcircuit.leoncountyfl.gov/circuitJudges.php".into()),
        4 => Some(
            "https://www.jud4.org/circuit-and-county-judges-of-the-fourth-judicial-circuit".into(),
        ),
        5 => Some("https://www.circuit5.org/courts-judges/".into()),
        7 => Some("https://circuit7.org/judges/".into()),
        10 => Some("https://www.jud10.flcourts.org/gallery/judges".into()),
        12 => Some("https://www.jud12.flcourts.org/About/Judges".into()),
        15 => Some("https://www.15thcircuit.com/judges".into()),
        17 => Some("https://www.17th.flcourts.org/judges-and-judicial-staff/".into()),
        18 => Some("https://flcourts18.org/directory/".into()),
        20 => Some("https://www.ca.cjis20.org/About-The-Court/judiciary.aspx".into()),
        _ => None,
    }
}

/// Florida Bar public member search (portal only — often JS/Cloudflare; not auto-parsed).
pub fn fl_bar_member_search_url(name: &str) -> Option<String> {
    let parts: Vec<&str> = name.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }
    let first = parts[0].trim_matches(|c: char| !c.is_alphabetic());
    let last = parts[parts.len() - 1].trim_matches(|c: char| !c.is_alphabetic());
    if first.len() < 2 || last.len() < 2 {
        return None;
    }
    Some(format!(
        "https://www.floridabar.org/directories/find-mbr/?fName={}&lName={}&sdx=N",
        urlencoding_minimal(first),
        urlencoding_minimal(last)
    ))
}

/// One named external portal chip (Decisions empty / dossier).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NamedPortal {
    pub label: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Official opinion / court-record portals for a FL judicial office (link-out only).
pub fn fl_judicial_opinion_portals(office: &str) -> Vec<NamedPortal> {
    let o = office.trim().to_ascii_lowercase();
    let mut out = Vec::new();
    if o.contains("supreme") {
        out.push(NamedPortal {
            label: "FL Supreme Court opinions".into(),
            url: "https://supremecourt.flcourts.gov/Opinions".into(),
            note: Some("Official published opinions (link out).".into()),
        });
        return out;
    }
    if o.contains("district court of appeal") || o.contains(" dca") || o.contains("dca ") {
        if let Ok(re) = regex::Regex::new(r"(?i)(?:district|dca)\s*(\d{1,2})") {
            if let Some(c) = re.captures(&o) {
                if let Ok(n) = c.get(1).map(|m| m.as_str()).unwrap_or("").parse::<u32>() {
                    if (1..=6).contains(&n) {
                        out.push(NamedPortal {
                            label: format!("{n}DCA opinions"),
                            url: format!("https://{n}dca.flcourts.gov/Opinions"),
                            note: Some("District court published opinions (link out).".into()),
                        });
                    }
                }
            }
        }
    }
    out.push(NamedPortal {
        label: "FL court records search".into(),
        url: "https://www.flcourts.gov/Resources-Services/Court-Records/Court-Records-Search"
            .into(),
        note: Some("Statewide court records portal (not a bulk opinions API).".into()),
    });
    out
}

/// Decisions-tab portal chips for a Florida judge (directory + Bar + opinions).
pub fn fl_judge_decision_portals(office: &str, person_name: &str) -> Vec<NamedPortal> {
    let mut out = Vec::new();
    if let Some(u) = fl_courts_index_url(office) {
        out.push(NamedPortal {
            label: "Florida courts judge directory".into(),
            url: u,
            note: Some("Official court roster / bio pages when published.".into()),
        });
    } else {
        out.push(NamedPortal {
            label: "Florida Courts (flcourts.gov)".into(),
            url: "https://www.flcourts.gov/".into(),
            note: Some("Statewide court information.".into()),
        });
    }
    if let Some(bar) = fl_bar_member_search_url(person_name) {
        out.push(NamedPortal {
            label: "Florida Bar member search".into(),
            url: bar,
            note: Some(
                "Public lawyer directory. Auto-parse blocked by bot wall — open in browser.".into(),
            ),
        });
    }
    for p in fl_judicial_opinion_portals(office) {
        if !out.iter().any(|x| x.url == p.url) {
            out.push(p);
        }
    }
    out
}

fn fl_courts_next_json(html: &str) -> Option<Value> {
    let marker = r#"id="__NEXT_DATA__""#;
    let i = html
        .find(marker)
        .or_else(|| html.find("id='__NEXT_DATA__'"))?;
    let after = &html[i..];
    let start = after.find('>')? + 1;
    let rest = &after[start..];
    let end = rest.find("</script>")?;
    serde_json::from_str(rest[..end].trim()).ok()
}

fn fl_courts_abs(base_host: &str, path: &str) -> String {
    let p = path.trim();
    if p.starts_with("http://") || p.starts_with("https://") {
        return p.to_string();
    }
    if p.starts_with("//") {
        return format!("https:{p}");
    }
    let host = base_host.trim_end_matches('/');
    if p.starts_with('/') {
        format!("{host}{p}")
    } else {
        format!("{host}/{p}")
    }
}

fn fl_courts_base_from_page(page_url: &str) -> String {
    // https://5dca.flcourts.gov/Judges/foo → https://5dca.flcourts.gov
    if let Ok(re) = regex::Regex::new(r"(?i)^(https?://[^/]+)") {
        if let Some(c) = re.captures(page_url) {
            return c.get(1).map(|m| m.as_str()).unwrap_or(page_url).to_string();
        }
    }
    page_url.to_string()
}

/// Parse flcourts Next.js judges/justices index → name + URL links.
pub fn parse_fl_courts_next_index(html: &str, index_url: &str) -> Vec<FlCourtsJudgeLink> {
    let mut out = Vec::new();
    let Some(nd) = fl_courts_next_json(html) else {
        return out;
    };
    let base = fl_courts_base_from_page(index_url);
    let kids = nd
        .pointer("/props/pageProps/childrenInfos")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    for kid in kids {
        let typ = kid
            .pointer("/props/content/typeIdentifier")
            .or_else(|| kid.pointer("/content/typeIdentifier"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let name = kid
            .pointer("/props/content/name")
            .or_else(|| kid.pointer("/content/name"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let path = kid
            .pointer("/props/location/url")
            .or_else(|| kid.pointer("/props/content/url"))
            .or_else(|| kid.pointer("/content/url"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if name.is_empty() || path.is_empty() {
            continue;
        }
        if path.starts_with("http") && !path.contains("flcourts") {
            continue; // social links
        }
        let name_l = name.to_ascii_lowercase();
        if name_l.contains("former")
            || name_l.contains("senior")
            || name_l.contains("news")
            || name_l.contains("chronological")
            || name_l.contains("download")
            || name_l.contains("retention")
            || name_l.contains("transition")
            || name_l.contains("succession")
            || name_l.contains("listing of")
        {
            continue;
        }
        let kind = if typ == "judge" {
            "judge"
        } else if typ == "folder"
            && (name_l.contains("justice") || name_l.contains("judge") || name_l.contains("chief"))
        {
            "folder"
        } else if typ == "judge" || name_l.contains("judge") || name_l.contains("justice") {
            "judge"
        } else {
            continue;
        };
        out.push(FlCourtsJudgeLink {
            name,
            url: fl_courts_abs(&base, &path),
            kind: kind.into(),
        });
    }
    out
}

/// Circuit WP directory: collect `*-biography` profile links.
fn title_case_slug_words(slug: &str) -> String {
    slug.replace('-', " ")
        .split_whitespace()
        .filter(|w| {
            let l = w.to_ascii_lowercase();
            !matches!(
                l.as_str(),
                "judge" | "hon" | "the" | "honorable" | "biography"
            )
        })
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn push_circuit_judge_link(
    out: &mut Vec<FlCourtsJudgeLink>,
    name: String,
    url: String,
    kind: &str,
) {
    let name = name.trim().to_string();
    let url = url.trim().to_string();
    if name.len() < 3 || url.len() < 8 {
        return;
    }
    let low_u = url.to_ascii_lowercase();
    if low_u.contains("vacant")
        || low_u.contains("/feed")
        || low_u.contains("judge-tbd")
        || low_u.ends_with("/judge-tbd")
        || low_u.contains("retired-judges")
        || low_u.contains("past-chief")
    {
        return;
    }
    let low_n = name.to_ascii_lowercase();
    if low_n.contains("vacant") || low_n == "tbd" || low_n.starts_with("judge tbd") {
        return;
    }
    if out.iter().any(|x| x.url == url) {
        return;
    }
    out.push(FlCourtsJudgeLink {
        name,
        url,
        kind: kind.into(),
    });
}

/// Parse circuit court directory HTML for per-judge bio links.
/// Hosts: C18 `*-biography`, C7 `/judges/judge-*`, C12 `Judges-Magistrates/Judge-*`,
/// C10 gallery `/gallery/{slug}` + portrait alts.
pub fn parse_fl_circuit_directory_links(html: &str, index_url: &str) -> Vec<FlCourtsJudgeLink> {
    let mut out = Vec::new();
    let base = fl_courts_base_from_page(index_url);

    // C18-style: …-biography/ with optional anchor text
    if let Ok(re) =
        regex::Regex::new(r#"(?is)href=["']([^"']*biography[^"']*)["'][^>]*>([^<]{0,120})<"#)
    {
        for cap in re.captures_iter(html) {
            let href = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            let mut label = strip_tags(cap.get(2).map(|m| m.as_str()).unwrap_or(""))
                .trim()
                .to_string();
            if href.is_empty() {
                continue;
            }
            let url = fl_courts_abs(&base, href);
            if !url.to_ascii_lowercase().contains("biography") {
                continue;
            }
            if label.is_empty() || label.to_ascii_lowercase().starts_with("biography") {
                if let Some(slug) = url
                    .trim_end_matches('/')
                    .rsplit('/')
                    .next()
                    .map(|s| s.trim_end_matches("-biography"))
                {
                    label = title_case_slug_words(slug);
                }
            }
            if let Some(rest) = label.strip_prefix("Biography:") {
                label = rest.trim().to_string();
                if let Some((last, first)) = label.split_once(',') {
                    label = format!("{} {}", first.trim(), last.trim());
                }
            }
            push_circuit_judge_link(&mut out, label, url, "wp_bio");
        }
    }
    if let Ok(re2) = regex::Regex::new(r#"(?i)href=["']([^"']+-biography/?)["']"#) {
        for cap in re2.captures_iter(html) {
            let href = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            let url = fl_courts_abs(&base, href);
            let slug = url
                .trim_end_matches('/')
                .rsplit('/')
                .next()
                .unwrap_or("")
                .trim_end_matches("-biography");
            push_circuit_judge_link(&mut out, title_case_slug_words(slug), url, "wp_bio");
        }
    }

    // C7-style: /judges/judge-first-last/ with "Judge Name" anchor
    if let Ok(re) = regex::Regex::new(
        r#"(?is)href=["']([^"']*/judges/judge-[^"'#?]+/?)["'][^>]*>([^<]{0,120})<"#,
    ) {
        for cap in re.captures_iter(html) {
            let href = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            let mut label = strip_tags(cap.get(2).map(|m| m.as_str()).unwrap_or(""))
                .trim()
                .to_string();
            let url = fl_courts_abs(&base, href);
            if label.is_empty() || label.eq_ignore_ascii_case("read more") {
                let slug = url
                    .trim_end_matches('/')
                    .rsplit('/')
                    .next()
                    .unwrap_or("")
                    .trim_start_matches("judge-");
                label = title_case_slug_words(slug);
            } else {
                label = label
                    .trim_start_matches("Judge ")
                    .trim_start_matches("JUDGE ")
                    .trim()
                    .to_string();
            }
            push_circuit_judge_link(&mut out, label, url, "wp_bio");
        }
    }
    if let Ok(re) = regex::Regex::new(r#"(?i)href=["']([^"']*/judges/judge-[^"'#?]+/?)["']"#) {
        for cap in re.captures_iter(html) {
            let href = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            let url = fl_courts_abs(&base, href);
            let slug = url
                .trim_end_matches('/')
                .rsplit('/')
                .next()
                .unwrap_or("")
                .trim_start_matches("judge-");
            push_circuit_judge_link(&mut out, title_case_slug_words(slug), url, "wp_bio");
        }
    }

    // C12 DNN: /About-the-Court/Judges-Magistrates/Judge-First-Last
    if let Ok(re) = regex::Regex::new(
        r#"(?is)href=["']([^"']*Judges-Magistrates/Judge-[^"'#?]+)["'][^>]*>([^<]{0,120})<"#,
    ) {
        for cap in re.captures_iter(html) {
            let href = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            let mut label = strip_tags(cap.get(2).map(|m| m.as_str()).unwrap_or(""))
                .trim()
                .to_string();
            let url = fl_courts_abs(&base, href);
            if label.is_empty() || !label.to_ascii_lowercase().contains("judge") {
                if let Some(slug) = url.rsplit('/').next() {
                    label = title_case_slug_words(slug.trim_start_matches("Judge-"));
                }
            } else {
                label = label
                    .trim_start_matches("Judge ")
                    .trim_start_matches("JUDGE ")
                    .trim()
                    .to_string();
            }
            // Skip county-seat stubs like "Judge DeSoto County"
            let low = label.to_ascii_lowercase();
            if low.ends_with(" county") || low.contains(" requirements") {
                continue;
            }
            push_circuit_judge_link(&mut out, label, url, "wp_bio");
        }
    }
    if let Ok(re) =
        regex::Regex::new(r#"(?i)href=["']([^"']*Judges-Magistrates/Judge-[^"'#?]+)["']"#)
    {
        for cap in re.captures_iter(html) {
            let href = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            let url = fl_courts_abs(&base, href);
            let slug = url.rsplit('/').next().unwrap_or("");
            if slug.eq_ignore_ascii_case("Judge-TBD") {
                continue;
            }
            let name = title_case_slug_words(slug.trim_start_matches("Judge-"));
            let low = name.to_ascii_lowercase();
            if low.ends_with(" county") {
                continue;
            }
            push_circuit_judge_link(&mut out, name, url, "wp_bio");
        }
    }

    // C10 Drupal gallery: href /gallery/slug + alt "Portrait of Judge Name"
    if let Ok(re) = regex::Regex::new(
        r#"(?is)<a\b[^>]+href=["']([^"']+/gallery/[^"'#?]+)["'][^>]*>[\s\S]{0,400}?<img\b[^>]+alt=["']([^"']+)["']"#,
    ) {
        for cap in re.captures_iter(html) {
            let href = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            let alt = strip_tags(cap.get(2).map(|m| m.as_str()).unwrap_or(""))
                .trim()
                .to_string();
            let url = fl_courts_abs(&base, href);
            let low_u = url.to_ascii_lowercase();
            if low_u.ends_with("/gallery/judges")
                || low_u.ends_with("/gallery/")
                || low_u.contains("/gallery/chief")
                || low_u.contains("/gallery/county")
                || low_u.contains("/gallery/senior")
                || low_u.contains("magistrat")
            {
                continue;
            }
            let mut name = alt;
            for prefix in [
                "Portrait of Judge ",
                "Portrait of ",
                "Photo of Judge ",
                "Judge ",
            ] {
                if let Some(rest) = name.strip_prefix(prefix) {
                    name = rest.trim().to_string();
                    break;
                }
            }
            if name.eq_ignore_ascii_case("no photo available") || name.len() < 3 {
                // slug fallback
                let slug = url.trim_end_matches('/').rsplit('/').next().unwrap_or("");
                name = title_case_slug_words(slug);
            }
            push_circuit_judge_link(&mut out, name, url, "wp_bio");
        }
    }
    // Bare gallery person paths (no img alt)
    if let Ok(re) = regex::Regex::new(
        r#"(?i)href=["']((?:https?://[^"']+)?/gallery/[a-z0-9][a-z0-9\-]+)/?["']"#,
    ) {
        for cap in re.captures_iter(html) {
            let href = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            let url = fl_courts_abs(&base, href);
            let slug = url.trim_end_matches('/').rsplit('/').next().unwrap_or("");
            let skip = matches!(
                slug,
                "judges" | "chief_judge" | "chief-judge" | "county" | "senior" | "gallery"
            ) || slug.contains("magistrat");
            if skip {
                continue;
            }
            push_circuit_judge_link(&mut out, title_case_slug_words(slug), url, "wp_bio");
        }
    }

    out
}

/// High-precision unique match of person to court index links.
pub fn match_fl_courts_judge_link<'a>(
    links: &'a [FlCourtsJudgeLink],
    person_name: &str,
) -> Option<&'a FlCourtsJudgeLink> {
    let mut hits: Vec<&FlCourtsJudgeLink> = links
        .iter()
        .filter(|l| gp_title_matches_person(person_name, &l.name))
        .collect();
    if hits.is_empty() {
        // try first+last only
        let bases = bp_name_base_variants(person_name);
        hits = links
            .iter()
            .filter(|l| bases.iter().any(|b| gp_title_matches_person(b, &l.name)))
            .collect();
    }
    if hits.len() == 1 {
        return Some(hits[0]);
    }
    None
}

fn fl_courts_push_fact(out: &mut MemberBioParse, kind: &str, text: String, src: &str, url: &str) {
    let text = text.trim().to_string();
    if text.len() < 3 {
        return;
    }
    if out
        .facts
        .iter()
        .any(|f| f.kind == kind && f.text.eq_ignore_ascii_case(&text))
    {
        return;
    }
    out.facts.push(BioFact {
        kind: kind.into(),
        text,
        source: src.into(),
        source_url: Some(url.into()),
        ..Default::default()
    });
}

fn fl_courts_html5_items(html5: &str) -> Vec<String> {
    let mut items = Vec::new();
    if let Ok(re) = regex::Regex::new(r"(?is)<li\b[^>]*>(.*?)</li>") {
        for cap in re.captures_iter(html5) {
            let t = strip_tags(cap.get(1).map(|m| m.as_str()).unwrap_or(""));
            if t.len() >= 3 {
                items.push(t);
            }
        }
    }
    if items.is_empty() {
        let t = strip_tags(html5);
        if t.len() >= 3 {
            // split on ; or newlines already collapsed — keep as one or split ". "
            for part in t.split(';') {
                let p = part.trim();
                if p.len() >= 3 {
                    items.push(p.to_string());
                }
            }
        }
    }
    items
}

fn fl_courts_media_download_url(path_or_url: &str) -> Option<String> {
    let u = path_or_url.trim();
    if u.is_empty() {
        return None;
    }
    let low = u.to_ascii_lowercase();
    if low.contains("seal") || low.contains("demo-hero") || low.contains("logo") {
        return None;
    }
    if u.starts_with("http://") || u.starts_with("https://") {
        // Normalize known media hosts; drop query junk
        let clean = u.split('?').next().unwrap_or(u);
        return Some(clean.to_string());
    }
    if u.contains("/storage/images/") || u.starts_with("/var/site/") {
        let path = if let Some(i) = u.find("/var/site/") {
            &u[i..]
        } else {
            u
        };
        // Live SC/DCA embeds use flcourts-media.flcourts.gov
        return Some(format!(
            "https://flcourts-media.flcourts.gov/image/download{path}"
        ));
    }
    None
}

fn fl_courts_media_image_url(base: &str, image_val: &Value) -> Option<String> {
    let uri = image_val
        .pointer("/image/uri")
        .or_else(|| image_val.pointer("/image/url"))
        .or_else(|| image_val.pointer("/uri"))
        .or_else(|| image_val.pointer("/url"))
        .and_then(|v| v.as_str())?;
    fl_courts_media_download_url(uri).or_else(|| {
        let u = uri.trim();
        if u.is_empty() {
            None
        } else {
            Some(fl_courts_abs(base, u))
        }
    })
}

/// Recover headshot URLs from CMS description HTML (entity-encoded ibexa embeds).
fn fl_courts_photo_from_html_blob(desc_html: &str, short_html: &str, base: &str) -> Option<String> {
    let raw = format!("{desc_html}\n{short_html}");
    let decoded = html_unescape_deep(&raw);
    let mut candidates: Vec<String> = Vec::new();

    // Full media download URLs (prefer formal / larger aliases later)
    if let Ok(re) = regex::Regex::new(
        r#"https?://flcourts-media\.(?:flcourts\.gov|ccplatform\.net)/image/download(/var/site/storage/images/[A-Za-z0-9_./\-]+)"#,
    ) {
        for cap in re.captures_iter(&decoded) {
            let path = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            if let Some(u) = fl_courts_media_download_url(path) {
                candidates.push(u);
            }
        }
    }
    // Bare /var/site/storage paths
    if let Ok(re) =
        regex::Regex::new(r"(/var/site/storage/images/[A-Za-z0-9_./\-]+\.(?:jpg|jpeg|png|webp))")
    {
        for cap in re.captures_iter(&decoded) {
            let path = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            if let Some(u) = fl_courts_media_download_url(path) {
                candidates.push(u);
            }
        }
    }
    // Plain href/src images
    if let Ok(re) =
        regex::Regex::new(r#"(?i)(?:href|src)=["']([^"']+\.(?:jpg|jpeg|png|webp)[^"']*)["']"#)
    {
        for cap in re.captures_iter(&decoded) {
            let u = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            if let Some(abs) = fl_courts_media_download_url(u) {
                candidates.push(abs);
            } else if u.starts_with("http") && !u.to_ascii_lowercase().contains("seal") {
                candidates.push(u.split('?').next().unwrap_or(u).to_string());
            } else if u.starts_with('/') {
                let abs = fl_courts_abs(base, u);
                if !abs.to_ascii_lowercase().contains("seal") {
                    candidates.push(abs);
                }
            }
        }
    }

    // Prefer formal portrait / non-alias full-res
    let score = |u: &str| -> i32 {
        let l = u.to_ascii_lowercase();
        let mut s = 0;
        if l.contains("formal") {
            s += 10;
        }
        if l.contains("portrait") {
            s += 5;
        }
        if l.contains("_aliases") {
            s -= 3;
        }
        if l.contains("informal") {
            s += 2;
        }
        if l.ends_with(".jpg") || l.ends_with(".jpeg") {
            s += 1;
        }
        s
    };
    candidates.sort_by(|a, b| score(b).cmp(&score(a)));
    candidates.into_iter().next()
}

/// Split prose into sentences without breaking on initials / abbrevs (`A.`, `U.S.`, `St.`).
/// Manual scan — `regex` crate has no lookbehind.
fn fl_courts_split_sentences(text: &str) -> Vec<String> {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    if chars.is_empty() {
        return Vec::new();
    }
    let is_abbr = |w: &str| -> bool {
        let l = w.to_ascii_lowercase();
        l.len() <= 1
            || matches!(
                l.as_str(),
                "st" | "mr"
                    | "mrs"
                    | "ms"
                    | "dr"
                    | "jr"
                    | "sr"
                    | "vs"
                    | "us"
                    | "dc"
                    | "no"
                    | "gen"
                    | "gov"
                    | "sen"
                    | "rep"
                    | "hon"
                    | "ave"
                    | "blvd"
            )
    };
    let mut out = Vec::new();
    let mut start_i = 0usize; // index into chars
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i].1 == '.' {
            // whitespace then uppercase?
            let mut j = i + 1;
            while j < chars.len() && chars[j].1.is_whitespace() {
                j += 1;
            }
            let has_ws = j > i + 1;
            let next_upper =
                j < chars.len() && chars[j].1.is_uppercase() && chars[j].1.is_alphabetic();
            if has_ws && next_upper {
                // word immediately before '.'
                let mut k = i;
                while k > start_i && chars[k - 1].1.is_alphanumeric() {
                    k -= 1;
                }
                let word: String = chars[k..i].iter().map(|(_, c)| *c).collect();
                if !is_abbr(&word) {
                    let from = chars[start_i].0;
                    let to = chars[i].0;
                    let chunk = text[from..to].trim();
                    if chunk.len() >= 20 {
                        out.push(chunk.trim_end_matches('.').trim().to_string());
                    }
                    start_i = j;
                    i = j;
                    continue;
                }
            }
        }
        i += 1;
    }
    let from = chars[start_i].0;
    let tail = text[from..].trim();
    if tail.len() >= 20 {
        out.push(tail.trim_end_matches('.').trim().to_string());
    }
    out
}

/// Classify one prose sentence from a justice/judge narrative.
fn fl_courts_classify_sentence(s: &str) -> Option<&'static str> {
    let low = s.to_ascii_lowercase();
    if low.len() < 25 {
        return None;
    }
    // Skip chambers / contact boilerplate
    if low.contains("office information")
        || low.contains("phone number")
        || low.contains("mailing address")
        || low.contains("judicial assistant")
        || low.contains("staff attorney")
        || low.contains("clerkship") && low.contains("interested")
        || low.contains("publicinformation@")
        || low.contains("fellows program")
        || low.contains("law clerk recruitment")
    {
        return None;
    }
    if low.contains("wife")
        || low.contains("husband")
        || low.contains("spouse")
        || low.contains("children")
        || low.contains(" son")
        || low.contains("daughter")
    {
        return Some("family");
    }
    // Origin / childhood before school keywords
    if low.contains("grew up")
        || low.contains("birthplace")
        || low.contains("hometown")
        || low.contains("born ")
    {
        return Some("personal");
    }
    // Private practice / firm tenure → legal (before generic career verbs).
    if low.contains("private practice")
        || low.contains("law firm")
        || low.contains("law practice")
        || (low.contains("partner")
            && (low.contains("law")
                || low.contains("firm")
                || low.contains("llp")
                || low.contains("p.a")
                || low.contains("p.a.")))
        || (low.contains("associate")
            && (low.contains("law") || low.contains("firm") || low.contains("attorney")))
        || (low.contains("shareholder") && (low.contains("law") || low.contains("firm")))
        || low.contains("practiced law")
        || low.contains("practice of law")
        || low.contains("litigation attorney")
        || low.contains("trial attorney")
        || (low.contains("attorney at") || low.contains("attorney with"))
            && !low.contains("attorney general")
    {
        return Some("legal");
    }
    // Career verbs beat “law school” mentions inside clerkship sentences.
    if low.contains("clerked")
        || low.contains("appointed")
        || low.contains("served as")
        || low.contains("prior to joining")
        || low.contains("general counsel of")
        || low.contains("deputy attorney general")
        || low.contains("solicitor general")
    {
        return Some("office");
    }
    if low.contains("graduate of")
        || low.contains("graduated from")
        || low.contains("graduated ")
        || low.contains("received his j.d")
        || low.contains("received her j.d")
        || low.contains("earned a")
        || low.contains("j.d.,")
        || low.contains("j.d. ")
        || low.contains("ll.m")
        || (low.contains("university")
            && (low.contains("degree")
                || low.contains("b.a")
                || low.contains("b.s")
                || low.contains("m.a")
                || low.contains("ph.d")
                || low.contains("college of law")
                || low.contains("graduate")))
        || (low.contains("attended")
            && (low.contains("university") || low.contains("college of"))
            && !low.contains("clerk")
            && !low.contains("high school"))
    {
        return Some("education");
    }
    if low.contains("lives in") && !low.contains("wife") && !low.contains("husband") {
        return Some("personal");
    }
    if low.contains("justice")
        || low.contains("judge")
        || low.contains("general counsel")
        || low.contains("attorney general")
        || low.contains("solicitor")
        || low.contains("prior to")
        || low.contains("career in")
        || low.contains("worked as an attorney")
    {
        return Some("office");
    }
    None
}

fn fl_courts_parse_years_of_service(s: &str) -> (Option<i32>, Option<i32>) {
    let re = regex::Regex::new(r"(?i)((?:19|20)\d{2})\s*[-–—]\s*(present|(?:19|20)\d{2})").ok();
    if let Some(re) = re {
        if let Some(c) = re.captures(s) {
            let start = c.get(1).and_then(|m| m.as_str().parse().ok());
            let end_s = c.get(2).map(|m| m.as_str()).unwrap_or("");
            let end = if end_s.eq_ignore_ascii_case("present") {
                None
            } else {
                end_s.parse().ok()
            };
            return (start, end);
        }
    }
    // "Jan 2012 - Present"
    if let Ok(re) = regex::Regex::new(r"(?i)\b((?:19|20)\d{2})\b") {
        let years: Vec<i32> = re
            .captures_iter(s)
            .filter_map(|c| c.get(1)?.as_str().parse().ok())
            .collect();
        if let Some(&y) = years.first() {
            return (Some(y), None);
        }
    }
    (None, None)
}

/// Parse flcourts judge or justice page (`__NEXT_DATA__`) → bio.
pub fn parse_fl_courts_judge_html(html: &str, page_url: &str) -> MemberBioParse {
    const SRC: &str = "Florida Courts";
    let page = page_url.trim();
    let mut out = MemberBioParse {
        photo_source: Some(SRC.into()),
        photo_source_url: if page.is_empty() {
            None
        } else {
            Some(page.into())
        },
        ..Default::default()
    };
    let Some(nd) = fl_courts_next_json(html) else {
        return out;
    };
    let base = fl_courts_base_from_page(page);
    let pd = nd
        .pointer("/props/pageProps/pageData")
        .cloned()
        .unwrap_or(Value::Null);
    if pd.is_null() {
        return out;
    }

    let desc_html = pd
        .pointer("/description/html5")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let short_html = pd
        .pointer("/shortDescription/html5")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Photo — structured fields first, then CMS embeds in description HTML.
    if let Some(img) = pd.get("image").or_else(|| pd.get("altImage")) {
        if let Some(u) = fl_courts_media_image_url(&base, img) {
            if !u.to_ascii_lowercase().contains("seal") {
                out.photo_url = Some(u);
            }
        }
    }
    if out.photo_url.is_none() {
        if let Some(u) = fl_courts_photo_from_html_blob(desc_html, short_html, &base) {
            out.photo_url = Some(u);
        }
    }

    // yearsOfService → political span
    if let Some(yos) = pd.get("yearsOfService").and_then(|v| v.as_str()) {
        let yos = yos.trim();
        if !yos.is_empty() {
            fl_courts_push_fact(
                &mut out,
                "office",
                format!("Years of service: {yos}"),
                SRC,
                page,
            );
            let (start, end) = fl_courts_parse_years_of_service(yos);
            let label = pd
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Judge")
                .to_string();
            out.spans.push(CareerSpan::new(
                LifeCategory::Political,
                label,
                start,
                end,
                SRC,
                Some(page.into()),
            ));
        }
    }

    // degrees → education facts
    if let Some(h) = pd.pointer("/degrees/html5").and_then(|v| v.as_str()) {
        for item in fl_courts_html5_items(h) {
            fl_courts_push_fact(
                &mut out,
                "education",
                format!("Education: {item}"),
                SRC,
                page,
            );
        }
    }

    // offices / positions → career spans + work facts
    if let Some(h) = pd
        .pointer("/officesPositions/html5")
        .and_then(|v| v.as_str())
    {
        for item in fl_courts_html5_items(h) {
            let low = item.to_ascii_lowercase();
            let cat = if low.contains("judge")
                || low.contains("justice")
                || low.contains("solicitor")
                || low.contains("attorney general")
                || low.contains("appointed")
            {
                LifeCategory::Political
            } else if low.contains("attorney")
                || low.contains("counsel")
                || low.contains("law")
                || low.contains("partner")
                || low.contains("associate")
                || low.contains("private practice")
                || low.contains("firm")
            {
                LifeCategory::Legal
            } else {
                LifeCategory::Work
            };
            let (start, end) = fl_courts_parse_years_of_service(&item);
            out.spans.push(CareerSpan::new(
                cat,
                item.clone(),
                start,
                end,
                SRC,
                Some(page.into()),
            ));
            let fact_kind = match cat {
                LifeCategory::Legal => "legal",
                LifeCategory::Political => "office",
                _ => "work",
            };
            fl_courts_push_fact(&mut out, fact_kind, item, SRC, page);
        }
    }

    // Prose description (SC justices folders often only this)
    // Prefer real <p> blocks so contact sections stay separate from bio.
    let mut paragraphs: Vec<String> = Vec::new();
    for h in [desc_html, short_html] {
        if h.is_empty() {
            continue;
        }
        let mut found = false;
        if let Ok(re) = regex::Regex::new(r"(?is)<p\b[^>]*>(.*?)</p>") {
            for cap in re.captures_iter(h) {
                let t = strip_tags(cap.get(1).map(|m| m.as_str()).unwrap_or(""));
                if t.len() >= 25 {
                    paragraphs.push(t);
                    found = true;
                }
            }
        }
        if !found {
            let t = strip_tags(h);
            if t.len() >= 40 {
                paragraphs.push(t);
            }
        }
    }
    for para in &paragraphs {
        let low_p = para.to_ascii_lowercase();
        if low_p.contains("office information")
            || low_p.starts_with("the phone number")
            || low_p.contains("mailing address is")
        {
            continue;
        }
        for s in fl_courts_split_sentences(para) {
            if s.len() < 30 {
                continue;
            }
            let Some(kind) = fl_courts_classify_sentence(&s) else {
                continue;
            };
            let text = if kind == "education" && !s.to_ascii_lowercase().starts_with("education") {
                format!("Education: {s}")
            } else {
                s.clone()
            };
            fl_courts_push_fact(&mut out, kind, text, SRC, page);

            // Private practice / firm lines → dated Legal career spans when years present.
            if kind == "legal" {
                let (start, end) = fl_courts_parse_years_of_service(&s);
                if start.is_some() {
                    let label = if s.len() > 160 {
                        format!("{}…", s.chars().take(157).collect::<String>())
                    } else {
                        s.clone()
                    };
                    let dup = out.spans.iter().any(|sp| {
                        sp.category == "legal"
                            && sp.start_year == start
                            && sp.end_year == end
                            && sp.label == label
                    });
                    if !dup {
                        out.spans.push(CareerSpan::new(
                            LifeCategory::Legal,
                            label,
                            start,
                            end,
                            SRC,
                            Some(page.into()),
                        ));
                    }
                }
            }

            // Appointment → dated political span (require "appointed" + nearby year, not "vote in 2026")
            if kind == "office" {
                let low = s.to_ascii_lowercase();
                if low.contains("appointed") && !low.contains("merit-retention") {
                    if let Ok(re) = regex::Regex::new(
                        r"(?i)appointed(?:\s+\w+){0,12}\s+(?:on\s+)?(?:(?:January|February|March|April|May|June|July|August|September|October|November|December)\s+\d{1,2},\s+)?((?:19|20)\d{2})\b",
                    ) {
                        if let Some(c) = re.captures(&s) {
                            if let Ok(y) = c.get(1).map(|m| m.as_str()).unwrap_or("").parse::<i32>()
                            {
                                let label = pd
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("Florida Supreme Court")
                                    .to_string();
                                if !out.spans.iter().any(|sp| {
                                    sp.category == "political" && sp.start_year == Some(y)
                                }) {
                                    out.spans.push(CareerSpan::new(
                                        LifeCategory::Political,
                                        label,
                                        Some(y),
                                        None,
                                        SRC,
                                        Some(page.into()),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Structured description fields: Birthplace / Spouse / Children (DCA-style)
    if let Ok(re) = regex::Regex::new(
        r"(?i)<strong[^>]*>\s*(Birthplace|Hometown|Spouse|Children)\s*:?\s*</strong>\s*([^<]+)",
    ) {
        for cap in re.captures_iter(desc_html) {
            let k = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            let v = strip_tags(cap.get(2).map(|m| m.as_str()).unwrap_or(""));
            if v.is_empty() {
                continue;
            }
            let kind = if k.eq_ignore_ascii_case("spouse") || k.eq_ignore_ascii_case("children") {
                "family"
            } else {
                "personal"
            };
            fl_courts_push_fact(&mut out, kind, format!("{k}: {v}"), SRC, page);
        }
    }

    out
}

/// Circuit WordPress judge “biography” page (often chambers + photo; bio prose when present).
pub fn parse_fl_circuit_wp_bio_html(html: &str, page_url: &str) -> MemberBioParse {
    const SRC: &str = "Florida circuit court";
    let page = page_url.trim();
    let mut out = MemberBioParse {
        photo_source: Some(SRC.into()),
        photo_source_url: if page.is_empty() {
            None
        } else {
            Some(page.into())
        },
        ..Default::default()
    };
    if html.len() < 200 {
        return out;
    }
    // Photo: Judge_Name uploads / gallery portraits
    if let Ok(re) = regex::Regex::new(r#"(?is)<img\b[^>]+src=["']([^"']+)["'][^>]*>"#) {
        for cap in re.captures_iter(html) {
            let src = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let low = src.to_ascii_lowercase();
            if !(low.contains("judge")
                || low.contains("wp-content/uploads")
                || low.contains("/galleries/")
                || low.contains("portrait"))
            {
                continue;
            }
            if low.contains("logo") || low.contains("icon") || low.contains("seal") {
                continue;
            }
            if low.ends_with(".jpg")
                || low.ends_with(".jpeg")
                || low.ends_with(".png")
                || low.ends_with(".webp")
                || low.contains(".jpg")
                || low.contains(".png")
                || low.contains(".webp")
            {
                let abs = if src.starts_with("http") {
                    src.split('?').next().unwrap_or(src).to_string()
                } else {
                    fl_courts_abs(&fl_courts_base_from_page(page), src)
                };
                out.photo_url = Some(abs);
                break;
            }
        }
    }
    // Title / Honorable name
    if let Ok(re) = regex::Regex::new(
        r"(?is)(?:The Honorable|CIRCUIT JUDGE|COUNTY JUDGE|COUNTY COURT JUDGE)\s+([^<\n]{3,80})",
    ) {
        if let Some(c) = re.captures(html) {
            let n = strip_tags(c.get(1).map(|m| m.as_str()).unwrap_or(""));
            if n.len() >= 3 {
                fl_courts_push_fact(&mut out, "office", format!("Listed as: {n}"), SRC, page);
            }
        }
    }
    // Division / assignment lines
    if let Ok(re) = regex::Regex::new(r"(?i)(?:Division|Circuit Assignment)\s*:\s*([^<\n]{2,120})")
    {
        if let Some(c) = re.captures(html) {
            let d = strip_tags(c.get(1).map(|m| m.as_str()).unwrap_or(""));
            if d.len() >= 2 {
                fl_courts_push_fact(&mut out, "office", format!("Assignment: {d}"), SRC, page);
            }
        }
    }
    // Prefer <p> bio blocks; fall back to full plain text sentences.
    let cleaned = regex::Regex::new(r"(?is)<script[\s\S]*?</script>|<style[\s\S]*?</style>")
        .map(|re| re.replace_all(html, " ").into_owned())
        .unwrap_or_else(|_| html.to_string());
    let mut paragraphs: Vec<String> = Vec::new();
    if let Ok(re) = regex::Regex::new(r"(?is)<p\b[^>]*>(.*?)</p>") {
        for cap in re.captures_iter(&cleaned) {
            let t = strip_tags(cap.get(1).map(|m| m.as_str()).unwrap_or(""));
            if t.len() >= 40 {
                paragraphs.push(t);
            }
        }
    }
    if paragraphs.is_empty() {
        let plain = strip_tags(&cleaned);
        if plain.len() >= 40 {
            paragraphs.push(plain);
        }
    }
    for para in &paragraphs {
        let low_p = para.to_ascii_lowercase();
        if low_p.contains("public records")
            || low_p.contains("e-mail addresses are public")
            || low_p.contains("disclaimer")
            || low_p.contains("jacs")
            || low_p.contains("scheduling")
            || low_p.contains("zoom")
            || low_p.contains("webmaster")
            || low_p.contains("do not send electronic mail")
        {
            continue;
        }
        for s in fl_courts_split_sentences(para) {
            if s.len() < 40 || s.len() > 500 {
                continue;
            }
            let Some(kind) = fl_courts_classify_sentence(&s) else {
                // Keep strong appointment / education lines the classifier misses.
                let low = s.to_ascii_lowercase();
                if low.contains("appointed")
                    || low.contains("graduated")
                    || low.contains("juris doctor")
                    || low.contains("earned his")
                    || low.contains("earned her")
                {
                    let k = if low.contains("graduated")
                        || low.contains("university")
                        || low.contains("degree")
                        || low.contains("juris")
                        || low.contains("earned")
                    {
                        "education"
                    } else {
                        "office"
                    };
                    fl_courts_push_fact(&mut out, k, s.clone(), SRC, page);
                }
                continue;
            };
            let text = if kind == "education" && !s.to_ascii_lowercase().starts_with("education") {
                format!("Education: {s}")
            } else {
                s.clone()
            };
            fl_courts_push_fact(&mut out, kind, text, SRC, page);
            if kind == "legal" {
                let (start, end) = fl_courts_parse_years_of_service(&s);
                if start.is_some() {
                    let label = if s.len() > 160 {
                        format!("{}…", s.chars().take(157).collect::<String>())
                    } else {
                        s.clone()
                    };
                    if !out.spans.iter().any(|sp| {
                        sp.category == "legal"
                            && sp.start_year == start
                            && sp.end_year == end
                            && sp.label == label
                    }) {
                        out.spans.push(CareerSpan::new(
                            LifeCategory::Legal,
                            label,
                            start,
                            end,
                            SRC,
                            Some(page.into()),
                        ));
                    }
                }
            }
            if kind == "office" {
                let low = s.to_ascii_lowercase();
                if low.contains("appointed") {
                    if let Ok(re) = regex::Regex::new(
                        r"(?i)appointed(?:\s+\w+){0,12}\s+(?:on\s+)?(?:(?:January|February|March|April|May|June|July|August|September|October|November|December)\s+\d{1,2},\s+)?((?:19|20)\d{2})\b",
                    ) {
                        if let Some(c) = re.captures(&s) {
                            if let Ok(y) = c.get(1).map(|m| m.as_str()).unwrap_or("").parse::<i32>()
                            {
                                if !out.spans.iter().any(|sp| {
                                    sp.category == "political" && sp.start_year == Some(y)
                                }) {
                                    out.spans.push(CareerSpan::new(
                                        LifeCategory::Political,
                                        "Florida circuit/county bench",
                                        Some(y),
                                        None,
                                        SRC,
                                        Some(page.into()),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    out
}

/// Merge facts into a dossier (dedupe by kind+text). Refresh empty_notes.
pub fn merge_facts_into_dossier(d: &mut PersonDossier, facts: &[BioFact]) {
    for f in facts {
        let dup = d.facts.iter().any(|x| x.kind == f.kind && x.text == f.text);
        if !dup {
            d.facts.push(f.clone());
        }
    }
    let has_family = d.facts.iter().any(|f| f.kind == "family");
    let has_edu = d.facts.iter().any(|f| f.kind == "education");
    let has_work = d
        .facts
        .iter()
        .any(|f| matches!(f.kind.as_str(), "work" | "business" | "legal"));
    d.empty_notes.retain(|n| {
        let l = n.to_ascii_lowercase();
        if has_family && l.starts_with("family:") {
            return false;
        }
        if (has_edu || has_work) && l.starts_with("education / work / business / legal") {
            return false;
        }
        true
    });
    if has_edu && !has_work {
        // keep partial note? skip — facts section shows what's known
    }
}

/// Source priority for coalesced text richness (lower = stronger).
fn coalesce_source_rank(name: &str) -> u8 {
    let n = name.trim().to_ascii_lowercase();
    if n.contains("house.gov")
        || n.contains("senate.gov")
        || n.contains("official")
        || n.contains("fl house")
        || n.contains("fl senate")
        || n.contains("myfloridahouse")
        || n.contains("flsenate")
    {
        return 0;
    }
    if n.contains("ballotpedia") {
        return 1;
    }
    if n.contains("campaign") {
        return 2;
    }
    if n.contains("wikipedia") {
        return 3;
    }
    if n.contains("wikidata") {
        return 4;
    }
    if n.contains("dbpedia") {
        return 5;
    }
    if n.contains("openfec") || n.contains("fec") {
        return 6;
    }
    if n.contains("grokipedia") {
        return 7;
    }
    if n.contains("open states") {
        return 5;
    }
    8
}

fn coalesce_norm_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = false;
    for ch in s.chars() {
        let c = match ch {
            'Á' | 'À' | 'Ä' | 'Â' | 'Ã' | 'á' | 'à' | 'ä' | 'â' | 'ã' => 'a',
            'É' | 'È' | 'Ë' | 'Ê' | 'é' | 'è' | 'ë' | 'ê' => 'e',
            'Í' | 'Ì' | 'Ï' | 'Î' | 'í' | 'ì' | 'ï' | 'î' => 'i',
            'Ó' | 'Ò' | 'Ö' | 'Ô' | 'Õ' | 'ó' | 'ò' | 'ö' | 'ô' | 'õ' => 'o',
            'Ú' | 'Ù' | 'Ü' | 'Û' | 'ú' | 'ù' | 'ü' | 'û' => 'u',
            'Ñ' | 'ñ' => 'n',
            'Ç' | 'ç' => 'c',
            _ => ch,
        };
        let c = c.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_space = false;
        } else if c.is_whitespace() || c == '-' || c == '/' {
            if !prev_space && !out.is_empty() {
                out.push(' ');
                prev_space = true;
            }
        }
        // drop other punctuation
    }
    out.trim().to_string()
}

fn coalesce_strip_label(text: &str) -> &str {
    text.split_once(": ").map(|(_, v)| v).unwrap_or(text).trim()
}

fn coalesce_edu_degrees(text: &str) -> Vec<&'static str> {
    let t = text.to_ascii_lowercase();
    let mut deg = Vec::new();
    let checks: &[(&[&str], &str)] = &[
        (
            &[
                "juris doctor",
                "j.d.",
                " jd",
                "jd,",
                "law degree",
                "law school",
            ],
            "jd",
        ),
        (&["m.d.", " md", "doctor of medicine"], "md"),
        (&["ph.d.", "phd", "doctorate"], "phd"),
        (&["ll.m.", "llm"], "llm"),
        (
            &["master of", "master's", "masters", "m.a.", "m.s.", "mba"],
            "masters",
        ),
        (
            &[
                "bachelor of",
                "bachelor's",
                "bachelors",
                "b.a.",
                "b.s.",
                "ba ",
                "bs ",
            ],
            "bachelors",
        ),
        (&["associate", "a.a.", "a.s."], "associates"),
        (&["high school", "secondary school"], "hs"),
    ];
    for (pats, tag) in checks {
        if pats.iter().any(|p| t.contains(p)) {
            deg.push(*tag);
        }
    }
    deg
}

fn coalesce_edu_school_key(text: &str) -> Option<String> {
    let raw = coalesce_strip_label(text);
    let mut s = coalesce_norm_text(raw);
    if s.is_empty() {
        return None;
    }
    // Degree-only rows (no school token) → no school key.
    let deg_only = {
        let d = coalesce_edu_degrees(text);
        !d.is_empty()
            && !s.contains("university")
            && !s.contains("college")
            && !s.contains("school")
            && !s.contains("institute")
            && !s.contains("academy")
            && s.split_whitespace().count() <= 4
    };
    if deg_only {
        return None;
    }
    for prefix in [
        "university of ",
        "the university of ",
        "college of ",
        "the college of ",
    ] {
        if let Some(rest) = s.strip_prefix(prefix) {
            s = rest.to_string();
            break;
        }
    }
    // Drop degree tokens from school key
    for tok in [
        "juris doctor",
        "bachelor of arts",
        "bachelor of science",
        "bachelor of",
        "bachelors",
        "bachelor s",
        "bachelor",
        "master of arts",
        "master of science",
        "master of",
        "masters",
        "master s",
        "master",
        "doctor of",
        "ph d",
        "phd",
        "j d",
        "jd",
        "m d",
        "md",
        "ll m",
        "llm",
        "mba",
        "b a",
        "b s",
        "m a",
        "m s",
        "degree",
        "graduated",
        "attended",
    ] {
        s = s.replace(tok, " ");
    }
    // Drop years
    let re = regex::Regex::new(r"\b(?:19|20)\d{2}\b").ok();
    if let Some(re) = re {
        s = re.replace_all(&s, " ").to_string();
    }
    s = coalesce_norm_text(&s);
    // Collapse common aliases
    if s.contains("stetson") {
        return Some("stetson".into());
    }
    if s.contains("florida") && (s.contains("uf") || s.split_whitespace().count() <= 2) {
        // "florida" alone after stripping "university of"
        if s == "florida" || s.starts_with("florida ") {
            return Some("florida".into());
        }
    }
    if s.contains("tarpon springs") {
        return Some("tarpon springs high".into());
    }
    // St. Petersburg Junior College ≈ St. Petersburg College
    if s.contains("petersburg") {
        let t = s
            .replace("junior college", "college")
            .replace(" community college", " college");
        return Some(coalesce_norm_text(&t));
    }
    if s.is_empty() {
        return None;
    }
    // First 3 significant tokens
    let key: String = s
        .split_whitespace()
        .filter(|w| w.len() > 1)
        .take(4)
        .collect::<Vec<_>>()
        .join(" ");
    if key.is_empty() {
        None
    } else {
        Some(key)
    }
}

fn coalesce_edu_cluster_key(text: &str) -> String {
    if let Some(school) = coalesce_edu_school_key(text) {
        return format!("school:{school}");
    }
    let degs = coalesce_edu_degrees(text);
    if !degs.is_empty() {
        return format!("deg:{}", degs.join("+"));
    }
    format!("raw:{}", coalesce_norm_text(coalesce_strip_label(text)))
}

fn coalesce_work_profession_key(text: &str) -> Option<String> {
    let raw = coalesce_strip_label(text);
    let n = coalesce_norm_text(raw);
    if n.is_empty() {
        return None;
    }
    // Employer rows stay separate
    if text.to_ascii_lowercase().contains("employer") {
        return Some(format!("employer:{}", n));
    }
    let synonyms: &[(&[&str], &str)] = &[
        (
            &[
                "attorney",
                "lawyer",
                "jurist",
                "legal practice",
                "law practice",
                "practicing law",
            ],
            "profession:attorney",
        ),
        (
            &["physician", "doctor", "medical doctor"],
            "profession:physician",
        ),
        (
            &["teacher", "educator", "professor", "instructor"],
            "profession:educator",
        ),
        (
            &[
                "businessman",
                "businesswoman",
                "business owner",
                "entrepreneur",
            ],
            "profession:business",
        ),
        (
            &["farmer", "rancher", "agriculture"],
            "profession:agriculture",
        ),
        (&["engineer"], "profession:engineer"),
        (
            &["journalist", "reporter", "author", "writer"],
            "profession:writer",
        ),
        (&["pastor", "minister", "clergy"], "profession:clergy"),
    ];
    for (pats, key) in synonyms {
        if pats.iter().any(|p| n.contains(p)) {
            return Some((*key).into());
        }
    }
    // Generic occupation/profession line
    let low = text.to_ascii_lowercase();
    if low.starts_with("occupation") || low.starts_with("profession") || low.starts_with("work") {
        return Some(format!("profession:{}", n));
    }
    None
}

fn coalesce_work_cluster_key(text: &str) -> String {
    if let Some(k) = coalesce_work_profession_key(text) {
        return k;
    }
    format!("work:{}", coalesce_norm_text(coalesce_strip_label(text)))
}

fn coalesce_family_cluster_key(text: &str) -> String {
    let low = text.to_ascii_lowercase();
    let body = coalesce_norm_text(coalesce_strip_label(text));
    if low.contains("spouse")
        || low.starts_with("married")
        || low.contains("wife")
        || low.contains("husband")
    {
        return "family:spouse".into();
    }
    if low.contains("children")
        || low.contains("child")
        || low.contains("son")
        || low.contains("daughter")
    {
        return "family:children".into();
    }
    if low.contains("parent") || low.contains("father") || low.contains("mother") {
        return "family:parents".into();
    }
    format!("family:other:{body}")
}

fn coalesce_other_cluster_key(text: &str) -> String {
    let pref = other_fact_prefix(text).to_ascii_lowercase();
    let pref = pref.trim_end_matches(':').trim();
    if pref.is_empty() {
        format!("other:{}", coalesce_norm_text(text))
    } else {
        format!("other:{pref}")
    }
}

fn coalesce_text_richness(text: &str) -> (i32, i32, i32) {
    // Prefer longer informative text, more digits (years), degree tokens.
    let len = text.chars().filter(|c| !c.is_whitespace()).count() as i32;
    let digits = text.chars().filter(|c| c.is_ascii_digit()).count() as i32;
    let deg = coalesce_edu_degrees(text).len() as i32;
    (deg * 100 + digits * 10 + len, digits, deg)
}

fn coalesce_prefer_text(a: &str, b: &str, a_src: &str, b_src: &str) -> bool {
    // true if a should beat b
    let ra = coalesce_text_richness(a);
    let rb = coalesce_text_richness(b);
    if ra != rb {
        return ra > rb;
    }
    let sa = coalesce_source_rank(a_src);
    let sb = coalesce_source_rank(b_src);
    if sa != sb {
        return sa < sb;
    }
    // Prefer Profession: over Occupation:
    let al = a.to_ascii_lowercase();
    let bl = b.to_ascii_lowercase();
    let a_prof = al.starts_with("profession");
    let b_prof = bl.starts_with("profession");
    if a_prof != b_prof {
        return a_prof;
    }
    a.len() >= b.len()
}

fn coalesce_merge_sources(into: &mut Vec<BioFactSource>, extra: &[BioFactSource]) {
    for s in extra {
        let name_l = s.name.trim().to_ascii_lowercase();
        if name_l.is_empty() {
            continue;
        }
        if let Some(existing) = into
            .iter_mut()
            .find(|e| e.name.trim().eq_ignore_ascii_case(s.name.trim()))
        {
            if existing.url.is_none() && s.url.is_some() {
                existing.url = s.url.clone();
            }
            continue;
        }
        into.push(s.clone());
    }
    into.sort_by(|a, b| {
        coalesce_source_rank(&a.name)
            .cmp(&coalesce_source_rank(&b.name))
            .then_with(|| {
                a.name
                    .to_ascii_lowercase()
                    .cmp(&b.name.to_ascii_lowercase())
            })
    });
}

fn coalesce_absorb_degree_into_school(school_text: &str, deg_text: &str) -> String {
    let school_degs = coalesce_edu_degrees(school_text);
    let deg_degs = coalesce_edu_degrees(deg_text);
    if deg_degs.is_empty() {
        return school_text.to_string();
    }
    // Already has this degree class
    if deg_degs.iter().all(|d| school_degs.contains(d)) {
        return school_text.to_string();
    }
    let add = coalesce_strip_label(deg_text).trim();
    if add.is_empty() {
        return school_text.to_string();
    }
    // Append missing degree token if school line lacks it
    if school_text
        .to_ascii_lowercase()
        .contains(&add.to_ascii_lowercase())
    {
        return school_text.to_string();
    }
    format!("{school_text}, {add}")
}

/// Merge duplicate bio signals into one row per cluster; multi-cite on each.
/// Pure — does not invent facts. Source priority for text: official ≥ BP ≥ campaign ≥
/// Wikipedia/Wikidata ≥ DBpedia ≥ FEC ≥ Grokipedia.
pub fn coalesce_dossier_facts(facts: &[BioFact]) -> Vec<BioFact> {
    if facts.is_empty() {
        return Vec::new();
    }

    #[derive(Clone)]
    struct Cluster {
        kind: String,
        text: String,
        sources: Vec<BioFactSource>,
        primary_source: String,
        order: usize,
        /// education: school key if any
        edu_school: Option<String>,
        /// education: is degree-only (no school)
        edu_deg_only: bool,
    }

    let mut clusters: Vec<Cluster> = Vec::new();
    // key -> index
    let mut index: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for (order, f) in facts.iter().enumerate() {
        let kind = f.kind.as_str();
        let key = match kind {
            "education" => format!("education:{}", coalesce_edu_cluster_key(&f.text)),
            "work" | "legal" | "business" => {
                // Normalize legal/work profession synonyms into one bucket kind=legal when attorney
                let ck = coalesce_work_cluster_key(&f.text);
                let bucket = if ck == "profession:attorney" {
                    "legal"
                } else {
                    kind
                };
                format!("{bucket}:{ck}")
            }
            "family" => format!("{}", coalesce_family_cluster_key(&f.text)),
            "other" => coalesce_other_cluster_key(&f.text),
            _ => format!("{kind}:{}", coalesce_norm_text(&f.text)),
        };

        let srcs = f.all_sources();
        let primary = srcs
            .first()
            .map(|s| s.name.clone())
            .unwrap_or_else(|| f.source.clone());

        if let Some(&idx) = index.get(&key) {
            let c = &mut clusters[idx];
            // Prefer richer text
            if coalesce_prefer_text(&f.text, &c.text, &primary, &c.primary_source) {
                c.text = f.text.clone();
                c.primary_source = primary;
                // Prefer attorney kind over work when merging profession
                if key.contains("profession:attorney") {
                    c.kind = "legal".into();
                } else if c.kind == "work" && kind == "legal" {
                    c.kind = "legal".into();
                }
            } else if kind == "legal" && c.kind == "work" && key.contains("profession:attorney") {
                c.kind = "legal".into();
            }
            coalesce_merge_sources(&mut c.sources, &srcs);
            continue;
        }

        let edu_school = if kind == "education" {
            coalesce_edu_school_key(&f.text)
        } else {
            None
        };
        let edu_deg_only = kind == "education"
            && edu_school.is_none()
            && !coalesce_edu_degrees(&f.text).is_empty();

        let mut kind_out = f.kind.clone();
        if key.contains("profession:attorney") {
            kind_out = "legal".into();
        }

        let c = Cluster {
            kind: kind_out,
            text: f.text.clone(),
            sources: srcs,
            primary_source: primary,
            order,
            edu_school,
            edu_deg_only,
        };
        index.insert(key, clusters.len());
        clusters.push(c);
    }

    // Second pass: absorb degree-only education into matching school rows that share a degree class
    // or when only one school exists for that degree type.
    let deg_idxs: Vec<usize> = clusters
        .iter()
        .enumerate()
        .filter(|(_, c)| c.kind == "education" && c.edu_deg_only)
        .map(|(i, _)| i)
        .collect();
    let mut drop: BTreeSet<usize> = BTreeSet::new();
    for di in deg_idxs {
        let deg_text = clusters[di].text.clone();
        let deg_srcs = clusters[di].sources.clone();
        let degs = coalesce_edu_degrees(&deg_text);
        // Find school clusters that don't already list this degree, prefer one missing it
        let mut best: Option<usize> = None;
        for (si, sc) in clusters.iter().enumerate() {
            if sc.kind != "education" || sc.edu_deg_only || sc.edu_school.is_none() {
                continue;
            }
            let school_degs = coalesce_edu_degrees(&sc.text);
            // Absorb into school that lacks this degree, or any school if JD and school looks like law
            let school_l = sc.text.to_ascii_lowercase();
            let lawish = school_l.contains("law") || school_l.contains("stetson");
            let match_jd =
                degs.contains(&"jd") && (lawish || !school_degs.iter().any(|d| *d == "jd"));
            let match_other =
                degs.iter().any(|d| *d != "jd") && degs.iter().any(|d| !school_degs.contains(d));
            if match_jd || match_other {
                // Prefer school missing the degree entirely
                let score = (
                    if degs.iter().all(|d| school_degs.contains(d)) {
                        0
                    } else {
                        1
                    },
                    coalesce_text_richness(&sc.text).0,
                );
                match best {
                    None => best = Some(si),
                    Some(bi) => {
                        let bscore = (
                            if degs
                                .iter()
                                .all(|d| coalesce_edu_degrees(&clusters[bi].text).contains(d))
                            {
                                0
                            } else {
                                1
                            },
                            coalesce_text_richness(&clusters[bi].text).0,
                        );
                        if score > bscore {
                            best = Some(si);
                        }
                    }
                }
            }
        }
        // If JD degree-only and a law school exists, attach even if already has jd text
        if best.is_none() && degs.contains(&"jd") {
            best = clusters.iter().enumerate().find_map(|(si, sc)| {
                if sc.kind == "education" && !sc.edu_deg_only {
                    let l = sc.text.to_ascii_lowercase();
                    if l.contains("law") || l.contains("stetson") {
                        Some(si)
                    } else {
                        None
                    }
                } else {
                    None
                }
            });
        }
        // Bachelor degree-only → Florida / primary undergrad school lacking bachelors tag
        if best.is_none()
            && degs
                .iter()
                .any(|d| matches!(*d, "bachelors" | "masters" | "phd" | "md"))
        {
            best = clusters.iter().enumerate().find_map(|(si, sc)| {
                if sc.kind == "education" && !sc.edu_deg_only {
                    let sd = coalesce_edu_degrees(&sc.text);
                    if degs.iter().any(|d| !sd.contains(d)) {
                        Some(si)
                    } else {
                        None
                    }
                } else {
                    None
                }
            });
        }
        if let Some(si) = best {
            let merged = coalesce_absorb_degree_into_school(&clusters[si].text, &deg_text);
            clusters[si].text = merged;
            let mut srcs = clusters[si].sources.clone();
            coalesce_merge_sources(&mut srcs, &deg_srcs);
            clusters[si].sources = srcs;
            drop.insert(di);
        }
    }

    // Third pass: bare school absorbed into richer school row with same school key
    let school_groups: std::collections::HashMap<String, Vec<usize>> = {
        let mut m: std::collections::HashMap<String, Vec<usize>> = std::collections::HashMap::new();
        for (i, c) in clusters.iter().enumerate() {
            if drop.contains(&i) {
                continue;
            }
            if c.kind == "education" {
                if let Some(ref sk) = c.edu_school {
                    m.entry(sk.clone()).or_default().push(i);
                }
            }
        }
        m
    };
    for (_sk, idxs) in school_groups {
        if idxs.len() < 2 {
            continue;
        }
        // Keep richest
        let mut best = idxs[0];
        for &i in &idxs[1..] {
            if coalesce_prefer_text(
                &clusters[i].text,
                &clusters[best].text,
                &clusters[i].primary_source,
                &clusters[best].primary_source,
            ) {
                best = i;
            }
        }
        for &i in &idxs {
            if i == best {
                continue;
            }
            let mut srcs = clusters[best].sources.clone();
            coalesce_merge_sources(&mut srcs, &clusters[i].sources);
            // If loser has extra degree info, absorb
            let merged =
                coalesce_absorb_degree_into_school(&clusters[best].text, &clusters[i].text);
            // Also reverse if loser was richer in degrees but we already picked by prefer
            let merged2 = coalesce_absorb_degree_into_school(&merged, &clusters[best].text);
            clusters[best].text =
                if coalesce_text_richness(&merged2) > coalesce_text_richness(&merged) {
                    merged2
                } else {
                    merged
                };
            clusters[best].sources = srcs;
            drop.insert(i);
        }
    }

    let mut out: Vec<(usize, BioFact)> = Vec::new();
    for (i, c) in clusters.into_iter().enumerate() {
        if drop.contains(&i) {
            continue;
        }
        let mut sources = c.sources;
        if sources.is_empty() {
            sources.push(BioFactSource {
                name: c.primary_source.clone(),
                url: None,
            });
        }
        let primary = sources.first().cloned().unwrap_or_default();
        out.push((
            c.order,
            BioFact {
                kind: c.kind,
                text: c.text,
                source: primary.name,
                source_url: primary.url,
                sources,
                ..Default::default()
            },
        ));
    }
    out.sort_by_key(|(o, _)| *o);
    out.into_iter().map(|(_, f)| f).collect()
}

/// Coalesce facts in place on a dossier (I1/I2). Safe to call repeatedly.
pub fn coalesce_dossier_inplace(d: &mut PersonDossier) {
    if d.facts.len() < 2 {
        // Still normalize single-fact sources list
        for f in &mut d.facts {
            if f.sources.is_empty() && !f.source.is_empty() {
                f.sources = vec![BioFactSource {
                    name: f.source.clone(),
                    url: f.source_url.clone(),
                }];
            }
        }
    } else {
        d.facts = coalesce_dossier_facts(&d.facts);
    }
    refresh_dossier_snapshot_fields(d);
}

fn sources_gp_only(sources: &[BioFactSource]) -> bool {
    !sources.is_empty()
        && sources
            .iter()
            .all(|s| s.name.to_ascii_lowercase().contains("grokipedia"))
}

fn word_num_to_u32(s: &str) -> Option<u32> {
    match s.trim().to_ascii_lowercase().as_str() {
        "one" | "a" | "an" => Some(1),
        "two" => Some(2),
        "three" => Some(3),
        "four" => Some(4),
        "five" => Some(5),
        "six" => Some(6),
        "seven" => Some(7),
        "eight" => Some(8),
        "nine" => Some(9),
        "ten" => Some(10),
        _ => s.trim().parse().ok(),
    }
}

fn is_children_noise_token(s: &str) -> bool {
    let t = s.trim().trim_matches(|c: char| c == '.' || c == ';');
    if t.is_empty() {
        return true;
    }
    if t.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    matches!(
        t.to_ascii_lowercase().as_str(),
        "one"
            | "two"
            | "three"
            | "four"
            | "five"
            | "six"
            | "seven"
            | "eight"
            | "nine"
            | "ten"
            | "a"
            | "an"
            | "children"
            | "child"
            | "kids"
            | "kid"
            | "son"
            | "sons"
            | "daughter"
            | "daughters"
            | "and"
    )
}

/// Split named children on commas and "and" / "&" (e.g. "Brandon and Connor", "A, B, and C").
fn parse_children_names(body: &str) -> Vec<String> {
    let body = body.trim();
    if body.is_empty() || body.chars().all(|c| c.is_ascii_digit()) {
        return Vec::new();
    }
    // Reject pure count phrases with no proper names.
    let low = body.to_ascii_lowercase();
    if regex::Regex::new(
        r"(?i)^(one|two|three|four|five|six|seven|eight|nine|ten|\d+)\s+(children|child|kids|sons|son|daughters|daughter)$",
    )
    .ok()
    .and_then(|re| re.find(low.as_str()))
    .is_some()
    {
        return Vec::new();
    }

    let mut names: Vec<String> = Vec::new();
    for comma_part in body.split(',') {
        let mut part = comma_part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some(rest) = part
            .strip_prefix("and ")
            .or_else(|| part.strip_prefix("And "))
            .or_else(|| part.strip_prefix("AND "))
            .or_else(|| part.strip_prefix("& "))
        {
            part = rest.trim();
        }
        if part.is_empty() {
            continue;
        }
        let low_part = part.to_ascii_lowercase();
        let sub_parts: Vec<&str> = if low_part.contains(" and ") {
            // Case-insensitive split on " and "
            let mut out = Vec::new();
            let mut rest = part;
            while let Some(idx) = rest.to_ascii_lowercase().find(" and ") {
                out.push(rest[..idx].trim());
                rest = rest[idx + 5..].trim();
            }
            if !rest.is_empty() {
                out.push(rest);
            }
            out
        } else if part.contains(" & ") {
            part.split(" & ").collect()
        } else {
            vec![part]
        };
        for s in sub_parts {
            let s = s.trim().trim_matches(|c: char| c == '.' || c == ';');
            if is_children_noise_token(s) {
                continue;
            }
            if !s.chars().any(|c| c.is_alphabetic()) {
                continue;
            }
            names.push(s.to_string());
        }
    }

    // Drop if nothing looks like a personal name (no uppercase letter anywhere).
    if !names.is_empty() && !names.iter().any(|n| n.chars().any(|c| c.is_uppercase())) {
        return Vec::new();
    }
    names
}

/// Education display rank: lower = higher / professional first (JD/MD/PhD → … → HS).
pub fn education_degree_rank(text: &str) -> u8 {
    let degs = coalesce_edu_degrees(text);
    if degs
        .iter()
        .any(|d| *d == "jd" || *d == "md" || *d == "phd" || *d == "llm")
    {
        return 0;
    }
    if degs.iter().any(|d| *d == "masters") {
        return 1;
    }
    if degs.iter().any(|d| *d == "bachelors") {
        return 2;
    }
    if degs.iter().any(|d| *d == "associates") {
        return 3;
    }
    let t = text.to_ascii_lowercase();
    if t.contains("law") || t.contains("stetson") || t.contains("medical") {
        return 0;
    }
    if degs.iter().any(|d| *d == "hs") || t.contains("high school") {
        return 5;
    }
    4
}

fn sort_education_facts_inplace(facts: &mut [BioFact]) {
    let edu_idx: Vec<usize> = facts
        .iter()
        .enumerate()
        .filter(|(_, f)| f.kind == "education")
        .map(|(i, _)| i)
        .collect();
    if edu_idx.len() < 2 {
        return;
    }
    let mut edu_facts: Vec<BioFact> = edu_idx.iter().map(|&i| facts[i].clone()).collect();
    edu_facts.sort_by(|a, b| {
        education_degree_rank(&a.text)
            .cmp(&education_degree_rank(&b.text))
            .then_with(|| b.text.len().cmp(&a.text.len()))
    });
    for (slot, fact) in edu_idx.into_iter().zip(edu_facts.into_iter()) {
        facts[slot] = fact;
    }
}

/// Family snapshot from family facts only. Never invents unmarried/childless.
/// Grokipedia alone is not enough (same rule as citizenship/orientation).
pub fn family_summary_from_facts(facts: &[BioFact]) -> FamilySummary {
    let mut spouse: Option<(String, Vec<BioFactSource>)> = None;
    let mut children: Option<(Option<u32>, Option<String>, Vec<BioFactSource>)> = None;

    for f in facts.iter().filter(|f| f.kind == "family") {
        let srcs = f.all_sources();
        if sources_gp_only(&srcs) {
            continue;
        }
        let key = coalesce_family_cluster_key(&f.text);
        let body = coalesce_strip_label(&f.text).trim().to_string();
        if key == "family:spouse" {
            let name = if body.is_empty()
                || body.eq_ignore_ascii_case("yes")
                || body.eq_ignore_ascii_case("married")
            {
                String::new()
            } else {
                body
            };
            match &mut spouse {
                None => spouse = Some((name, srcs)),
                Some((cur, cur_s)) => {
                    if name.len() > cur.len() {
                        *cur = name;
                    }
                    coalesce_merge_sources(cur_s, &srcs);
                }
            }
            continue;
        }
        if key == "family:children" {
            let low = f.text.to_ascii_lowercase();
            let mut count: Option<u32> = None;
            let mut detail: Option<String> = None;
            let mut named_count = false;
            // "4 children" / "Children: 4" / "two children"
            if let Some(re) = regex::Regex::new(
                r"(?i)\b(one|two|three|four|five|six|seven|eight|nine|ten|\d+)\s+children\b",
            )
            .ok()
            {
                if let Some(c) = re.captures(&low) {
                    count = word_num_to_u32(c.get(1).map(|m| m.as_str()).unwrap_or(""));
                }
            }
            if count.is_none() {
                if let Some(re) = regex::Regex::new(r"(?i)^children:\s*(\d+)\s*$").ok() {
                    if let Some(c) = re.captures(&low) {
                        count = c.get(1).and_then(|m| m.as_str().parse().ok());
                    }
                }
            }
            // Named list: "Theo, Henry" / "Brandon and Connor" / "A, B, and C"
            let names = parse_children_names(&body);
            if !names.is_empty() {
                // Prefer name-list length over a conflicting bare "1 child" count.
                count = Some(names.len() as u32);
                named_count = true;
                detail = Some(names.join(", "));
            }
            if count.is_none() && detail.is_none() {
                // "Children: 4" already handled; bare "has children" → disclosed count unknown — skip count
                if low.contains("child") {
                    // keep as disclosed-without-count only if we have something citable
                    if body.is_empty() {
                        continue;
                    }
                    detail = Some(body.clone());
                } else {
                    continue;
                }
            }
            match &mut children {
                None => children = Some((count, detail, srcs)),
                Some((c, d, cur_s)) => {
                    if named_count {
                        // Named kids win over bare numeric conflicts ("1 child" vs two names).
                        *c = count;
                    } else if count.is_some() && (*c).is_none() {
                        *c = count;
                    } else if let (Some(a), Some(b)) = (*c, count) {
                        // Prefer larger bare counts only when no names yet; if names exist keep name count.
                        if d.is_none() {
                            *c = Some(a.max(b));
                        }
                    }
                    if let Some(nd) = detail {
                        let prefer = d
                            .as_ref()
                            .map(|x| {
                                // Prefer longer named lists; bare "4" loses to "Brandon, Connor".
                                let x_names = parse_children_names(x).len();
                                let n_names = parse_children_names(&nd).len();
                                n_names > x_names || (n_names == x_names && nd.len() > x.len())
                            })
                            .unwrap_or(true);
                        if prefer {
                            *d = Some(nd);
                        }
                    }
                    coalesce_merge_sources(cur_s, &srcs);
                }
            }
        }
    }

    if spouse.is_none() && children.is_none() {
        return FamilySummary::default();
    }

    let mut sources = Vec::new();
    let mut parts = Vec::new();
    let spouse_out = if let Some((name, s)) = spouse {
        coalesce_merge_sources(&mut sources, &s);
        if name.is_empty() {
            parts.push("Married".into());
            None
        } else {
            parts.push(format!("Married to {name}"));
            Some(name)
        }
    } else {
        None
    };
    let (children_count, children_detail) = if let Some((c, d, s)) = children {
        coalesce_merge_sources(&mut sources, &s);
        // If detail is a named list, align count to parsed names.
        let named = d
            .as_ref()
            .map(|s| parse_children_names(s))
            .unwrap_or_default();
        let (c, d) = if !named.is_empty() {
            let joined = named.join(", ");
            (Some(named.len() as u32), Some(joined))
        } else {
            (c, d)
        };
        match (c, d) {
            (Some(n), Some(names)) if n > 0 => {
                let word = if n == 1 { "child" } else { "children" };
                // Show names when known so glance count ≡ detail.
                parts.push(format!("{n} {word} ({names})"));
                (Some(n), Some(names))
            }
            (Some(n), d) if n > 0 => {
                parts.push(format!("{n} {}", if n == 1 { "child" } else { "children" }));
                (Some(n), d)
            }
            (_, Some(names)) => {
                parts.push(format!("Children: {names}"));
                (None, Some(names))
            }
            (c, d) => (c, d),
        }
    } else {
        (None, None)
    };

    if parts.is_empty() {
        return FamilySummary::default();
    }

    FamilySummary {
        disclosed: true,
        display: parts.join(" · "),
        note: String::new(),
        spouse: spouse_out,
        children_count,
        children_detail,
        sources,
    }
}

/// Orientation only when a public source **explicitly** states it. No spouse-gender inference.
/// Grokipedia alone is never enough.
pub fn orientation_from_facts(facts: &[BioFact]) -> OrientationRecord {
    // Explicit phrases only — rare in filings.
    let patterns: &[(&str, &str)] = &[
        (r"(?i)\bopenly\s+gay\b", "Openly gay"),
        (r"(?i)\bopenly\s+lesbian\b", "Openly lesbian"),
        (r"(?i)\bopenly\s+bisexual\b", "Openly bisexual"),
        (r"(?i)\bopenly\s+transgender\b", "Openly transgender"),
        (r"(?i)\bopenly\s+queer\b", "Openly queer"),
        (r"(?i)\bis\s+a\s+lesbian\b", "Lesbian"),
        (r"(?i)\bis\s+gay\b", "Gay"),
        (r"(?i)\bis\s+bisexual\b", "Bisexual"),
        (r"(?i)\bis\s+transgender\b", "Transgender"),
        (
            r"(?i)\bidentifies\s+as\s+(gay|lesbian|bisexual|queer|transgender|pansexual|asexual)\b",
            "",
        ),
        (r"(?i)^orientation:\s*(.+)$", ""),
        (r"(?i)^sexual orientation:\s*(.+)$", ""),
    ];

    for f in facts {
        let srcs = f.all_sources();
        if sources_gp_only(&srcs) {
            continue;
        }
        // Never take orientation from spouse/children family lines alone
        if f.kind == "family" {
            continue;
        }
        let text = f.text.trim();
        if text.is_empty() {
            continue;
        }
        for (pat, label) in patterns {
            let Ok(re) = regex::Regex::new(pat) else {
                continue;
            };
            if let Some(caps) = re.captures(text) {
                let lab = if !label.is_empty() {
                    (*label).to_string()
                } else if let Some(m) = caps.get(1) {
                    let raw = m.as_str().trim();
                    if raw.is_empty() {
                        continue;
                    }
                    // Title-case first letter
                    let mut c = raw.chars();
                    match c.next() {
                        None => continue,
                        Some(ch) => format!("{}{}", ch.to_ascii_uppercase(), c.as_str()),
                    }
                } else {
                    continue;
                };
                // Reject obvious non-orientation values
                let low = lab.to_ascii_lowercase();
                if low.contains("straight") || low.contains("heterosexual") {
                    // Explicit straight is rare but citable if stated
                } else if !(low.contains("gay")
                    || low.contains("lesbian")
                    || low.contains("bisexual")
                    || low.contains("trans")
                    || low.contains("queer")
                    || low.contains("lgbt")
                    || low.contains("pansexual")
                    || low.contains("asexual")
                    || low.contains("homosexual"))
                {
                    continue;
                }
                return OrientationRecord {
                    disclosed: true,
                    label: Some(lab),
                    note: String::new(),
                    sources: srcs,
                };
            }
        }
    }
    OrientationRecord::default()
}

/// Refresh family summary, orientation, education sort, empty_notes (I3).
pub fn refresh_dossier_snapshot_fields(d: &mut PersonDossier) {
    sort_education_facts_inplace(&mut d.facts);
    d.family_summary = family_summary_from_facts(&d.facts);
    d.orientation = orientation_from_facts(&d.facts);
    if d.family_summary.disclosed {
        d.empty_notes
            .retain(|n| !n.to_ascii_lowercase().starts_with("family:"));
    }
}

/// Record a bio host we actually consulted (for empty-state copy).
pub fn note_source_checked(d: &mut PersonDossier, label: &str) {
    let lab = label.trim();
    if lab.is_empty() {
        return;
    }
    let exists = d
        .sources_checked
        .iter()
        .any(|s| s.eq_ignore_ascii_case(lab));
    if !exists {
        d.sources_checked.push(lab.to_string());
    }
}

/// Empty-field copy: “Checked Ballotpedia / official site / Wikidata — not found.”
pub fn empty_not_found_copy(sources_checked: &[String]) -> String {
    let mut seen = BTreeSet::new();
    let mut parts: Vec<&str> = Vec::new();
    for s in sources_checked {
        let t = s.trim();
        if t.is_empty() {
            continue;
        }
        let key = t.to_ascii_lowercase();
        if seen.insert(key) {
            parts.push(t);
        }
    }
    if parts.is_empty() {
        return "Not disclosed in sources we check.".into();
    }
    format!("Checked {} — not found.", parts.join(" / "))
}

/// Polish generic empty_notes once sources_checked is known (I4).
pub fn polish_dossier_empty_notes(d: &mut PersonDossier) {
    let checked = empty_not_found_copy(&d.sources_checked);
    let has_edu = d.facts.iter().any(|f| f.kind == "education");
    let has_work = d
        .facts
        .iter()
        .any(|f| matches!(f.kind.as_str(), "work" | "business" | "legal"));
    let has_family = d.family_summary.disclosed || d.facts.iter().any(|f| f.kind == "family");
    let has_photo = d.photo_url.as_ref().is_some_and(|u| !u.is_empty());

    // Drop stale bucket notes we are about to rewrite or that are filled.
    d.empty_notes.retain(|n| {
        let l = n.to_ascii_lowercase();
        if l.starts_with("photo:") || l.starts_with("family:") || l.starts_with("education / work")
        {
            return false;
        }
        true
    });

    if !has_photo {
        d.empty_notes.insert(0, format!("Photo: {checked}"));
    }
    if !has_family {
        d.empty_notes.push(format!("Family: {checked}"));
    }
    if !has_edu && !has_work {
        d.empty_notes.push(format!("Education / work: {checked}"));
    }
}

/// Pull dated education/work/legal/business facts into career spans for fractions.
/// Single graduation year → 1-year span; ranges preserved when present.
pub fn spans_from_bio_facts(facts: &[BioFact]) -> Vec<CareerSpan> {
    let mut out: Vec<CareerSpan> = Vec::new();
    for f in facts {
        let cat = match f.kind.as_str() {
            "education" => LifeCategory::Education,
            "work" => LifeCategory::Work,
            "business" => LifeCategory::Business,
            "legal" => LifeCategory::Legal,
            _ => continue,
        };
        let Some((start, end)) = year_span_from_fact_text(&f.text) else {
            continue;
        };
        let label = f.text.trim();
        if label.is_empty() {
            continue;
        }
        let span = CareerSpan::new(
            cat,
            label,
            Some(start),
            end,
            f.source.as_str(),
            f.source_url.clone(),
        );
        let dup = out.iter().any(|b| {
            b.category == span.category
                && b.label == span.label
                && b.start_year == span.start_year
                && b.end_year == span.end_year
        });
        if !dup {
            out.push(span);
        }
    }
    out
}

/// Years from a fact line: "…, 1986", "1981–1983", "in 1989", bare trailing year.
fn year_span_from_fact_text(text: &str) -> Option<(i32, Option<i32>)> {
    let t = text.trim();
    if t.is_empty() {
        return None;
    }
    let (start, end) = years_from_label(t);
    if let Some(s) = start {
        return Some((s, end.or(Some(s))));
    }
    // ", 1986" / "(1986)" / " in 1986" — collect all 19xx/20xx
    let re = regex::Regex::new(r"\b((?:19|20)\d{2})\b").ok()?;
    let mut years: Vec<i32> = re
        .captures_iter(t)
        .filter_map(|c| c.get(1).and_then(|m| m.as_str().parse().ok()))
        .filter(|y| (1900..=2100).contains(y))
        .collect();
    years.sort_unstable();
    years.dedup();
    match years.as_slice() {
        [] => None,
        [y] => Some((*y, Some(*y))),
        // Short range (undergrad / law school window)
        [a, b] if *b >= *a && (*b - *a) <= 10 => Some((*a, Some(*b))),
        // Multiple discrete graduation years on one line — use last (most recent credential)
        ys => {
            let y = *ys.last()?;
            Some((y, Some(y)))
        }
    }
}

fn birth_year_from_facts(facts: &[BioFact]) -> Option<i32> {
    for f in facts {
        let low = f.text.to_ascii_lowercase();
        if f.kind == "other" && (low.starts_with("born") || low.starts_with("birth")) {
            if let Some(y) = year_from_born_text(&f.text) {
                return Some(y);
            }
        }
    }
    None
}

fn reassess_dossier_career(
    d: &mut PersonDossier,
    bio_spans: &[CareerSpan],
    birth: Option<i32>,
    as_of_year: i32,
) {
    coalesce_dossier_inplace(d);
    let fact_spans = spans_from_bio_facts(&d.facts);
    let spans = merge_career_spans(&merge_career_spans(&d.career.spans, bio_spans), &fact_spans);
    let birth = birth
        .or(d.career.birth_year)
        .or_else(|| birth_year_from_facts(&d.facts));
    d.career = assess_career(&spans, birth, as_of_year);
    if d.career
        .fractions
        .iter()
        .any(|f| f.category != "political" && f.years > 0.0)
    {
        d.empty_notes.retain(|n| {
            !n.to_ascii_lowercase()
                .starts_with("education / work / business / legal")
        });
    }
}

/// Apply a MemberBioParse onto an existing dossier (photo, facts, re-assess career).
pub fn apply_member_bio_to_dossier(d: &mut PersonDossier, bio: &MemberBioParse, as_of_year: i32) {
    if let Some(ref url) = bio.photo_url {
        if !url.is_empty() && d.photo_url.is_none() {
            d.photo_url = Some(url.clone());
            d.photo_source = bio.photo_source.clone();
            d.photo_source_url = bio.photo_source_url.clone();
            d.empty_notes
                .retain(|n| !n.to_ascii_lowercase().starts_with("photo:"));
        }
    }
    merge_facts_into_dossier(d, &bio.facts);
    coalesce_dossier_inplace(d);
    if let Some(ref c) = bio.citizenship {
        if c.disclosed && !d.citizenship.disclosed {
            d.citizenship = c.clone();
        }
    }
    apply_citizenship_from_facts(d);
    reassess_dossier_career(d, &bio.spans, bio.birth_year, as_of_year);
}

/// Set photo on dossier if empty.
pub fn apply_photo_to_dossier(
    d: &mut PersonDossier,
    photo_url: Option<String>,
    photo_source: Option<String>,
    photo_source_url: Option<String>,
) {
    let Some(url) = photo_url.filter(|u| !u.trim().is_empty()) else {
        return;
    };
    if d.photo_url.is_some() {
        return;
    }
    d.photo_url = Some(url);
    d.photo_source = photo_source;
    d.photo_source_url = photo_source_url;
    d.empty_notes
        .retain(|n| !n.to_ascii_lowercase().starts_with("photo:"));
}

// --- Wikidata (G1): structured public claims → dossier facts/spans (cite or omit) ---

const WD_SOURCE: &str = "Wikidata";
const WD_ENTITY_PREFIX: &str = "https://www.wikidata.org/wiki/";

/// Canonical entity page URL for a Q-id.
pub fn wikidata_entity_url(qid: &str) -> Option<String> {
    let id = qid.trim();
    if !looks_like_wikidata_id(id) {
        return None;
    }
    Some(format!("{WD_ENTITY_PREFIX}{id}"))
}

fn looks_like_wikidata_id(id: &str) -> bool {
    let id = id.trim();
    if id.len() < 2 || !id.as_bytes()[0].eq_ignore_ascii_case(&b'Q') {
        return false;
    }
    id[1..].chars().all(|c| c.is_ascii_digit())
}

/// Q-ids referenced by claims we parse — JS fetches labels in a second request.
pub fn wikidata_label_ids_needed(entity_json: &str) -> Vec<String> {
    let Ok(root) = serde_json::from_str::<Value>(entity_json) else {
        return Vec::new();
    };
    let Some(entity) = wikidata_pick_entity(&root) else {
        return Vec::new();
    };
    let Some(claims) = entity.get("claims").and_then(|c| c.as_object()) else {
        return Vec::new();
    };
    let mut ids = BTreeSet::new();
    for prop in ["P26", "P40", "P69", "P106", "P39", "P27", "P108"] {
        let Some(arr) = claims.get(prop).and_then(|v| v.as_array()) else {
            continue;
        };
        for c in arr {
            if let Some(id) = snak_entity_id(c.get("mainsnak")) {
                ids.insert(id);
            }
            if let Some(quals) = c.get("qualifiers").and_then(|q| q.as_object()) {
                for (_k, qarr) in quals {
                    if let Some(list) = qarr.as_array() {
                        for q in list {
                            if let Some(id) = snak_entity_id(Some(q)) {
                                ids.insert(id);
                            }
                        }
                    }
                }
            }
        }
    }
    ids.into_iter().collect()
}

fn wikidata_pick_entity(root: &Value) -> Option<&Value> {
    if root.get("claims").is_some() {
        return Some(root);
    }
    let ents = root.get("entities")?.as_object()?;
    ents.values().next()
}

fn snak_entity_id(snak: Option<&Value>) -> Option<String> {
    let v = snak?.get("datavalue")?.get("value")?;
    if let Some(id) = v.get("id").and_then(|x| x.as_str()) {
        if looks_like_wikidata_id(id) {
            return Some(id.to_string());
        }
    }
    None
}

fn snak_time_year(snak: Option<&Value>) -> Option<i32> {
    let t = snak?
        .get("datavalue")?
        .get("value")?
        .get("time")?
        .as_str()?;
    year_from_date(t)
}

fn label_for(labels: &std::collections::HashMap<String, String>, qid: &str) -> String {
    labels
        .get(qid)
        .cloned()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| qid.to_string())
}

/// Occupations that are political roles (not private work).
fn wikidata_occupation_is_political(qid: &str, label: &str) -> bool {
    matches!(
        qid,
        "Q82955" // politician
            | "Q15686806" // senator (generic)
            | "Q4416090" // US senator
            | "Q13218630" // US representative
            | "Q11774202" // legislative aide-ish
    ) || {
        let l = label.to_ascii_lowercase();
        l.contains("politician")
            || l.contains("senator")
            || l.contains("representative")
            || l.contains("legislator")
            || l.contains("congressman")
            || l.contains("congresswoman")
            || l.contains("mayor")
            || l.contains("governor")
    }
}

fn wikidata_occupation_category(qid: &str, label: &str) -> Option<(LifeCategory, &'static str)> {
    if wikidata_occupation_is_political(qid, label) {
        return None;
    }
    let l = label.to_ascii_lowercase();
    if matches!(qid, "Q40348" | "Q185351")
        || l.contains("lawyer")
        || l.contains("attorney")
        || l.contains("jurist")
        || l.contains("judge")
    {
        return Some((LifeCategory::Legal, "legal"));
    }
    if l.contains("business")
        || l.contains("entrepreneur")
        || l.contains("executive")
        || l.contains("ceo")
    {
        return Some((LifeCategory::Business, "business"));
    }
    Some((LifeCategory::Work, "work"))
}

/// Parse a Wikidata entity (+ label map) into bio facts/spans.
/// `entity_json`: single entity object or full `wbgetentities` response.
/// `labels_json`: `{"Q30":"United States",...}` for referenced items.
pub fn parse_wikidata_entity_bio(entity_json: &str, labels_json: &str) -> MemberBioParse {
    let mut out = MemberBioParse {
        photo_url: None,
        photo_source: None,
        photo_source_url: None,
        birth_year: None,
        facts: Vec::new(),
        spans: Vec::new(),
        citizenship: None,
    };
    let Ok(root) = serde_json::from_str::<Value>(entity_json) else {
        return out;
    };
    let Some(entity) = wikidata_pick_entity(&root) else {
        return out;
    };
    let qid = entity
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let src_url = if looks_like_wikidata_id(&qid) {
        Some(format!("{WD_ENTITY_PREFIX}{qid}"))
    } else {
        None
    };
    let labels: std::collections::HashMap<String, String> =
        serde_json::from_str(labels_json).unwrap_or_default();
    let Some(claims) = entity.get("claims").and_then(|c| c.as_object()) else {
        return out;
    };

    // P569 date of birth
    if let Some(arr) = claims.get("P569").and_then(|v| v.as_array()) {
        for c in arr {
            if let Some(y) = snak_time_year(c.get("mainsnak")) {
                out.birth_year = Some(y);
                break;
            }
        }
    }

    // P26 spouse
    if let Some(arr) = claims.get("P26").and_then(|v| v.as_array()) {
        for c in arr {
            if let Some(id) = snak_entity_id(c.get("mainsnak")) {
                let name = label_for(&labels, &id);
                if name.starts_with('Q') && name.len() < 12 {
                    continue; // unresolved label
                }
                out.facts.push(BioFact {
                    kind: "family".into(),
                    text: format!("Spouse: {name}"),
                    source: WD_SOURCE.into(),
                    source_url: src_url.clone(),
                    ..Default::default()
                });
            }
        }
    }

    // P40 children
    if let Some(arr) = claims.get("P40").and_then(|v| v.as_array()) {
        let mut kids = Vec::new();
        for c in arr {
            if let Some(id) = snak_entity_id(c.get("mainsnak")) {
                let name = label_for(&labels, &id);
                if !(name.starts_with('Q') && name.chars().skip(1).all(|ch| ch.is_ascii_digit())) {
                    kids.push(name);
                }
            }
        }
        if !kids.is_empty() {
            out.facts.push(BioFact {
                kind: "family".into(),
                text: format!("Children: {}", kids.join(", ")),
                source: WD_SOURCE.into(),
                source_url: src_url.clone(),
                ..Default::default()
            });
        }
    }

    // P69 educated at (+ optional P512 degree)
    if let Some(arr) = claims.get("P69").and_then(|v| v.as_array()) {
        for c in arr {
            let Some(id) = snak_entity_id(c.get("mainsnak")) else {
                continue;
            };
            let school = label_for(&labels, &id);
            if school.starts_with('Q') && school.chars().skip(1).all(|ch| ch.is_ascii_digit()) {
                continue;
            }
            let mut bits = vec![school.clone()];
            if let Some(quals) = c.get("qualifiers").and_then(|q| q.as_object()) {
                if let Some(deg_arr) = quals.get("P512").and_then(|v| v.as_array()) {
                    for d in deg_arr {
                        if let Some(did) = snak_entity_id(Some(d)) {
                            let dl = label_for(&labels, &did);
                            if !(dl.starts_with('Q')
                                && dl.chars().skip(1).all(|ch| ch.is_ascii_digit()))
                            {
                                bits.push(dl);
                            }
                        }
                    }
                }
            }
            let start = c
                .get("qualifiers")
                .and_then(|q| q.get("P580"))
                .and_then(|a| a.as_array())
                .and_then(|a| a.first())
                .and_then(|s| snak_time_year(Some(s)));
            let end = c
                .get("qualifiers")
                .and_then(|q| q.get("P582"))
                .and_then(|a| a.as_array())
                .and_then(|a| a.first())
                .and_then(|s| snak_time_year(Some(s)));
            out.facts.push(BioFact {
                kind: "education".into(),
                text: format!("Education: {}", bits.join(", ")),
                source: WD_SOURCE.into(),
                source_url: src_url.clone(),
                ..Default::default()
            });
            out.spans.push(CareerSpan::new(
                LifeCategory::Education,
                format!("Educated at {}", bits.join(", ")),
                start,
                end,
                WD_SOURCE,
                src_url.clone(),
            ));
        }
    }

    // P106 occupation
    if let Some(arr) = claims.get("P106").and_then(|v| v.as_array()) {
        for c in arr {
            let Some(id) = snak_entity_id(c.get("mainsnak")) else {
                continue;
            };
            let lab = label_for(&labels, &id);
            if lab.starts_with('Q') && lab.chars().skip(1).all(|ch| ch.is_ascii_digit()) {
                continue;
            }
            let Some((cat, kind)) = wikidata_occupation_category(&id, &lab) else {
                continue;
            };
            out.facts.push(BioFact {
                kind: kind.into(),
                text: format!("Occupation: {lab}"),
                source: WD_SOURCE.into(),
                source_url: src_url.clone(),
                ..Default::default()
            });
            out.spans.push(CareerSpan::new(
                cat,
                lab.clone(),
                None,
                None,
                WD_SOURCE,
                src_url.clone(),
            ));
        }
    }

    // P108 employer
    if let Some(arr) = claims.get("P108").and_then(|v| v.as_array()) {
        for c in arr {
            let Some(id) = snak_entity_id(c.get("mainsnak")) else {
                continue;
            };
            let lab = label_for(&labels, &id);
            if lab.starts_with('Q') && lab.chars().skip(1).all(|ch| ch.is_ascii_digit()) {
                continue;
            }
            let start = c
                .get("qualifiers")
                .and_then(|q| q.get("P580"))
                .and_then(|a| a.as_array())
                .and_then(|a| a.first())
                .and_then(|s| snak_time_year(Some(s)));
            let end = c
                .get("qualifiers")
                .and_then(|q| q.get("P582"))
                .and_then(|a| a.as_array())
                .and_then(|a| a.first())
                .and_then(|s| snak_time_year(Some(s)));
            out.facts.push(BioFact {
                kind: "work".into(),
                text: format!("Employer: {lab}"),
                source: WD_SOURCE.into(),
                source_url: src_url.clone(),
                ..Default::default()
            });
            out.spans.push(CareerSpan::new(
                LifeCategory::Work,
                format!("Employed at {lab}"),
                start,
                end,
                WD_SOURCE,
                src_url.clone(),
            ));
        }
    }

    // P39 position held → political spans (prefer dated; cap noise vs CL terms).
    if let Some(arr) = claims.get("P39").and_then(|v| v.as_array()) {
        let mut dated = Vec::new();
        let mut undated = Vec::new();
        for c in arr {
            let Some(id) = snak_entity_id(c.get("mainsnak")) else {
                continue;
            };
            let lab = label_for(&labels, &id);
            if lab.starts_with('Q') && lab.chars().skip(1).all(|ch| ch.is_ascii_digit()) {
                continue;
            }
            let start = c
                .get("qualifiers")
                .and_then(|q| q.get("P580"))
                .and_then(|a| a.as_array())
                .and_then(|a| a.first())
                .and_then(|s| snak_time_year(Some(s)));
            let end = c
                .get("qualifiers")
                .and_then(|q| q.get("P582"))
                .and_then(|a| a.as_array())
                .and_then(|a| a.first())
                .and_then(|s| snak_time_year(Some(s)));
            let span = CareerSpan::new(
                LifeCategory::Political,
                lab,
                start,
                end,
                WD_SOURCE,
                src_url.clone(),
            );
            if start.is_some() {
                dated.push(span);
            } else {
                undated.push(span);
            }
        }
        // Newest first when dated.
        dated.sort_by(|a, b| b.start_year.cmp(&a.start_year));
        const MAX_WD_POLITICAL: usize = 10;
        for s in dated.into_iter().take(MAX_WD_POLITICAL) {
            out.spans.push(s);
        }
        let room = MAX_WD_POLITICAL.saturating_sub(
            out.spans
                .iter()
                .filter(|s| s.category == "political" && s.source == WD_SOURCE)
                .count(),
        );
        for s in undated.into_iter().take(room) {
            out.spans.push(s);
        }
    }

    // P27 country of citizenship — only multi or explicit non-single-US disclosure
    if let Some(arr) = claims.get("P27").and_then(|v| v.as_array()) {
        let mut countries = Vec::new();
        for c in arr {
            if let Some(id) = snak_entity_id(c.get("mainsnak")) {
                let lab = label_for(&labels, &id);
                if lab.starts_with('Q') && lab.chars().skip(1).all(|ch| ch.is_ascii_digit()) {
                    continue;
                }
                if !countries
                    .iter()
                    .any(|x: &String| x.eq_ignore_ascii_case(&lab))
                {
                    countries.push(lab);
                }
            }
        }
        let us_only = countries.len() == 1
            && (countries[0].eq_ignore_ascii_case("United States")
                || countries[0].eq_ignore_ascii_case("United States of America")
                || countries[0].eq_ignore_ascii_case("USA")
                || countries[0].eq_ignore_ascii_case("U.S.")
                || countries[0].eq_ignore_ascii_case("U.S.A."));
        if countries.len() >= 2 {
            let note = format!("Citizenship (Wikidata): {}", countries.join(", "));
            out.citizenship = Some(CitizenshipRecord {
                countries: countries.clone(),
                disclosed: true,
                note: note.clone(),
                source: Some(WD_SOURCE.into()),
                source_url: src_url.clone(),
            });
            out.facts.push(BioFact {
                kind: "other".into(),
                text: note,
                source: WD_SOURCE.into(),
                source_url: src_url.clone(),
                ..Default::default()
            });
        } else if countries.len() == 1 && !us_only {
            let note = format!(
                "Citizenship claim (Wikidata): {}. Not treated as dual unless additional countries are listed.",
                countries[0]
            );
            out.citizenship = Some(CitizenshipRecord {
                countries: countries.clone(),
                disclosed: true,
                note: note.clone(),
                source: Some(WD_SOURCE.into()),
                source_url: src_url.clone(),
            });
            out.facts.push(BioFact {
                kind: "other".into(),
                text: note,
                source: WD_SOURCE.into(),
                source_url: src_url.clone(),
                ..Default::default()
            });
        }
        // Single US: leave default not-disclosed (do not invent exclusivity).
    }

    out
}

// --- Ballotpedia (H1): person infobox + Biography prose → MemberBioParse ---

const BP_SOURCE: &str = "Ballotpedia";

/// Canonical Ballotpedia person page URL (spaces → underscores).
pub fn ballotpedia_member_url(title: &str) -> Option<String> {
    crate::govtrack::ballotpedia_page_url(title)
}

fn bp_abs_url(src: &str) -> String {
    let p = src.trim();
    if p.starts_with("http://") || p.starts_with("https://") {
        p.to_string()
    } else if p.starts_with("//") {
        format!("https:{p}")
    } else if p.starts_with('/') {
        format!("https://ballotpedia.org{p}")
    } else {
        format!("https://ballotpedia.org/{p}")
    }
}

fn bp_infobox_slice(html: &str) -> &str {
    let lower = html.to_ascii_lowercase();
    let start = lower
        .find("infobox person")
        .or_else(|| lower.find("class=\"infobox\""))
        .unwrap_or(0);
    let from = start.saturating_sub(40);
    let rest = &html[from..];
    let rest_l = rest.to_ascii_lowercase();
    let end = rest_l
        .find("id=\"hono-css\"")
        .or_else(|| rest_l.find("id=\"toc\""))
        .or_else(|| rest_l.find("<h2"))
        .unwrap_or(rest.len().min(80_000));
    &rest[..end]
}

fn bp_fact_value_core(text: &str) -> &str {
    text.split_once(": ").map(|(_, v)| v).unwrap_or(text).trim()
}

fn bp_school_core(val: &str) -> String {
    val.split(',')
        .next()
        .unwrap_or(val)
        .trim()
        .to_ascii_lowercase()
}

fn bp_push_fact(
    out: &mut MemberBioParse,
    kind: &str,
    label: &str,
    value: &str,
    page: &Option<String>,
) {
    let v = value.trim();
    if v.is_empty() || v.len() > 500 {
        return;
    }
    let text = if label.is_empty() {
        v.to_string()
    } else {
        format!("{label}: {v}")
    };
    if out.facts.iter().any(|f| f.kind == kind && f.text == text) {
        return;
    }
    // Education: drop bare school when a year-bearing row for same school exists (or replace bare).
    if kind == "education" {
        let core = bp_school_core(v);
        let new_has_year = v.chars().any(|c| c.is_ascii_digit());
        if let Some(pos) = out.facts.iter().position(|f| {
            f.kind == "education" && bp_school_core(bp_fact_value_core(&f.text)) == core
        }) {
            let existing = &out.facts[pos].text;
            let ex_has_year = existing.chars().any(|c| c.is_ascii_digit());
            if ex_has_year && !new_has_year {
                return;
            }
            if new_has_year && !ex_has_year {
                out.facts.remove(pos);
            } else {
                return;
            }
        }
    }
    out.facts.push(BioFact {
        kind: kind.into(),
        text,
        source: BP_SOURCE.into(),
        source_url: page.clone(),
        ..Default::default()
    });
}

fn bp_key_kind(key: &str) -> Option<(&'static str, String)> {
    let k = key.trim().to_ascii_lowercase();
    let k = k.trim_end_matches(':').trim();
    match k {
        "high school" | "associates" | "associate's" | "associate" | "bachelor's" | "bachelors"
        | "bachelor" | "master's" | "masters" | "master" | "ph.d." | "phd" | "doctorate"
        | "graduate" | "law" | "j.d." | "jd" | "m.d." | "md" | "education" | "other education"
        | "military" => {
            let label = if k == "education" || k == "other education" {
                "Education".into()
            } else {
                // Preserve display casing lightly
                let mut c = key.trim().chars();
                match c.next() {
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    None => "Education".into(),
                }
            };
            Some(("education", label))
        }
        "profession" | "occupation" => Some(("work", "Profession".into())),
        "spouse" | "spouse(s)" => Some(("family", "Spouse".into())),
        "children" | "child" | "family" => Some(("family", "Family".into())),
        "religion" => Some(("other", "Religion".into())),
        "birthplace" | "place of birth" => Some(("other", "Birthplace".into())),
        "residence" | "home city" | "city of residence" => Some(("other", "Residence".into())),
        "birthday" | "date of birth" | "born" | "birth date" => Some(("other", "Born".into())),
        "net worth" | "base salary" | "last election" | "next election" | "campaign website"
        | "official website" | "official facebook" | "official x" | "official twitter"
        | "official instagram" | "official youtube" | "campaign facebook" | "campaign x"
        | "personal facebook" | "assumed office" | "term ends" | "years in position" => None,
        _ => None,
    }
}

/// Parse Ballotpedia person page HTML → photo + infobox facts + Biography blurb.
/// Cite or omit; never invent family/citizenship. Pure — no network.
pub fn parse_ballotpedia_member_html(html: &str, page_url: &str) -> MemberBioParse {
    let page = page_url.trim();
    let src_url = if page.is_empty() {
        None
    } else {
        Some(page.to_string())
    };
    let mut out = MemberBioParse {
        photo_source: Some(BP_SOURCE.into()),
        photo_source_url: src_url.clone(),
        ..Default::default()
    };
    if html.len() < 80 {
        return out;
    }

    let box_html = bp_infobox_slice(html);

    // Photo: first real headshot in infobox (skip placeholders / submit-photo).
    if let Ok(re) = regex::Regex::new(r#"(?is)<img\b([^>]+)>"#) {
        for cap in re.captures_iter(box_html) {
            let attrs = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let src = attr_value(attrs, "src").unwrap_or_default();
            if src.is_empty() {
                continue;
            }
            let low = src.to_ascii_lowercase();
            if low.contains("placeholder")
                || low.contains("submitphoto")
                || low.contains("submit-photo")
                || low.contains("logo")
                || low.contains("icon")
                || low.contains("sprite")
                || low.contains("seal")
            {
                continue;
            }
            if !(low.contains(".jpg")
                || low.contains(".jpeg")
                || low.contains(".png")
                || low.contains(".webp"))
            {
                continue;
            }
            let cls = attr_value(attrs, "class")
                .unwrap_or_default()
                .to_ascii_lowercase();
            let alt = attr_value(attrs, "alt")
                .unwrap_or_default()
                .to_ascii_lowercase();
            let hint = format!("{low} {cls} {alt}");
            if hint.contains("widget-img")
                || hint.contains("ballotpedia-api")
                || hint.contains("thumb")
                || hint.contains("portrait")
                || hint.contains("photo")
                || !alt.contains("silhouette")
            {
                out.photo_url = Some(bp_abs_url(&src));
                break;
            }
        }
    }

    // widget-key / widget-value rows (Education, Personal, etc.).
    if let Ok(re) = regex::Regex::new(
        r#"(?is)<div\s+class="widget-key"[^>]*>\s*([^<]{1,80}?)\s*</div>\s*<div\s+class="widget-value"[^>]*>([\s\S]{0,800}?)</div>"#,
    ) {
        for cap in re.captures_iter(box_html) {
            let key = strip_tags(cap.get(1).map(|m| m.as_str()).unwrap_or(""));
            let val = strip_tags(cap.get(2).map(|m| m.as_str()).unwrap_or(""));
            if key.is_empty() || val.is_empty() {
                continue;
            }
            let Some((kind, label)) = bp_key_kind(&key) else {
                continue;
            };
            if kind == "other" && (label == "Born" || key.to_ascii_lowercase().contains("birth")) {
                if let Some(y) = year_from_born_text(&val) {
                    out.birth_year = Some(y);
                }
            }
            bp_push_fact(&mut out, kind, &label, &val, &src_url);
        }
    }

    // Prior offices / tenure spans (political).
    // "Years in office: 2007 - 2013" after a bold office title.
    if let Ok(re) = regex::Regex::new(
        r#"(?is)(?:font-weight:\s*bold[^>]*>\s*([^<]{3,120}?)\s*</div>\s*<div[^>]*>\s*Years in office:\s*(\d{4})\s*[-–—]\s*(\d{4}|[Pp]resent|[Cc]urrent))"#,
    ) {
        for cap in re.captures_iter(box_html) {
            let label = strip_tags(cap.get(1).map(|m| m.as_str()).unwrap_or(""));
            let start = cap.get(2).and_then(|m| m.as_str().parse::<i32>().ok());
            let end_raw = cap.get(3).map(|m| m.as_str()).unwrap_or("");
            let end = if end_raw.eq_ignore_ascii_case("present")
                || end_raw.eq_ignore_ascii_case("current")
            {
                None
            } else {
                end_raw.parse::<i32>().ok()
            };
            if label.is_empty() || start.is_none() {
                continue;
            }
            out.spans.push(CareerSpan::new(
                LifeCategory::Political,
                label,
                start,
                end,
                BP_SOURCE,
                src_url.clone(),
            ));
        }
    }
    // Current office tenure block: office heading then Tenure / "2013 - Present"
    if let Ok(re) = regex::Regex::new(
        r#"(?is)widget-row value-only[^>]*>\s*([^<]{5,120}?)\s*</div>\s*<div[^>]*>\s*Tenure\s*</div>\s*<div[^>]*>\s*(\d{4})\s*[-–—]\s*(\d{4}|[Pp]resent|[Cc]urrent)"#,
    ) {
        for cap in re.captures_iter(box_html) {
            let label = strip_tags(cap.get(1).map(|m| m.as_str()).unwrap_or(""));
            let start = cap.get(2).and_then(|m| m.as_str().parse::<i32>().ok());
            let end_raw = cap.get(3).map(|m| m.as_str()).unwrap_or("");
            let end = if end_raw.eq_ignore_ascii_case("present")
                || end_raw.eq_ignore_ascii_case("current")
            {
                None
            } else {
                end_raw.parse::<i32>().ok()
            };
            let low = label.to_ascii_lowercase();
            if label.is_empty()
                || start.is_none()
                || low.contains("candidate")
                || low.contains("prior offices")
                || low.contains("compensation")
                || low.contains("education")
                || low.contains("personal")
                || low.contains("contact")
                || low.contains("elections")
            {
                continue;
            }
            let dup = out
                .spans
                .iter()
                .any(|s| s.label == label && s.start_year == start && s.end_year == end);
            if dup {
                continue;
            }
            out.spans.push(CareerSpan::new(
                LifeCategory::Political,
                label,
                start,
                end,
                BP_SOURCE,
                src_url.clone(),
            ));
        }
    }

    // Biography section first paragraph — short public blurb only.
    if let Some(idx) = html.find("id=\"Biography\"") {
        let after = &html[idx..];
        if let Some(h2_end) = after.find("</h2>") {
            let rest = after[h2_end + 5..].trim_start();
            if let Some(p_start) = rest.find("<p") {
                let from_p = &rest[p_start..];
                if let Some(p_end) = from_p.find("</p>") {
                    let raw = &from_p[..p_end + 4];
                    let text = strip_tags(raw);
                    if text.len() >= 40 && text.len() <= 900 {
                        let clipped = if text.len() > 500 {
                            format!("{}…", &text[..497])
                        } else {
                            text
                        };
                        bp_push_fact(&mut out, "other", "", &clipped, &src_url);
                    }
                }
            }
        }
    }

    out
}

// --- Ballotpedia challengers (H6): title candidates from name+state+office; campaign About ---

fn bp_state_full_name(code: &str) -> Option<&'static str> {
    match code.trim().to_ascii_uppercase().as_str() {
        "AL" => Some("Alabama"),
        "AK" => Some("Alaska"),
        "AZ" => Some("Arizona"),
        "AR" => Some("Arkansas"),
        "CA" => Some("California"),
        "CO" => Some("Colorado"),
        "CT" => Some("Connecticut"),
        "DE" => Some("Delaware"),
        "FL" => Some("Florida"),
        "GA" => Some("Georgia"),
        "HI" => Some("Hawaii"),
        "ID" => Some("Idaho"),
        "IL" => Some("Illinois"),
        "IN" => Some("Indiana"),
        "IA" => Some("Iowa"),
        "KS" => Some("Kansas"),
        "KY" => Some("Kentucky"),
        "LA" => Some("Louisiana"),
        "ME" => Some("Maine"),
        "MD" => Some("Maryland"),
        "MA" => Some("Massachusetts"),
        "MI" => Some("Michigan"),
        "MN" => Some("Minnesota"),
        "MS" => Some("Mississippi"),
        "MO" => Some("Missouri"),
        "MT" => Some("Montana"),
        "NE" => Some("Nebraska"),
        "NV" => Some("Nevada"),
        "NH" => Some("New Hampshire"),
        "NJ" => Some("New Jersey"),
        "NM" => Some("New Mexico"),
        "NY" => Some("New York"),
        "NC" => Some("North Carolina"),
        "ND" => Some("North Dakota"),
        "OH" => Some("Ohio"),
        "OK" => Some("Oklahoma"),
        "OR" => Some("Oregon"),
        "PA" => Some("Pennsylvania"),
        "RI" => Some("Rhode Island"),
        "SC" => Some("South Carolina"),
        "SD" => Some("South Dakota"),
        "TN" => Some("Tennessee"),
        "TX" => Some("Texas"),
        "UT" => Some("Utah"),
        "VT" => Some("Vermont"),
        "VA" => Some("Virginia"),
        "WA" => Some("Washington"),
        "WV" => Some("West Virginia"),
        "WI" => Some("Wisconsin"),
        "WY" => Some("Wyoming"),
        "DC" => Some("District of Columbia"),
        _ => None,
    }
}

fn bp_strip_nicknames(name: &str) -> String {
    let re = match regex::Regex::new(r#"["']([^"']+)["']"#) {
        Ok(r) => r,
        Err(_) => {
            return name.split_whitespace().collect::<Vec<_>>().join(" ");
        }
    };
    let s = re.replace_all(name, " ");
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn bp_name_base_variants(name: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut push = |s: String| {
        let t = s.split_whitespace().collect::<Vec<_>>().join(" ");
        if t.len() < 3 {
            return;
        }
        if !out.iter().any(|x: &String| x.eq_ignore_ascii_case(&t)) {
            out.push(t);
        }
    };
    let raw = name.trim();
    if raw.is_empty() {
        return out;
    }
    push(raw.to_string());
    let stripped = bp_strip_nicknames(raw);
    push(stripped.clone());
    // Nickname-only given: Ronrico "Rico" Smith → Rico Smith
    if let Ok(re) = regex::Regex::new(r#"["']([^"']+)["']"#) {
        if let Some(cap) = re.captures(raw) {
            let nick = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            let parts: Vec<&str> = stripped.split_whitespace().collect();
            if nick.len() >= 2 && parts.len() >= 2 {
                let last = parts[parts.len() - 1];
                push(format!("{nick} {last}"));
            }
        }
    }
    // Drop middle initials: Gus M. Bilirakis → Gus Bilirakis
    let parts: Vec<&str> = stripped.split_whitespace().collect();
    if parts.len() >= 3 {
        let first = parts[0];
        let last = parts[parts.len() - 1];
        let middles = &parts[1..parts.len() - 1];
        let all_initials = middles.iter().all(|m| {
            let t = m.trim_end_matches('.');
            t.len() == 1 && t.chars().all(|c| c.is_ascii_alphabetic())
        });
        if all_initials {
            push(format!("{first} {last}"));
        } else {
            // Keep first + last only as weaker fallback.
            push(format!("{first} {last}"));
        }
    }
    out
}

/// Office-aware Ballotpedia disambiguator tags (pure). Empty when no clear role.
fn bp_office_role_tags(chamber: &str, office: &str) -> Vec<&'static str> {
    let ch = chamber.trim().to_ascii_lowercase();
    let off = office.trim().to_ascii_lowercase();
    let mut tags = Vec::new();
    let push = |tags: &mut Vec<&'static str>, t: &'static str| {
        if !tags.iter().any(|x| *x == t) {
            tags.push(t);
        }
    };

    // Statewide / executive (office text only — never assume chamber alone)
    if off.contains("lieutenant governor")
        || off.contains("lt. governor")
        || off.contains("lt governor")
    {
        push(&mut tags, "Lieutenant Governor");
    } else if off.contains("governor") {
        push(&mut tags, "Governor");
    }
    if off.contains("attorney general") {
        push(&mut tags, "Attorney General");
    }
    if off.contains("chief financial") || off == "cfo" || off.contains("cfo") {
        push(&mut tags, "Chief Financial Officer");
    }
    if off.contains("agriculture") || off.contains("commissioner of agriculture") {
        push(&mut tags, "Agriculture Commissioner");
    }
    if off.contains("secretary of state") {
        push(&mut tags, "Secretary of State");
    }
    if off.contains("state attorney") {
        push(&mut tags, "State Attorney");
    }
    if off.contains("public defender") {
        push(&mut tags, "Public Defender");
    }

    // Legislature (title pages sometimes use chamber role)
    if ch == "state_senate" || off.contains("state senator") || off.contains("state senate") {
        push(&mut tags, "politician");
    }
    if ch == "state_house"
        || off.contains("state representative")
        || off.contains("state house")
        || off.contains("state assembly")
    {
        push(&mut tags, "politician");
    }

    // Judicial
    if ch == "judicial"
        || off.contains("judge")
        || off.contains("justice")
        || off.contains("court of appeal")
        || off.contains("supreme court")
    {
        if off.contains("supreme") {
            push(&mut tags, "judge");
        } else if off.contains("circuit") {
            push(&mut tags, "judge");
        } else if off.contains("county") {
            push(&mut tags, "judge");
        } else {
            push(&mut tags, "judge");
        }
    }

    // Local
    if off.contains("mayor") {
        push(&mut tags, "Mayor");
    }
    if off.contains("county commissioner") || off.contains("board of county") {
        push(&mut tags, "County Commissioner");
    }
    if off.contains("school board") {
        push(&mut tags, "School Board");
    }
    if off.contains("city council") || off.contains("council member") || off.contains("councilor") {
        push(&mut tags, "City Council");
    }
    if off.contains("sheriff") {
        push(&mut tags, "Sheriff");
    }
    if off.contains("clerk") && (ch == "county" || off.contains("clerk of")) {
        push(&mut tags, "Clerk");
    }
    if off.contains("property appraiser") {
        push(&mut tags, "Property Appraiser");
    }
    if off.contains("tax collector") {
        push(&mut tags, "Tax Collector");
    }
    if off.contains("supervisor of elections") {
        push(&mut tags, "Supervisor of Elections");
    }

    tags
}

/// Ordered Ballotpedia title guesses when CL `id.ballotpedia` is missing (challengers).
/// Pure — no network. Caller fetches + validates with `ballotpedia_html_matches_person`.
pub fn ballotpedia_title_candidates(
    name: &str,
    state_code: Option<&str>,
    chamber: Option<&str>,
    office: Option<&str>,
) -> Vec<String> {
    let mut titles = Vec::new();
    let mut push_title = |t: String| {
        let t = t.trim().to_string();
        if t.len() < 3 {
            return;
        }
        if !titles.iter().any(|x: &String| x.eq_ignore_ascii_case(&t)) {
            titles.push(t);
        }
    };

    let bases = bp_name_base_variants(name);
    for b in &bases {
        push_title(b.clone());
    }

    let st = state_code
        .map(|s| s.trim().to_ascii_uppercase())
        .filter(|s| s.len() == 2);
    let full = st.as_deref().and_then(bp_state_full_name);
    if let Some(full) = full {
        for b in &bases {
            push_title(format!("{b} ({full})"));
        }
    }

    let ch = chamber.unwrap_or("").to_ascii_lowercase();
    let off = office.unwrap_or("").to_ascii_lowercase();
    let federal = ch == "house"
        || ch == "senate"
        || off.contains("u.s. house")
        || off.contains("u.s. senate")
        || off.contains("united states house")
        || off.contains("united states senate")
        || off.contains("us house")
        || off.contains("us senate");
    if federal {
        for b in &bases {
            push_title(format!("{b} (politician)"));
        }
    } else {
        // J2: statewide / judicial / local office disambiguators.
        let roles = bp_office_role_tags(chamber.unwrap_or(""), office.unwrap_or(""));
        for role in &roles {
            for b in &bases {
                push_title(format!("{b} ({role})"));
                if let Some(full) = full {
                    // "Name (Florida Governor)" / "Name (Governor of Florida)"
                    if *role == "Governor" {
                        push_title(format!("{b} (Governor of {full})"));
                    } else if *role == "Lieutenant Governor" {
                        push_title(format!("{b} (Lieutenant Governor of {full})"));
                    } else if *role == "Attorney General" {
                        push_title(format!("{b} (Attorney General of {full})"));
                    } else {
                        push_title(format!("{b} ({full} {role})"));
                    }
                }
            }
        }
        // Weak generic when no role tag matched.
        if roles.is_empty() {
            for b in &bases {
                push_title(format!("{b} (politician)"));
            }
        }
    }
    // Cap attempts — browser should not hammer BP.
    titles.truncate(8);
    titles
}

fn bp_page_title_from_html(html: &str) -> Option<String> {
    if let Ok(re) =
        regex::Regex::new(r#"(?is)<meta\s+property=["']og:title["']\s+content=["']([^"']+)["']"#)
    {
        if let Some(c) = re.captures(html) {
            let t = html_unescape_basic(c.get(1).map(|m| m.as_str()).unwrap_or(""))
                .trim()
                .to_string();
            if !t.is_empty() {
                return Some(t);
            }
        }
    }
    if let Ok(re) = regex::Regex::new(r#"(?is)class=["']mw-page-title-main["'][^>]*>([^<]+)<"#) {
        if let Some(c) = re.captures(html) {
            let t = strip_tags(c.get(1).map(|m| m.as_str()).unwrap_or(""))
                .trim()
                .to_string();
            if !t.is_empty() {
                return Some(t);
            }
        }
    }
    if let Ok(re) = regex::Regex::new(r"(?is)<title[^>]*>([^<]+)</title>") {
        if let Some(c) = re.captures(html) {
            let mut t = html_unescape_basic(c.get(1).map(|m| m.as_str()).unwrap_or(""));
            if let Some(idx) = t.find(" - Ballotpedia") {
                t = t[..idx].to_string();
            } else if let Some(idx) = t.find(" — Ballotpedia") {
                t = t[..idx].to_string();
            }
            let t = t.trim().to_string();
            if !t.is_empty() {
                return Some(t);
            }
        }
    }
    None
}

/// True when HTML is a person page for this name (and optional state). Ambiguous → false.
pub fn ballotpedia_html_matches_person(
    html: &str,
    person_name: &str,
    state_code: Option<&str>,
) -> bool {
    if html.len() < 200 {
        return false;
    }
    let low = html.to_ascii_lowercase();
    if low.contains("may refer to")
        || low.contains("disambiguation")
        || low.contains("there were no results matching")
        || low.contains("search results") && low.contains("there is currently no text")
    {
        // Disambiguation / search — skip (ambiguous).
        if low.contains("may refer to") || low.contains("disambiguation pages") {
            return false;
        }
    }
    if !low.contains("infobox person") {
        return false;
    }
    let Some(title) = bp_page_title_from_html(html) else {
        return false;
    };
    // Title must be a high-precision name match (reuse Grokipedia token rules).
    if !gp_title_matches_person(person_name, &title)
        && !gp_title_matches_person(person_name, &bp_strip_nicknames(&title))
    {
        // Also try each name variant against title.
        let ok = bp_name_base_variants(person_name)
            .iter()
            .any(|v| gp_title_matches_person(v, &title));
        if !ok {
            return false;
        }
    }
    if let Some(code) = state_code.map(|s| s.trim()).filter(|s| s.len() == 2) {
        let code_u = code.to_ascii_uppercase();
        let full = bp_state_full_name(&code_u).unwrap_or("");
        let head = html
            .chars()
            .take(40_000)
            .collect::<String>()
            .to_ascii_lowercase();
        let code_l = code_u.to_ascii_lowercase();
        let full_l = full.to_ascii_lowercase();
        let has_state = (!full_l.is_empty() && head.contains(&full_l))
            || head.contains(&format!(" {code_l}"))
            || head.contains(&format!("({code_l})"))
            || head.contains(&format!(", {code_l}"))
            || head.contains(&format!("-{code_l}"))
            || head.contains(&format!("_{code_l}"));
        if !has_state {
            return false;
        }
    }
    true
}

/// Campaign website href from Ballotpedia person infobox Contact row (if present).
pub fn ballotpedia_campaign_website(html: &str) -> Option<String> {
    // <a href="https://…">Campaign website</a>
    let re = regex::Regex::new(
        r#"(?is)<a\s+[^>]*href=["']([^"']+)["'][^>]*>\s*Campaign\s+website\s*</a>"#,
    )
    .ok()?;
    for cap in re.captures_iter(html) {
        let href = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
        if href.starts_with("http://") || href.starts_with("https://") {
            if is_campaign_site_url(href) {
                return Some(href.to_string());
            }
        }
    }
    None
}

/// True when URL looks like a candidate campaign site (not gov / encyclopedia / social).
pub fn is_campaign_site_url(url: &str) -> bool {
    let u = url.trim();
    if !(u.starts_with("http://") || u.starts_with("https://")) {
        return false;
    }
    let low = u.to_ascii_lowercase();
    // Host-ish slice
    let host = low
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("")
        .split('@')
        .next()
        .unwrap_or("")
        .trim_start_matches("www.");
    if host.is_empty() || !host.contains('.') {
        return false;
    }
    const BLOCK: &[&str] = &[
        "ballotpedia.org",
        "wikipedia.org",
        "wikidata.org",
        "dbpedia.org",
        "grokipedia.com",
        "house.gov",
        "senate.gov",
        "congress.gov",
        "govtrack.us",
        "openstates.org",
        "followthemoney.org",
        "facebook.com",
        "fb.com",
        "twitter.com",
        "x.com",
        "instagram.com",
        "youtube.com",
        "youtu.be",
        "linkedin.com",
        "threads.net",
        "tiktok.com",
        "google.com",
        "goo.gl",
        "bit.ly",
        "actblue.com",
        "winred.com",
        "myflorida.com",
        "flsenate.gov",
        "flhouse.gov",
        "myfloridahouse.gov",
        "dos.elections.myflorida.com",
        "ncsbe.gov",
        "elections.maryland.gov",
        "azcleanelections.gov",
        "azsos.gov",
        "fec.gov",
    ];
    for b in BLOCK {
        if host == *b || host.ends_with(&format!(".{b}")) {
            return false;
        }
    }
    if host.ends_with(".gov") || host.contains(".gov.") || host.starts_with("state.") {
        return false;
    }
    if host.ends_with(".mil") {
        return false;
    }
    // Obvious campaign host tokens → yes
    if host.contains("forcongress")
        || host.contains("forsenate")
        || host.contains("forhouse")
        || host.contains("elect")
        || host.contains("campaign")
        || host.contains("vote")
    {
        return true;
    }
    // Personal-looking domains (candidate sites) — allow non-gov http(s).
    true
}

/// About/bio path candidates on a campaign site (same idea as official_about_urls).
pub fn campaign_about_urls(site_url: &str) -> Vec<String> {
    let base = normalize_campaign_site_url(site_url);
    if base.len() < 12 {
        return Vec::new();
    }
    let mut out = vec![
        format!("{base}/about"),
        format!("{base}/about/"),
        format!("{base}/about-me"),
        format!("{base}/about-me/"),
        format!("{base}/biography"),
        format!("{base}/bio"),
        format!("{base}/meet"),
        format!("{base}/who-is"),
        // Root last — often a long homepage bio.
        base.clone(),
    ];
    let mut seen = std::collections::HashSet::new();
    out.retain(|u| seen.insert(u.clone()));
    out
}

fn normalize_campaign_site_url(site_url: &str) -> String {
    let t = site_url.trim();
    if t.is_empty() {
        return String::new();
    }
    let mut u = t.to_string();
    if !(u.starts_with("http://") || u.starts_with("https://")) {
        u = format!("https://{u}");
    }
    // Strip fragment/query; trim trailing slash for base join.
    if let Some(i) = u.find('#') {
        u = u[..i].to_string();
    }
    if let Some(i) = u.find('?') {
        u = u[..i].to_string();
    }
    while u.ends_with('/') && u.len() > 10 {
        u.pop();
    }
    u
}

/// Parse campaign About/homepage HTML → MemberBioParse (cite campaign URL).
/// Lower trust than official — apply with fill_gaps only.
pub fn parse_campaign_about_html(html: &str, page_url: &str) -> MemberBioParse {
    let page = page_url.trim();
    let src = "Campaign website".to_string();
    let src_url = if page.is_empty() {
        None
    } else {
        Some(page.to_string())
    };
    let mut out = MemberBioParse {
        photo_source: Some(src.clone()),
        photo_source_url: src_url.clone(),
        ..Default::default()
    };
    if html.len() < 120 {
        return out;
    }
    // Reuse official About extractors (prose + photo), then re-label.
    let mut prose = parse_official_member_about_html(html, page);
    for f in &mut prose.facts {
        f.source = src.clone();
        f.source_url = src_url.clone();
    }
    out.photo_url = prose.photo_url;
    out.photo_source = Some(src);
    out.photo_source_url = src_url;
    out.birth_year = prose.birth_year;
    out.facts = prose.facts;
    // Career spans from campaign copy are often vague — keep empty unless official parser found dated ones.
    out.spans = prose.spans;
    // Never take citizenship from campaign alone.
    out.citizenship = None;
    out
}

/// Wikipedia REST summary URL for a page title (spaces → underscores). No fetch.
pub fn wikipedia_summary_api_url(title: &str) -> Option<String> {
    let t = title.trim();
    if t.is_empty() {
        return None;
    }
    let slug: String = t
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_")
        .chars()
        .map(|c| if c == ' ' { '_' } else { c })
        .collect();
    // Encode path-unsafe chars but keep common title punctuation.
    let enc = slug
        .chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => {
                let mut b = [0u8; 4];
                let s = c.encode_utf8(&mut b);
                s.bytes().map(|x| format!("%{x:02X}")).collect::<String>()
            }
        })
        .collect::<String>();
    if enc.is_empty() {
        return None;
    }
    Some(format!(
        "https://en.wikipedia.org/api/rest_v1/page/summary/{enc}"
    ))
}

/// Human Wikipedia article URL for a title.
pub fn wikipedia_article_url(title: &str) -> Option<String> {
    let t = title.trim();
    if t.is_empty() {
        return None;
    }
    let slug = t.split_whitespace().collect::<Vec<_>>().join("_");
    let enc = slug
        .chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => {
                let mut b = [0u8; 4];
                let s = c.encode_utf8(&mut b);
                s.bytes().map(|x| format!("%{x:02X}")).collect::<String>()
            }
        })
        .collect::<String>();
    Some(format!("https://en.wikipedia.org/wiki/{enc}"))
}

// --- Official member About (H3): house.gov / senate.gov About HTML via Wisp ---

fn official_source_label(page_url: &str) -> String {
    let u = page_url.to_ascii_lowercase();
    if u.contains("senate.gov") {
        "U.S. Senate (official)".into()
    } else if u.contains("house.gov") {
        "U.S. House (official)".into()
    } else {
        "Official member site".into()
    }
}

fn strip_scripts_styles(html: &str) -> String {
    let mut s = html.to_string();
    if let Ok(re) = regex::Regex::new(r"(?is)<script\b[^>]*>.*?</script>") {
        s = re.replace_all(&s, " ").into_owned();
    }
    if let Ok(re) = regex::Regex::new(r"(?is)<style\b[^>]*>.*?</style>") {
        s = re.replace_all(&s, " ").into_owned();
    }
    if let Ok(re) = regex::Regex::new(r"(?is)<noscript\b[^>]*>.*?</noscript>") {
        s = re.replace_all(&s, " ").into_owned();
    }
    s
}

fn official_main_html_slice(html: &str) -> String {
    let clean = strip_scripts_styles(html);
    let lower = clean.to_ascii_lowercase();
    // Prefer <main>, then role=main, then article, then body.
    for (open_pat, close_pat) in [
        ("<main", "</main>"),
        ("role=\"main\"", "</div>"),
        ("<article", "</article>"),
    ] {
        if let Some(start) = lower.find(open_pat) {
            let from = &clean[start..];
            let from_l = from.to_ascii_lowercase();
            if let Some(end) = from_l.find(close_pat) {
                let slice = &from[..end + close_pat.len()];
                if slice.len() > 200 {
                    return slice.to_string();
                }
            }
        }
    }
    // Divi / common CMS text blocks.
    if let Ok(re) = regex::Regex::new(r#"(?is)<div class="et_pb_text_inner">(.*?)</div>"#) {
        let mut parts = Vec::new();
        for cap in re.captures_iter(&clean) {
            parts.push(cap.get(1).map(|m| m.as_str()).unwrap_or(""));
        }
        if !parts.is_empty() {
            return parts.join("\n");
        }
    }
    if let Some(start) = lower.find("<body") {
        return clean[start..].to_string();
    }
    clean
}

fn official_plain_text(html_slice: &str) -> String {
    // Prefer real paragraphs; also keep long CMS text blocks / bare prose.
    let mut chunks = Vec::new();
    if let Ok(re) = regex::Regex::new(r"(?is)<p\b[^>]*>(.*?)</p>") {
        for cap in re.captures_iter(html_slice) {
            let t = strip_tags(cap.get(1).map(|m| m.as_str()).unwrap_or(""));
            if t.len() >= 40 {
                chunks.push(t);
            }
        }
    }
    if chunks.is_empty() {
        if let Ok(re) = regex::Regex::new(r#"(?is)<div class="et_pb_text_inner">(.*?)</div>"#) {
            for cap in re.captures_iter(html_slice) {
                let t = strip_tags(cap.get(1).map(|m| m.as_str()).unwrap_or(""));
                if t.len() >= 40 {
                    chunks.push(t);
                }
            }
        }
    }
    if chunks.is_empty() {
        // Fall back to full strip (Drupal/evo often uses bare text nodes).
        let t = strip_tags(html_slice);
        return t.split_whitespace().collect::<Vec<_>>().join(" ");
    }
    chunks.join(" ")
}

fn official_og_image(html: &str) -> Option<String> {
    let re = regex::Regex::new(
        r#"(?is)<meta\b[^>]+property=["']og:image["'][^>]+content=["']([^"']+)["']"#,
    )
    .ok()?;
    if let Some(cap) = re.captures(html) {
        let u = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
        if u.starts_with("http") {
            return Some(u.split('?').next().unwrap_or(u).to_string());
        }
    }
    let re2 = regex::Regex::new(
        r#"(?is)<meta\b[^>]+content=["']([^"']+)["'][^>]+property=["']og:image["']"#,
    )
    .ok()?;
    if let Some(cap) = re2.captures(html) {
        let u = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
        if u.starts_with("http") {
            return Some(u.split('?').next().unwrap_or(u).to_string());
        }
    }
    None
}

/// Parse official House/Senate member About HTML → bio facts/photo.
/// Authoritative source; cite page URL. Pure — no network.
pub fn parse_official_member_about_html(html: &str, page_url: &str) -> MemberBioParse {
    let page = page_url.trim();
    let src = official_source_label(page);
    let src_url = if page.is_empty() {
        None
    } else {
        Some(page.to_string())
    };
    let mut out = MemberBioParse {
        photo_source: Some(src.clone()),
        photo_source_url: src_url.clone(),
        ..Default::default()
    };
    if html.len() < 120 {
        return out;
    }

    if let Some(img) = official_og_image(html) {
        let low = img.to_ascii_lowercase();
        if !low.contains("logo") && !low.contains("icon") && !low.contains("seal") {
            out.photo_url = Some(img);
        }
    }

    let slice = official_main_html_slice(html);
    let text = official_plain_text(&slice);
    if text.len() < 40 {
        return out;
    }

    // Reuse prose extractors, then re-label source as official.
    let mut prose = parse_wikipedia_plain_extract(&text, page);
    for f in &mut prose.facts {
        f.source = src.clone();
        f.source_url = src_url.clone();
    }
    out.birth_year = prose.birth_year;
    out.facts = prose.facts;
    out.spans = prose.spans;
    out.citizenship = prose.citizenship;

    // Official narrative blurb: first long paragraph if we still have little.
    if out.facts.iter().filter(|f| f.kind == "other").count() == 0 {
        if let Ok(re) = regex::Regex::new(r"(?is)<p\b[^>]*>(.*?)</p>") {
            for cap in re.captures_iter(&slice) {
                let t = strip_tags(cap.get(1).map(|m| m.as_str()).unwrap_or(""));
                if t.len() >= 80 && t.len() <= 900 {
                    let low = t.to_ascii_lowercase();
                    // Keep member bios; skip pure committee-assignment laundry lists.
                    let looks_bio = low.contains("represent")
                        || low.contains("elected")
                        || low.contains("grew up")
                        || low.contains("born")
                        || low.contains("is a ")
                        || low.contains("serves as");
                    let laundry = low.matches("committee").count() >= 3
                        && !looks_bio
                        && !low.contains("grew");
                    if laundry {
                        continue;
                    }
                    let clipped = if t.len() > 500 {
                        format!("{}…", &t[..497])
                    } else {
                        t
                    };
                    out.facts.push(BioFact {
                        kind: "other".into(),
                        text: clipped,
                        source: src.clone(),
                        source_url: src_url.clone(),
                        ..Default::default()
                    });
                    break;
                }
            }
        }
    }

    // Also try absolute photo from content img when og:image missing.
    if out.photo_url.is_none() {
        if let Ok(re) = regex::Regex::new(r#"(?is)<img\b([^>]+)>"#) {
            for cap in re.captures_iter(&slice) {
                let attrs = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let src_a = attr_value(attrs, "src").unwrap_or_default();
                if src_a.is_empty() {
                    continue;
                }
                let low = src_a.to_ascii_lowercase();
                if low.contains("logo")
                    || low.contains("icon")
                    || low.contains("seal")
                    || low.contains("sprite")
                {
                    continue;
                }
                if !(low.contains(".jpg")
                    || low.contains(".jpeg")
                    || low.contains(".png")
                    || low.contains(".webp"))
                {
                    continue;
                }
                let alt = attr_value(attrs, "alt")
                    .unwrap_or_default()
                    .to_ascii_lowercase();
                let hint = format!("{low} {alt}");
                if hint.contains("official")
                    || hint.contains("photo")
                    || hint.contains("portrait")
                    || hint.contains("congressman")
                    || hint.contains("congresswoman")
                    || hint.contains("senator")
                    || hint.contains("representative")
                {
                    let abs = if src_a.starts_with("http") {
                        src_a
                    } else if src_a.starts_with("//") {
                        format!("https:{src_a}")
                    } else if let Some(base) = src_url.as_ref() {
                        // resolve against site origin
                        if let Some(origin) = base
                            .split('/')
                            .take(3)
                            .collect::<Vec<_>>()
                            .get(0..3)
                            .map(|p| p.join("/"))
                        {
                            if src_a.starts_with('/') {
                                format!("{origin}{src_a}")
                            } else {
                                format!("{origin}/{src_a}")
                            }
                        } else {
                            src_a
                        }
                    } else {
                        src_a
                    };
                    out.photo_url = Some(abs);
                    break;
                }
            }
        }
    }

    out
}

// --- Wikipedia plain extract (H2): denser prose facts; fill gaps only vs BP/official ---

const WP_SOURCE: &str = "Wikipedia";

fn wikipedia_title_slug(title: &str) -> Option<String> {
    let t = title.trim();
    if t.is_empty() {
        return None;
    }
    let slug = t.split_whitespace().collect::<Vec<_>>().join("_");
    let enc = slug
        .chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => {
                let mut b = [0u8; 4];
                let s = c.encode_utf8(&mut b);
                s.bytes().map(|x| format!("%{x:02X}")).collect::<String>()
            }
        })
        .collect::<String>();
    if enc.is_empty() {
        None
    } else {
        Some(enc)
    }
}

/// MediaWiki extracts API URL (plain text, CORS `origin=*`). Caps length via exchars.
pub fn wikipedia_extract_api_url(title: &str) -> Option<String> {
    let enc = wikipedia_title_slug(title)?;
    Some(format!(
        "https://en.wikipedia.org/w/api.php?action=query&prop=extracts&explaintext=1\
         &exsectionformat=plain&redirects=1&format=json&origin=*&exchars=3000&titles={enc}"
    ))
}

fn wp_push(out: &mut MemberBioParse, kind: &str, text: String, page: &Option<String>) {
    let t = text.trim();
    if t.is_empty() || t.len() > 400 {
        return;
    }
    if out.facts.iter().any(|f| f.kind == kind && f.text == t) {
        return;
    }
    out.facts.push(BioFact {
        kind: kind.into(),
        text: t.to_string(),
        source: WP_SOURCE.into(),
        source_url: page.clone(),
        ..Default::default()
    });
}

/// Parse plain Wikipedia extract prose → birth / family / education / profession facts.
/// Cite or omit; no invented family. Pure — no network.
pub fn parse_wikipedia_plain_extract(text: &str, page_url: &str) -> MemberBioParse {
    let page = page_url.trim();
    let src_url = if page.is_empty() {
        None
    } else {
        Some(page.to_string())
    };
    let mut out = MemberBioParse {
        photo_source: Some(WP_SOURCE.into()),
        photo_source_url: src_url.clone(),
        ..Default::default()
    };
    let raw = text.trim();
    if raw.len() < 40 {
        return out;
    }
    // Collapse whitespace for sentence scans.
    let t = raw.split_whitespace().collect::<Vec<_>>().join(" ");

    // Birth date → year (+ Born fact).
    if let Ok(re) = regex::Regex::new(r"(?i)\bborn\s+(?:on\s+)?([A-Z][a-z]+\s+\d{1,2},\s+\d{4})") {
        if let Some(cap) = re.captures(&t) {
            let date = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            if let Some(y) = year_from_born_text(date) {
                out.birth_year = Some(y);
            }
            wp_push(&mut out, "other", format!("Born: {date}"), &src_url);
        }
    }

    // Birthplace: "born in Gainesville, Florida" (not "born February").
    if let Ok(re) = regex::Regex::new(
        r"(?i)\bborn\s+in\s+([A-Z][^.;]{2,60}?)(?:\.|;|\s+and\s+grew|\s+to\s|\s+the\s+son|\s+the\s+daughter)",
    ) {
        if let Some(cap) = re.captures(&t) {
            let place = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            let place = place.trim_end_matches(',').trim();
            if place.len() >= 3
                && !place.chars().next().is_some_and(|c| c.is_ascii_digit())
                && !place.to_ascii_lowercase().starts_with("february")
                && !place.to_ascii_lowercase().starts_with("january")
            {
                wp_push(&mut out, "other", format!("Birthplace: {place}"), &src_url);
            }
        }
    }

    // née Maiden
    if let Ok(re) = regex::Regex::new(r"(?i)\bn[eé]e\s+([A-Z][A-Za-z'''\-]{1,40})") {
        if let Some(cap) = re.captures(&t) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            if name.len() >= 2 {
                wp_push(&mut out, "family", format!("Née: {name}"), &src_url);
            }
        }
    }

    // son/daughter of A and B
    if let Ok(re) = regex::Regex::new(
        r"(?i)\b(?:the\s+)?(?:son|daughter)\s+of\s+([A-Z][^.;]{3,100}?)(?:\.|;|\s+He\s|\s+She\s)",
    ) {
        if let Some(cap) = re.captures(&t) {
            let parents = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            let parents = parents
                .trim_end_matches(',')
                .split(", the")
                .next()
                .unwrap_or(parents)
                .trim();
            if parents.len() >= 5 && parents.len() <= 120 {
                wp_push(&mut out, "family", format!("Parents: {parents}"), &src_url);
            }
        }
    }

    // his/her father X / mother X (single)
    if let Ok(re) = regex::Regex::new(
        r"(?i)\b(?:his|her)\s+(father|mother)\s+([A-Z][A-Za-z .'''\-]{2,60}?)(?:\.|,|;|\s+and\s|\s+who\s|\s+in\s)",
    ) {
        for cap in re.captures_iter(&t) {
            let rel = cap.get(1).map(|m| m.as_str()).unwrap_or("parent");
            let name = cap.get(2).map(|m| m.as_str()).unwrap_or("").trim();
            let name = name.trim_end_matches(',').trim();
            if name.len() >= 3 && name.len() <= 80 {
                let label = if rel.eq_ignore_ascii_case("father") {
                    "Father"
                } else {
                    "Mother"
                };
                wp_push(&mut out, "family", format!("{label}: {name}"), &src_url);
            }
        }
    }

    // married to X / wife X / husband X / her husband X / his wife X
    if let Ok(re) = regex::Regex::new(
        r"(?i)\b(?:(?:she|he)\s+and\s+)?(?:(?:his|her)\s+)?(?:married(?:\s+to)?|wife|husband)\s+([A-Z][A-Za-z .'''\-]{2,50}?)(?:\s+in\s+\d{4}|\s+have\b|\s+has\b|\.|,|;|\s+and\s)",
    ) {
        if let Some(cap) = re.captures(&t) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            if name.len() >= 2
                && !name.eq_ignore_ascii_case("the")
                && !name.to_ascii_lowercase().starts_with("member")
            {
                wp_push(&mut out, "family", format!("Spouse: {name}"), &src_url);
            }
        }
    }

    // children: "two boys, Theo and Henry" / "three children" / "sons A and B"
    if let Ok(re) = regex::Regex::new(
        r"(?i)\b(?:have|has)\s+(?:two|three|four|five|\d+)\s+(?:boys|girls|sons|daughters|children)(?:,\s*([A-Z][^.;]{2,80}?))?(?:\.|;|$)",
    ) {
        if let Some(cap) = re.captures(&t) {
            if let Some(names) = cap.get(1).map(|m| m.as_str().trim()) {
                let names = names.trim_end_matches('.').trim();
                if names.len() >= 3 && names.len() <= 100 {
                    wp_push(&mut out, "family", format!("Children: {names}"), &src_url);
                }
            }
        }
    }

    // Education: graduated from X; bachelor's … from X; J.D. from X; attended X
    if let Ok(re) = regex::Regex::new(
        r"(?i)\b(?:graduated from|received (?:his|her|a|an) (?:J\.?D\.?|M\.?D\.?|Ph\.?D\.?|bachelor'?s|master'?s|law)(?:\s+degree)? from(?: the)?|earned (?:his|her|a|an) (?:J\.?D\.?|bachelor'?s|master'?s|law)(?:\s+degree)? from(?: the)?|attended(?: the)?)\s+([A-Z][^.;]{3,90}?)(?:\s+in\s+(\d{4})|\s+with\s|\.|;|,?\s+and\s+from\s|\s+and\s+from\s)",
    ) {
        for cap in re.captures_iter(&t) {
            let school = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            let school = school
                .trim_end_matches(',')
                .trim_end_matches(" where")
                .trim();
            if school.len() < 3 || school.len() > 90 {
                continue;
            }
            let year = cap.get(2).map(|m| m.as_str());
            let text = if let Some(y) = year {
                format!("Education: {school}, {y}")
            } else {
                format!("Education: {school}")
            };
            wp_push(&mut out, "education", text, &src_url);
        }
    }
    // "graduated from A and from B" second school
    if let Ok(re) =
        regex::Regex::new(r"(?i)\band from(?: the)?\s+([A-Z][^.;]{3,90}?)(?:\s+in\s+(\d{4})|\.|;)")
    {
        for cap in re.captures_iter(&t) {
            let school = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            let school = school.trim_end_matches(',').trim();
            if school.len() < 3 || school.len() > 90 {
                continue;
            }
            let year = cap.get(2).map(|m| m.as_str());
            let text = if let Some(y) = year {
                format!("Education: {school}, {y}")
            } else {
                format!("Education: {school}")
            };
            wp_push(&mut out, "education", text, &src_url);
        }
    }

    // Profession from lead appositive: "is an American lawyer and politician"
    if let Ok(re) = regex::Regex::new(
        r"(?i)\bis an? (?:American |British |Canadian )?((?:lawyer|attorney|physician|doctor|farmer|teacher|educator|businessman|businesswoman|engineer|journalist|author|professor|pastor|minister|rancher)(?:\s+and\s+(?:lawyer|attorney|politician|businessman))?)",
    ) {
        if let Some(cap) = re.captures(&t) {
            let mut prof = cap
                .get(1)
                .map(|m| m.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            // Drop trailing "and politician"
            if let Some(idx) = prof.to_ascii_lowercase().find(" and politician") {
                prof = prof[..idx].trim().to_string();
            }
            if !prof.is_empty() && !prof.eq_ignore_ascii_case("politician") {
                let kind = if prof.to_ascii_lowercase().contains("lawyer")
                    || prof.to_ascii_lowercase().contains("attorney")
                {
                    "legal"
                } else {
                    "work"
                };
                let label = {
                    let mut c = prof.chars();
                    match c.next() {
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                        None => prof.clone(),
                    }
                };
                wp_push(&mut out, kind, format!("Profession: {label}"), &src_url);
            }
        }
    }

    out
}

/// Parse MediaWiki `action=query&prop=extracts` JSON → MemberBioParse.
pub fn parse_wikipedia_extract_json(json: &str) -> MemberBioParse {
    let mut empty = MemberBioParse::default();
    let Ok(v) = serde_json::from_str::<Value>(json) else {
        return empty;
    };
    let pages = v.pointer("/query/pages").and_then(|p| p.as_object());
    let Some(pages) = pages else {
        return empty;
    };
    for (_id, page) in pages {
        if page.get("missing").is_some() {
            continue;
        }
        let title = page
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .trim();
        let extract = page
            .get("extract")
            .and_then(|e| e.as_str())
            .unwrap_or("")
            .trim();
        if extract.is_empty() {
            continue;
        }
        let page_url = if title.is_empty() {
            "https://en.wikipedia.org/".into()
        } else {
            wikipedia_article_url(title).unwrap_or_else(|| "https://en.wikipedia.org/".into())
        };
        return parse_wikipedia_plain_extract(extract, &page_url);
    }
    empty.photo_source = None;
    empty
}

/// Whether a dossier already has facts in a gap bucket (for weaker-source fill-in).
fn dossier_has_fact_kind(d: &PersonDossier, kind: &str) -> bool {
    match kind {
        "family" => d.facts.iter().any(|f| f.kind == "family"),
        "education" => d.facts.iter().any(|f| f.kind == "education"),
        "work" | "business" | "legal" => d
            .facts
            .iter()
            .any(|f| matches!(f.kind.as_str(), "work" | "business" | "legal")),
        "other" => {
            // Allow multiple "other" only when distinct prefixes (Religion / Birthplace / Born).
            false
        }
        _ => d.facts.iter().any(|f| f.kind == kind),
    }
}

/// Weaker-source fact clusters with an existing row (multi-cite) rather than a novel claim.
fn fact_coalesce_compatible(existing: &[BioFact], incoming: &BioFact) -> bool {
    match incoming.kind.as_str() {
        "education" => {
            let ik = coalesce_edu_cluster_key(&incoming.text);
            if ik.starts_with("deg:") {
                // Degree-only may attach to a school row.
                return existing
                    .iter()
                    .any(|e| e.kind == "education" && coalesce_edu_school_key(&e.text).is_some());
            }
            existing
                .iter()
                .any(|e| e.kind == "education" && coalesce_edu_cluster_key(&e.text) == ik)
        }
        "work" | "legal" | "business" => {
            let ik = coalesce_work_cluster_key(&incoming.text);
            existing.iter().any(|e| {
                if !matches!(e.kind.as_str(), "work" | "legal" | "business") {
                    return false;
                }
                coalesce_work_cluster_key(&e.text) == ik
            })
        }
        _ => false,
    }
}

fn other_fact_prefix(text: &str) -> &str {
    text.split(':').next().unwrap_or(text).trim()
}

/// Apply bio only into empty gaps (photo/birth/kind buckets). Does not clobber BP/official.
pub fn apply_member_bio_fill_gaps(d: &mut PersonDossier, bio: &MemberBioParse, as_of_year: i32) {
    if let Some(ref url) = bio.photo_url {
        if !url.is_empty() && d.photo_url.is_none() {
            d.photo_url = Some(url.clone());
            d.photo_source = bio.photo_source.clone();
            d.photo_source_url = bio.photo_source_url.clone();
            d.empty_notes
                .retain(|n| !n.to_ascii_lowercase().starts_with("photo:"));
        }
    }

    let mut filtered = Vec::new();
    for f in &bio.facts {
        if f.kind == "other" {
            let pref = other_fact_prefix(&f.text).to_ascii_lowercase();
            let has_pref = d.facts.iter().any(|x| {
                x.kind == "other"
                    && other_fact_prefix(&x.text).eq_ignore_ascii_case(other_fact_prefix(&f.text))
            });
            // Skip long free-form blurbs when we already have any other narrative-ish fact
            // or stronger education/work — keep structured Born/Birthplace/Religion.
            let structured = matches!(
                pref.as_str(),
                "born" | "birthplace" | "religion" | "residence"
            );
            if has_pref {
                continue;
            }
            if !structured
                && d.facts
                    .iter()
                    .any(|x| x.kind == "other" && x.text.len() > 80)
            {
                continue;
            }
            filtered.push(f.clone());
            continue;
        }
        if dossier_has_fact_kind(d, &f.kind) {
            // Education/work: admit only cluster-compatible rows for multi-cite merge.
            // Family and novel schools stay gap-only (weaker source must not invent).
            if matches!(f.kind.as_str(), "education" | "work" | "business" | "legal")
                && fact_coalesce_compatible(&d.facts, f)
            {
                filtered.push(f.clone());
            }
            continue;
        }
        filtered.push(f.clone());
    }
    merge_facts_into_dossier(d, &filtered);
    coalesce_dossier_inplace(d);

    if let Some(ref c) = bio.citizenship {
        if c.disclosed && !d.citizenship.disclosed {
            d.citizenship = c.clone();
        }
    }
    apply_citizenship_from_facts(d);

    // Always reassess: fact years (edu/work) feed fractions even when bio.spans empty.
    reassess_dossier_career(d, &bio.spans, bio.birth_year, as_of_year);
}

// --- DBpedia (H4): structured infobox densify — gap-only vs BP/official/wiki ---

const DBP_SOURCE: &str = "DBpedia";

/// DBpedia resource page URL (human).
pub fn dbpedia_page_url(title: &str) -> Option<String> {
    let enc = wikipedia_title_slug(title)?;
    Some(format!("https://dbpedia.org/page/{enc}"))
}

/// DBpedia N-Triples data URL for a Wikipedia title (`/data/{Title}.ntriples`).
/// JSON `/data/*.json` is often 503; N-Triples/Turtle remain available.
pub fn dbpedia_ntriples_url(title: &str) -> Option<String> {
    let enc = wikipedia_title_slug(title)?;
    Some(format!("https://dbpedia.org/data/{enc}.ntriples"))
}

/// SPARQL DESCRIBE fallback (N-Triples) when `/data/` is down.
pub fn dbpedia_describe_ntriples_url(title: &str) -> Option<String> {
    let enc = wikipedia_title_slug(title)?;
    let resource = format!("http://dbpedia.org/resource/{enc}");
    let q = format!("DESCRIBE <{resource}>");
    Some(format!(
        "https://dbpedia.org/sparql?default-graph-uri=http%3A%2F%2Fdbpedia.org&query={}&format=application%2Fn-triples",
        urlencoding_minimal(&q)
    ))
}

fn urlencoding_minimal(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "%20".into(),
            _ => {
                let mut b = [0u8; 4];
                let enc = c.encode_utf8(&mut b);
                enc.bytes().map(|x| format!("%{x:02X}")).collect::<String>()
            }
        })
        .collect()
}

fn dbp_resource_label(uri_or_text: &str) -> String {
    let s = uri_or_text.trim();
    let s = s.trim_start_matches('<').trim_end_matches('>').trim();
    let leaf = s
        .rsplit('/')
        .next()
        .unwrap_or(s)
        .split('#')
        .next()
        .unwrap_or(s);
    let leaf = leaf.split('?').next().unwrap_or(leaf);
    // Percent-decode common cases lightly.
    let decoded = leaf
        .replace("%27", "'")
        .replace("%28", "(")
        .replace("%29", ")");
    decoded.replace('_', " ").trim().to_string()
}

fn dbp_parse_object(obj: &str) -> Option<String> {
    let o = obj.trim().trim_end_matches('.').trim();
    if o.is_empty() {
        return None;
    }
    // URI
    if o.starts_with('<') && o.ends_with('>') {
        let lab = dbp_resource_label(o);
        if lab.is_empty() {
            return None;
        }
        return Some(lab);
    }
    // Typed literal "val"^^<datatype>
    if let Some(rest) = o.strip_prefix('"') {
        let end = rest.find('"')?;
        let val = rest[..end].trim();
        if val.is_empty() {
            return None;
        }
        // Skip pure year integers used as spouse marriage year noise when labeled spouse elsewhere.
        return Some(html_unescape_basic(val));
    }
    // bare token
    if o.chars().all(|c| c.is_ascii_digit() || c == '-') {
        return Some(o.to_string());
    }
    None
}

fn dbp_pred_local(pred: &str) -> String {
    let p = pred.trim().trim_start_matches('<').trim_end_matches('>');
    p.rsplit('/')
        .next()
        .unwrap_or(p)
        .rsplit('#')
        .next()
        .unwrap_or(p)
        .to_ascii_lowercase()
}

fn dbp_push(out: &mut MemberBioParse, kind: &str, text: String, page: &Option<String>) {
    let t = text.trim();
    if t.is_empty() || t.len() > 400 {
        return;
    }
    // Skip pure integers for non-children kinds.
    if kind != "family" && t.chars().all(|c| c.is_ascii_digit()) {
        return;
    }
    if out.facts.iter().any(|f| f.kind == kind && f.text == t) {
        return;
    }
    out.facts.push(BioFact {
        kind: kind.into(),
        text: t.to_string(),
        source: DBP_SOURCE.into(),
        source_url: page.clone(),
        ..Default::default()
    });
}

/// Parse DBpedia N-Triples (DESCRIBE or `/data/*.ntriples`) for one person title.
/// Gap densify only at apply time — this extracts spouse/education/birth/children.
pub fn parse_dbpedia_ntriples(nt: &str, title: &str) -> MemberBioParse {
    let title = title.trim();
    let slug = wikipedia_title_slug(title).unwrap_or_default();
    let page = dbpedia_page_url(title);
    let mut out = MemberBioParse {
        photo_source: Some(DBP_SOURCE.into()),
        photo_source_url: page.clone(),
        ..Default::default()
    };
    if nt.len() < 40 || slug.is_empty() {
        return out;
    }
    // Reject HTML error pages.
    let head = nt.chars().take(80).collect::<String>().to_ascii_lowercase();
    if head.contains("<html") || head.contains("service unavailable") {
        return out;
    }

    let subj_exact = format!("<http://dbpedia.org/resource/{slug}>");

    for raw_line in nt.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Only triples about this person as subject.
        let Some(rest) = line.strip_prefix(&subj_exact) else {
            continue;
        };
        let rest = rest.trim_start();
        let (pred, obj) = if rest.starts_with('<') {
            let end = match rest.find('>') {
                Some(i) => i,
                None => continue,
            };
            let pred = &rest[..=end];
            let obj = rest[end + 1..].trim();
            (pred, obj)
        } else {
            continue;
        };
        let local = dbp_pred_local(pred);
        let Some(val) = dbp_parse_object(obj) else {
            continue;
        };
        if val.is_empty() {
            continue;
        }

        match local.as_str() {
            "spouse" => {
                // Skip marriage-year integers and empty.
                if val.chars().all(|c| c.is_ascii_digit()) {
                    continue;
                }
                if val.len() < 2 {
                    continue;
                }
                dbp_push(&mut out, "family", format!("Spouse: {val}"), &page);
            }
            "children" | "child" => {
                if val.chars().all(|c| c.is_ascii_digit()) {
                    dbp_push(&mut out, "family", format!("Children: {val}"), &page);
                } else if val.len() >= 2 {
                    dbp_push(&mut out, "family", format!("Children: {val}"), &page);
                }
            }
            "relatives" | "relation" => {
                if val.len() >= 2 && !val.chars().all(|c| c.is_ascii_digit()) {
                    dbp_push(&mut out, "family", format!("Relative: {val}"), &page);
                }
            }
            "education" | "almamater" => {
                if val.len() >= 2 && !val.chars().all(|c| c.is_ascii_digit()) {
                    dbp_push(&mut out, "education", format!("Education: {val}"), &page);
                }
            }
            "birthdate" => {
                if let Some(y) = year_from_born_text(&val) {
                    out.birth_year = Some(y);
                }
                dbp_push(&mut out, "other", format!("Born: {val}"), &page);
            }
            "birthplace" => {
                // Prefer city-level strings over bare state resource labels.
                let low = val.to_ascii_lowercase();
                if low == "florida"
                    || low == "new york"
                    || low == "california"
                    || low == "texas"
                    || low == "united states"
                {
                    continue;
                }
                dbp_push(&mut out, "other", format!("Birthplace: {val}"), &page);
            }
            "birthname" => {
                if val.len() >= 3 {
                    dbp_push(&mut out, "other", format!("Birth name: {val}"), &page);
                }
            }
            "religion" => {
                if val.len() >= 3 {
                    dbp_push(&mut out, "other", format!("Religion: {val}"), &page);
                }
            }
            "occupation" | "profession" => {
                let low = val.to_ascii_lowercase();
                if low == "politician" || low == "politics" {
                    continue;
                }
                let kind = if low.contains("lawyer") || low.contains("attorney") {
                    "legal"
                } else {
                    "work"
                };
                dbp_push(&mut out, kind, format!("Profession: {val}"), &page);
            }
            _ => {}
        }
    }

    out
}

// --- Grokipedia (H5): typeahead + page HTML via Wisp; high-precision only; never sole family/citizenship ---

const GP_SOURCE: &str = "Grokipedia";

/// Unique typeahead hit (slug + human title + cite URL).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GrokipediaHit {
    pub slug: String,
    pub title: String,
    pub page_url: String,
}

/// Typeahead JSON API (no CORS — browser needs Wisp).
pub fn grokipedia_typeahead_url(query: &str) -> Option<String> {
    let q = query.trim();
    if q.len() < 2 {
        return None;
    }
    let enc = urlencoding_minimal(q);
    if enc.is_empty() {
        return None;
    }
    Some(format!("https://grokipedia.com/api/typeahead?query={enc}"))
}

/// Human page URL for a slug (`Gus_Bilirakis`).
pub fn grokipedia_page_url(slug: &str) -> Option<String> {
    let s = slug.trim().trim_start_matches('/');
    if s.is_empty() || s.contains("://") || s.contains(' ') {
        return None;
    }
    // Allow slug chars only.
    if !s
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return None;
    }
    Some(format!("https://grokipedia.com/page/{s}"))
}

/// Fold common Latin diacritics so `Muniz` matches `Muñiz` (BP / court titles).
fn fold_name_char(c: char) -> char {
    let c = c.to_lowercase().next().unwrap_or(c);
    match c {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' => 'a',
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ė' | 'ę' => 'e',
        'ì' | 'í' | 'î' | 'ï' | 'ī' | 'į' => 'i',
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' => 'o',
        'ù' | 'ú' | 'û' | 'ü' | 'ū' => 'u',
        'ý' | 'ÿ' => 'y',
        'ñ' | 'ń' => 'n',
        'ç' | 'ć' | 'č' => 'c',
        'š' | 'ś' => 's',
        'ž' | 'ź' | 'ż' => 'z',
        'ł' => 'l',
        'đ' => 'd',
        'ß' => 's',
        _ => c,
    }
}

fn gp_name_tokens(s: &str) -> Vec<String> {
    s.chars()
        .map(|c| {
            let f = fold_name_char(c);
            if f.is_ascii_alphanumeric() || f == '\'' {
                f
            } else if c.is_ascii_alphanumeric() || c == '\'' {
                c.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .filter(|t| {
            t.len() >= 1
                && *t != "jr"
                && *t != "sr"
                && *t != "ii"
                && *t != "iii"
                && *t != "iv"
                && *t != "judge"
                && *t != "justice"
                && *t != "chief"
                && *t != "hon"
                && *t != "honorable"
                && *t != "the"
        })
        .map(|t| t.trim_matches('\'').to_string())
        .filter(|t| !t.is_empty())
        .collect()
}

fn gp_given_token_match(a: &str, b: &str) -> bool {
    a == b || (a.len() == 1 && b.starts_with(a)) || (b.len() == 1 && a.starts_with(b))
}

/// True when title is a high-precision match for person_name (shared last name + given name).
/// Allows middle names on either side: "Robert Alan Segal" ↔ "Robert Segal".
fn gp_title_matches_person(person_name: &str, title: &str) -> bool {
    let p = gp_name_tokens(person_name);
    let t = gp_name_tokens(title);
    if p.len() < 2 || t.len() < 2 {
        return false;
    }
    let p_last = p.last().unwrap();
    let t_last = t.last().unwrap();
    if p_last != t_last {
        return false;
    }
    let p_given = &p[..p.len() - 1];
    let t_given = &t[..t.len() - 1];
    // First given name must align (not last-name-only).
    let first_ok = gp_given_token_match(&p_given[0], &t_given[0]);
    if !first_ok {
        return false;
    }
    // Person givens ⊆ title givens OR title givens ⊆ person givens (middle names either side).
    let person_in_title = p_given
        .iter()
        .all(|pg| t_given.iter().any(|tg| gp_given_token_match(pg, tg)));
    let title_in_person = t_given
        .iter()
        .all(|tg| p_given.iter().any(|pg| gp_given_token_match(pg, tg)));
    person_in_title || title_in_person
}

/// Pick a unique high-precision typeahead hit. Ambiguous / last-name-only → None.
pub fn match_grokipedia_typeahead(json: &str, person_name: &str) -> Option<GrokipediaHit> {
    let name = person_name.trim();
    if name.len() < 3 {
        return None;
    }
    let v: Value = serde_json::from_str(json).ok()?;
    let results = v.get("results")?.as_array()?;
    if results.is_empty() {
        return None;
    }
    let mut hits: Vec<GrokipediaHit> = Vec::new();
    for r in results {
        let slug = r.get("slug").and_then(|x| x.as_str()).unwrap_or("").trim();
        let title = r.get("title").and_then(|x| x.as_str()).unwrap_or("").trim();
        if slug.is_empty() || title.is_empty() {
            continue;
        }
        if !gp_title_matches_person(name, title) {
            continue;
        }
        let Some(page_url) = grokipedia_page_url(slug) else {
            continue;
        };
        hits.push(GrokipediaHit {
            slug: slug.to_string(),
            title: title.to_string(),
            page_url,
        });
    }
    if hits.len() == 1 {
        return hits.pop();
    }
    // Multiple given-name matches (rare): require exact normalized title == name.
    let exact: Vec<_> = hits
        .into_iter()
        .filter(|h| gp_name_tokens(&h.title) == gp_name_tokens(name))
        .collect();
    if exact.len() == 1 {
        return exact.into_iter().next();
    }
    None
}

fn gp_strip_cite_markers(s: &str) -> String {
    let re = match regex::Regex::new(r"\[\d+\]") {
        Ok(r) => r,
        Err(_) => return s.to_string(),
    };
    re.replace_all(s, "").into_owned()
}

/// Plain text from first `<article>` (scripts/styles stripped). Pure.
pub fn grokipedia_article_plain(html: &str) -> String {
    let low = html.to_ascii_lowercase();
    let start = match low.find("<article") {
        Some(i) => i,
        None => return String::new(),
    };
    let after_start = &html[start..];
    let open_end = match after_start.find('>') {
        Some(i) => i + 1,
        None => return String::new(),
    };
    let rest = &after_start[open_end..];
    let rest_low = rest.to_ascii_lowercase();
    let end = match rest_low.find("</article>") {
        Some(i) => i,
        None => rest.len().min(200_000),
    };
    let mut body = rest[..end].to_string();
    if let Ok(re) = regex::Regex::new(r"(?is)<script[^>]*>.*?</script>") {
        body = re.replace_all(&body, " ").into_owned();
    }
    if let Ok(re) = regex::Regex::new(r"(?is)<style[^>]*>.*?</style>") {
        body = re.replace_all(&body, " ").into_owned();
    }
    let plain = strip_tags(&body);
    let plain = gp_strip_cite_markers(&plain);
    plain.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn gp_push(out: &mut MemberBioParse, kind: &str, text: String, page: &Option<String>) {
    let t = text.trim();
    if t.is_empty() || t.len() > 400 {
        return;
    }
    if out.facts.iter().any(|f| f.kind == kind && f.text == t) {
        return;
    }
    out.facts.push(BioFact {
        kind: kind.into(),
        text: t.to_string(),
        source: GP_SOURCE.into(),
        source_url: page.clone(),
        ..Default::default()
    });
}

/// High-precision Grokipedia page HTML → MemberBioParse.
/// Education / profession / birth only — **no family, no citizenship** (never sole source).
pub fn parse_grokipedia_page_html(html: &str, page_url: &str) -> MemberBioParse {
    let page = page_url.trim();
    let src_url = if page.is_empty() {
        None
    } else {
        Some(page.to_string())
    };
    let mut out = MemberBioParse {
        photo_source: Some(GP_SOURCE.into()),
        photo_source_url: src_url.clone(),
        ..Default::default()
    };
    if html.len() < 80 {
        return out;
    }
    let head = html
        .chars()
        .take(120)
        .collect::<String>()
        .to_ascii_lowercase();
    if head.contains("page not found") {
        return out;
    }
    let raw = grokipedia_article_plain(html);
    if raw.len() < 40 {
        return out;
    }
    // Cap scan window — lead + early life/education is enough; avoid whole-page noise.
    let t = if raw.len() > 6000 {
        raw.chars().take(6000).collect::<String>()
    } else {
        raw
    };

    if let Ok(re) = regex::Regex::new(r"(?i)\bborn\s+(?:on\s+)?([A-Z][a-z]+\s+\d{1,2},\s+\d{4})") {
        if let Some(cap) = re.captures(&t) {
            let date = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            if let Some(y) = year_from_born_text(date) {
                out.birth_year = Some(y);
            }
            gp_push(&mut out, "other", format!("Born: {date}"), &src_url);
        }
    }

    if let Ok(re) = regex::Regex::new(
        r"(?i)\bborn\s+(?:on\s+[A-Z][a-z]+\s+\d{1,2},\s+\d{4},\s+)?in\s+([A-Z][^.;]{2,60}?)(?:\.|;|\s+as\s|\s+and\s+grew|\s+to\s|\s+the\s+son|\s+the\s+daughter)",
    ) {
        if let Some(cap) = re.captures(&t) {
            let place = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            let place = place.trim_end_matches(',').trim();
            if place.len() >= 3
                && !place.chars().next().is_some_and(|c| c.is_ascii_digit())
                && !place.to_ascii_lowercase().starts_with("february")
                && !place.to_ascii_lowercase().starts_with("january")
                && !place.to_ascii_lowercase().starts_with("december")
            {
                gp_push(&mut out, "other", format!("Birthplace: {place}"), &src_url);
            }
        }
    }

    // "earned a Bachelor of Arts degree in 1986" / "obtaining his Juris Doctor in 1989"
    // + graduated from / J.D. from / attended (same family as WP).
    if let Ok(re) = regex::Regex::new(
        r"(?i)\b(?:graduated from|received (?:his|her|a|an) (?:J\.?D\.?|M\.?D\.?|Ph\.?D\.?|bachelor'?s|master'?s|law)(?:\s+degree)? from(?: the)?|earned (?:his|her|a|an) (?:J\.?D\.?|Bachelor of Arts|Bachelor of Science|bachelor'?s|master'?s|law)(?:\s+degree)?(?:\s+from(?: the)?)?|obtaining (?:his|her) (?:Juris Doctor|J\.?D\.?)|attended(?: the)?)\s+([A-Z][^.;]{3,90}?)(?:\s+in\s+(\d{4})|\s+with\s|\.|;|,?\s+and\s+from\s|\s+and\s+from\s|,?\s+where\s|\s+before\s|\s+from\s+\d)",
    ) {
        for cap in re.captures_iter(&t) {
            let school = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            let school = school
                .trim_end_matches(',')
                .trim_end_matches(" where")
                .trim();
            // "earned a Bachelor… in 1986" may capture year-only body — skip non-schools.
            if school.len() < 3 || school.len() > 90 {
                continue;
            }
            if school.chars().all(|c| c.is_ascii_digit() || c == ' ') {
                continue;
            }
            let low = school.to_ascii_lowercase();
            if low.starts_with("bachelor")
                || low.starts_with("master")
                || low.starts_with("juris")
                || low.starts_with("degree")
            {
                continue;
            }
            let year = cap.get(2).map(|m| m.as_str());
            let text = if let Some(y) = year {
                format!("Education: {school}, {y}")
            } else {
                format!("Education: {school}")
            };
            gp_push(&mut out, "education", text, &src_url);
        }
    }

    // Explicit: "University of Florida, where he earned a Bachelor of Arts degree in 1986"
    if let Ok(re) = regex::Regex::new(
        r"(?i)\b((?:University|College|School|Institute)\s+of\s+[A-Z][^,.;]{2,60}?|(?:[A-Z][A-Za-z.]+(?:\s+[A-Z][A-Za-z.]+){0,5}\s+(?:University|College)(?:\s+of\s+Law)?))\s*,?\s+where\s+(?:he|she)\s+earned\s+(?:a|an|his|her)\s+[^.]{0,40}?\b(?:in|in\s+)\s*(\d{4})",
    ) {
        for cap in re.captures_iter(&t) {
            let school = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            let year = cap.get(2).map(|m| m.as_str());
            if school.len() < 5 {
                continue;
            }
            let text = if let Some(y) = year {
                format!("Education: {school}, {y}")
            } else {
                format!("Education: {school}")
            };
            gp_push(&mut out, "education", text, &src_url);
        }
    }

    // "at Stetson University College of Law, obtaining his Juris Doctor in 1989"
    if let Ok(re) = regex::Regex::new(
        r"(?i)\bat\s+([A-Z][^,.;]{5,80}?(?:University|College|School)[^,.;]{0,40}?),\s+obtaining\s+(?:his|her)\s+(?:Juris Doctor|J\.?D\.?)\s+in\s+(\d{4})",
    ) {
        for cap in re.captures_iter(&t) {
            let school = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            let year = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            if school.len() >= 5 {
                gp_push(
                    &mut out,
                    "education",
                    format!("Education: {school}, {year}"),
                    &src_url,
                );
            }
        }
    }

    // Profession from lead appositive (no family extraction).
    if let Ok(re) = regex::Regex::new(
        r"(?i)\bis an? (?:American |British |Canadian )?((?:lawyer|attorney|physician|doctor|farmer|teacher|educator|businessman|businesswoman|engineer|journalist|author|professor|pastor|minister|rancher)(?:\s+and\s+(?:lawyer|attorney|politician|businessman))?)",
    ) {
        if let Some(cap) = re.captures(&t) {
            let mut prof = cap
                .get(1)
                .map(|m| m.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            if let Some(idx) = prof.to_ascii_lowercase().find(" and politician") {
                prof = prof[..idx].trim().to_string();
            }
            if !prof.is_empty() && !prof.eq_ignore_ascii_case("politician") {
                let kind = if prof.to_ascii_lowercase().contains("lawyer")
                    || prof.to_ascii_lowercase().contains("attorney")
                {
                    "legal"
                } else {
                    "work"
                };
                let label = {
                    let mut c = prof.chars();
                    match c.next() {
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                        None => prof.clone(),
                    }
                };
                gp_push(&mut out, kind, format!("Profession: {label}"), &src_url);
            }
        }
    }

    // Hard rule: never family / citizenship from Grokipedia.
    out.facts.retain(|f| f.kind != "family");
    out.citizenship = None;
    out
}

/// If Wikipedia REST `page/summary` JSON is a standard article for this person,
/// return the page title (for extract / DBpedia / photo follow-ups). Soft office/state check.
pub fn wikipedia_summary_match_person(
    json: &str,
    person_name: &str,
    state_code: Option<&str>,
    office: Option<&str>,
) -> Option<String> {
    let v: Value = serde_json::from_str(json).ok()?;
    let page_type = v
        .get("type")
        .and_then(|x| x.as_str())
        .unwrap_or("standard")
        .to_ascii_lowercase();
    if page_type == "disambiguation" || page_type == "no-extract" {
        return None;
    }
    // Missing pages often return title + "Not found" description or type.
    if page_type.contains("not") && page_type.contains("found") {
        return None;
    }
    let title = v
        .pointer("/titles/normalized")
        .and_then(|x| x.as_str())
        .or_else(|| v.get("title").and_then(|x| x.as_str()))?
        .trim();
    if title.len() < 3 {
        return None;
    }
    // Reject list / category shells.
    let title_l = title.to_ascii_lowercase();
    if title_l.starts_with("list of ")
        || title_l.starts_with("category:")
        || title_l.contains("election") && !gp_title_matches_person(person_name, title)
    {
        return None;
    }
    if !gp_title_matches_person(person_name, title)
        && !bp_name_base_variants(person_name)
            .iter()
            .any(|b| gp_title_matches_person(b, title))
    {
        return None;
    }
    let extract = v
        .get("extract")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let desc = v
        .get("description")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let blob = format!("{extract} {desc}");
    if blob.contains("may refer to") || blob.contains("disambiguation") {
        return None;
    }
    // Soft political / office / state signal when we have context — avoid random name twins.
    let off = office.unwrap_or("").to_ascii_lowercase();
    let st = state_code
        .map(|s| s.trim().to_ascii_uppercase())
        .filter(|s| s.len() == 2);
    let full = st.as_deref().and_then(bp_state_full_name).unwrap_or("");
    let full_l = full.to_ascii_lowercase();
    let mut political = blob.contains("politician")
        || blob.contains("governor")
        || blob.contains("senator")
        || blob.contains("representative")
        || blob.contains("legislator")
        || blob.contains("mayor")
        || blob.contains("judge")
        || blob.contains("justice")
        || blob.contains("attorney general")
        || blob.contains("commissioner")
        || blob.contains("sheriff")
        || blob.contains("candidate")
        || blob.contains("elected")
        || blob.contains("congress")
        || blob.contains("assembly")
        || blob.contains("council");
    if !off.is_empty() {
        // Office keyword fragments in extract (e.g. "governor", "school board")
        for tok in off.split(|c: char| !c.is_ascii_alphanumeric()) {
            if tok.len() >= 5 && blob.contains(tok) {
                political = true;
                break;
            }
        }
    }
    let mut has_state = full_l.is_empty();
    if !full_l.is_empty() {
        has_state = blob.contains(&full_l);
        if let Some(code) = st.as_deref() {
            let code_l = code.to_ascii_lowercase();
            has_state = has_state
                || blob.contains(&format!(" {code_l}"))
                || blob.contains(&format!("({code_l})"))
                || blob.contains(&format!(", {code_l}"));
        }
    }
    // Accept if political signal, or state matches and extract is non-trivial, or no state known.
    if political || (has_state && extract.len() > 80) || (st.is_none() && extract.len() > 120) {
        return Some(title.to_string());
    }
    // Description-only hits (some summaries thin) with state in description.
    if has_state && !desc.is_empty() && (political || desc.contains("american")) {
        return Some(title.to_string());
    }
    None
}

/// Parse Wikipedia REST `page/summary` JSON → optional photo URL + source page.
/// Prefer `originalimage` then `thumbnail`.
pub fn parse_wikipedia_summary_photo(json: &str) -> Option<(String, String)> {
    let v: Value = serde_json::from_str(json).ok()?;
    let thr = v
        .pointer("/originalimage/source")
        .and_then(|x| x.as_str())
        .or_else(|| v.pointer("/thumbnail/source").and_then(|x| x.as_str()))?
        .trim();
    if thr.is_empty() || !thr.starts_with("http") {
        return None;
    }
    // Drop tracking query junk when present.
    let photo = thr.split('?').next().unwrap_or(thr).to_string();
    let page = v
        .pointer("/content_urls/desktop/page")
        .and_then(|x| x.as_str())
        .or_else(|| {
            v.get("titles")
                .and_then(|t| t.get("canonical"))
                .and_then(|x| x.as_str())
                .map(|_| "")
        })
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| {
            v.pointer("/titles/canonical")
                .and_then(|x| x.as_str())
                .and_then(wikipedia_article_url)
        })
        .unwrap_or_else(|| "https://en.wikipedia.org/".into());
    Some((photo, page))
}

/// House Clerk / Senate eFD search portals for federal members.
pub fn federal_disclosure_portals(chamber: Option<&str>) -> Vec<DisclosurePortal> {
    let ch = chamber.unwrap_or("").to_ascii_lowercase();
    let mut out = Vec::new();
    if ch.contains("house") || ch == "h" || ch.is_empty() {
        out.push(DisclosurePortal {
            label: "House Clerk financial disclosure search".into(),
            url: "https://disclosures-clerk.house.gov/FinancialDisclosure".into(),
            note: "Official House personal financial disclosures (PDF). Annual FD Original Schedule A may auto-fill when searchable text PDF is reachable; scanned image PDFs stay portal-only. Not campaign FEC totals.".into(),
        });
    }
    if ch.contains("senate") || ch == "s" || ch.is_empty() {
        out.push(DisclosurePortal {
            label: "Senate eFD search".into(),
            url: "https://efdsearch.senate.gov/search/home/".into(),
            note: "Official Senate electronic financial disclosures. Annual Part 3 assets may auto-fill when eFD search is reachable; otherwise search by name.".into(),
        });
    }
    if out.is_empty() {
        out.push(DisclosurePortal {
            label: "House Clerk financial disclosure search".into(),
            url: "https://disclosures-clerk.house.gov/FinancialDisclosure".into(),
            note: "Federal personal financial disclosures when applicable.".into(),
        });
    }
    out
}

/// One row from eFD DataTables `/search/report/data/` (or fixture).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EfdReportHit {
    pub filer_name: String,
    pub office: String,
    pub state: String,
    pub report_type: String,
    pub date_filed: String,
    /// Absolute or site-relative report path, e.g. `/search/view/annual/{uuid}/`.
    pub report_path: String,
}

const EFD_ORIGIN: &str = "https://efdsearch.senate.gov";

/// Absolute eFD URL from a path or full URL.
pub fn efd_abs_url(path_or_url: &str) -> String {
    let p = path_or_url.trim();
    if p.starts_with("http://") || p.starts_with("https://") {
        p.to_string()
    } else if p.starts_with('/') {
        format!("{EFD_ORIGIN}{p}")
    } else if p.is_empty() {
        format!("{EFD_ORIGIN}/search/home/")
    } else {
        format!("{EFD_ORIGIN}/{p}")
    }
}

fn efd_strip_cell(html: &str) -> String {
    strip_tags(html).trim().to_string()
}

fn efd_href_from_cell(html: &str) -> Option<String> {
    let re = regex::Regex::new(r#"(?i)href\s*=\s*["']([^"']+)["']"#).ok()?;
    re.captures(html)
        .and_then(|c| c.get(1).map(|m| m.as_str().trim().to_string()))
        .filter(|s| !s.is_empty())
}

/// Office label from eFD display name, e.g. `Gillibrand, Kirsten E. (Senator)` → `Senator`.
fn efd_office_from_display_name(name: &str) -> String {
    let re = match regex::Regex::new(r"\(([^)]+)\)\s*$") {
        Ok(r) => r,
        Err(_) => return String::new(),
    };
    re.captures(name.trim())
        .and_then(|c| c.get(1).map(|m| m.as_str().trim().to_string()))
        .unwrap_or_default()
}

/// Parse DataTables JSON body from POST `/search/report/data/`.
///
/// Two layouts observed:
/// - **Live (2026):** `[first, last, "Last, First (Senator)", "<a>Annual Report…</a>", date]`
/// - **Legacy/fixture:** `["<a>Last, First</a>", office, state, "Annual", date]`
pub fn parse_efd_search_data_json(json: &str) -> Result<Vec<EfdReportHit>, String> {
    let v: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("efd search json: {e}"))?;
    let rows = v
        .get("data")
        .and_then(|d| d.as_array())
        .ok_or_else(|| "efd search json: missing data[]".to_string())?;
    let mut out = Vec::new();
    for row in rows {
        let cells: Vec<String> = match row {
            serde_json::Value::Array(arr) => arr
                .iter()
                .map(|c| match c {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .collect(),
            _ => continue,
        };
        if cells.is_empty() {
            continue;
        }

        let c0 = cells.first().map(|s| s.as_str()).unwrap_or("");
        let c1 = cells.get(1).map(|s| s.as_str()).unwrap_or("");
        let c2 = cells.get(2).map(|s| s.as_str()).unwrap_or("");
        let c3 = cells.get(3).map(|s| s.as_str()).unwrap_or("");
        let c4 = cells.get(4).map(|s| s.as_str()).unwrap_or("");

        let path0 = efd_href_from_cell(c0);
        let path3 = efd_href_from_cell(c3);

        let (filer_name, office, state, report_type, date_filed, report_path) = if path0.is_some() {
            // Legacy: name+link | office | state | report type | date
            (
                efd_strip_cell(c0),
                efd_strip_cell(c1),
                efd_strip_cell(c2),
                efd_strip_cell(c3),
                efd_strip_cell(c4),
                path0.unwrap_or_default(),
            )
        } else if path3.is_some() {
            // Live: first | last | "Last, First (Office)" | report link | date
            let display = efd_strip_cell(c2);
            let filer = if !display.is_empty() {
                display
            } else {
                let first = efd_strip_cell(c0);
                let last = efd_strip_cell(c1);
                if last.is_empty() {
                    first
                } else if first.is_empty() {
                    last
                } else {
                    format!("{last}, {first}")
                }
            };
            let office = efd_office_from_display_name(&filer);
            (
                filer,
                office,
                String::new(), // state not in live row; search already filtered
                efd_strip_cell(c3),
                efd_strip_cell(c4),
                path3.unwrap_or_default(),
            )
        } else {
            // Best-effort: first cell with href is the report link
            let mut path = String::new();
            let mut report_type = String::new();
            for cell in &cells {
                if let Some(p) = efd_href_from_cell(cell) {
                    path = p;
                    report_type = efd_strip_cell(cell);
                    break;
                }
            }
            let filer = cells
                .iter()
                .map(|s| efd_strip_cell(s))
                .find(|s| s.contains(','))
                .unwrap_or_else(|| efd_strip_cell(c0));
            if filer.is_empty() && path.is_empty() {
                continue;
            }
            (
                filer.clone(),
                efd_office_from_display_name(&filer),
                String::new(),
                report_type,
                cells
                    .iter()
                    .rev()
                    .map(|s| efd_strip_cell(s))
                    .find(|s| efd_parse_mdy(s).is_some())
                    .unwrap_or_default(),
                path,
            )
        };

        if filer_name.is_empty() && report_path.is_empty() {
            continue;
        }
        out.push(EfdReportHit {
            filer_name,
            office,
            state,
            report_type,
            date_filed,
            report_path,
        });
    }
    Ok(out)
}

/// Holdings snapshot filings: yearly Annual, New Filer (first disclosure), and
/// candidate reports. eFD UI groups New Filer under the Annual checkbox.
/// Exclude PTR / periodic transaction lists.
fn efd_is_holdings_report(report_type: &str, report_path: &str) -> bool {
    let t = report_type.to_ascii_lowercase();
    let p = report_path.to_ascii_lowercase();
    if t.contains("periodic")
        || t.contains("transaction")
        || t.contains("ptr")
        || p.contains("/ptr/")
        || p.contains("/view/ptr")
    {
        return false;
    }
    if t.contains("annual")
        || t.contains("new filer")
        || t.contains("candidate report")
        || t.contains("amendment") && t.contains("annual")
    {
        return true;
    }
    // Live rows often link under /search/view/annual/{uuid}/ even for New Filer.
    p.contains("/view/annual/")
}

fn efd_parse_mdy(s: &str) -> Option<(i32, u32, u32)> {
    let re = regex::Regex::new(r"(\d{1,2})/(\d{1,2})/(\d{4})").ok()?;
    let c = re.captures(s.trim())?;
    let m: u32 = c.get(1)?.as_str().parse().ok()?;
    let d: u32 = c.get(2)?.as_str().parse().ok()?;
    let y: i32 = c.get(3)?.as_str().parse().ok()?;
    if (1..=12).contains(&m) && (1..=31).contains(&d) && (2012..=2100).contains(&y) {
        Some((y, m, d))
    } else {
        None
    }
}

/// Split ballot/FEC-style name into (first, last) best-effort.
pub fn efd_split_person_name(name: &str) -> (String, String) {
    let n = name.trim();
    if n.is_empty() {
        return (String::new(), String::new());
    }
    // "LAST, FIRST MIDDLE" (FEC)
    if let Some((last, rest)) = n.split_once(',') {
        let last = last.trim().to_string();
        let first = rest
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_matches(|c: char| !c.is_alphabetic() && c != '\'')
            .to_string();
        return (first, last);
    }
    let parts: Vec<&str> = n.split_whitespace().collect();
    if parts.len() == 1 {
        return (parts[0].to_string(), parts[0].to_string());
    }
    let last = parts.last().copied().unwrap_or("").to_string();
    let first = parts.first().copied().unwrap_or("").to_string();
    (first, last)
}

fn efd_is_honorific_token(tok: &str) -> bool {
    let t = tok
        .trim_matches(|c: char| !c.is_alphabetic())
        .to_ascii_lowercase();
    matches!(
        t.as_str(),
        "hon" | "honorable" | "mr" | "mrs" | "ms" | "miss" | "dr" | "rep" | "sen" | "prof" | "rev"
    )
}

fn efd_name_matches(filer_name: &str, candidate_name: &str) -> bool {
    let (c_first, c_last) = efd_split_person_name(candidate_name);
    if c_last.is_empty() {
        return false;
    }
    let filer = filer_name.to_ascii_lowercase();
    let c_last_l = c_last.to_ascii_lowercase();
    let c_first_l = c_first.to_ascii_lowercase();
    // eFD: "Tillis, Thomas (R)"; House Clerk: "Pelosi, Hon.. Nancy"
    let (f_last, f_rest) = if let Some((a, b)) = filer_name.split_once(',') {
        (a.trim().to_ascii_lowercase(), b.trim().to_ascii_lowercase())
    } else {
        // fallback: last token
        let parts: Vec<&str> = filer_name.split_whitespace().collect();
        (
            parts
                .last()
                .copied()
                .unwrap_or("")
                .trim_matches(|c: char| !c.is_alphabetic())
                .to_ascii_lowercase(),
            filer.to_ascii_lowercase(),
        )
    };
    if f_last != c_last_l {
        // allow hyphenated / suffix noise
        if !f_last.starts_with(&c_last_l) && !c_last_l.starts_with(&f_last) {
            return false;
        }
    }
    if c_first_l.is_empty() {
        return true;
    }
    // first name starts-with either direction (Kirsten / K.); skip Hon./Mr./…
    let f_first = f_rest
        .split_whitespace()
        .map(|t| {
            t.trim_matches(|c: char| !c.is_alphabetic() && c != '\'')
                .to_ascii_lowercase()
        })
        .find(|t| !t.is_empty() && !efd_is_honorific_token(t))
        .unwrap_or_default();
    if f_first.is_empty() {
        return filer.contains(&c_first_l);
    }
    f_first.starts_with(&c_first_l)
        || c_first_l.starts_with(&f_first)
        || f_first.chars().next() == c_first_l.chars().next()
}

fn efd_state_matches(hit_state: &str, want: &str) -> bool {
    let w = want.trim().to_ascii_uppercase();
    if w.is_empty() {
        return true;
    }
    let h = hit_state.trim().to_ascii_uppercase();
    if h.is_empty() {
        return true;
    }
    if h == w {
        return true;
    }
    // full name vs abbrev — light map for common
    let full = match w.as_str() {
        "NY" => "NEW YORK",
        "NC" => "NORTH CAROLINA",
        "CA" => "CALIFORNIA",
        "FL" => "FLORIDA",
        "TX" => "TEXAS",
        "OH" => "OHIO",
        "PA" => "PENNSYLVANIA",
        "IL" => "ILLINOIS",
        "AZ" => "ARIZONA",
        "MD" => "MARYLAND",
        _ => "",
    };
    !full.is_empty() && h.contains(full)
}

/// Pick the latest holdings snapshot report (Annual / New Filer / candidate) for a
/// uniquely matching filer name (+ optional state).
/// Ambiguous multi-person match → `None` (caller keeps portal link only).
pub fn pick_efd_annual_report(
    hits: &[EfdReportHit],
    candidate_name: &str,
    state: Option<&str>,
) -> Option<EfdReportHit> {
    let st = state.unwrap_or("").trim();
    let matched: Vec<&EfdReportHit> = hits
        .iter()
        .filter(|h| efd_is_holdings_report(&h.report_type, &h.report_path))
        .filter(|h| !h.report_path.is_empty())
        .filter(|h| efd_name_matches(&h.filer_name, candidate_name))
        .filter(|h| efd_state_matches(&h.state, st))
        .collect();
    if matched.is_empty() {
        return None;
    }
    // Distinct filer name keys — if more than one person, skip.
    let mut people = std::collections::BTreeSet::new();
    for h in &matched {
        let key = h
            .filer_name
            .split('(')
            .next()
            .unwrap_or(&h.filer_name)
            .trim()
            .to_ascii_lowercase();
        people.insert(key);
    }
    if people.len() > 1 {
        return None;
    }
    matched
        .into_iter()
        .max_by(|a, b| {
            let da = efd_parse_mdy(&a.date_filed).unwrap_or((0, 0, 0));
            let db = efd_parse_mdy(&b.date_filed).unwrap_or((0, 0, 0));
            da.cmp(&db)
        })
        .cloned()
}

fn efd_holding_kind(asset_type: &str) -> String {
    let t = asset_type.to_ascii_lowercase();
    if t.contains("stock")
        || t.contains("corporate securities")
        || t.contains("mutual fund")
        || t.contains("exchange traded")
        || t.contains("etf")
        || t.contains("bond")
    {
        "stock".into()
    } else if t.contains("real estate") || t.contains("property") || t.contains("residential") {
        "property".into()
    } else if t.contains("business") || t.contains("partnership") || t.contains("llc") {
        "business".into()
    } else {
        "other".into()
    }
}

/// Parse Senate eFD annual report HTML → Part 3 asset rows as `PersonalHolding`.
/// `source_url` should be the absolute report URL (cited on every holding).
pub fn parse_senate_efd_annual_html(html: &str, source_url: &str) -> Vec<PersonalHolding> {
    let src = "Senate eFD";
    let url = {
        let u = source_url.trim();
        if u.is_empty() {
            None
        } else {
            Some(efd_abs_url(u))
        }
    };

    // Prefer #grid_items tbody; fall back to first table after "Part 3. Assets".
    let tbody = if let Some(m) =
        regex::Regex::new(r#"(?is)id\s*=\s*["']grid_items["'][\s\S]*?<tbody>([\s\S]*?)</tbody>"#)
            .ok()
            .and_then(|re| re.captures(html))
    {
        m.get(1).map(|x| x.as_str()).unwrap_or("").to_string()
    } else if let Some(idx) = html.to_ascii_lowercase().find("part 3") {
        let slice = &html[idx..];
        regex::Regex::new(r"(?is)<tbody>([\s\S]*?)</tbody>")
            .ok()
            .and_then(|re| re.captures(slice))
            .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
            .unwrap_or_default()
    } else {
        String::new()
    };
    if tbody.is_empty() {
        return Vec::new();
    }

    let row_re = regex::Regex::new(r"(?is)<tr\b[^>]*>([\s\S]*?)</tr>").ok();
    let cell_re = regex::Regex::new(r"(?is)<t[dh]\b[^>]*>([\s\S]*?)</t[dh]>").ok();
    let Some(row_re) = row_re else {
        return Vec::new();
    };
    let Some(cell_re) = cell_re else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for row in row_re.captures_iter(&tbody) {
        let row_html = row.get(1).map(|m| m.as_str()).unwrap_or("");
        let cells: Vec<String> = cell_re
            .captures_iter(row_html)
            .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
            .collect();
        // [#], Asset, Asset Type, Owner, Value, Income Type, Income
        if cells.len() < 5 {
            continue;
        }
        let asset_html = cells.get(1).map(|s| s.as_str()).unwrap_or("");
        let type_html = cells.get(2).map(|s| s.as_str()).unwrap_or("");
        let value = efd_strip_cell(cells.get(4).map(|s| s.as_str()).unwrap_or(""));
        let asset_name = efd_strip_cell(asset_html);
        if asset_name.is_empty() {
            continue;
        }
        // Skip container rows (brokerage parents) and non-dollar placeholders.
        let value_l = value.to_ascii_lowercase();
        let has_amount =
            value.contains('$') || value_l.contains("over $") || value_l.starts_with("over ");
        if !has_amount {
            continue;
        }

        let type_plain = efd_strip_cell(type_html);
        let kind = efd_holding_kind(&type_plain);
        let owner = efd_strip_cell(cells.get(3).map(|s| s.as_str()).unwrap_or(""));
        let mut desc = asset_name;
        if !type_plain.is_empty() {
            desc = format!("{desc} ({type_plain})");
        }
        if !owner.is_empty() && !owner.eq_ignore_ascii_case("self") {
            desc = format!("{desc} · owner: {owner}");
        }
        out.push(PersonalHolding {
            kind,
            description: desc,
            amount_display: Some(value),
            source: src.into(),
            source_url: url.clone(),
        });
    }
    // Cap display bulk — annuals can list 100+ rows; keep first 40 unique by description.
    let mut seen = std::collections::BTreeSet::new();
    out.retain(|h| seen.insert(h.description.clone()));
    if out.len() > 40 {
        out.truncate(40);
    }
    out
}

/// Merge holdings into dossier; refresh empty_notes. Does not invent when empty.
pub fn apply_holdings_to_dossier(d: &mut PersonDossier, holdings: Vec<PersonalHolding>) {
    if holdings.is_empty() {
        return;
    }
    d.holdings = holdings;
    d.empty_notes.retain(|n| {
        let l = n.to_ascii_lowercase();
        !l.contains("personal holdings") && !l.contains("no public disclosure parsed")
    });
}

// --- House Clerk financial disclosure (PDF Schedule A assets) ---

const HOUSE_CLERK_ORIGIN: &str = "https://disclosures-clerk.house.gov";

/// One filing row from House Clerk member/candidate search HTML.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HouseClerkFilingHit {
    pub filer_name: String,
    /// e.g. `CA11` or `FL12`.
    pub office: String,
    pub filing_year: String,
    /// e.g. `FD Original`, `PTR Original`.
    pub filing_type: String,
    /// Site-relative PDF path, e.g. `public_disc/financial-pdfs/2025/10075701.pdf`.
    pub pdf_path: String,
}

/// Absolute House Clerk URL from a relative path or full URL.
pub fn house_clerk_abs_url(path_or_url: &str) -> String {
    let p = path_or_url.trim();
    if p.starts_with("http://") || p.starts_with("https://") {
        p.to_string()
    } else if p.starts_with('/') {
        format!("{HOUSE_CLERK_ORIGIN}{p}")
    } else if p.is_empty() {
        format!("{HOUSE_CLERK_ORIGIN}/FinancialDisclosure")
    } else {
        format!("{HOUSE_CLERK_ORIGIN}/{p}")
    }
}

fn house_clerk_is_fd_original(filing_type: &str) -> bool {
    let t = filing_type.to_ascii_lowercase();
    // Prefer annual FD / original annual — never PTR transaction lists as holdings.
    (t.contains("fd") || t.contains("annual"))
        && !t.contains("ptr")
        && !t.contains("periodic")
        && !t.contains("transaction")
}

fn house_clerk_office_state(office: &str) -> String {
    office
        .chars()
        .take_while(|c| c.is_ascii_alphabetic())
        .collect::<String>()
        .to_ascii_uppercase()
}

fn house_clerk_office_district(office: &str) -> Option<u32> {
    let digits: String = office.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

/// Parse House Clerk `ViewMemberSearchResult` / candidate search HTML table rows.
pub fn parse_house_clerk_search_html(html: &str) -> Vec<HouseClerkFilingHit> {
    let row_re =
        match regex::Regex::new(r#"(?is)<tr\b[^>]*role\s*=\s*["']row["'][^>]*>([\s\S]*?)</tr>"#) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };
    let cell_re = match regex::Regex::new(r"(?is)<t[dh]\b[^>]*>([\s\S]*?)</t[dh]>") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let href_re = match regex::Regex::new(r#"(?i)href\s*=\s*["']([^"']+)["']"#) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    for row in row_re.captures_iter(html) {
        let row_html = row.get(1).map(|m| m.as_str()).unwrap_or("");
        let cells: Vec<String> = cell_re
            .captures_iter(row_html)
            .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
            .collect();
        if cells.len() < 4 {
            continue;
        }
        let name_html = cells.first().map(|s| s.as_str()).unwrap_or("");
        let path = href_re
            .captures(name_html)
            .and_then(|c| c.get(1).map(|m| m.as_str().trim().to_string()))
            .unwrap_or_default();
        if path.is_empty() || !path.to_ascii_lowercase().contains(".pdf") {
            continue;
        }
        let filer_name = efd_strip_cell(name_html);
        if filer_name.is_empty() {
            continue;
        }
        out.push(HouseClerkFilingHit {
            filer_name,
            office: efd_strip_cell(cells.get(1).map(|s| s.as_str()).unwrap_or("")),
            filing_year: efd_strip_cell(cells.get(2).map(|s| s.as_str()).unwrap_or("")),
            filing_type: efd_strip_cell(cells.get(3).map(|s| s.as_str()).unwrap_or("")),
            pdf_path: path,
        });
    }
    out
}

/// Pick latest **FD Original** (annual) for a uniquely matching filer.
/// `district` is optional House CD number for office disambiguation (e.g. 11 for CA11).
pub fn pick_house_clerk_fd_report(
    hits: &[HouseClerkFilingHit],
    candidate_name: &str,
    state: Option<&str>,
    district: Option<u32>,
) -> Option<HouseClerkFilingHit> {
    let st = state.unwrap_or("").trim().to_ascii_uppercase();
    let matched: Vec<&HouseClerkFilingHit> = hits
        .iter()
        .filter(|h| house_clerk_is_fd_original(&h.filing_type))
        .filter(|h| !h.pdf_path.is_empty())
        // financial-pdfs = annual FD; ptr-pdfs = transactions
        .filter(|h| {
            let p = h.pdf_path.to_ascii_lowercase();
            p.contains("financial-pdfs") || !p.contains("ptr-pdfs")
        })
        .filter(|h| efd_name_matches(&h.filer_name, candidate_name))
        .filter(|h| {
            if st.is_empty() {
                return true;
            }
            let ost = house_clerk_office_state(&h.office);
            ost.is_empty() || ost == st
        })
        .filter(|h| {
            let Some(want) = district else {
                return true;
            };
            match house_clerk_office_district(&h.office) {
                Some(d) => d == want,
                None => true,
            }
        })
        .collect();
    if matched.is_empty() {
        return None;
    }
    let mut people = std::collections::BTreeSet::new();
    for h in &matched {
        let key = h
            .filer_name
            .split(',')
            .next()
            .unwrap_or(&h.filer_name)
            .trim()
            .to_ascii_lowercase();
        people.insert(key);
    }
    if people.len() > 1 {
        return None;
    }
    matched
        .into_iter()
        .max_by(|a, b| {
            let ya = a.filing_year.trim().parse::<i32>().unwrap_or(0);
            let yb = b.filing_year.trim().parse::<i32>().unwrap_or(0);
            ya.cmp(&yb).then_with(|| a.pdf_path.cmp(&b.pdf_path))
        })
        .cloned()
}

fn house_clerk_holding_kind(code: &str) -> String {
    match code.trim().to_ascii_uppercase().as_str() {
        "ST" | "CS" | "PS" | "OP" => "stock".into(),
        "RP" => "property".into(),
        "OL" | "AB" => "business".into(),
        _ => "other".into(),
    }
}

fn house_clerk_scrub_pdf_text(text: &str) -> String {
    text.chars()
        .map(|c| if c == '\0' { ' ' } else { c })
        .collect::<String>()
}

/// Parse House Clerk FD PDF **text** (Schedule A assets) → holdings.
/// Skips PTR-style rows that embed trade dates. Image-only PDFs yield empty.
pub fn parse_house_clerk_fd_text(text: &str, source_url: &str) -> Vec<PersonalHolding> {
    let src = "House Clerk FD";
    let url = {
        let u = source_url.trim();
        if u.is_empty() {
            None
        } else {
            Some(house_clerk_abs_url(u))
        }
    };
    let scrubbed = house_clerk_scrub_pdf_text(text);
    // Collapse soft line breaks: join lines that continue a wrapped value range.
    let raw_lines: Vec<&str> = scrubbed
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    let mut lines: Vec<String> = Vec::new();
    let mut i = 0;
    while i < raw_lines.len() {
        let mut cur = raw_lines[i].to_string();
        // Merge "… $1,000,001 -" + "$25,000,000 …"
        while cur.contains('$')
            && cur.trim_end().ends_with('-')
            && i + 1 < raw_lines.len()
            && raw_lines[i + 1].starts_with('$')
        {
            i += 1;
            cur.push(' ');
            cur.push_str(raw_lines[i]);
        }
        // Merge asset name line + next line starting with [CODE]
        if !cur.contains('[')
            && i + 1 < raw_lines.len()
            && raw_lines[i + 1].starts_with('[')
            && raw_lines[i + 1].contains(']')
        {
            i += 1;
            cur.push(' ');
            cur.push_str(raw_lines[i]);
            while cur.contains('$')
                && cur.trim_end().ends_with('-')
                && i + 1 < raw_lines.len()
                && raw_lines[i + 1].starts_with('$')
            {
                i += 1;
                cur.push(' ');
                cur.push_str(raw_lines[i]);
            }
        }
        lines.push(cur);
        i += 1;
    }

    // Asset line: Name [CODE] OWNER? $range
    let asset_re = match regex::Regex::new(
        r"(?x)
        ^(?P<name>.+?)\s*
        \[(?P<code>[A-Za-z]{2})\]\s*
        (?:(?P<owner>SP|JT|DC|Self|Joint)\s+)?
        (?P<val>
            (?:None)
            |(?:Over\s+\$[\d,]+)
            |(?:\$[\d,]+(?:\s*-\s*\$[\d,]+)?)
        )
        ",
    ) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let date_re = regex::Regex::new(r"\b\d{1,2}/\d{1,2}/\d{4}\b").ok();
    let mut out = Vec::new();
    for line in &lines {
        let l = line.trim();
        if l.len() < 8 {
            continue;
        }
        let low = l.to_ascii_lowercase();
        if low.starts_with("asset owner")
            || low.starts_with("name:")
            || low.starts_with("status:")
            || low.starts_with("filing ")
            || low.starts_with("location:")
            || low.starts_with("description:")
            || low.starts_with("clerk of the house")
        {
            continue;
        }
        // PTR / transaction rows embed trade dates — not Schedule A snapshot.
        if date_re.as_ref().is_some_and(|re| re.is_match(l)) {
            continue;
        }
        let Some(caps) = asset_re.captures(l) else {
            continue;
        };
        let name = caps
            .name("name")
            .map(|m| m.as_str().trim())
            .unwrap_or("")
            .trim_matches(|c: char| c == '|' || c.is_control())
            .trim();
        // Drop leading row numbers ("11 Zinfandel…")
        let name = {
            let re = regex::Regex::new(r"^\d{1,3}\s+").ok();
            match re {
                Some(r) => r.replace(name, "").to_string(),
                None => name.to_string(),
            }
        };
        let name = name.trim();
        if name.is_empty() || name.len() < 2 {
            continue;
        }
        if name.eq_ignore_ascii_case("asset") {
            continue;
        }
        let code = caps
            .name("code")
            .map(|m| m.as_str())
            .unwrap_or("")
            .to_ascii_uppercase();
        let owner = caps
            .name("owner")
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        let mut val = caps
            .name("val")
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        // Skip empty/None values — not useful holdings display
        if val.is_empty() || val.eq_ignore_ascii_case("none") {
            continue;
        }
        // Normalize "Over $5,000,000"
        if val.to_ascii_lowercase().starts_with("over ") {
            val = val.replacen("Over ", "Over ", 1);
        }
        // Collapse whitespace in multi-line joined ranges
        let val = regex::Regex::new(r"\s+")
            .ok()
            .map(|re| re.replace_all(&val, " ").to_string())
            .unwrap_or(val);

        let kind = house_clerk_holding_kind(&code);
        let mut desc = format!("{name} [{code}]");
        if !owner.is_empty() {
            desc = format!("{desc} · owner: {owner}");
        }

        out.push(PersonalHolding {
            kind,
            description: desc,
            amount_display: Some(val),
            source: src.into(),
            source_url: url.clone(),
        });
    }
    let mut seen = std::collections::BTreeSet::new();
    out.retain(|h| seen.insert(h.description.clone()));
    if out.len() > 40 {
        out.truncate(40);
    }
    out
}

/// Extract text from House Clerk FD PDF bytes then parse Schedule A holdings.
pub fn parse_house_clerk_fd_pdf(bytes: &[u8], source_url: &str) -> Vec<PersonalHolding> {
    let text = match crate::states::north_carolina::extract_pdf_text(bytes) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    parse_house_clerk_fd_text(&text, source_url)
}

/// Measure sponsor/oppose committees → endorsement rows (financial backers as public signal).
/// `top_support` optional named donors/committees on the support side (cited).
pub fn endorsements_from_measure_sides(
    sponsor_name: &str,
    sponsor_url: Option<&str>,
    source: &str,
    oppose: &[(String, Option<String>)],
) -> Vec<Endorsement> {
    let mut out = Vec::new();
    let sn = sponsor_name.trim();
    if !sn.is_empty()
        && !sn.eq_ignore_ascii_case("unknown")
        && !sn.eq_ignore_ascii_case("support (ftm total)")
    {
        out.push(Endorsement {
            org: sn.into(),
            stance: "support".into(),
            source: source.into(),
            source_url: sponsor_url
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            kind: Some("committee".into()),
            trust: Some("filing".into()),
            date: None,
        });
    }
    for (name, url) in oppose {
        let n = name.trim();
        if n.is_empty() {
            continue;
        }
        out.push(Endorsement {
            org: n.into(),
            stance: "oppose".into(),
            source: source.into(),
            source_url: url
                .as_ref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            kind: Some("committee".into()),
            trust: Some("filing".into()),
            date: None,
        });
    }
    out
}

/// Scan cited bio facts for **explicit** dual/multi citizenship only. Never guess from birthplace.
pub fn citizenship_from_facts(facts: &[BioFact]) -> Option<CitizenshipRecord> {
    let re = regex::Regex::new(
        r"(?i)\b(dual\s+citiz(?:en|enship)|citizen\s+of\s+both|holds?\s+citizenship\s+in|naturalized\s+(?:u\.?s\.?\s+)?citizen|citizenship:\s*([a-z][a-z\s,]{2,80}))\b",
    )
    .ok()?;
    for f in facts {
        let t = f.text.trim();
        if t.is_empty() {
            continue;
        }
        if !re.is_match(t)
            && !t.to_ascii_lowercase().contains("dual national")
            && !t.to_ascii_lowercase().contains("multiple citizenship")
        {
            continue;
        }
        // Extract country-ish tokens after "citizen of"
        let mut countries = Vec::new();
        if let Ok(cre) = regex::Regex::new(
            r"(?i)(?:citizen(?:ship)?\s+of|citizenship\s+in)\s+([A-Za-z][A-Za-z\s]{1,40}?)(?:\s+and\s+([A-Za-z][A-Za-z\s]{1,40}))?(?:\.|,|$)",
        ) {
            if let Some(c) = cre.captures(t) {
                for i in 1..=2 {
                    if let Some(m) = c.get(i) {
                        let s = m.as_str().trim().trim_end_matches('.').to_string();
                        if !s.is_empty() {
                            countries.push(s);
                        }
                    }
                }
            }
        }
        if countries.is_empty() && t.to_ascii_lowercase().contains("dual") {
            countries.push("(see cited text)".into());
        }
        return Some(CitizenshipRecord {
            countries,
            disclosed: true,
            note: t.to_string(),
            source: Some(f.source.clone()),
            source_url: f.source_url.clone(),
        });
    }
    None
}

/// Apply citizenship from facts when not already disclosed.
pub fn apply_citizenship_from_facts(d: &mut PersonDossier) {
    if d.citizenship.disclosed {
        return;
    }
    if let Some(c) = citizenship_from_facts(&d.facts) {
        d.citizenship = c;
    }
}

/// Merge endorsements into dossier (dedupe stance+org).
pub fn merge_endorsements(d: &mut PersonDossier, extra: &[Endorsement]) {
    for e in extra {
        let dup = d.endorsements.iter().any(|x| {
            x.stance.eq_ignore_ascii_case(&e.stance) && x.org.eq_ignore_ascii_case(&e.org)
        });
        if !dup {
            d.endorsements.push(e.clone());
        }
    }
    if !d.endorsements.is_empty() {
        d.empty_notes
            .retain(|n| !n.to_ascii_lowercase().starts_with("endorsements:"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn year_parse() {
        assert_eq!(year_from_date("1970-05-01"), Some(1970));
        assert_eq!(year_from_date("1985"), Some(1985));
        assert_eq!(year_from_date("nope"), None);
        assert_eq!(year_from_date("+1971-05-28T00:00:00Z"), Some(1971));
    }

    #[test]
    fn parse_wikidata_entity_sample() {
        let entity = r#"{
          "id": "Q324546",
          "claims": {
            "P569": [{"mainsnak": {"datavalue": {"value": {"time": "+1971-05-28T00:00:00Z"}}}}],
            "P26": [{"mainsnak": {"datavalue": {"value": {"id": "Q20176884"}}}}],
            "P40": [{"mainsnak": {"datavalue": {"value": {"id": "Q131876433"}}}}],
            "P69": [{
              "mainsnak": {"datavalue": {"value": {"id": "Q501758"}}},
              "qualifiers": {
                "P512": [{"datavalue": {"value": {"id": "Q1765120"}}}],
                "P582": [{"datavalue": {"value": {"time": "+1993-01-01T00:00:00Z"}}}]
              }
            }],
            "P106": [
              {"mainsnak": {"datavalue": {"value": {"id": "Q40348"}}}},
              {"mainsnak": {"datavalue": {"value": {"id": "Q82955"}}}}
            ],
            "P39": [{
              "mainsnak": {"datavalue": {"value": {"id": "Q4416090"}}},
              "qualifiers": {
                "P580": [{"datavalue": {"value": {"time": "+2011-01-03T00:00:00Z"}}}]
              }
            }],
            "P27": [
              {"mainsnak": {"datavalue": {"value": {"id": "Q30"}}}},
              {"mainsnak": {"datavalue": {"value": {"id": "Q241"}}}}
            ]
          }
        }"#;
        let labels = r#"{
          "Q20176884": "Jeanette Dousdebes Rubio",
          "Q131876433": "Anthony Rubio",
          "Q501758": "University of Florida",
          "Q1765120": "Bachelor of Arts",
          "Q40348": "lawyer",
          "Q82955": "politician",
          "Q4416090": "United States senator",
          "Q30": "United States",
          "Q241": "Cuba"
        }"#;
        let p = parse_wikidata_entity_bio(entity, labels);
        assert_eq!(p.birth_year, Some(1971));
        assert!(p
            .facts
            .iter()
            .any(|f| f.kind == "family" && f.text.contains("Jeanette")));
        assert!(p
            .facts
            .iter()
            .any(|f| f.kind == "family" && f.text.contains("Anthony")));
        assert!(p
            .facts
            .iter()
            .any(|f| f.kind == "education" && f.text.contains("Florida")));
        assert!(p
            .facts
            .iter()
            .any(|f| f.kind == "legal" && f.text.contains("lawyer")));
        assert!(!p
            .facts
            .iter()
            .any(|f| f.text.to_ascii_lowercase().contains("politician")));
        assert!(p
            .spans
            .iter()
            .any(|s| s.category == "political" && s.label.contains("senator")));
        assert!(p
            .citizenship
            .as_ref()
            .is_some_and(|c| c.disclosed && c.countries.len() == 2));
        let ids = wikidata_label_ids_needed(entity);
        assert!(ids.iter().any(|id| id == "Q501758"));
        assert!(wikidata_entity_url("Q324546").unwrap().contains("Q324546"));
    }

    #[test]
    fn wikidata_us_only_citizenship_not_disclosed() {
        let entity = r#"{
          "id": "Q1",
          "claims": {
            "P27": [{"mainsnak": {"datavalue": {"value": {"id": "Q30"}}}}]
          }
        }"#;
        let labels = r#"{"Q30":"United States"}"#;
        let p = parse_wikidata_entity_bio(entity, labels);
        assert!(p.citizenship.is_none());
    }

    #[test]
    fn wikipedia_summary_photo_parse() {
        let url = wikipedia_summary_api_url("Gus Bilirakis").unwrap();
        assert!(url.contains("Gus_Bilirakis"));
        let json = r#"{
          "title": "Gus Bilirakis",
          "thumbnail": {"source": "https://upload.wikimedia.org/wikipedia/commons/thumb/x/x.jpg/330px-x.jpg"},
          "originalimage": {"source": "https://upload.wikimedia.org/wikipedia/commons/x/x.jpg"},
          "content_urls": {"desktop": {"page": "https://en.wikipedia.org/wiki/Gus_Bilirakis"}},
          "titles": {"canonical": "Gus_Bilirakis"}
        }"#;
        let (photo, page) = parse_wikipedia_summary_photo(json).unwrap();
        assert!(photo.ends_with("x.jpg"));
        assert!(page.contains("Gus_Bilirakis"));
        assert!(wikipedia_article_url("Paul Gosar")
            .unwrap()
            .contains("Paul_Gosar"));
    }

    #[test]
    fn wikipedia_summary_match_person_soft() {
        let hit = r#"{
          "type": "standard",
          "title": "Ron DeSantis",
          "titles": {"normalized": "Ron DeSantis"},
          "description": "American politician and governor of Florida",
          "extract": "Ronald Dion DeSantis is an American politician serving as the 46th governor of Florida since 2019."
        }"#;
        assert_eq!(
            wikipedia_summary_match_person(hit, "Ron DeSantis", Some("FL"), Some("Governor"))
                .as_deref(),
            Some("Ron DeSantis")
        );
        // Wrong person name
        assert!(
            wikipedia_summary_match_person(hit, "Jane Doe", Some("FL"), Some("Governor")).is_none()
        );
        // Disambiguation rejected
        let dis = r#"{
          "type": "disambiguation",
          "title": "John Smith",
          "titles": {"normalized": "John Smith"},
          "extract": "John Smith may refer to:"
        }"#;
        assert!(
            wikipedia_summary_match_person(dis, "John Smith", Some("FL"), Some("Mayor")).is_none()
        );
    }

    /// G5 smoke: FL federal (Bilirakis) + NY federal (Gillibrand) Wikidata fixtures.
    #[test]
    fn g5_smoke_wikidata_fl_and_ny_federal() {
        let fl = parse_wikidata_entity_bio(
            include_str!("testdata/g5/wikidata_bilirakis.json"),
            include_str!("testdata/g5/wikidata_bilirakis_labels.json"),
        );
        assert_eq!(fl.birth_year, Some(1963));
        assert!(
            fl.facts.iter().any(|f| f.kind == "education"),
            "FL education facts: {:?}",
            fl.facts
        );
        assert!(
            fl.spans.iter().any(|s| s.category == "political"),
            "FL political spans"
        );
        assert!(
            fl.spans
                .iter()
                .filter(|s| s.category == "political" && s.source == "Wikidata")
                .count()
                <= 10
        );
        // single-US citizenship stays undisclosed
        assert!(fl.citizenship.is_none());

        let ny = parse_wikidata_entity_bio(
            include_str!("testdata/g5/wikidata_gillibrand.json"),
            include_str!("testdata/g5/wikidata_gillibrand_labels.json"),
        );
        assert_eq!(ny.birth_year, Some(1966));
        assert!(ny.facts.iter().any(|f| f.kind == "education"));
        assert!(ny.spans.iter().any(|s| s.category == "political"));

        let fl_photo =
            parse_wikipedia_summary_photo(include_str!("testdata/g5/wiki_bilirakis_summary.json"));
        assert!(fl_photo.is_some(), "FL wiki photo");
        let ny_photo =
            parse_wikipedia_summary_photo(include_str!("testdata/g5/wiki_gillibrand_summary.json"));
        assert!(ny_photo.is_some(), "NY wiki photo");

        // Merge into dossier like enrich does
        let mut d = empty_dossier(2026);
        apply_member_bio_to_dossier(&mut d, &fl, 2026);
        assert!(d.career.birth_year.is_some());
        assert!(!d.facts.is_empty());
        if let Some((url, page)) = fl_photo {
            apply_photo_to_dossier(&mut d, Some(url), Some("Wikipedia".into()), Some(page));
        }
        assert!(d.photo_url.is_some());
    }

    #[test]
    fn career_politician_majority_adult_life() {
        // Born 1960 → adult from 1978; as_of 2026 → 48 adult years.
        // Political 2000–2026 = 27 years → ~56% → career.
        let spans = vec![CareerSpan::new(
            LifeCategory::Political,
            "U.S. House (FL-8)",
            Some(2000),
            Some(2026),
            "test",
            None,
        )];
        let a = assess_career(&spans, Some(1960), 2026);
        assert!(a.is_career_politician, "{:?}", a);
        assert_eq!(a.banner.as_deref(), Some("CAREER POLITICIAN"));
        assert!(a.blurb.contains("not a side job"));
        assert!(a.political_fraction.unwrap() >= 0.5);
    }

    #[test]
    fn not_career_when_fraction_low() {
        // Born 1980 → adult 2000; as_of 2026 → 26 adult yr; 2 years political.
        let spans = vec![CareerSpan::new(
            LifeCategory::Political,
            "State House",
            Some(2024),
            Some(2026),
            "test",
            None,
        )];
        let a = assess_career(&spans, Some(1980), 2026);
        assert!(!a.is_career_politician);
        assert!(a.banner.is_none());
        assert!(a.blurb.contains("Below the career-politician"));
    }

    #[test]
    fn bench_time_is_political() {
        let spans = vec![CareerSpan::new(
            LifeCategory::Political,
            "Circuit judge",
            Some(2005),
            Some(2026),
            "ballot",
            None,
        )];
        let a = assess_career(&spans, Some(1965), 2026);
        // adult = 2026-1965-18 = 43; political = 22 years → ~51%
        assert!(a.political_years >= 20.0);
        assert!(a.is_career_politician);
    }

    #[test]
    fn without_birth_long_service_labeled() {
        let spans = vec![CareerSpan::new(
            LifeCategory::Political,
            "Senate",
            Some(2000),
            Some(2026),
            "test",
            None,
        )];
        let a = assess_career(&spans, None, 2026);
        assert!(a.is_career_politician);
        assert!(a.blurb.contains("not a side job"));
    }

    #[test]
    fn overlapping_years_unioned() {
        let spans = vec![
            CareerSpan::new(
                LifeCategory::Political,
                "House",
                Some(2010),
                Some(2015),
                "a",
                None,
            ),
            CareerSpan::new(
                LifeCategory::Political,
                "Senate",
                Some(2012),
                Some(2018),
                "b",
                None,
            ),
        ];
        // 2010..=2018 = 9 years
        assert_eq!(years_covered(&spans, "political", 2026), 9.0);
    }

    #[test]
    fn cl_person_extract_and_assess() {
        let json = r#"[
          {
            "id": { "bioguide": "X1", "govtrack": 1, "fec": ["H8FL08042"] },
            "name": { "official_full": "Test Member" },
            "bio": { "birthday": "1955-03-01" },
            "terms": [
              {
                "type": "rep",
                "start": "1995-01-03",
                "end": "2027-01-03",
                "state": "FL",
                "district": 8,
                "party": "Republican"
              }
            ]
          }
        ]"#;
        let a = assess_from_congress_legislators(json, "H8FL08042", 2026).expect("assess");
        assert!(a.birth_year == Some(1955));
        assert!(a.political_years >= 20.0);
        assert!(a.is_career_politician);
        assert!(a.spans.iter().any(|s| s.label.contains("FL-8")));
    }

    #[test]
    fn openstates_roles_to_spans() {
        let person = r#"{
          "id": "ocd-person/abc",
          "name": "Jane Doe",
          "image": "https://example.com/jane.jpg",
          "birth_date": "1970-01-01",
          "roles": [
            {
              "type": "upper",
              "title": "Senator",
              "org_classification": "upper",
              "district": "10",
              "start_date": "2016-11-08",
              "end_date": "2024-11-05"
            }
          ]
        }"#;
        let (career, photo, src) = assess_from_openstates_person_json(person, 2026).expect("os");
        assert_eq!(photo.as_deref(), Some("https://example.com/jane.jpg"));
        assert_eq!(src.as_deref(), Some("Open States"));
        assert!(career.political_years >= 8.0);
        assert!(!career.spans.is_empty());
    }

    #[test]
    fn ie_support_endorsement() {
        let e = endorsements_from_ie_support(
            "AMERICANS FOR EXAMPLE",
            "Support",
            "https://www.fec.gov/data/committee/C001/",
        )
        .unwrap();
        assert_eq!(e.stance, "support");
        assert!(e.org.contains("EXAMPLE"));
    }

    #[test]
    fn empty_dossier_honest() {
        let d = empty_dossier(2026);
        assert!(!d.career.is_career_politician);
        assert!(!d.empty_notes.is_empty());
        assert!(!d.citizenship.disclosed);
    }

    #[test]
    fn unitedstates_photo_url() {
        let u = unitedstates_congress_photo_url("R000609").unwrap();
        assert!(u.contains("unitedstates.github.io"));
        assert!(u.contains("R000609.jpg"));
        assert!(unitedstates_congress_photo_url("").is_none());
        assert!(unitedstates_congress_photo_url("../x").is_none());
    }

    #[test]
    fn measure_side_endorsements() {
        let e = endorsements_from_measure_sides(
            "Yes on 4 Committee",
            Some("https://example.com/yes"),
            "FL DOS TreFin",
            &[("No on 4 PAC".into(), Some("https://example.com/no".into()))],
        );
        assert_eq!(e.len(), 2);
        assert_eq!(e[0].stance, "support");
        assert_eq!(e[1].stance, "oppose");
    }

    #[test]
    fn citizenship_only_when_explicit() {
        let facts = vec![
            BioFact {
                kind: "other".into(),
                text: "Born: in Fort Lauderdale, Florida".into(),
                source: "FL Senate".into(),
                source_url: None,
                ..Default::default()
            },
            BioFact {
                kind: "other".into(),
                text: "Dual citizen of the United States and Canada".into(),
                source: "chamber bio".into(),
                source_url: Some("https://example.com".into()),
                ..Default::default()
            },
        ];
        assert!(citizenship_from_facts(&facts[..1]).is_none());
        let c = citizenship_from_facts(&facts).unwrap();
        assert!(c.disclosed);
        assert!(c
            .countries
            .iter()
            .any(|x| x.contains("United") || x.contains("Canada") || x.contains("see cited")));
    }

    #[test]
    fn federal_disclosure_portals_house() {
        let p = federal_disclosure_portals(Some("us_house"));
        assert!(p.iter().any(|x| x.url.contains("house.gov")));
    }

    #[test]
    fn efd_search_pick_latest_annual_and_parse_assets() {
        // Legacy layout fixture
        let json = include_str!("../../../testdata/efd_search_tillis_sample.json");
        let hits = parse_efd_search_data_json(json).expect("search json");
        assert_eq!(hits.len(), 3);
        assert!(hits[0].report_path.contains("/search/view/annual/"));

        let pick = pick_efd_annual_report(&hits, "Thomas Tillis", Some("NC")).expect("pick");
        assert!(pick.date_filed.contains("2021"));
        assert!(pick.report_path.contains("008b142d"));

        // PTR-only should not win
        let ptr_only: Vec<EfdReportHit> = hits
            .iter()
            .filter(|h| h.report_type.to_ascii_lowercase().contains("periodic"))
            .cloned()
            .collect();
        assert!(pick_efd_annual_report(&ptr_only, "Thomas Tillis", Some("NC")).is_none());

        // Ambiguous two different last names → None
        let mut amb = hits.clone();
        amb.push(EfdReportHit {
            filer_name: "Smith, Thomas (R)".into(),
            office: "Senator".into(),
            state: "North Carolina".into(),
            report_type: "Annual".into(),
            date_filed: "05/01/2021".into(),
            report_path: "/search/view/annual/zzzzzzzz-zzzz-zzzz-zzzz-zzzzzzzzzzzz/".into(),
        });
        // name match is Tillis-only so still unique
        assert!(pick_efd_annual_report(&amb, "Thomas Tillis", Some("NC")).is_some());

        let html = include_str!("../../../testdata/efd_annual_tillis_2020_sample.html");
        let url =
            "https://efdsearch.senate.gov/search/view/annual/008b142d-54f3-4e9b-8aca-f378eb9b9cf0/";
        let holds = parse_senate_efd_annual_html(html, url);
        assert!(
            holds.len() >= 3,
            "expected several Part 3 assets, got {}",
            holds.len()
        );
        assert!(holds.iter().any(|h| h.kind == "stock"));
        assert!(holds.iter().all(|h| h.source == "Senate eFD"));
        assert!(holds.iter().all(|h| {
            h.source_url
                .as_deref()
                .is_some_and(|u| u.contains("efdsearch.senate.gov"))
        }));
        assert!(holds.iter().any(|h| {
            h.description.to_ascii_lowercase().contains("ibm")
                && h.amount_display.as_deref().is_some_and(|a| a.contains('$'))
        }));
        // Container rows with "--" value must not appear
        assert!(!holds.iter().any(|h| {
            h.amount_display
                .as_deref()
                .is_some_and(|a| a.trim() == "--")
        }));

        let mut d = empty_dossier(2026);
        apply_holdings_to_dossier(&mut d, holds.clone());
        assert_eq!(d.holdings.len(), holds.len());
        assert!(!d
            .empty_notes
            .iter()
            .any(|n| n.to_ascii_lowercase().contains("personal holdings")));
    }

    #[test]
    fn efd_new_filer_moody_is_holdings_report() {
        let json = include_str!("../../../testdata/efd_search_moody_sample.json");
        let hits = parse_efd_search_data_json(json).expect("moody search");
        assert_eq!(hits.len(), 1);
        assert!(
            hits[0]
                .report_type
                .to_ascii_lowercase()
                .contains("new filer"),
            "type={}",
            hits[0].report_type
        );
        assert!(hits[0].report_path.contains("/view/annual/"));
        let pick = pick_efd_annual_report(&hits, "Ashley Moody", Some("FL")).expect("pick moody");
        assert!(pick.report_path.contains("2a8e3a84"));
        // Must not reject New Filer as non-annual
        assert!(efd_is_holdings_report(&pick.report_type, &pick.report_path));

        let html = include_str!("../../../testdata/efd_annual_moody_new_filer_sample.html");
        let url = efd_abs_url(&pick.report_path);
        let holds = parse_senate_efd_annual_html(html, &url);
        assert!(
            holds
                .iter()
                .any(|h| h.amount_display.as_deref().is_some_and(|a| a.contains('$'))),
            "expected $ assets from New Filer Part 3, got {}",
            holds.len()
        );
        assert!(!holds.iter().any(|h| {
            h.amount_display
                .as_deref()
                .is_some_and(|a| a.trim() == "--")
        }));
    }

    #[test]
    fn efd_live_layout_gillibrand_search_and_annual() {
        // Live DataTables row shape (probed 2026-08-08)
        let json = include_str!("../../../testdata/efd_search_gillibrand_sample.json");
        let hits = parse_efd_search_data_json(json).expect("gillibrand search");
        assert_eq!(hits.len(), 5);
        assert!(hits[0]
            .filer_name
            .to_ascii_lowercase()
            .contains("gillibrand"));
        assert!(hits[0].report_path.contains("/search/view/annual/2beb582f"));
        assert!(
            efd_is_holdings_report(&hits[0].report_type, &hits[0].report_path),
            "report_type={}",
            hits[0].report_type
        );
        assert!(hits[0].office.to_ascii_lowercase().contains("senator"));

        let pick = pick_efd_annual_report(&hits, "GILLIBRAND, KIRSTEN E", Some("NY"))
            .expect("pick gillibrand");
        assert!(pick.report_path.contains("2beb582f"));
        assert!(pick.date_filed.contains("2026"));

        let html = include_str!("../../../testdata/efd_annual_gillibrand_2025_sample.html");
        let url =
            "https://efdsearch.senate.gov/search/view/annual/2beb582f-859a-4e8c-98fb-b0887f52a494/";
        let holds = parse_senate_efd_annual_html(html, url);
        assert!(
            holds.len() >= 3,
            "expected Part 3 bank deposits, got {}",
            holds.len()
        );
        assert!(holds.iter().all(|h| h.source == "Senate eFD"));
        assert!(holds.iter().any(|h| {
            h.description.to_ascii_lowercase().contains("citi")
                && h.amount_display
                    .as_deref()
                    .is_some_and(|a| a.contains("$250,001"))
        }));
        assert!(holds.iter().any(|h| {
            h.description
                .to_ascii_lowercase()
                .contains("senate federal credit")
        }));
    }

    #[test]
    fn efd_name_split_fec_style() {
        let (f, l) = efd_split_person_name("GILLIBRAND, KIRSTEN E");
        assert_eq!(l.to_ascii_uppercase(), "GILLIBRAND");
        assert!(f.to_ascii_uppercase().starts_with('K'));
        assert!(efd_name_matches(
            "Gillibrand, Kirsten E.",
            "GILLIBRAND, KIRSTEN E"
        ));
        assert!(!efd_name_matches("Tillis, Thomas", "GILLIBRAND, KIRSTEN E"));
    }

    #[test]
    fn house_clerk_search_pick_and_parse_pelosi_text() {
        let html = include_str!("../../../testdata/house_clerk_search_pelosi_2025.html");
        let hits = parse_house_clerk_search_html(html);
        assert!(
            hits.iter().any(|h| h.filing_type.contains("FD")),
            "expected FD rows, got {:?}",
            hits.iter().map(|h| &h.filing_type).collect::<Vec<_>>()
        );
        let pick = pick_house_clerk_fd_report(&hits, "PELOSI, NANCY", Some("CA"), Some(11))
            .expect("pick pelosi FD");
        assert!(pick.pdf_path.contains("financial-pdfs"));
        assert!(pick.pdf_path.ends_with(".pdf"));
        assert_eq!(pick.filing_year, "2025");
        // PTR must not win
        assert!(house_clerk_is_fd_original(&pick.filing_type));

        let bil = include_str!("../../../testdata/house_clerk_search_bilirakis_2025.html");
        let bil_hits = parse_house_clerk_search_html(bil);
        let bil_pick =
            pick_house_clerk_fd_report(&bil_hits, "BILIRAKIS, GUS M", Some("FL"), Some(12))
                .expect("bilirakis");
        assert!(bil_pick.pdf_path.contains("9116162"));

        let text = include_str!("../../../testdata/house_clerk_fd_pelosi_2025.txt");
        let url = house_clerk_abs_url(&pick.pdf_path);
        let holds = parse_house_clerk_fd_text(text, &url);
        assert!(
            holds.len() >= 10,
            "expected many Schedule A assets, got {}",
            holds.len()
        );
        assert!(holds.iter().all(|h| h.source == "House Clerk FD"));
        assert!(holds.iter().all(|h| {
            h.source_url
                .as_deref()
                .is_some_and(|u| u.contains("disclosures-clerk.house.gov"))
        }));
        assert!(holds.iter().any(|h| h.kind == "stock"
            && h.description.to_ascii_lowercase().contains("apple")
            && h.amount_display.as_deref().is_some_and(|a| a.contains('$'))));
        assert!(holds
            .iter()
            .any(|h| h.kind == "property" && h.description.to_ascii_lowercase().contains("lobos")));
        // No PTR trade-date rows
        assert!(!holds.iter().any(|h| {
            h.description.contains("01/14/2025") || h.description.contains("12/24/2025")
        }));

        let pdf = include_bytes!("../../../testdata/house_clerk_fd_pelosi_2025.pdf");
        let from_pdf = parse_house_clerk_fd_pdf(pdf, &url);
        assert!(
            from_pdf.len() >= 10,
            "pdf-extract path should yield holdings, got {}",
            from_pdf.len()
        );
    }

    #[test]
    fn parse_fl_house_member_minimal() {
        let html = r#"
          <html><body>
          <img class="member-photo" src="/images/members/rep123.jpg" alt="Representative Jane Doe" />
          <tr><td>Occupation:</td><td>Attorney</td></tr>
          <tr><td>Education:</td><td>UF, J.D., 2001</td></tr>
          <tr><td>Spouse:</td><td>John Doe</td></tr>
          </body></html>
        "#;
        let p = parse_fl_house_member_html(
            html,
            "https://www.flhouse.gov/Sections/Representatives/details.aspx?MemberId=1",
        );
        assert!(p.photo_url.as_deref().unwrap_or("").contains("rep123.jpg"));
        assert!(p.facts.iter().any(|f| f.kind == "work"));
        assert!(p.facts.iter().any(|f| f.kind == "family"));
    }

    #[test]
    fn parse_fl_senate_member_sample() {
        let html = r#"
            <img src="/PublishedContent/Senators/2024-2026/Photos/s17_5256.jpg" width="185" alt='Senator Smith'>
            <h4>Legislative Service</h4>
            <ul class="fls_list_bulleted">
              <li>Elected to the Senate in 2024</li>
              <li>House of Representatives, 2016-2022</li>
            </ul>
            <h4>Affiliations</h4>
            <ul>
              <li>Legislative Assistant, Florida State Representative Joe Saunders, 2012-2014</li>
              <li>Men's Wearhouse and Tux, Store Manager, 2006-2010</li>
            </ul>
            <h4>Biographical Information</h4>
            <table><tbody>
              <tr><td scope="row" class="bold">Occupation: </td><td>Non-profit management</td></tr>
              <tr><td scope="row" class="bold">Spouse: </td><td>Alex Example</td></tr>
              <tr><td scope="row" class="bold">Education: </td>
                <td><ul class="noBullet"><li>University of Central Florida, B.S., Business Administration, 2003</li></ul></td>
              </tr>
              <tr><td scope="row" class="bold">Born: </td><td> in Fort Lauderdale, Florida</td></tr>
            </tbody></table>
        "#;
        let page = "https://www.flsenate.gov/Senators/2024-2026/S17";
        let p = parse_fl_senate_member_html(html, page);
        assert!(p
            .photo_url
            .as_deref()
            .unwrap_or("")
            .contains("Photos/s17_5256.jpg"));
        assert!(p
            .facts
            .iter()
            .any(|f| f.kind == "family" && f.text.contains("Alex")));
        assert!(p
            .facts
            .iter()
            .any(|f| f.kind == "education" && f.text.contains("Central Florida")));
        assert!(p
            .facts
            .iter()
            .any(|f| f.kind == "work" && f.text.contains("Non-profit")));
        assert!(p
            .spans
            .iter()
            .any(|s| s.category == "political" && s.label.contains("2016")));
        assert!(p
            .spans
            .iter()
            .any(|s| s.category == "political" && s.label.contains("Legislative Assistant")));
        assert!(p
            .spans
            .iter()
            .any(|s| s.category == "business" || s.category == "work"));

        let mut d = empty_dossier(2026);
        apply_member_bio_to_dossier(&mut d, &p, 2026);
        assert!(d.photo_url.is_some());
        assert!(!d.facts.is_empty());
        assert!(d.career.political_years > 0.0);
    }

    #[test]
    fn ballotpedia_url_helper() {
        assert_eq!(
            ballotpedia_member_url("Gus M. Bilirakis").as_deref(),
            Some("https://ballotpedia.org/Gus_M._Bilirakis")
        );
    }

    #[test]
    fn parse_official_about_house_and_senate_fixtures() {
        let house = include_str!("../../../testdata/official_about_bilirakis_house.html");
        let hp = parse_official_member_about_html(house, "https://bilirakis.house.gov/about");
        assert!(
            hp.facts
                .iter()
                .any(|f| f.source.contains("House") || f.source.contains("official")),
            "official source label"
        );
        assert!(
            hp.facts
                .iter()
                .any(|f| f.kind == "other" && f.text.contains("Florida")),
            "house about blurb"
        );

        let sen = include_str!("../../../testdata/official_about_gillibrand_senate.html");
        let sp = parse_official_member_about_html(sen, "https://www.gillibrand.senate.gov/about/");
        assert!(sp.facts.iter().any(|f| f.source.contains("Senate")));
        assert!(
            sp.facts
                .iter()
                .any(|f| f.kind == "family" && f.text.contains("Jonathan")),
            "spouse from official about: {:?}",
            sp.facts
        );
        assert!(
            sp.facts.iter().any(
                |f| f.kind == "family" && (f.text.contains("Theo") || f.text.contains("Henry"))
            ),
            "children from official about"
        );

        let mut d = empty_dossier(2026);
        apply_member_bio_to_dossier(&mut d, &sp, 2026);
        assert!(d.facts.iter().any(|f| f.kind == "family"));
    }

    #[test]
    fn dbpedia_ntriples_parse_and_fill_gaps() {
        assert!(dbpedia_ntriples_url("Gus Bilirakis")
            .unwrap()
            .contains("Gus_Bilirakis.ntriples"));
        assert!(dbpedia_describe_ntriples_url("Gus Bilirakis")
            .unwrap()
            .contains("DESCRIBE"));

        let fl = parse_dbpedia_ntriples(
            include_str!("testdata/g5/dbpedia_bilirakis.nt"),
            "Gus Bilirakis",
        );
        assert!(
            fl.facts
                .iter()
                .any(|f| f.kind == "family" && f.text.contains("Eva")),
            "spouse: {:?}",
            fl.facts
        );
        assert!(fl.facts.iter().any(|f| f.kind == "education"));
        assert_eq!(fl.birth_year, Some(1963));
        assert!(fl.facts.iter().any(|f| f.text.contains("Children")));
        assert!(fl.facts.iter().all(|f| f.source == "DBpedia"));

        let ny = parse_dbpedia_ntriples(
            include_str!("testdata/g5/dbpedia_gillibrand.nt"),
            "Kirsten Gillibrand",
        );
        assert!(ny
            .facts
            .iter()
            .any(|f| f.kind == "family" && f.text.contains("Jonathan")));
        assert!(ny
            .facts
            .iter()
            .any(|f| f.kind == "education" && f.text.contains("Dartmouth")));
        assert_eq!(ny.birth_year, Some(1966));

        // Gap-only: existing education keeps BP; spouse fills.
        let mut d = empty_dossier(2026);
        d.facts.push(BioFact {
            kind: "education".into(),
            text: "Bachelor's: University of Florida, 1986".into(),
            source: "Ballotpedia".into(),
            source_url: None,
            ..Default::default()
        });
        apply_member_bio_fill_gaps(&mut d, &fl, 2026);
        assert_eq!(d.facts.iter().filter(|f| f.kind == "education").count(), 1);
        assert!(d
            .facts
            .iter()
            .any(|f| f.kind == "family" && f.text.contains("Eva")));
    }

    #[test]
    fn wikipedia_extract_url_and_plain_parse() {
        let url = wikipedia_extract_api_url("Gus Bilirakis").unwrap();
        assert!(url.contains("prop=extracts"));
        assert!(url.contains("origin=*"));
        assert!(url.contains("Gus_Bilirakis"));

        let text = "Gus Michael Bilirakis (born February 8, 1963) is an American lawyer and politician \
            serving as the U.S. representative for Florida's 12th congressional district. \
            He succeeded his father Michael Bilirakis. Bilirakis was born in Gainesville, Florida, \
            and grew up in Tarpon Springs, the son of Evelyn (née Miaoulis) and Michael Bilirakis. \
            Bilirakis graduated from Tarpon Springs High School. He then attended the University of Florida, \
            where he graduated in 1986 with a bachelor's degree. He received his J.D. degree from the \
            Stetson University College of Law in 1989.";
        let page = "https://en.wikipedia.org/wiki/Gus_Bilirakis";
        let p = parse_wikipedia_plain_extract(text, page);
        assert_eq!(p.birth_year, Some(1963));
        assert!(p.facts.iter().any(|f| f.text.contains("1963")));
        assert!(p.facts.iter().any(|f| f.text.contains("Gainesville")));
        assert!(p
            .facts
            .iter()
            .any(|f| f.kind == "family" && f.text.contains("Michael")));
        assert!(p
            .facts
            .iter()
            .any(|f| f.kind == "education" && f.text.contains("Florida")));
        assert!(p
            .facts
            .iter()
            .any(|f| f.kind == "education" && f.text.contains("Stetson")));
        assert!(p
            .facts
            .iter()
            .any(|f| matches!(f.kind.as_str(), "legal" | "work") && f.text.contains("Lawyer")));
        assert!(p.facts.iter().all(|f| f.source == "Wikipedia"));
    }

    #[test]
    fn wikipedia_extract_json_fixtures_and_fill_gaps() {
        let fl =
            parse_wikipedia_extract_json(include_str!("testdata/g5/wiki_extract_bilirakis.json"));
        assert_eq!(fl.birth_year, Some(1963));
        assert!(fl
            .facts
            .iter()
            .any(|f| f.kind == "education" || f.text.contains("Florida")));
        assert!(fl.facts.iter().any(|f| f.kind == "family"));

        let ny =
            parse_wikipedia_extract_json(include_str!("testdata/g5/wiki_extract_gillibrand.json"));
        assert_eq!(ny.birth_year, Some(1966));
        assert!(ny
            .facts
            .iter()
            .any(|f| f.kind == "education" && f.text.contains("Dartmouth")));
        assert!(ny.facts.iter().any(|f| f.kind == "family"
            && f.text.to_ascii_lowercase().contains("née")
            || f.text.contains("Rutnik")
            || f.kind == "family"));

        // Fill gaps: BP already has education → wiki education skipped; family fills.
        let mut d = empty_dossier(2026);
        d.facts.push(BioFact {
            kind: "education".into(),
            text: "Bachelor's: University of Florida, 1986".into(),
            source: "Ballotpedia".into(),
            source_url: Some("https://ballotpedia.org/Gus_M._Bilirakis".into()),
            ..Default::default()
        });
        apply_member_bio_fill_gaps(&mut d, &fl, 2026);
        assert!(d.facts.iter().any(|f| f.kind == "family"));
        assert_eq!(
            d.facts.iter().filter(|f| f.kind == "education").count(),
            1,
            "must not clobber BP education"
        );
        assert_eq!(d.career.birth_year, Some(1963));
    }

    #[test]
    fn parse_ballotpedia_member_minimal() {
        let html = r#"
          <div class="infobox person">
            <div class="widget-row"><img src="https://s3.amazonaws.com/ballotpedia-api4/files/thumbs/200/300/Example.jpg" class="widget-img" /></div>
            <div class="widget-row value-only Republican">Education</div>
            <div class="widget-row"><div class="widget-key">Bachelor&#39;s</div><div class="widget-value">University of Florida, 1986</div></div>
            <div class="widget-row"><div class="widget-key">Bachelor&#39;s</div><div class="widget-value">University of Florida</div></div>
            <div class="widget-row"><div class="widget-key">Law</div><div class="widget-value">Stetson Law School, 1989</div></div>
            <div class="widget-row value-only Republican">Personal</div>
            <div class="widget-row"><div class="widget-key">Birthplace</div><div class="widget-value">Gainesville, FL</div></div>
            <div class="widget-row"><div class="widget-key">Religion</div><div class="widget-value">Greek Orthodox</div></div>
            <div class="widget-row"><div class="widget-key">Profession</div><div class="widget-value">Attorney</div></div>
            <div class="widget-row"><div class="widget-key">Spouse</div><div class="widget-value">Eva Example</div></div>
            <div class="widget-row value-only Republican">U.S. House Florida District 12</div>
            <div style="font-weight: bold;text-align:center;">Tenure</div>
            <div style="text-align:center;">2013 - Present</div>
            <div class="widget-row value-only Republican">Prior offices</div>
            <div style="font-weight: bold; text-align: center;">Florida House of Representatives District 48</div>
            <div style="font-size: 13px; text-align:center;">Years in office: 1999 - 2007</div>
          </div>
          <h2><span class="mw-headline" id="Biography">Biography</span></h2>
          <p>Example Person was born in Gainesville, Florida. He earned a bachelor&#39;s degree from the University of Florida in 1986 and a law degree from Stetson Law School in 1989. He owned a law practice focused on estate planning.</p>
        "#;
        let page = "https://ballotpedia.org/Gus_M._Bilirakis";
        let p = parse_ballotpedia_member_html(html, page);
        assert!(p.photo_url.as_deref().unwrap_or("").contains("Example.jpg"));
        assert!(p
            .facts
            .iter()
            .any(|f| f.kind == "education" && f.text.contains("1986")));
        assert!(
            !p.facts
                .iter()
                .any(|f| f.kind == "education" && f.text == "Bachelor's: University of Florida"),
            "bare school should lose to year-bearing row"
        );
        assert!(p
            .facts
            .iter()
            .any(|f| f.kind == "work" && f.text.contains("Attorney")));
        assert!(p
            .facts
            .iter()
            .any(|f| f.kind == "family" && f.text.contains("Eva")));
        assert!(p.facts.iter().any(|f| f.text.contains("Greek Orthodox")));
        assert!(p.facts.iter().any(|f| f.text.contains("Gainesville")));
        assert!(p
            .facts
            .iter()
            .any(|f| f.kind == "other" && f.text.contains("estate planning")));
        assert!(p.spans.iter().any(|s| {
            s.category == "political"
                && s.label.contains("Florida House")
                && s.start_year == Some(1999)
        }));
        assert!(p.spans.iter().any(|s| {
            s.category == "political"
                && s.label.contains("U.S. House")
                && s.start_year == Some(2013)
        }));
        assert!(p.facts.iter().all(|f| f.source == "Ballotpedia"));

        let mut d = empty_dossier(2026);
        apply_member_bio_to_dossier(&mut d, &p, 2026);
        assert!(d.photo_url.is_some());
        assert!(d.facts.iter().any(|f| f.kind == "education"));
    }

    #[test]
    fn parse_ballotpedia_bilirakis_fixture() {
        let html = include_str!("../../../testdata/ballotpedia_bilirakis_sample.html");
        let page = "https://ballotpedia.org/Gus_M._Bilirakis";
        let p = parse_ballotpedia_member_html(html, page);
        assert!(p.photo_url.as_deref().unwrap_or("").contains("Bilirakis"));
        assert!(p
            .facts
            .iter()
            .any(|f| f.kind == "education" && f.text.contains("Florida")));
        assert!(p
            .facts
            .iter()
            .any(|f| f.kind == "work" && f.text.contains("Attorney")));
        assert!(p.facts.iter().any(|f| f.text.contains("Greek Orthodox")));
        assert!(p.facts.iter().any(|f| f.text.contains("Gainesville")));
    }

    #[test]
    fn grokipedia_typeahead_unique_and_ambiguous() {
        assert!(grokipedia_typeahead_url("Gus Bilirakis")
            .unwrap()
            .contains("typeahead?query=Gus"));
        assert_eq!(
            grokipedia_page_url("Gus_Bilirakis").as_deref(),
            Some("https://grokipedia.com/page/Gus_Bilirakis")
        );
        assert!(grokipedia_page_url("bad slug").is_none());

        let hit = match_grokipedia_typeahead(
            include_str!("testdata/g5/grokipedia_typeahead_bilirakis.json"),
            "Gus Bilirakis",
        )
        .expect("unique Gus");
        assert_eq!(hit.slug, "Gus_Bilirakis");
        assert!(hit.page_url.contains("Gus_Bilirakis"));

        let g = match_grokipedia_typeahead(
            include_str!("testdata/g5/grokipedia_typeahead_gillibrand.json"),
            "Kirsten Gillibrand",
        )
        .expect("unique Kirsten");
        assert_eq!(g.slug, "Kirsten_Gillibrand");

        // Last-name-only typeahead returns Gus + Michael → skip.
        assert!(match_grokipedia_typeahead(
            include_str!("testdata/g5/grokipedia_typeahead_bilirakis_ambig.json"),
            "Gus Bilirakis",
        )
        .is_some()); // given name still unique among results
        assert!(match_grokipedia_typeahead(
            include_str!("testdata/g5/grokipedia_typeahead_bilirakis_ambig.json"),
            "Bilirakis",
        )
        .is_none());
        assert!(match_grokipedia_typeahead(
            include_str!("testdata/g5/grokipedia_typeahead_smith_ambig.json"),
            "John Smith",
        )
        .is_none());
    }

    #[test]
    fn grokipedia_page_parse_no_family_citizenship() {
        let fl = parse_grokipedia_page_html(
            include_str!("testdata/g5/grokipedia_bilirakis.html"),
            "https://grokipedia.com/page/Gus_Bilirakis",
        );
        assert_eq!(fl.birth_year, Some(1963));
        assert!(fl.facts.iter().any(|f| f.text.contains("Gainesville")));
        assert!(
            fl.facts
                .iter()
                .any(|f| f.kind == "education" && f.text.contains("Florida")),
            "edu: {:?}",
            fl.facts
        );
        assert!(
            fl.facts
                .iter()
                .any(|f| f.kind == "education" && f.text.contains("Stetson")),
            "stetson: {:?}",
            fl.facts
        );
        assert!(fl
            .facts
            .iter()
            .any(|f| matches!(f.kind.as_str(), "legal" | "work") && f.text.contains("Lawyer")));
        assert!(
            fl.facts.iter().all(|f| f.kind != "family"),
            "must not emit family from Grokipedia: {:?}",
            fl.facts
        );
        assert!(fl.citizenship.is_none());
        assert!(fl.facts.iter().all(|f| f.source == "Grokipedia"));
        assert!(fl
            .facts
            .iter()
            .all(|f| f.source_url.as_deref() == Some("https://grokipedia.com/page/Gus_Bilirakis")));

        let ny = parse_grokipedia_page_html(
            include_str!("testdata/g5/grokipedia_gillibrand.html"),
            "https://grokipedia.com/page/Kirsten_Gillibrand",
        );
        assert_eq!(ny.birth_year, Some(1966));
        assert!(ny
            .facts
            .iter()
            .any(|f| f.kind == "education" && f.text.contains("Dartmouth")));
        assert!(ny.facts.iter().all(|f| f.kind != "family"));
        assert!(ny.citizenship.is_none());

        // Gap-only merge: existing education kept; birth fills.
        let mut d = empty_dossier(2026);
        d.facts.push(BioFact {
            kind: "education".into(),
            text: "Bachelor's: University of Florida, 1986".into(),
            source: "Ballotpedia".into(),
            source_url: None,
            ..Default::default()
        });
        apply_member_bio_fill_gaps(&mut d, &fl, 2026);
        assert_eq!(d.facts.iter().filter(|f| f.kind == "education").count(), 1);
        assert_eq!(d.career.birth_year, Some(1963));
        assert!(d
            .facts
            .iter()
            .any(|f| f.text.contains("Lawyer") || f.text.contains("Born")));
    }

    #[test]
    fn fact_years_feed_career_fractions() {
        let mut d = empty_dossier(2026);
        d.career = assess_career(
            &[CareerSpan::new(
                LifeCategory::Political,
                "U.S. House",
                Some(2007),
                None,
                "CL",
                None,
            )],
            Some(1963),
            2026,
        );
        let bio = MemberBioParse {
            birth_year: Some(1963),
            facts: vec![
                BioFact {
                    kind: "education".into(),
                    text: "Bachelor's: University of Florida, 1986".into(),
                    source: "Ballotpedia".into(),
                    source_url: Some("https://ballotpedia.org/Gus_M._Bilirakis".into()),
                    ..Default::default()
                },
                BioFact {
                    kind: "education".into(),
                    text: "Law: Stetson Law School, 1989".into(),
                    source: "Ballotpedia".into(),
                    source_url: None,
                    ..Default::default()
                },
                BioFact {
                    kind: "legal".into(),
                    text: "Profession: Attorney".into(),
                    source: "Ballotpedia".into(),
                    source_url: None,
                    ..Default::default()
                },
            ],
            spans: vec![],
            ..Default::default()
        };
        apply_member_bio_to_dossier(&mut d, &bio, 2026);
        let edu = d
            .career
            .fractions
            .iter()
            .find(|f| f.category == "education")
            .expect("edu fraction");
        assert!(
            edu.years >= 2.0,
            "education years from fact dates: {:?}",
            edu
        );
        assert!(d.career.spans.iter().any(|s| {
            s.category == "education" && s.label.contains("Florida") && s.start_year == Some(1986)
        }));
        // Gap-fill with birth-only still keeps edu spans.
        let mut d2 = d.clone();
        apply_member_bio_fill_gaps(
            &mut d2,
            &MemberBioParse {
                birth_year: Some(1963),
                ..Default::default()
            },
            2026,
        );
        assert!(d2
            .career
            .fractions
            .iter()
            .any(|f| f.category == "education" && f.years >= 2.0));
    }

    #[test]
    fn ballotpedia_challenger_titles_and_match() {
        let t = ballotpedia_title_candidates(
            "Joe Gruters",
            Some("FL"),
            Some("state_senate"),
            Some("Florida State Senate District 22"),
        );
        assert!(t.iter().any(|x| x == "Joe Gruters"));
        assert!(t.iter().any(|x| x == "Joe Gruters (Florida)"));
        assert!(t.len() <= 8);

        let fed = ballotpedia_title_candidates(
            r#"Gus M. Bilirakis"#,
            Some("FL"),
            Some("house"),
            Some("U.S. House Florida District 12"),
        );
        assert!(fed.iter().any(|x| x == "Gus M. Bilirakis"));
        assert!(fed.iter().any(|x| x == "Gus Bilirakis"));
        assert!(fed.iter().any(|x| x.contains("(politician)")));

        // J2: statewide / judicial / local office disambiguators
        let gov = ballotpedia_title_candidates(
            "Ron DeSantis",
            Some("FL"),
            Some("statewide"),
            Some("Governor"),
        );
        assert!(gov.iter().any(|x| x == "Ron DeSantis"));
        assert!(gov.iter().any(|x| x == "Ron DeSantis (Florida)"));
        assert!(gov.iter().any(|x| x.contains("Governor")));
        assert!(gov.len() <= 8);

        let judge = ballotpedia_title_candidates(
            "Melanie Chase",
            Some("FL"),
            Some("judicial"),
            Some("Circuit Judge"),
        );
        assert!(judge.iter().any(|x| x == "Melanie Chase (Florida)"));
        assert!(judge
            .iter()
            .any(|x| x.to_ascii_lowercase().contains("judge")));

        let local = ballotpedia_title_candidates(
            "Alice Smith",
            Some("FL"),
            Some("county"),
            Some("County Commissioner"),
        );
        assert!(local.iter().any(|x| x.contains("County Commissioner")));
        assert!(local.len() <= 8);

        let html = r#"
          <html><head>
            <meta property="og:title" content="Joe Gruters"/>
            <title>Joe Gruters - Ballotpedia</title>
          </head><body>
            <div class="infobox person">
              <div class="widget-row value-only Republican">Joe Gruters</div>
              <div class="widget-row value-only Republican">Florida State Senate District 22</div>
              <div class="widget-row value-only white">
                <a href="https://joegruters.com/" target="_blank">Campaign website</a>
              </div>
            </div>
            <p>Joe Gruters is a member of the Florida State Senate.</p>
          </body></html>
        "#;
        assert!(ballotpedia_html_matches_person(
            html,
            "Joe Gruters",
            Some("FL")
        ));
        assert!(!ballotpedia_html_matches_person(
            html,
            "Joe Gruters",
            Some("NY")
        ));
        assert!(!ballotpedia_html_matches_person(
            html,
            "Michael Bilirakis",
            Some("FL")
        ));
        assert_eq!(
            ballotpedia_campaign_website(html).as_deref(),
            Some("https://joegruters.com/")
        );

        let disambig = r#"
          <title>John Smith - Ballotpedia</title>
          <p>John Smith may refer to:</p>
          <ul><li>John Smith (Alabama)</li></ul>
        "#;
        assert!(!ballotpedia_html_matches_person(
            disambig,
            "John Smith",
            Some("AL")
        ));
        // Accent fold: Muniz ↔ Muñiz
        assert!(gp_title_matches_person(
            "Carlos G. Muniz",
            "Carlos G. Muñiz"
        ));
        assert!(ballotpedia_html_matches_person(
            r#"<html><head><meta property="og:title" content="Carlos Muñiz"/>
               <title>Carlos Muñiz - Ballotpedia</title></head>
               <body><div class="infobox person">
                 <div class="widget-row">Carlos Muñiz</div>
                 <div class="widget-row">Florida Supreme Court</div>
               </div></body></html>"#,
            "Carlos G. Muniz",
            Some("FL"),
        ));
    }

    #[test]
    fn fl_courts_index_urls_and_parse() {
        assert_eq!(
            fl_courts_index_url("Supreme Court Justice"),
            Some("https://supremecourt.flcourts.gov/Justices".into())
        );
        assert_eq!(
            fl_courts_index_url("District Court of Appeal (District 5)"),
            Some("https://5dca.flcourts.gov/Judges".into())
        );
        assert_eq!(
            fl_courts_index_url("Circuit Judge (Circuit 18, Group 13)"),
            Some("https://flcourts18.org/directory/".into())
        );
        assert_eq!(
            fl_courts_index_url("Circuit Judge (Circuit 7, Group 2)"),
            Some("https://circuit7.org/judges/".into())
        );
        assert_eq!(
            fl_courts_index_url("Circuit Judge (Circuit 12, Group 1)"),
            Some("https://www.jud12.flcourts.org/About/Judges".into())
        );
        assert_eq!(
            fl_courts_index_url("Circuit Judge (Circuit 10, Group 3)"),
            Some("https://www.jud10.flcourts.org/gallery/judges".into())
        );
        assert!(fl_courts_index_url("Circuit Judge (Circuit 9, Group 1)").is_none());

        let c7 = r#"
          <a href="https://circuit7.org/judges/judge-benjamin-j-rich/">Judge Benjamin J. Rich</a>
          <a href="https://circuit7.org/judges/division-62-vacant/">Judge George A. Young</a>
          <a href="https://circuit7.org/judges/feed/">Feed</a>
        "#;
        let c7links = parse_fl_circuit_directory_links(c7, "https://circuit7.org/judges/");
        assert!(c7links
            .iter()
            .any(|l| l.name.contains("Rich") && l.url.contains("benjamin")));
        assert!(!c7links
            .iter()
            .any(|l| l.url.contains("vacant") || l.url.contains("feed")));

        let c12 = r#"
          <a href="/About-the-Court/Judges-Magistrates/Judge-Andrea-Johnson">Judge Andrea Johnson</a>
          <a href="/About-the-Court/Judges-Magistrates/Judge-TBD">Judge TBD</a>
          <a href="/About-the-Court/Judges-Magistrates/Judge-DeSoto-County">Judge DeSoto County</a>
        "#;
        let c12links =
            parse_fl_circuit_directory_links(c12, "https://www.jud12.flcourts.org/About/Judges");
        assert!(c12links.iter().any(|l| l.name.contains("Andrea")));
        assert!(!c12links
            .iter()
            .any(|l| l.url.contains("TBD") || l.name.contains("DeSoto")));

        let c10 = r#"
          <a href="/gallery/j-kevin-abdoney"><img alt="Portrait of Judge J. Kevin Abdoney" src="/sites/x.jpg"></a>
          <a href="/gallery/judges">All judges</a>
        "#;
        let c10links =
            parse_fl_circuit_directory_links(c10, "https://www.jud10.flcourts.org/gallery/judges");
        assert!(c10links.iter().any(|l| l.name.contains("Abdoney")));

        let c7bio = r#"
          <html><body>
          <p>St. Johns County Court Judge Benjamin Rich was appointed to the bench in 2025.
          He earned his undergraduate degree from the University of Central Florida and his
          Juris Doctor from Florida Coastal School of Law. He previously practiced law at Smith &amp; Jones LLP from 2010 to 2024.</p>
          <p>Under Florida law, e-mail addresses are public records. If you do not want your e-mail address released...</p>
          </body></html>
        "#;
        let bio = parse_fl_circuit_wp_bio_html(
            c7bio,
            "https://circuit7.org/judges/judge-benjamin-j-rich/",
        );
        assert!(bio
            .facts
            .iter()
            .any(|f| f.kind == "education" && f.text.contains("Central Florida")));
        assert!(bio
            .facts
            .iter()
            .any(|f| f.kind == "office" || f.text.to_ascii_lowercase().contains("appointed")));
        assert!(bio
            .facts
            .iter()
            .all(|f| !f.text.to_ascii_lowercase().contains("public records")));

        let portals =
            fl_judge_decision_portals("District Court of Appeal (District 5)", "Scott Makar");
        assert!(portals
            .iter()
            .any(|p| p.url.contains("5dca") && p.label.contains("opinions")));
        assert!(portals.iter().any(|p| p.url.contains("floridabar.org")));
        let sc_ops = fl_judicial_opinion_portals("Justice of the Supreme Court");
        assert!(sc_ops
            .iter()
            .any(|p| p.url.contains("supremecourt") && p.url.contains("Opinions")));

        let index = r#"
          <script id="__NEXT_DATA__" type="application/json">
          {"props":{"pageProps":{"childrenInfos":[
            {"props":{"content":{"name":"Judge Scott Makar","typeIdentifier":"judge","url":"/Judges/judge-scott-makar"},
              "location":{"url":"/Judges/judge-scott-makar"}}},
            {"props":{"content":{"name":"Former Judges","typeIdentifier":"folder","url":"/Judges/former"},
              "location":{"url":"/Judges/former"}}},
            {"props":{"content":{"name":"Justice Carlos G. Muñiz","typeIdentifier":"folder","url":"/justices/muniz"},
              "location":{"url":"/justices/muniz"}}}
          ]}}}
          </script>
        "#;
        let links = parse_fl_courts_next_index(index, "https://5dca.flcourts.gov/Judges");
        assert!(links.iter().any(|l| l.name.contains("Makar")));
        assert!(links
            .iter()
            .any(|l| l.name.contains("Muñiz") || l.name.contains("Muniz")));
        assert!(!links.iter().any(|l| l.name.contains("Former")));
        let hit = match_fl_courts_judge_link(&links, "Scott D. Makar").unwrap();
        assert!(hit.url.contains("makar"));
        let hit2 = match_fl_courts_judge_link(&links, "Carlos G. Muniz").unwrap();
        assert!(hit2.url.contains("muniz"));
        // Middle name on ballot, short form on roster
        let roster = vec![FlCourtsJudgeLink {
            name: "Robert Segal".into(),
            url: "https://flcourts18.org/robert-segal-biography/".into(),
            kind: "wp_bio".into(),
        }];
        assert!(
            match_fl_courts_judge_link(&roster, "Robert Alan Segal").map(|l| l.name.as_str())
                == Some("Robert Segal")
        );

        let page = r#"
          <script id="__NEXT_DATA__" type="application/json">
          {"props":{"pageProps":{"pageData":{
            "name":"Judge Scott Makar",
            "yearsOfService":"Jan 2012 - Present",
            "image":{"image":{"uri":"/var/site/storage/images/media/images/judge-makar.jpg"}},
            "degrees":{"html5":"<ul><li>J.D., University of Florida, 1987</li><li>B.S., Mercer University, 1980</li></ul>"},
            "officesPositions":{"html5":"<ul><li>Judge, Fifth District Court of Appeal, 2012-present</li><li>Solicitor General, State of Florida, 2007-2012</li></ul>"}
          }}}}
          </script>
        "#;
        let bio =
            parse_fl_courts_judge_html(page, "https://5dca.flcourts.gov/Judges/judge-scott-makar");
        assert!(bio.photo_url.as_ref().unwrap().contains("judge-makar"));
        assert!(bio
            .facts
            .iter()
            .any(|f| f.kind == "education" && f.text.contains("Florida")));
        assert!(bio.spans.iter().any(|s| s.category == "political"));
        assert!(bio.facts.iter().all(|f| f.source == "Florida Courts"));

        // SC justice folder: entity-encoded media + paragraph glue + classification
        let muniz = r#"
          <script id="__NEXT_DATA__" type="application/json">
          {"props":{"pageProps":{"pageData":{
            "name":"Justice Carlos G. Muñiz",
            "shortDescription":{"html5":"<p>Carlos G. Mu&ntilde;iz is the 89th Justice. Appointed in 2019.</p>"},
            "description":{"html5":"<p>Justice Mu&ntilde;iz was appointed to the Florida Supreme Court by Governor Ron DeSantis on January 22, 2019.</p><div class=\"ibexa-embed\" data-content=\"&#x7B;&quot;image&quot;&#x3A;&#x7B;&quot;url&quot;&#x3A;&quot;https&#x3A;&#x5C;&#x2F;&#x5C;&#x2F;flcourts-media.flcourts.gov&#x5C;&#x2F;image&#x5C;&#x2F;download&#x5C;&#x2F;var&#x5C;&#x2F;site&#x5C;&#x2F;storage&#x5C;&#x2F;images&#x5C;&#x2F;5&#x5C;&#x2F;1&#x5C;&#x2F;2&#x5C;&#x2F;0&#x5C;&#x2F;7660215-1-eng-US&#x5C;&#x2F;muniz-formal-original-2019.jpg&quot;&#x7D;&#x7D;\"></div><p>He served as the deputy attorney general and chief of staff to Attorney General Pam Bondi.</p><p>Justice Mu&ntilde;iz is a graduate of the University of Virginia and of Yale Law School.</p><p>After law school, he clerked for Judge Jos&eacute; A. Cabranes of the U.S. Court of Appeals for the Second Circuit and for Judge Thomas A. Flannery of the U.S. District Court for the District of Columbia.</p><p>Justice Mu&ntilde;iz lives in Tallahassee with his wife, Katie Mu&ntilde;iz, and their three children.</p><p>Office Information The phone number is (850) 488-0007.</p>"}
          }}}}
          </script>
        "#;
        let mb = parse_fl_courts_judge_html(
            muniz,
            "https://supremecourt.flcourts.gov/justices/justice-carlos-g.-muniz",
        );
        let photo = mb.photo_url.expect("muniz photo");
        assert!(
            photo.contains("muniz-formal") && photo.contains("flcourts-media"),
            "photo={photo}"
        );
        assert!(
            mb.facts
                .iter()
                .any(|f| f.kind == "education" && f.text.contains("Yale")),
            "edu: {:?}",
            mb.facts
        );
        assert!(
            mb.facts
                .iter()
                .any(|f| f.kind == "office" && f.text.contains("Bondi")),
            "career under office not edu: {:?}",
            mb.facts
        );
        assert!(
            mb.facts
                .iter()
                .any(|f| f.kind == "office" && f.text.contains("clerked")),
            "clerkship is office: {:?}",
            mb.facts
        );
        assert!(
            mb.facts
                .iter()
                .any(|f| f.kind == "family" && f.text.contains("Katie")),
            "family: {:?}",
            mb.facts
        );
        assert!(
            !mb.facts.iter().any(|f| f.text.contains("488-0007")),
            "skip office phone: {:?}",
            mb.facts
        );
        // No glued sentences
        assert!(
            !mb.facts
                .iter()
                .any(|f| f.text.contains(".Justice") || f.text.contains(".He ")),
            "glued prose: {:?}",
            mb.facts
        );
        assert!(mb.spans.iter().any(|s| s.start_year == Some(2019)));
    }

    #[test]
    fn campaign_site_about_helpers() {
        assert!(is_campaign_site_url("https://joegruters.com/"));
        assert!(is_campaign_site_url("https://katforcongress.com"));
        assert!(!is_campaign_site_url("https://bilirakis.house.gov/about"));
        assert!(!is_campaign_site_url("https://ballotpedia.org/Joe_Gruters"));
        assert!(!is_campaign_site_url("https://www.facebook.com/JoeGruters"));
        assert!(!is_campaign_site_url(
            "https://dos.elections.myflorida.com/x"
        ));

        let urls = campaign_about_urls("https://joegruters.com/");
        assert!(urls.iter().any(|u| u.ends_with("/about")));
        assert!(urls.iter().any(|u| u == "https://joegruters.com"));

        let html = r#"
          <html><head><meta property="og:image" content="https://cdn.example/me.jpg"/></head>
          <body><main>
            <p>Jane Example is an American lawyer and small-business owner. She was born on May 1, 1975
            in Tampa, Florida. She graduated from the University of Florida in 1997 and received her J.D.
            degree from the Stetson University College of Law in 2000. She and her husband have two children.</p>
          </main></body></html>
        "#;
        let p = parse_campaign_about_html(html, "https://janeexample.com/about");
        assert!(p.facts.iter().all(|f| f.source == "Campaign website"));
        assert!(p.citizenship.is_none());
        assert!(
            p.facts
                .iter()
                .any(|f| f.kind == "education" && f.text.contains("Florida"))
                || p.birth_year == Some(1975)
                || p.facts.iter().any(|f| f.text.contains("Lawyer")),
            "campaign facts: {:?}",
            p.facts
        );
    }

    #[test]
    fn coalesce_bilirakis_like_education_work_family() {
        // Noisy multi-source dump (BP + WD + DBpedia + GP) → one FL, one Stetson, one attorney.
        let raw = vec![
            BioFact {
                kind: "education".into(),
                text: "Bachelor's: University of Florida, 1986".into(),
                source: "Ballotpedia".into(),
                source_url: Some("https://ballotpedia.org/Gus_M._Bilirakis".into()),
                ..Default::default()
            },
            BioFact {
                kind: "education".into(),
                text: "Education: University of Florida".into(),
                source: "DBpedia".into(),
                source_url: Some("https://dbpedia.org/page/Gus_Bilirakis".into()),
                ..Default::default()
            },
            BioFact {
                kind: "education".into(),
                text: "Education: University of Florida, Bachelor of Arts".into(),
                source: "Wikidata".into(),
                source_url: Some("https://www.wikidata.org/wiki/Q455807".into()),
                ..Default::default()
            },
            BioFact {
                kind: "education".into(),
                text: "Education: Bachelor of Arts".into(),
                source: "DBpedia".into(),
                source_url: None,
                ..Default::default()
            },
            BioFact {
                kind: "education".into(),
                text: "Law: Stetson Law School, 1989".into(),
                source: "Ballotpedia".into(),
                source_url: None,
                ..Default::default()
            },
            BioFact {
                kind: "education".into(),
                text: "Education: Stetson University College of Law, 1989".into(),
                source: "Grokipedia".into(),
                source_url: Some("https://grokipedia.com/page/Gus_Bilirakis".into()),
                ..Default::default()
            },
            BioFact {
                kind: "education".into(),
                text: "Education: Juris Doctor".into(),
                source: "DBpedia".into(),
                source_url: None,
                ..Default::default()
            },
            BioFact {
                kind: "work".into(),
                text: "Profession: Attorney".into(),
                source: "Ballotpedia".into(),
                source_url: None,
                ..Default::default()
            },
            BioFact {
                kind: "legal".into(),
                text: "Occupation: lawyer".into(),
                source: "Wikidata".into(),
                source_url: Some("https://www.wikidata.org/wiki/Q455807".into()),
                ..Default::default()
            },
            BioFact {
                kind: "work".into(),
                text: "Profession: Attorney".into(),
                source: "Grokipedia".into(),
                source_url: None,
                ..Default::default()
            },
            BioFact {
                kind: "legal".into(),
                text: "Occupation: jurist".into(),
                source: "Wikidata".into(),
                source_url: None,
                ..Default::default()
            },
            BioFact {
                kind: "family".into(),
                text: "Spouse: Eva Lialios".into(),
                source: "Ballotpedia".into(),
                source_url: None,
                ..Default::default()
            },
            BioFact {
                kind: "family".into(),
                text: "Spouse: Eva Lialios Bilirakis".into(),
                source: "DBpedia".into(),
                source_url: None,
                ..Default::default()
            },
            BioFact {
                kind: "family".into(),
                text: "Children: 4".into(),
                source: "Ballotpedia".into(),
                source_url: None,
                ..Default::default()
            },
            BioFact {
                kind: "other".into(),
                text: "Born: February 8, 1963".into(),
                source: "Wikidata".into(),
                source_url: None,
                ..Default::default()
            },
            BioFact {
                kind: "other".into(),
                text: "Born: 1963-02-08".into(),
                source: "DBpedia".into(),
                source_url: None,
                ..Default::default()
            },
        ];

        let out = coalesce_dossier_facts(&raw);
        let edu: Vec<_> = out.iter().filter(|f| f.kind == "education").collect();
        assert_eq!(
            edu.len(),
            2,
            "one Florida + one Stetson, got: {:?}",
            edu.iter().map(|f| &f.text).collect::<Vec<_>>()
        );
        assert!(
            edu.iter().any(|f| {
                let t = f.text.to_ascii_lowercase();
                t.contains("florida") && !t.contains("stetson")
            }),
            "florida row: {:?}",
            edu
        );
        assert!(
            edu.iter()
                .any(|f| f.text.to_ascii_lowercase().contains("stetson")),
            "stetson row: {:?}",
            edu
        );
        // Florida multi-cite
        let fl = edu
            .iter()
            .find(|f| f.text.to_ascii_lowercase().contains("florida"))
            .unwrap();
        assert!(
            fl.sources.len() >= 2,
            "florida multi-cite: {:?}",
            fl.sources
        );
        assert!(
            fl.sources.iter().any(|s| s.name.contains("Ballotpedia")),
            "{:?}",
            fl.sources
        );

        let workish: Vec<_> = out
            .iter()
            .filter(|f| matches!(f.kind.as_str(), "work" | "legal" | "business"))
            .collect();
        assert_eq!(
            workish.len(),
            1,
            "one attorney profession, got: {:?}",
            workish
                .iter()
                .map(|f| (&f.kind, &f.text))
                .collect::<Vec<_>>()
        );
        assert!(
            workish[0].text.to_ascii_lowercase().contains("attorney")
                || workish[0].text.to_ascii_lowercase().contains("lawyer"),
            "{:?}",
            workish[0].text
        );
        assert!(workish[0].sources.len() >= 2, "{:?}", workish[0].sources);

        let spouses: Vec<_> = out
            .iter()
            .filter(|f| f.kind == "family" && f.text.to_ascii_lowercase().contains("spouse"))
            .collect();
        assert_eq!(spouses.len(), 1, "{:?}", spouses);
        assert!(spouses[0].text.contains("Eva"));
        assert!(spouses[0].sources.len() >= 2);

        let born: Vec<_> = out
            .iter()
            .filter(|f| f.kind == "other" && f.text.to_ascii_lowercase().starts_with("born"))
            .collect();
        assert_eq!(born.len(), 1, "{:?}", born);

        // Apply path: stack then coalesce; fractions count edu years once.
        let mut d = empty_dossier(2026);
        d.career = assess_career(
            &[CareerSpan::new(
                LifeCategory::Political,
                "U.S. House",
                Some(2007),
                None,
                "CL",
                None,
            )],
            Some(1963),
            2026,
        );
        apply_member_bio_to_dossier(
            &mut d,
            &MemberBioParse {
                birth_year: Some(1963),
                facts: raw.clone(),
                spans: vec![],
                ..Default::default()
            },
            2026,
        );
        assert_eq!(
            d.facts.iter().filter(|f| f.kind == "education").count(),
            2,
            "dossier edu: {:?}",
            d.facts
                .iter()
                .filter(|f| f.kind == "education")
                .map(|f| &f.text)
                .collect::<Vec<_>>()
        );
        let edu_frac = d
            .career
            .fractions
            .iter()
            .find(|f| f.category == "education")
            .expect("edu fraction");
        // 1986 + 1989 → at least 2 years, not doubled by synonym rows
        assert!(
            edu_frac.years >= 2.0 && edu_frac.years <= 4.0,
            "edu years not double-counted: {:?}",
            edu_frac
        );
    }

    #[test]
    fn coalesce_gillibrand_like_family_and_edu() {
        let raw = vec![
            BioFact {
                kind: "family".into(),
                text: "Spouse: Jonathan Gillibrand".into(),
                source: "Ballotpedia".into(),
                source_url: None,
                ..Default::default()
            },
            BioFact {
                kind: "family".into(),
                text: "Spouse: Jonathan Gillibrand".into(),
                source: "Wikidata".into(),
                source_url: None,
                ..Default::default()
            },
            BioFact {
                kind: "family".into(),
                text: "Children: Theodore, Henry".into(),
                source: "Wikipedia".into(),
                source_url: None,
                ..Default::default()
            },
            BioFact {
                kind: "education".into(),
                text: "Education: Dartmouth College".into(),
                source: "DBpedia".into(),
                source_url: None,
                ..Default::default()
            },
            BioFact {
                kind: "education".into(),
                text: "Bachelor's: Dartmouth College, 1988".into(),
                source: "Ballotpedia".into(),
                source_url: None,
                ..Default::default()
            },
            BioFact {
                kind: "education".into(),
                text: "Education: UCLA School of Law".into(),
                source: "Wikidata".into(),
                source_url: None,
                ..Default::default()
            },
            BioFact {
                kind: "education".into(),
                text: "Law: UCLA School of Law, 1991".into(),
                source: "Ballotpedia".into(),
                source_url: None,
                ..Default::default()
            },
        ];
        let out = coalesce_dossier_facts(&raw);
        assert_eq!(
            out.iter().filter(|f| f.kind == "education").count(),
            2,
            "{:?}",
            out.iter()
                .filter(|f| f.kind == "education")
                .map(|f| &f.text)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            out.iter()
                .filter(|f| f.kind == "family" && f.text.to_ascii_lowercase().contains("spouse"))
                .count(),
            1
        );
        let spouse = out
            .iter()
            .find(|f| f.kind == "family" && f.text.contains("Spouse"))
            .unwrap();
        assert!(spouse.sources.len() >= 2);
    }

    #[test]
    fn family_summary_married_and_kids() {
        let facts = vec![
            BioFact {
                kind: "family".into(),
                text: "Spouse: Eva Lialios".into(),
                source: "Ballotpedia".into(),
                source_url: Some("https://ballotpedia.org/Gus".into()),
                ..Default::default()
            },
            BioFact {
                kind: "family".into(),
                text: "Children: 4".into(),
                source: "DBpedia".into(),
                source_url: None,
                ..Default::default()
            },
        ];
        let s = family_summary_from_facts(&facts);
        assert!(s.disclosed);
        assert!(s.display.contains("Married to Eva Lialios"));
        assert!(s.display.contains("4 children"));
        assert_eq!(s.spouse.as_deref(), Some("Eva Lialios"));
        assert_eq!(s.children_count, Some(4));
        assert!(s.sources.len() >= 2);

        // No invent unmarried when empty
        assert!(!family_summary_from_facts(&[]).disclosed);

        // GP alone insufficient
        let gp_only = vec![BioFact {
            kind: "family".into(),
            text: "Spouse: Someone".into(),
            source: "Grokipedia".into(),
            source_url: None,
            ..Default::default()
        }];
        assert!(!family_summary_from_facts(&gp_only).disclosed);

        // Named children list (comma)
        let named = vec![BioFact {
            kind: "family".into(),
            text: "Children: Theodore, Henry".into(),
            source: "Wikipedia".into(),
            source_url: None,
            ..Default::default()
        }];
        let s2 = family_summary_from_facts(&named);
        assert!(s2.disclosed);
        assert_eq!(s2.children_count, Some(2));
        assert!(s2.display.contains("2 children"));
        assert!(s2.display.contains("Theodore"));
        assert!(s2.display.contains("Henry"));
        assert_eq!(s2.children_detail.as_deref(), Some("Theodore, Henry"));

        // "A and B" without commas (Gillibrand-like)
        let and_names = vec![BioFact {
            kind: "family".into(),
            text: "Children: Brandon and Connor".into(),
            source: "Ballotpedia".into(),
            source_url: None,
            ..Default::default()
        }];
        let s3 = family_summary_from_facts(&and_names);
        assert_eq!(s3.children_count, Some(2));
        assert_eq!(s3.children_detail.as_deref(), Some("Brandon, Connor"));
        assert!(s3.display.contains("2 children (Brandon, Connor)"));

        // Oxford comma list
        let oxford = vec![BioFact {
            kind: "family".into(),
            text: "Children: Ava, Beau, and Cole".into(),
            source: "Wikipedia".into(),
            source_url: None,
            ..Default::default()
        }];
        let s4 = family_summary_from_facts(&oxford);
        assert_eq!(s4.children_count, Some(3));
        assert_eq!(s4.children_detail.as_deref(), Some("Ava, Beau, Cole"));

        // Named list wins over conflicting bare "1 child"
        let conflict = vec![
            BioFact {
                kind: "family".into(),
                text: "Children: 1".into(),
                source: "DBpedia".into(),
                source_url: None,
                ..Default::default()
            },
            BioFact {
                kind: "family".into(),
                text: "Children: Brandon and Connor".into(),
                source: "Ballotpedia".into(),
                source_url: None,
                ..Default::default()
            },
        ];
        let s5 = family_summary_from_facts(&conflict);
        assert_eq!(s5.children_count, Some(2));
        assert!(s5.display.contains("Brandon"));
        assert!(s5.display.contains("Connor"));
    }

    #[test]
    fn orientation_explicit_only_never_inferred() {
        // Spouse present ≠ orientation
        let family = vec![BioFact {
            kind: "family".into(),
            text: "Spouse: Jonathan Gillibrand".into(),
            source: "Ballotpedia".into(),
            source_url: None,
            ..Default::default()
        }];
        assert!(!orientation_from_facts(&family).disclosed);

        // GP alone insufficient
        let gp = vec![BioFact {
            kind: "other".into(),
            text: "is openly gay".into(),
            source: "Grokipedia".into(),
            source_url: None,
            ..Default::default()
        }];
        assert!(!orientation_from_facts(&gp).disclosed);

        // Explicit public cite
        let ok = vec![BioFact {
            kind: "other".into(),
            text: "is an openly gay American politician".into(),
            source: "Wikipedia".into(),
            source_url: Some("https://en.wikipedia.org/wiki/Example".into()),
            ..Default::default()
        }];
        let o = orientation_from_facts(&ok);
        assert!(o.disclosed);
        assert_eq!(o.label.as_deref(), Some("Openly gay"));
        assert!(!o.sources.is_empty());
    }

    #[test]
    fn education_sort_professional_first_and_snapshot_refresh() {
        let mut d = empty_dossier(2026);
        d.facts = vec![
            BioFact {
                kind: "education".into(),
                text: "Bachelor's: University of Florida, 1986".into(),
                source: "Ballotpedia".into(),
                source_url: None,
                ..Default::default()
            },
            BioFact {
                kind: "education".into(),
                text: "Law: Stetson Law School, 1989".into(),
                source: "Ballotpedia".into(),
                source_url: None,
                ..Default::default()
            },
            BioFact {
                kind: "family".into(),
                text: "Spouse: Eva Lialios".into(),
                source: "Ballotpedia".into(),
                source_url: None,
                ..Default::default()
            },
            BioFact {
                kind: "family".into(),
                text: "Children: four children".into(),
                source: "Wikidata".into(),
                source_url: None,
                ..Default::default()
            },
        ];
        refresh_dossier_snapshot_fields(&mut d);
        let edu: Vec<_> = d
            .facts
            .iter()
            .filter(|f| f.kind == "education")
            .map(|f| f.text.as_str())
            .collect();
        assert!(
            edu[0].to_ascii_lowercase().contains("stetson")
                || edu[0].to_ascii_lowercase().contains("law"),
            "JD/law first: {:?}",
            edu
        );
        assert!(d.family_summary.disclosed);
        assert!(d.family_summary.display.contains("Eva"));
        assert_eq!(d.family_summary.children_count, Some(4));
        assert!(!d.orientation.disclosed);
    }

    #[test]
    fn empty_not_found_copy_checked_hosts() {
        assert_eq!(
            empty_not_found_copy(&[]),
            "Not disclosed in sources we check."
        );
        let s = empty_not_found_copy(&[
            "Ballotpedia".into(),
            "official site".into(),
            "Wikidata".into(),
            "Ballotpedia".into(),
        ]);
        assert_eq!(
            s,
            "Checked Ballotpedia / official site / Wikidata — not found."
        );
    }

    #[test]
    fn i4_smoke_bilirakis_and_gillibrand_10001() {
        // Federal dossier pipeline (fixture order ≈ enrich stages) — FL House + NY Senate smoke.
        let mut fl = empty_dossier(2026);
        note_source_checked(&mut fl, "official site");
        apply_member_bio_to_dossier(
            &mut fl,
            &parse_official_member_about_html(
                include_str!("../../../testdata/official_about_bilirakis_house.html"),
                "https://bilirakis.house.gov/about",
            ),
            2026,
        );
        note_source_checked(&mut fl, "Ballotpedia");
        apply_member_bio_to_dossier(
            &mut fl,
            &parse_ballotpedia_member_html(
                include_str!("../../../testdata/ballotpedia_bilirakis_sample.html"),
                "https://ballotpedia.org/Gus_M._Bilirakis",
            ),
            2026,
        );
        note_source_checked(&mut fl, "Wikipedia");
        apply_member_bio_fill_gaps(
            &mut fl,
            &parse_wikipedia_extract_json(include_str!("testdata/g5/wiki_extract_bilirakis.json")),
            2026,
        );
        note_source_checked(&mut fl, "DBpedia");
        apply_member_bio_fill_gaps(
            &mut fl,
            &parse_dbpedia_ntriples(
                include_str!("testdata/g5/dbpedia_bilirakis.nt"),
                "Gus Bilirakis",
            ),
            2026,
        );
        note_source_checked(&mut fl, "Grokipedia");
        apply_member_bio_fill_gaps(
            &mut fl,
            &parse_grokipedia_page_html(
                include_str!("testdata/g5/grokipedia_bilirakis.html"),
                "https://grokipedia.com/page/Gus_Bilirakis",
            ),
            2026,
        );
        note_source_checked(&mut fl, "Wikidata");
        apply_member_bio_to_dossier(
            &mut fl,
            &parse_wikidata_entity_bio(
                include_str!("testdata/g5/wikidata_bilirakis.json"),
                include_str!("testdata/g5/wikidata_bilirakis_labels.json"),
            ),
            2026,
        );
        polish_dossier_empty_notes(&mut fl);

        let edu: Vec<_> = fl
            .facts
            .iter()
            .filter(|f| f.kind == "education")
            .map(|f| f.text.as_str())
            .collect();
        assert!(
            edu.len() >= 2 && edu.len() <= 6,
            "coalesced edu count: {:?}",
            edu
        );
        assert!(
            edu.iter()
                .any(|t| t.to_ascii_lowercase().contains("florida")),
            "{:?}",
            edu
        );
        assert!(
            edu.iter()
                .any(|t| t.to_ascii_lowercase().contains("stetson")
                    || t.to_ascii_lowercase().contains("law")),
            "{:?}",
            edu
        );
        // No stacked bare school + full school for University of Florida
        let fl_rows = edu
            .iter()
            .filter(|t| {
                let l = t.to_ascii_lowercase();
                l.contains("university of florida")
                    || (l.contains("florida")
                        && !l.contains("stetson")
                        && !l.contains("tarpon")
                        && !l.contains("petersburg"))
            })
            .count();
        assert_eq!(fl_rows, 1, "one Florida undergrad row: {:?}", edu);
        let stetson_rows = edu
            .iter()
            .filter(|t| t.to_ascii_lowercase().contains("stetson"))
            .count();
        assert_eq!(stetson_rows, 1, "one Stetson row: {:?}", edu);

        let workish: Vec<_> = fl
            .facts
            .iter()
            .filter(|f| matches!(f.kind.as_str(), "work" | "legal" | "business"))
            .map(|f| (&f.kind, f.text.as_str()))
            .collect();
        assert!(
            workish.iter().any(|(k, t)| {
                let l = t.to_ascii_lowercase();
                l.contains("attorney") || l.contains("lawyer") || *k == "legal"
            }),
            "{:?}",
            workish
        );
        // attorney synonym spam collapsed (≤2 workish: profession + optional employer)
        assert!(workish.len() <= 3, "work coalesced: {:?}", workish);

        if fl.family_summary.disclosed {
            assert!(
                fl.family_summary
                    .display
                    .to_ascii_lowercase()
                    .contains("married")
                    || fl.family_summary.spouse.is_some(),
                "{:?}",
                fl.family_summary
            );
        }
        assert!(!fl.orientation.disclosed);
        assert!(fl.sources_checked.iter().any(|s| s == "Ballotpedia"));
        assert!(fl.sources_checked.iter().any(|s| s == "Wikidata"));
        // Education present → no edu empty note with checked copy required
        assert!(
            !fl.empty_notes
                .iter()
                .any(|n| n.to_ascii_lowercase().starts_with("education / work:")),
            "{:?}",
            fl.empty_notes
        );

        // --- Gillibrand 10001 ---
        let mut ny = empty_dossier(2026);
        note_source_checked(&mut ny, "official site");
        apply_member_bio_to_dossier(
            &mut ny,
            &parse_official_member_about_html(
                include_str!("../../../testdata/official_about_gillibrand_senate.html"),
                "https://www.gillibrand.senate.gov/about",
            ),
            2026,
        );
        note_source_checked(&mut ny, "Wikipedia");
        apply_member_bio_fill_gaps(
            &mut ny,
            &parse_wikipedia_extract_json(include_str!("testdata/g5/wiki_extract_gillibrand.json")),
            2026,
        );
        note_source_checked(&mut ny, "DBpedia");
        apply_member_bio_fill_gaps(
            &mut ny,
            &parse_dbpedia_ntriples(
                include_str!("testdata/g5/dbpedia_gillibrand.nt"),
                "Kirsten Gillibrand",
            ),
            2026,
        );
        note_source_checked(&mut ny, "Wikidata");
        apply_member_bio_to_dossier(
            &mut ny,
            &parse_wikidata_entity_bio(
                include_str!("testdata/g5/wikidata_gillibrand.json"),
                include_str!("testdata/g5/wikidata_gillibrand_labels.json"),
            ),
            2026,
        );
        polish_dossier_empty_notes(&mut ny);

        assert!(
            ny.family_summary.disclosed || ny.facts.iter().any(|f| f.kind == "family"),
            "Gillibrand family denser: summary={:?} facts={:?}",
            ny.family_summary,
            ny.facts
                .iter()
                .filter(|f| f.kind == "family")
                .map(|f| &f.text)
                .collect::<Vec<_>>()
        );
        if ny.family_summary.disclosed {
            let d = ny.family_summary.display.to_ascii_lowercase();
            assert!(
                d.contains("married") || d.contains("children") || d.contains("child"),
                "{:?}",
                ny.family_summary
            );
        }
        let ny_edu = ny.facts.iter().filter(|f| f.kind == "education").count();
        assert!(ny_edu >= 1, "ny edu facts");
        // multi-cite on at least one fact when sources overlap
        assert!(
            ny.facts.iter().any(|f| f.sources.len() >= 2)
                || fl.facts.iter().any(|f| f.sources.len() >= 2),
            "expected multi-cite somewhere"
        );
        assert!(!ny.orientation.disclosed);
        let empty_fam = empty_not_found_copy(&ny.sources_checked);
        assert!(empty_fam.contains("Checked") && empty_fam.contains("not found"));
    }
}
