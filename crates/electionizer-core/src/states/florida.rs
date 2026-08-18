//! Florida pure mappers: DOS TSV, circuits, rosters, measures HTML (no HTTP).
use crate::models::{
    federal_election_date, normalize_party_label, GeoResolution, ResolvedJurisdiction,
    SnapshotCandidate, SnapshotMeasure,
};
use std::collections::HashMap;

pub const SENATE_URL: &str = "https://www.flsenate.gov/Senators";
pub const HOUSE_URL: &str = "https://www.flhouse.gov/Sections/Representatives/representatives.aspx";
pub const MEASURES_URL: &str = "https://constitutionalinitiatives.dos.fl.gov/";
pub const DOS_CTS_INDEX: &str = "https://dos.elections.myflorida.com/candidates/";
pub const DOS_EXTRACT_URL: &str =
    "https://dos.elections.myflorida.com/candidates/extractCanList.asp";
pub const DOS_DETAIL_BASE: &str =
    "https://dos.elections.myflorida.com/candidates/CanDetail.asp?account=";
pub const DOS_PUBLISHER: &str = "Florida Division of Elections";

/// DOS statuses included on the ballot report.
///
/// - `QUA` — qualified (contested or merit retention)
/// - `UNO` — unopposed (not printed on the sample ballot, but still takes the
///   seat; UI collapses these seats)
/// - `ACT` — rare active pre-status
///
/// `ELE` (already elected) is omitted.
pub const ACTIVE_STATUS: &[&str] = &["QUA", "UNO", "ACT"];



pub fn roster_fallback(
    geo: &GeoResolution,
    state_l: &str,
    senators: &HashMap<u32, Member>,
    representatives: &HashMap<u32, Member>,
    notes: &mut Vec<String>,
) -> Vec<SnapshotCandidate> {
    let mut candidates = Vec::new();
    if let Some(sd) = geo.state_senate_district {
        let sldu_ocd = format!("ocd-division/country:us/state:{state_l}/sldu:{sd}");
        if let Some(m) = senators.get(&sd) {
            candidates.push(member_to_candidate(
                m,
                &format!("Florida Senate (District {sd})"),
                "state_senate",
                &sldu_ocd,
                SENATE_URL,
                "Florida Senate",
            ));
        } else {
            notes.push(format!("No FL senator roster match for district {sd}."));
        }
    }
    if let Some(hd) = geo.state_house_district {
        let sldl_ocd = format!("ocd-division/country:us/state:{state_l}/sldl:{hd}");
        if let Some(m) = representatives.get(&hd) {
            candidates.push(member_to_candidate(
                m,
                &format!("Florida House (District {hd})"),
                "state_house",
                &sldl_ocd,
                &m.profile_url,
                "Florida House of Representatives",
            ));
        } else {
            notes.push(format!("No FL representative roster match for district {hd}."));
        }
    }
    if candidates.is_empty() {
        notes.push(
            "Could not match FL legislative districts for this ZIP — check Census district resolution."
                .into(),
        );
    }
    candidates
}

// --- DOS CTS download + parse ---

pub fn fl_gen_elec_id(cycle: i32) -> String {
    let date = federal_election_date(cycle); // YYYY-MM-DD
    format!("{}-GEN", date.replace('-', ""))
}

/// Candidate type codes on DOS extract form: state, multi-county, local.
pub const DOS_CANTYPES: &[&str] = &["STA", "MUL", "LOC"];



#[derive(Debug, Clone)]
pub struct DosFiling {
    pub acct: String,
    pub office_code: String,
    pub office_desc: String,
    pub juris1: Option<u32>,
    pub juris2: Option<u32>,
    pub status_desc: String,
    pub party_code: String,
    pub name: String,
    /// County name from DOS extract (local / multi-county rows).
    pub county: String,
}

pub fn parse_dos_tsv(tsv: &str) -> Vec<DosFiling> {
    let mut lines = tsv.lines();
    let Some(header_line) = lines.next() else {
        return Vec::new();
    };
    let headers: Vec<&str> = header_line.split('\t').map(|h| h.trim()).collect();
    let idx = |name: &str| headers.iter().position(|h| h.eq_ignore_ascii_case(name));

    let i_acct = idx("AcctNum");
    let i_code = idx("OfficeCode");
    let i_desc = idx("OfficeDesc");
    let i_j1 = idx("Juris1num");
    let i_j2 = idx("Juris2num");
    let i_st = idx("StatusCode");
    let i_std = idx("StatusDesc");
    let i_party = idx("PartyCode");
    let i_last = idx("NameLast");
    let i_first = idx("NameFirst");
    let i_mid = idx("NameMiddle");
    let i_county = idx("County");

    let mut out = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        let get = |i: Option<usize>| -> &str {
            i.and_then(|n| cols.get(n)).map(|s| s.trim()).unwrap_or("")
        };
        let status_code = get(i_st).to_ascii_uppercase();
        if !ACTIVE_STATUS.iter().any(|s| *s == status_code.as_str()) {
            continue;
        }
        let office_code = get(i_code).to_ascii_uppercase();
        // Federal races come from FEC; skip DOS USS/USR.
        if matches!(office_code.as_str(), "USS" | "USR") {
            continue;
        }
        if office_code.is_empty() {
            continue;
        }
        let name = dos_display_name(get(i_first), get(i_mid), get(i_last));
        if name.is_empty() {
            continue;
        }
        let status_desc = {
            let d = get(i_std);
            if d.is_empty() {
                status_code_label(&status_code)
            } else {
                d.to_string()
            }
        };
        out.push(DosFiling {
            acct: get(i_acct).to_string(),
            office_code,
            office_desc: get(i_desc).to_string(),
            juris1: parse_juris(get(i_j1)),
            juris2: parse_juris(get(i_j2)),
            status_desc,
            party_code: get(i_party).to_string(),
            name,
            county: get(i_county).to_string(),
        });
    }
    out
}

pub fn parse_juris(raw: &str) -> Option<u32> {
    let t = raw.trim();
    if t.is_empty() {
        return None;
    }
    t.parse::<u32>().ok()
}

pub fn status_code_label(code: &str) -> String {
    match code {
        "QUA" => "Qualified".into(),
        "UNO" => "Unopposed".into(),
        "ACT" => "Active".into(),
        "ELE" => "Elected".into(),
        other => other.to_string(),
    }
}

pub fn dos_display_name(first: &str, middle: &str, last: &str) -> String {
    let clean = |s: &str| {
        s.trim()
            .trim_matches('"')
            .replace('"', "")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    let first = clean(first);
    let middle = clean(middle);
    let last = clean(last);
    let mut parts = Vec::new();
    if !first.is_empty() {
        parts.push(first);
    }
    if !middle.is_empty() {
        parts.push(middle);
    }
    if !last.is_empty() {
        parts.push(last);
    }
    format_fl_name(&parts.join(" "))
}

pub fn fl_party_label(code: &str) -> String {
    match code.trim().to_ascii_uppercase().as_str() {
        "DEM" => "Democratic".into(),
        "REP" => "Republican".into(),
        "NPA" | "NOP" | "IND" => "Independent / Other".into(),
        "WRI" => "Write-In".into(),
        "LPF" => "Libertarian".into(),
        "GRE" | "GRN" => "Green".into(),
        other if other.is_empty() => "Independent / Other".into(),
        other => normalize_party_label(other),
    }
}

/// Judicial filings: NOP/NPA are nonpartisan seat designations, not “Independent.”
pub fn fl_judicial_party_label(code: &str) -> String {
    match code.trim().to_ascii_uppercase().as_str() {
        "DEM" => "Democratic".into(),
        "REP" => "Republican".into(),
        "NPA" | "NOP" | "IND" | "" => "Nonpartisan".into(),
        other => fl_party_label(other),
    }
}

pub fn map_filings_for_geo(
    filings: &[DosFiling],
    geo: &GeoResolution,
    state_ocd: &str,
    senators: &HashMap<u32, Member>,
    representatives: &HashMap<u32, Member>,
    extra_jurisdictions: &mut Vec<ResolvedJurisdiction>,
) -> Vec<SnapshotCandidate> {
    let state_l = "fl";
    let county_key = normalize_county_key(&geo.county);
    let circuit = county_to_circuit(&county_key);
    let dca = circuit.and_then(circuit_to_dca);

    if let Some(c) = circuit {
        push_jur(
            extra_jurisdictions,
            &format!("ocd-division/country:us/state:{state_l}/circuit:{c}"),
            &format!("Florida Judicial Circuit {c}"),
            "circuit",
        );
    }
    if let Some(d) = dca {
        push_jur(
            extra_jurisdictions,
            &format!("ocd-division/country:us/state:{state_l}/court_of_appeal:{d}"),
            &format!("Florida District Court of Appeal {d}"),
            "appellate",
        );
    }

    let mut out = Vec::new();
    for f in filings {
        let mapped = match f.office_code.as_str() {
            "STS" => {
                let Some(sd) = geo.state_senate_district else {
                    continue;
                };
                if f.juris1 != Some(sd) {
                    continue;
                }
                let ocd = format!("ocd-division/country:us/state:{state_l}/sldu:{sd}");
                let office = format!("Florida Senate (District {sd})");
                let incumb = senators
                    .get(&sd)
                    .map(|m| names_match(&m.name, &f.name))
                    .unwrap_or(false);
                Some((office, "state_senate", ocd, false, incumb))
            }
            "STR" => {
                let Some(hd) = geo.state_house_district else {
                    continue;
                };
                if f.juris1 != Some(hd) {
                    continue;
                }
                let ocd = format!("ocd-division/country:us/state:{state_l}/sldl:{hd}");
                let office = format!("Florida House (District {hd})");
                let incumb = representatives
                    .get(&hd)
                    .map(|m| names_match(&m.name, &f.name))
                    .unwrap_or(false);
                Some((office, "state_house", ocd, false, incumb))
            }
            "GOV" | "ATG" | "CFO" | "AGR" | "SCJ" => {
                let office = if f.office_desc.is_empty() {
                    office_code_label(&f.office_code).to_string()
                } else {
                    f.office_desc.clone()
                };
                let chamber = if f.office_code == "SCJ" {
                    "judicial"
                } else {
                    "statewide"
                };
                let is_judge = f.office_code == "SCJ";
                Some((office, chamber, state_ocd.to_string(), is_judge, false))
            }
            // DCA/CTJ: circuit- or district-wide — not precinct. Group (juris2) is a seat
            // number. Contested (QUA) and unopposed (UNO) both listed; UI opens contested
            // seats and collapses UNO so voters still see who takes the bench. A FL
            // sample primary may print only some Groups; UNO groups are report-only.
            "DCA" => {
                let Some(want) = dca else { continue };
                if f.juris1 != Some(want) {
                    continue;
                }
                let ocd =
                    format!("ocd-division/country:us/state:{state_l}/court_of_appeal:{want}");
                let office = format!("District Court of Appeal (District {want})");
                Some((office, "judicial", ocd, true, false))
            }
            "CTJ" => {
                let Some(want) = circuit else { continue };
                if f.juris1 != Some(want) {
                    continue;
                }
                let ocd = format!("ocd-division/country:us/state:{state_l}/circuit:{want}");
                let office = match f.juris2 {
                    Some(g) => format!("Circuit Judge (Circuit {want}, Group {g})"),
                    None => format!("Circuit Judge (Circuit {want})"),
                };
                Some((office, "judicial", ocd, true, false))
            }
            // County judge: county-wide seat; group is juris2 (DOS often leaves juris1 empty).
            "COJ" => {
                if !filing_matches_county(f, &county_key) {
                    continue;
                }
                let county_slug = slugify_simple(&county_key);
                let ocd =
                    format!("ocd-division/country:us/state:{state_l}/county:{county_slug}");
                let county_name = if geo.county.trim().is_empty() {
                    format!("{} County", title_case_words(&county_key))
                } else {
                    geo.county.clone()
                };
                push_jur(extra_jurisdictions, &ocd, &county_name, "county");
                let group = f.juris2.or(f.juris1);
                let office = match group {
                    Some(g) => format!("County Judge (Group {g})"),
                    None => "County Judge".into(),
                };
                Some((office, "judicial", ocd, true, false))
            }
            "STA" | "PUB" => {
                let Some(want) = circuit else { continue };
                if f.juris1 != Some(want) {
                    continue;
                }
                let ocd = format!("ocd-division/country:us/state:{state_l}/circuit:{want}");
                let office = match f.office_code.as_str() {
                    "STA" => format!("State Attorney (Circuit {want})"),
                    _ => format!("Public Defender (Circuit {want})"),
                };
                Some((office, "judicial", ocd, false, false))
            }
            _ => map_local_or_special_filing(f, geo, state_l, &county_key, extra_jurisdictions),
        };

        let Some((office, chamber, ocd, is_judge, is_incumbent)) = mapped else {
            continue;
        };

        let party = if is_judge {
            fl_judicial_party_label(&f.party_code)
        } else {
            fl_party_label(&f.party_code)
        };
        let dos_url = if f.acct.is_empty() {
            DOS_CTS_INDEX.to_string()
        } else {
            format!("{DOS_DETAIL_BASE}{}", f.acct)
        };
        // F8: prefer chamber member page for leg incumbents (dossier photo/bio).
        // Finance still uses external_id `fl:acct:*`, not source_url.
        let (source_url, source_publisher) = if is_incumbent {
            match chamber {
                "state_senate" => geo
                    .state_senate_district
                    .and_then(|sd| senators.get(&sd))
                    .filter(|m| !m.profile_url.is_empty())
                    .map(|m| (m.profile_url.clone(), "Florida Senate"))
                    .unwrap_or((dos_url, DOS_PUBLISHER)),
                "state_house" => geo
                    .state_house_district
                    .and_then(|hd| representatives.get(&hd))
                    .filter(|m| !m.profile_url.is_empty())
                    .map(|m| (m.profile_url.clone(), "Florida House of Representatives"))
                    .unwrap_or((dos_url, DOS_PUBLISHER)),
                _ => (dos_url, DOS_PUBLISHER),
            }
        } else {
            (dos_url, DOS_PUBLISHER)
        };

        out.push(SnapshotCandidate {
            office,
            chamber: Some(chamber.into()),
            jurisdiction_ocd: ocd,
            is_judicial: is_judge,
            name: f.name.clone(),
            party: party.clone(),
            is_incumbent,
            is_judge,
            summary: Some(format!(
                "{} · {} ({party}). Source: {DOS_PUBLISHER} Candidate Tracking System.",
                f.status_desc, f.office_desc
            )),
            source_url,
            source_publisher: Some(source_publisher.into()),
            external_id: fl_acct_external_id(&f.acct),
        });
    }
    out
}

pub fn office_code_label(code: &str) -> String {
    match code {
        "GOV" => "Governor".into(),
        "ATG" => "Attorney General".into(),
        "CFO" => "Chief Financial Officer".into(),
        "AGR" => "Commissioner of Agriculture".into(),
        "SCJ" => "Supreme Court Justice".into(),
        other => other.to_string(),
    }
}

/// County / municipal / special-district filings matched by DOS County column or office text.
pub fn map_local_or_special_filing(
    f: &DosFiling,
    geo: &GeoResolution,
    state_l: &str,
    county_key: &str,
    extra_jurisdictions: &mut Vec<ResolvedJurisdiction>,
) -> Option<(String, &'static str, String, bool, bool)> {
    if !filing_matches_county(f, county_key) {
        return None;
    }

    let (chamber, level) = classify_local_office(&f.office_code, &f.office_desc);
    let county_slug = slugify_simple(county_key);
    let county_ocd = format!("ocd-division/country:us/state:{state_l}/county:{county_slug}");
    let county_name = if geo.county.trim().is_empty() {
        format!("{} County", title_case_words(county_key))
    } else {
        geo.county.clone()
    };

    let (ocd, jur_name, jur_level) = match chamber {
        "municipal" => {
            let place = geo.city.trim();
            let place_slug = if place.is_empty() {
                "unknown".into()
            } else {
                slugify_simple(place)
            };
            let ocd = format!("ocd-division/country:us/state:{state_l}/place:{place_slug}");
            let name = if place.is_empty() {
                "Municipal".into()
            } else {
                place.to_string()
            };
            (ocd, name, "municipal")
        }
        "special_district" => {
            let dist = f
                .office_desc
                .split(['(', ')', ','])
                .map(str::trim)
                .find(|s| s.len() > 3)
                .unwrap_or("Special District");
            let slug = slugify_simple(dist);
            let ocd = format!(
                "ocd-division/country:us/state:{state_l}/county:{county_slug}/special_district:{slug}"
            );
            (ocd, dist.to_string(), "special_district")
        }
        _ => (county_ocd.clone(), county_name.clone(), level),
    };

    push_jur(extra_jurisdictions, &ocd, &jur_name, jur_level);
    if chamber != "municipal" {
        push_jur(extra_jurisdictions, &county_ocd, &county_name, "county");
    }

    let mut office = if f.office_desc.trim().is_empty() {
        office_code_label(&f.office_code)
    } else {
        f.office_desc.trim().to_string()
    };
    if let Some(d) = f.juris1 {
        if !office.to_ascii_lowercase().contains("district")
            && !office.to_ascii_lowercase().contains(&format!("group {d}"))
        {
            office = format!("{office} (District {d})");
        }
    }
    if let Some(g) = f.juris2 {
        if !office.to_ascii_lowercase().contains("group") {
            office = format!("{office} (Group {g})");
        }
    }

    Some((office, chamber, ocd, false, false))
}

pub fn filing_matches_county(f: &DosFiling, county_key: &str) -> bool {
    if county_key.is_empty() {
        return false;
    }
    let filing_county = normalize_county_key(&f.county);
    if !filing_county.is_empty() && filing_county == county_key {
        return true;
    }
    // Multi-county rows sometimes put the county only in OfficeDesc.
    let desc = f.office_desc.to_ascii_lowercase();
    let key = county_key.to_ascii_lowercase();
    if desc.contains(&key) {
        return true;
    }
    // e.g. county_key "miami dade" vs "miami-dade"
    let key_compact: String = key.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    let desc_compact: String = desc.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    !key_compact.is_empty() && desc_compact.contains(&key_compact)
}

pub fn classify_local_office(code: &str, desc: &str) -> (&'static str, &'static str) {
    let c = code.to_ascii_uppercase();
    let d = desc.to_ascii_lowercase();
    if matches!(
        c.as_str(),
        "MAY" | "MC" | "MCC" | "CCO" | "COU" | "CIT" | "CM"
    ) || d.contains("mayor")
        || d.contains("city council")
        || d.contains("city commission")
        || d.contains("town council")
        || d.contains("village")
    {
        return ("municipal", "municipal");
    }
    if matches!(
        c.as_str(),
        "CCM" | "CC" | "BCC" | "CLC" | "CLK" | "CL" | "SHC" | "SH" | "CPA" | "PA" | "CTC"
            | "TC" | "SOE" | "SE" | "SB" | "SBM" | "SBL" | "SCB"
    ) || d.contains("county commission")
        || d.contains("board of county")
        || d.contains("sheriff")
        || d.contains("clerk of")
        || d.contains("property appraiser")
        || d.contains("tax collector")
        || d.contains("supervisor of elections")
        || d.contains("school board")
        || d.contains("school district")
    {
        return ("county", "county");
    }
    if d.contains("special")
        || d.contains("authority")
        || d.contains("district")
        || d.contains("hospital")
        || d.contains("water")
        || d.contains("fire")
        || d.contains("community development")
        || matches!(c.as_str(), "SPE" | "SD" | "CDD" | "HSD" | "WMD" | "FIR")
    {
        return ("special_district", "special_district");
    }
    // Default local filings with a county to county level.
    ("county", "county")
}

pub fn slugify_simple(s: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('_');
            prev_dash = true;
        }
    }
    out.trim_matches('_').to_string()
}

pub fn push_jur(out: &mut Vec<ResolvedJurisdiction>, ocd: &str, name: &str, level: &str) {
    if out.iter().any(|j| j.ocd_id == ocd) {
        return;
    }
    out.push(ResolvedJurisdiction {
        ocd_id: ocd.to_string(),
        name: name.to_string(),
        level: level.to_string(),
        state: Some("FL".into()),
    });
}

pub fn names_match(a: &str, b: &str) -> bool {
    let na = normalize_name_key(a);
    let nb = normalize_name_key(b);
    if na.is_empty() || nb.is_empty() {
        return false;
    }
    if na == nb {
        return true;
    }
    // last-name + first-token match (handles missing middle)
    let pa: Vec<&str> = na.split_whitespace().collect();
    let pb: Vec<&str> = nb.split_whitespace().collect();
    if pa.len() >= 2 && pb.len() >= 2 {
        let la = pa.last().copied().unwrap_or("");
        let lb = pb.last().copied().unwrap_or("");
        return la == lb && pa[0] == pb[0];
    }
    false
}

pub fn normalize_name_key(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric() || c.is_ascii_whitespace())
        .collect::<String>()
        .split_whitespace()
        .map(|w| w.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join(" ")
}

// --- County → circuit / DCA ---

pub fn normalize_county_key(county: &str) -> String {
    county
        .trim()
        .trim_end_matches(" County")
        .trim_end_matches(" COUNTY")
        .to_ascii_lowercase()
        .replace('-', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Florida judicial circuit (1–20) by county name.
pub fn county_to_circuit(county: &str) -> Option<u32> {
    let c = county;
    let n = match c {
        "escambia" | "okaloosa" | "santa rosa" | "walton" => 1,
        "franklin" | "gadsden" | "jefferson" | "leon" | "liberty" | "wakulla" => 2,
        "columbia" | "dixie" | "hamilton" | "lafayette" | "madison" | "suwannee" | "taylor" => 3,
        "clay" | "duval" | "nassau" => 4,
        "citrus" | "hernando" | "lake" | "marion" | "sumter" => 5,
        "pasco" | "pinellas" => 6,
        "flagler" | "putnam" | "st johns" | "st. johns" | "volusia" => 7,
        "alachua" | "baker" | "bradford" | "gilchrist" | "levy" | "union" => 8,
        "orange" | "osceola" => 9,
        "hardee" | "highlands" | "polk" => 10,
        "miami dade" | "miami-dade" | "dade" => 11,
        "desoto" | "de soto" | "manatee" | "sarasota" => 12,
        "hillsborough" => 13,
        "bay" | "calhoun" | "gulf" | "holmes" | "jackson" | "washington" => 14,
        "palm beach" => 15,
        "monroe" => 16,
        "broward" => 17,
        "brevard" | "seminole" => 18,
        "indian river" | "martin" | "okeechobee" | "st lucie" | "st. lucie" => 19,
        "charlotte" | "collier" | "glades" | "hendry" | "lee" => 20,
        _ => return None,
    };
    Some(n)
}

/// DCA district from judicial circuit (current FL mapping).
pub fn circuit_to_dca(circuit: u32) -> Option<u32> {
    match circuit {
        1 | 2 | 3 | 4 | 8 | 14 => Some(1),
        6 | 10 | 12 | 13 | 20 => Some(2),
        11 | 16 => Some(3),
        15 | 17 | 19 => Some(4),
        5 | 7 | 9 | 18 => Some(5),
        _ => None,
    }
}

// --- Roster helpers (incumbent enrichment + fallback) ---

#[derive(Debug, Clone)]
pub struct Member {
    pub name: String,
    pub party: String,
    #[allow(dead_code)]
    pub district: u32,
    pub profile_url: String,
}

pub fn member_to_candidate(
    m: &Member,
    office: &str,
    chamber: &str,
    jurisdiction_ocd: &str,
    source_url: &str,
    publisher: &str,
) -> SnapshotCandidate {
    let party = normalize_party_label(&m.party);
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
            "Incumbent {office} ({party}). Source: {publisher}."
        )),
        source_url: source_url.to_string(),
        source_publisher: Some(publisher.into()),
        external_id: None,
    }
}

pub fn fl_measures_fallback(state_ocd: &str) -> Vec<SnapshotMeasure> {
    vec![SnapshotMeasure {
        title: "Florida constitutional amendments (see official list)".into(),
        measure_code: None,
        jurisdiction_ocd: state_ocd.to_string(),
        summary: Some(
            "Could not load the DOS constitutional initiatives list. Open the source for the current ballot amendments."
                .into(),
        ),
        source_url: MEASURES_URL.into(),
        source_publisher: Some(DOS_PUBLISHER.into()),
    }]
}



pub fn extract_verification_token(html: &str) -> Option<String> {
    let re = regex::Regex::new(
        r#"(?i)name\s*=\s*"__RequestVerificationToken"[^>]*value\s*=\s*"([^"]+)""#,
    )
    .ok()?;
    re.captures(html)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
        .or_else(|| {
            let re2 = regex::Regex::new(
                r#"(?i)value\s*=\s*"([^"]+)"[^>]*name\s*=\s*"__RequestVerificationToken""#,
            )
            .ok()?;
            re2.captures(html)
                .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
        })
}

#[derive(Debug, Clone)]
pub struct ParsedFlMeasure {
    pub title: String,
    pub measure_code: Option<String>,
    pub source_url: String,
    pub summary: Option<String>,
}

/// Parse filtered DOS constitutional initiatives table rows into ballot amendments.
pub fn parse_fl_measures_table(html: &str, cycle: i32) -> Vec<ParsedFlMeasure> {
    let row_re = match regex::Regex::new(r"(?is)<tr[^>]*>(.*?)</tr>") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let link_re = match regex::Regex::new(
        r#"(?is)href="((?:https://constitutionalinitiatives\.dos\.fl\.gov)?/Home/InitDetail\?account=[^"&]+&(?:amp;)?seqnum=\d+)"[^>]*>([^<]+)</a>"#,
    ) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let serial_re = match regex::Regex::new(r"\((\d+)\)") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let year_s = cycle.to_string();
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for row in row_re.captures_iter(html) {
        let row_html = row.get(1).map(|m| m.as_str()).unwrap_or("");
        if !row_html.contains("InitDetail") {
            continue;
        }
        // Defense: only keep rows that mention this cycle (e.g. "2026 GEN").
        if !row_html.contains(&year_s) {
            continue;
        }
        let Some(cap) = link_re.captures(row_html) else {
            continue;
        };
        let mut href = html_unescape(cap.get(1).map(|m| m.as_str()).unwrap_or("").trim());
        if href.starts_with("/Home/") {
            href = format!("https://constitutionalinitiatives.dos.fl.gov{href}");
        }
        href = href.replace("&amp;", "&");
        let title = html_unescape(cap.get(2).map(|m| m.as_str()).unwrap_or("").trim());
        if title.is_empty() || href.is_empty() {
            continue;
        }
        if !seen.insert(href.clone()) {
            continue;
        }
        let measure_code = serial_re
            .captures(row_html)
            .and_then(|c| c.get(1))
            .map(|m| format!("Amendment {}", m.as_str()));
        out.push(ParsedFlMeasure {
            title,
            measure_code,
            source_url: href,
            summary: None,
        });
    }

    out.sort_by(|a, b| {
        let an = a
            .measure_code
            .as_deref()
            .and_then(|c| c.rsplit(' ').next())
            .and_then(|n| n.parse::<u32>().ok())
            .unwrap_or(999);
        let bn = b
            .measure_code
            .as_deref()
            .and_then(|c| c.rsplit(' ').next())
            .and_then(|n| n.parse::<u32>().ok())
            .unwrap_or(999);
        an.cmp(&bn).then_with(|| a.title.cmp(&b.title))
    });
    out
}


pub fn fl_measure_detail_cache_key(detail_url: &str) -> String {
    let account = detail_url
        .split("account=")
        .nth(1)
        .and_then(|s| s.split(&['&', '#'][..]).next())
        .unwrap_or("x");
    let seq = detail_url
        .split("seqnum=")
        .nth(1)
        .and_then(|s| s.split(&['&', '#'][..]).next())
        .unwrap_or("x");
    format!("fl:measures:detail:{account}:{seq}")
}

/// Extract DOS committee `account` from InitDetail or ComDetail URL.
pub fn parse_dos_account_from_url(url: &str) -> Option<String> {
    let u = url.replace("&amp;", "&");
    let acct = u
        .split("account=")
        .nth(1)?
        .split(&['&', '#', '"', '\''][..])
        .next()?
        .trim();
    if acct.is_empty() {
        return None;
    }
    Some(acct.to_string())
}

/// Candidate Tracking / TreFin account → namespaced external_id (`fl:acct:88799`).
pub fn fl_acct_external_id(acct: &str) -> Option<String> {
    let a = acct.trim();
    if a.is_empty() || a == "0" {
        return None;
    }
    // Digit accounts only (reject placeholders).
    if !a.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(format!("fl:acct:{a}"))
}

/// Parse `fl:acct:{n}` → account number string.
pub fn parse_fl_acct_external_id(id: &str) -> Option<String> {
    let rest = id.trim().strip_prefix("fl:acct:")?;
    let a = rest.trim();
    if a.is_empty() || !a.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(a.to_string())
}

pub fn fl_com_detail_url(account: &str) -> String {
    format!("https://dos.elections.myflorida.com/committees/ComDetail.asp?account={account}")
}

/// Itemized contributions dump (amount-sorted) for a DOS committee account.
pub fn fl_trefin_contrib_url(account: &str) -> String {
    format!(
        "https://dos.elections.myflorida.com/cgi-bin/TreFin.exe?account={account}&seqnum=0&queryfor=1&queryorder=AMT&queryoutput=1"
    )
}

/// DOS committee name-search endpoint (POST form).
pub fn fl_com_lkup_by_name_url() -> &'static str {
    "https://dos.elections.myflorida.com/committees/ComLkupByName.asp"
}

/// `application/x-www-form-urlencoded` body for committee name search (containing).
pub fn fl_com_lkup_by_name_form(name: &str) -> String {
    // Index form uses comName; results form uses ComName — both accepted.
    format!(
        "searchtype=1&comName={}&LkupTypeName=C&NameSearchBtn=Search+by+Name",
        urlencoding_form(name)
    )
}

fn urlencoding_form(s: &str) -> String {
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FlCommitteeHit {
    pub account: String,
    pub name: String,
    pub committee_type: String,
    pub status: String,
}

/// Parse DOS `ComLkupByName.asp` HTML result table.
pub fn parse_com_lkup_by_name_html(html: &str) -> Vec<FlCommitteeHit> {
    let row_re = match regex::Regex::new(r"(?is)<tr[^>]*>(.*?)</tr>") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let link_re = match regex::Regex::new(
        r#"(?is)<a[^>]+HREF\s*=\s*["']ComDetail\.asp\?account=(\d+)["'][^>]*>(.*?)</a>"#,
    ) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let td_re = match regex::Regex::new(r"(?is)<td[^>]*>(.*?)</td>") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let tag_re = match regex::Regex::new(r"(?is)<[^>]+>") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for row in row_re.captures_iter(html) {
        let row_html = row.get(1).map(|m| m.as_str()).unwrap_or("");
        if !row_html.to_ascii_lowercase().contains("comdetail.asp") {
            continue;
        }
        let Some(cap) = link_re.captures(row_html) else {
            continue;
        };
        let account = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim().to_string();
        if account.is_empty() || !seen.insert(account.clone()) {
            continue;
        }
        let name = {
            let raw = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            let stripped = tag_re.replace_all(raw, " ");
            html_unescape(&stripped)
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        };
        if name.is_empty() {
            continue;
        }
        let tds: Vec<String> = td_re
            .captures_iter(row_html)
            .map(|c| {
                let raw = c.get(1).map(|m| m.as_str()).unwrap_or("");
                let stripped = tag_re.replace_all(raw, " ");
                html_unescape(&stripped)
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect();
        // Columns: Committee | Type | Status
        let committee_type = tds.get(1).cloned().unwrap_or_default();
        let status = tds.get(2).cloned().unwrap_or_default();
        out.push(FlCommitteeHit {
            account,
            name,
            committee_type,
            status,
        });
    }
    out
}

/// Amendment number from measure_code like `"Amendment 3"` / `"Amendment 10"`.
pub fn amendment_number_from_code(code: Option<&str>) -> Option<u32> {
    let c = code?.trim();
    let re = regex::Regex::new(r"(?i)\bamendment\s+#?(\d+)\b").ok()?;
    re.captures(c)
        .and_then(|cap| cap.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

/// Extract targeted amendment number from oppose-PAC style names (`No on 3`, `Vote No On #2`).
pub fn oppose_amendment_number_from_name(name: &str) -> Option<u32> {
    let re = regex::Regex::new(r"(?i)\bno\s+on\s+#?(\d+)\b").ok()?;
    re.captures(name)
        .and_then(|cap| cap.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

/// Pick oppose PACs for a ballot amendment from a DOS name-search hit list.
/// Prefers Active status, then PAC type; caps at `limit`.
pub fn select_oppose_committees(
    hits: &[FlCommitteeHit],
    amendment: u32,
    limit: usize,
) -> Vec<FlCommitteeHit> {
    let mut matched: Vec<FlCommitteeHit> = hits
        .iter()
        .filter(|h| oppose_amendment_number_from_name(&h.name) == Some(amendment))
        .cloned()
        .collect();
    matched.sort_by(|a, b| {
        status_rank(&a.status)
            .cmp(&status_rank(&b.status))
            .then_with(|| type_rank(&a.committee_type).cmp(&type_rank(&b.committee_type)))
            .then_with(|| a.name.cmp(&b.name))
    });
    matched.truncate(limit.max(1));
    matched
}

fn status_rank(s: &str) -> u8 {
    let u = s.trim().to_ascii_uppercase();
    if u.starts_with("ACTIVE") {
        0
    } else if u.starts_with("CLOSED") {
        2
    } else {
        1
    }
}

fn type_rank(t: &str) -> u8 {
    match t.trim().to_ascii_uppercase().as_str() {
        "PAC" => 0,
        "ECO" | "IXO" => 1,
        _ => 2,
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FlCommitteeFinance {
    pub account: String,
    pub contributions_sum: f64,
    pub contributions_sum_display: String,
    pub top_contributors: Vec<crate::models::ContributorRow>,
    pub line_count: usize,
    pub committee_url: String,
    pub trefin_url: String,
    pub note: String,
    /// Display name when known (oppose PAC search); empty for sponsor-only dumps.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub committee_name: String,
    /// `"sponsor"` or `"oppose"`.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub role: String,
}

/// Parse TreFin.exe contribution HTML (`<pre>` fixed-width or whitespace columns).
pub fn parse_trefin_contributions_html(html: &str, account: &str, limit: usize) -> FlCommitteeFinance {
    use crate::models::{format_usd, ContributorRow};
    use std::collections::HashMap;

    let text = {
        let re = regex::Regex::new(r"(?is)<pre[^>]*>(.*?)</pre>").ok();
        if let Some(re) = re {
            if let Some(c) = re.captures(html) {
                html_unescape(c.get(1).map(|m| m.as_str()).unwrap_or(""))
            } else {
                // Strip tags fallback
                let stripped = regex::Regex::new(r"(?is)<[^>]+>")
                    .ok()
                    .map(|r| r.replace_all(html, "\n").to_string())
                    .unwrap_or_else(|| html.to_string());
                html_unescape(&stripped)
            }
        } else {
            html.to_string()
        }
    };

    // Line shape (research sample):
    // 2024   Q2       05/13/2024    30,000.00 FLORIDA RIGHT TO LIFE          ...
    let line_re = regex::Regex::new(
        r"(?m)^\s*(\d{4})\s+\S+\s+(\d{2}/\d{2}/\d{4})\s+([0-9,]+\.\d{2})\s+(.+?)\s{2,}",
    )
    .ok();

    struct Agg {
        name: String,
        total: f64,
        count: u32,
        date: Option<String>,
    }
    let mut map: HashMap<String, Agg> = HashMap::new();
    let mut line_count = 0usize;
    let mut sum = 0.0f64;

    if let Some(re) = line_re {
        for cap in re.captures_iter(&text) {
            let date = cap.get(2).map(|m| m.as_str().to_string());
            let amount_s = cap.get(3).map(|m| m.as_str()).unwrap_or("0");
            let amount: f64 = amount_s.replace(',', "").parse().unwrap_or(0.0);
            if amount < 0.5 {
                continue;
            }
            let rest = cap.get(4).map(|m| m.as_str()).unwrap_or("").trim();
            // Contributor name is leading token run before address-like digits
            let name = rest
                .split_whitespace()
                .take_while(|t| {
                    !t.chars().next().is_some_and(|c| c.is_ascii_digit())
                        && !t.contains(',')
                })
                .collect::<Vec<_>>()
                .join(" ");
            let name = if name.len() < 2 {
                rest.chars().take(40).collect::<String>()
            } else {
                name
            };
            let name = name.trim().to_string();
            if name.is_empty() {
                continue;
            }
            line_count += 1;
            sum += amount;
            let key = name.to_ascii_uppercase();
            let e = map.entry(key).or_insert_with(|| Agg {
                name: name.clone(),
                total: 0.0,
                count: 0,
                date: date.clone(),
            });
            e.total += amount;
            e.count += 1;
            if let Some(ref d) = date {
                if e.date.as_ref().map(|x| d > x).unwrap_or(true) {
                    e.date = Some(d.clone());
                }
            }
        }
    }

    let mut rows: Vec<Agg> = map.into_values().collect();
    rows.sort_by(|a, b| {
        b.total
            .partial_cmp(&a.total)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.name.cmp(&b.name))
    });
    let top: Vec<ContributorRow> = rows
        .into_iter()
        .take(limit.max(1))
        .map(|a| ContributorRow {
            name: a.name,
            amount_display: format_usd(a.total),
            location: None,
            date: a.date,
            url: fl_com_detail_url(account),
            gift_count: if a.count > 1 { Some(a.count) } else { None },
        })
        .collect();

    let note = if line_count == 0 {
        "No itemized contribution lines parsed (legislature-referred measures often have empty PAC files, or TreFin HTML changed)."
            .into()
    } else {
        format!(
            "Sum of {line_count} itemized TreFin contribution line(s) — not a certified cash-on-hand total."
        )
    };

    FlCommitteeFinance {
        account: account.to_string(),
        contributions_sum: sum,
        contributions_sum_display: format_usd(sum),
        top_contributors: top,
        line_count,
        committee_url: fl_com_detail_url(account),
        trefin_url: fl_trefin_contrib_url(account),
        note,
        committee_name: String::new(),
        role: "sponsor".into(),
    }
}

/// Attach display metadata after TreFin parse (oppose PAC discovery).
pub fn fl_committee_finance_with_meta(
    mut fin: FlCommitteeFinance,
    committee_name: &str,
    role: &str,
) -> FlCommitteeFinance {
    if !committee_name.is_empty() {
        fin.committee_name = committee_name.to_string();
    }
    if !role.is_empty() {
        fin.role = role.to_string();
    }
    fin
}

// --- A2: FL DOS contrib.exe name-search fallback (when no fl:acct:*) ---

/// DOS campaign-finance contributions search endpoint (POST form).
pub fn fl_contrib_url() -> &'static str {
    "https://dos.elections.myflorida.com/cgi-bin/contrib.exe"
}

/// DOS candidate listing by election (POST `elecid`) — includes CanDetail account links.
pub fn fl_can_list_url() -> &'static str {
    "https://dos.elections.myflorida.com/candidates/CanList.asp"
}

/// Form body for CanList.asp (general election listing).
pub fn fl_can_list_form(elec_id: &str) -> String {
    format!(
        "elecid={}&GenSubmit=Submit",
        urlencoding_form(elec_id.trim())
    )
}

/// Candidate contribution-totals search (`search_on=3`, TSV).
///
/// Important: `party` must be `All` (empty party yields zero rows on live DOS).
pub fn fl_contrib_candidate_totals_form(
    election: &str,
    last_name: &str,
    first_name: &str,
    office_code: &str,
    district: &str,
) -> String {
    let office = {
        let o = office_code.trim();
        if o.is_empty() {
            "All"
        } else {
            o
        }
    };
    let nsrch = "2"; // last name starts with
    format!(
        "election={}&search_on=3&CanFName={}&CanLName={}&CanNameSrch={}&office={}&cdistrict={}&cgroup=&party=All&ComName=&ComNameSrch=2&committee=&cfname=&clname=&namesearch=2&ccity=&cstate=&czipcode=&coccupation=&cdollar_minimum=&cdollar_maximum=&rowlimit=100&csort1=NAM&csort2=CAN&cdatefrom=&cdateto=&queryformat=2&Submit=Submit",
        urlencoding_form(election.trim()),
        urlencoding_form(first_name.trim()),
        urlencoding_form(last_name.trim()),
        nsrch,
        urlencoding_form(office),
        urlencoding_form(district.trim()),
    )
}

/// One row from contrib.exe candidate totals TSV/HTML.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FlContribCandidateHit {
    pub name: String,
    pub party: String,
    /// DOS office code (`STS`, `STR`, `GOV`, `CTJ`, …).
    pub office_code: String,
    pub district: String,
    pub group: String,
    pub total_amount: f64,
    pub total_display: String,
}

/// Query fields used to disambiguate a ballot candidate against contrib/CanList hits.
#[derive(Debug, Clone, Default)]
pub struct FlNameSearchQuery {
    pub name: String,
    pub office: String,
    pub chamber: String,
    pub party: String,
    /// Numeric district when known (senate/house/circuit).
    pub district: Option<u32>,
    /// County key (normalized) when known — required match for local-only offices when hit has county.
    pub county: String,
}

/// Strict match outcome — never silently picks among multiple people.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FlNameMatch {
    /// Exactly one hit survived the rules.
    Unique { hit: FlContribCandidateHit },
    /// Zero hits after filters.
    None,
    /// More than one hit — caller must skip (no silent wrong person).
    Ambiguous { count: usize },
}

/// Parse contrib.exe candidate-totals tab file.
///
/// Header: `Candidate Name\tParty\tOffice\tDistrict\tGroup\tTotal Amount`
pub fn parse_contrib_candidate_totals_tsv(tsv: &str) -> Vec<FlContribCandidateHit> {
    let mut lines = tsv.lines();
    let Some(header) = lines.next() else {
        return Vec::new();
    };
    if !header.to_ascii_lowercase().contains("candidate name") {
        // Maybe no header (shouldn't happen) — still try body lines.
    }
    let mut out = Vec::new();
    for line in lines {
        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 6 {
            continue;
        }
        let name = cols[0].trim().to_string();
        if name.is_empty() {
            continue;
        }
        let party = cols[1].trim().to_string();
        let office_code = cols[2].trim().to_ascii_uppercase();
        let district = cols[3].trim().to_string();
        let group = cols[4].trim().to_string();
        let amount_raw = cols[5].trim().replace(',', "");
        let total_amount: f64 = amount_raw.parse().unwrap_or(0.0);
        out.push(FlContribCandidateHit {
            name,
            party,
            office_code,
            district,
            group,
            total_amount,
            total_display: crate::models::format_usd(total_amount),
        });
    }
    out
}

/// Parse contrib.exe candidate-totals HTML screen results (fallback when TSV unavailable).
pub fn parse_contrib_candidate_totals_html(html: &str) -> Vec<FlContribCandidateHit> {
    // Live HTML is a simple table after "Summary of Candidates"; prefer TSV in production.
    // Fallback: scan preformatted / table rows for name + office code patterns.
    if html.contains("Candidate Name") && html.contains('\t') {
        // Some gateways wrap TSV in HTML.
        if let Some(start) = html.find("Candidate Name") {
            return parse_contrib_candidate_totals_tsv(&html[start..]);
        }
    }
    let row_re = match regex::Regex::new(r"(?is)<tr[^>]*>(.*?)</tr>") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let td_re = match regex::Regex::new(r"(?is)<t[dh][^>]*>(.*?)</t[dh]>") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let tag_re = match regex::Regex::new(r"(?is)<[^>]+>") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    for row in row_re.captures_iter(html) {
        let row_html = row.get(1).map(|m| m.as_str()).unwrap_or("");
        let tds: Vec<String> = td_re
            .captures_iter(row_html)
            .map(|c| {
                let raw = c.get(1).map(|m| m.as_str()).unwrap_or("");
                let stripped = tag_re.replace_all(raw, " ");
                html_unescape(&stripped)
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect();
        if tds.len() < 6 {
            continue;
        }
        // Expect: Name | Party | Office | District | Group | Total
        let name = tds[0].trim().to_string();
        if name.is_empty() || name.eq_ignore_ascii_case("Candidate Name") {
            continue;
        }
        let office_code = tds[2].trim().to_ascii_uppercase();
        // Office column may be code or long label.
        let office_code = office_code_from_label(&office_code).unwrap_or(office_code);
        if office_code.len() > 6 && office_code_from_label(&office_code).is_none() {
            continue;
        }
        let amount_raw = tds[5].replace(['$', ','], "");
        let total_amount: f64 = amount_raw.trim().parse().unwrap_or(0.0);
        out.push(FlContribCandidateHit {
            name,
            party: tds[1].trim().to_string(),
            office_code,
            district: tds[3].trim().to_string(),
            group: tds[4].trim().to_string(),
            total_amount,
            total_display: crate::models::format_usd(total_amount),
        });
    }
    out
}

/// CanList / CTS row with DOS account (for TreFin after unique name match).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FlCanListHit {
    pub account: String,
    pub name: String,
    pub party: String,
    pub office_code: String,
    pub district: String,
    pub group: String,
    pub status: String,
}

/// Parse DOS `CanList.asp` HTML → candidates with CanDetail accounts.
pub fn parse_can_list_html(html: &str) -> Vec<FlCanListHit> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut office_code = String::new();

    // Section headers are bold labels before each results table.
    let section_re = match regex::Regex::new(
        r#"(?is)<b>\s*(United States Senator|United States Representative|Governor|Attorney General|Chief Financial Officer|Commissioner of Agriculture|Secretary of State|State Attorney|Public Defender|State Senator|State Representative|Supreme Court Justice|District Court of Appeal|Circuit Judge|County Commissioner|School Board|Mayor)[^<]*</b>"#,
    ) {
        Ok(r) => r,
        Err(_) => return out,
    };
    let row_re = match regex::Regex::new(r"(?is)<tr[^>]*>(.*?)</tr>") {
        Ok(r) => r,
        Err(_) => return out,
    };
    let link_re = match regex::Regex::new(
        r#"(?is)<a[^>]+href\s*=\s*["']CanDetail\.asp\?account=(\d+)["'][^>]*>(.*?)</a>"#,
    ) {
        Ok(r) => r,
        Err(_) => return out,
    };
    let tag_re = match regex::Regex::new(r"(?is)<[^>]+>") {
        Ok(r) => r,
        Err(_) => return out,
    };
    let party_re = match regex::Regex::new(r"\(([A-Z]{2,3})\)\s*$") {
        Ok(r) => r,
        Err(_) => return out,
    };

    // Walk the document in order: update office when a section header appears, then parse rows.
    let mut cursor = 0usize;
    let mut last_district = String::new();
    while cursor < html.len() {
        let rest = &html[cursor..];
        let next_section = section_re.find(rest).map(|m| (m.start(), m.end(), m.as_str()));
        let next_row = row_re.find(rest).map(|m| (m.start(), m.end(), m.as_str()));

        let take_section = match (&next_section, &next_row) {
            (Some((s, _, _)), Some((r, _, _))) => s < r,
            (Some(_), None) => true,
            _ => false,
        };

        if take_section {
            if let Some((_s, e, raw)) = next_section {
                let label = tag_re.replace_all(raw, " ");
                let label = html_unescape(&label)
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                if let Some(code) = office_code_from_label(&label) {
                    office_code = code;
                    last_district.clear();
                }
                cursor += e;
                continue;
            }
        }

        if let Some((_s, e, row_html)) = next_row {
            cursor += e;
            if office_code.is_empty() {
                continue;
            }
            if !row_html.to_ascii_lowercase().contains("candetail.asp") {
                continue;
            }
            // Collect all account links in row; name is concatenation of link texts.
            let mut account = String::new();
            let mut name_parts = Vec::new();
            for cap in link_re.captures_iter(row_html) {
                let acct = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
                if account.is_empty() {
                    account = acct.to_string();
                }
                let raw = cap.get(2).map(|m| m.as_str()).unwrap_or("");
                let t = tag_re.replace_all(raw, " ");
                let t = html_unescape(&t)
                    .replace('\u{a0}', " ")
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                if !t.is_empty() {
                    name_parts.push(t);
                }
            }
            if account.is_empty() || !seen.insert(account.clone()) {
                continue;
            }
            let name_joined = name_parts.join(" ");
            // "Gaetz , Don" / "Gaetz, Don J." style from split anchors.
            let name = normalize_canlist_display_name(&name_joined);
            if name.is_empty() {
                continue;
            }
            // Party from trailing (REP) outside links — look at full cell text.
            let cell_text = {
                let stripped = tag_re.replace_all(row_html, " ");
                html_unescape(&stripped)
                    .replace('\u{a0}', " ")
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
            };
            let party = party_re
                .captures(&cell_text)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            // District: first <td> often holds district number (may be &nbsp; for multi-cand).
            let tds: Vec<String> = regex::Regex::new(r"(?is)<td[^>]*>(.*?)</td>")
                .ok()
                .map(|re| {
                    re.captures_iter(row_html)
                        .map(|c| {
                            let raw = c.get(1).map(|m| m.as_str()).unwrap_or("");
                            let stripped = tag_re.replace_all(raw, " ");
                            html_unescape(&stripped)
                                .replace('\u{a0}', " ")
                                .split_whitespace()
                                .collect::<Vec<_>>()
                                .join(" ")
                        })
                        .collect()
                })
                .unwrap_or_default();
            let district_cell = tds.first().map(|s| s.trim().to_string()).unwrap_or_default();
            let district = if district_cell.chars().all(|c| c.is_ascii_digit())
                && !district_cell.is_empty()
            {
                last_district = district_cell.clone();
                district_cell
            } else {
                last_district.clone()
            };
            let status = tds.get(2).cloned().unwrap_or_default();

            // Circuit Judge tables may use District/Group columns differently — leave group empty unless present.
            let group = String::new();

            out.push(FlCanListHit {
                account,
                name,
                party,
                office_code: office_code.clone(),
                district,
                group,
                status,
            });
            continue;
        }
        break;
    }
    out
}

fn normalize_canlist_display_name(raw: &str) -> String {
    // "Gaetz , Don" / "Gaetz, Don J." → "Don J. Gaetz" (ballot-style first-last).
    let s = raw
        .replace('\u{a0}', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let s = s.replace(" ,", ",").replace(", ,", ",");
    if let Some((last, rest)) = s.split_once(',') {
        let last = last.trim();
        let rest = rest.trim().trim_matches(',').trim();
        // Strip trailing party if glued
        let rest = regex::Regex::new(r"\s*\([A-Z]{2,3}\)\s*$")
            .ok()
            .map(|re| re.replace(rest, "").to_string())
            .unwrap_or_else(|| rest.to_string());
        let rest = rest.trim();
        if !last.is_empty() && !rest.is_empty() {
            return format_fl_name(&format!("{rest} {last}"));
        }
        if !last.is_empty() {
            return format_fl_name(last);
        }
    }
    format_fl_name(&s)
}

/// Map office label or code → DOS short code.
pub fn office_code_from_label(label: &str) -> Option<String> {
    let u = label.trim().to_ascii_uppercase();
    if u.is_empty() {
        return None;
    }
    // Already a code?
    if matches!(
        u.as_str(),
        "STS"
            | "STR"
            | "GOV"
            | "ATG"
            | "CFO"
            | "AGR"
            | "SEC"
            | "USS"
            | "USR"
            | "STA"
            | "PUB"
            | "SCJ"
            | "DCA"
            | "CTJ"
            | "CCM"
            | "SB"
            | "MAY"
    ) {
        return Some(u);
    }
    let s = u.as_str();
    let code = if s.contains("UNITED STATES SENATOR") || s == "US SENATOR" {
        "USS"
    } else if s.contains("UNITED STATES REPRESENTATIVE") || s.contains("US REPRESENTATIVE") {
        "USR"
    } else if s.contains("STATE SENATOR") {
        "STS"
    } else if s.contains("STATE REPRESENTATIVE") {
        "STR"
    } else if s.contains("GOVERNOR") {
        "GOV"
    } else if s.contains("ATTORNEY GENERAL") {
        "ATG"
    } else if s.contains("CHIEF FINANCIAL") {
        "CFO"
    } else if s.contains("COMMISSIONER OF AGRICULTURE")
        || (s.contains("AGRICULTURE") && s.contains("COMMISSIONER"))
    {
        "AGR"
    } else if s.contains("SECRETARY OF STATE") {
        "SEC"
    } else if s.contains("STATE ATTORNEY") {
        "STA"
    } else if s.contains("PUBLIC DEFENDER") {
        "PUB"
    } else if s.contains("SUPREME COURT") {
        "SCJ"
    } else if s.contains("DISTRICT COURT OF APPEAL") || s.contains("COURT OF APPEAL") {
        "DCA"
    } else if s.contains("CIRCUIT JUDGE") {
        "CTJ"
    } else if s.contains("COUNTY JUDGE") {
        "COJ"
    } else if s.contains("COUNTY COMMISSION") {
        "CCM"
    } else if s.contains("SCHOOL BOARD") {
        "SB"
    } else if s.contains("MAYOR") {
        "MAY"
    } else {
        return None;
    };
    Some(code.into())
}

/// Infer DOS office code from ballot chamber + office title.
pub fn office_code_from_ballot(chamber: &str, office: &str) -> Option<String> {
    if let Some(c) = office_code_from_label(office) {
        return Some(c);
    }
    match chamber.trim() {
        "state_senate" => Some("STS".into()),
        "state_house" => Some("STR".into()),
        "statewide" | "state_exec" => office_code_from_label(office).or_else(|| {
            let o = office.to_ascii_lowercase();
            if o.contains("governor") {
                Some("GOV".into())
            } else if o.contains("attorney general") {
                Some("ATG".into())
            } else if o.contains("chief financial") || o.contains("cfo") {
                Some("CFO".into())
            } else if o.contains("agriculture") {
                Some("AGR".into())
            } else {
                None
            }
        }),
        "judicial" => {
            let o = office.to_ascii_lowercase();
            if o.contains("circuit") {
                Some("CTJ".into())
            } else if o.contains("county judge") || o.starts_with("county judge") {
                Some("COJ".into())
            } else if o.contains("appeal") {
                Some("DCA".into())
            } else if o.contains("supreme") {
                Some("SCJ".into())
            } else if o.contains("state attorney") {
                Some("STA".into())
            } else if o.contains("public defender") {
                Some("PUB".into())
            } else if o.contains("county") && o.contains("judge") {
                Some("COJ".into())
            } else {
                None
            }
        }
        "county" => {
            let o = office.to_ascii_lowercase();
            if o.contains("school") {
                Some("SB".into())
            } else if o.contains("commission") {
                Some("CCM".into())
            } else {
                None
            }
        }
        "municipal" => {
            if office.to_ascii_lowercase().contains("mayor") {
                Some("MAY".into())
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Split display name into (first, last) for DOS last-name search.
pub fn split_candidate_first_last(name: &str) -> (String, String) {
    let n = strip_name_nicknames(name);
    let parts: Vec<&str> = n.split_whitespace().filter(|p| !is_name_suffix(p)).collect();
    if parts.is_empty() {
        return (String::new(), String::new());
    }
    if parts.len() == 1 {
        return (String::new(), parts[0].to_string());
    }
    let last = parts[parts.len() - 1].to_string();
    let first = parts[..parts.len() - 1].join(" ");
    (first, last)
}

fn strip_name_nicknames(name: &str) -> String {
    let re = regex::Regex::new(r#"["']([^"']+)["']"#).ok();
    let s = if let Some(re) = re {
        re.replace_all(name, " ").to_string()
    } else {
        name.to_string()
    };
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn is_name_suffix(tok: &str) -> bool {
    matches!(
        tok.trim_end_matches('.').to_ascii_uppercase().as_str(),
        "JR" | "SR" | "II" | "III" | "IV" | "V" | "ESQ"
    )
}

/// Normalize person name for equality: uppercase, strip punct/nicknames/suffixes.
pub fn normalize_person_name(name: &str) -> String {
    let s = strip_name_nicknames(name);
    let s = s
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c.is_whitespace() {
                c
            } else if c == '-' {
                ' '
            } else {
                ' '
            }
        })
        .collect::<String>();
    let parts: Vec<String> = s
        .split_whitespace()
        .filter(|p| !is_name_suffix(p))
        .map(|p| p.to_ascii_uppercase())
        .collect();
    parts.join(" ")
}

fn name_tokens(name: &str) -> Vec<String> {
    normalize_person_name(name)
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

/// Last-name equality after normalization.
pub fn last_names_match(a: &str, b: &str) -> bool {
    let ta = name_tokens(a);
    let tb = name_tokens(b);
    match (ta.last(), tb.last()) {
        (Some(x), Some(y)) => x == y,
        _ => false,
    }
}

/// First-name compatibility: equal, or one is a prefix initial of the other.
pub fn first_names_compatible(a: &str, b: &str) -> bool {
    let ta = name_tokens(a);
    let tb = name_tokens(b);
    if ta.len() < 2 || tb.len() < 2 {
        // Only last name known on one side — allow (last already matched separately).
        return true;
    }
    let fa = &ta[..ta.len() - 1];
    let fb = &tb[..tb.len() - 1];
    if fa == fb {
        return true;
    }
    // Compare first tokens with initial tolerance: "J" vs "JOHN", "JOHN" vs "JOHNNY" not ok
    let a0 = fa[0].as_str();
    let b0 = fb[0].as_str();
    if a0 == b0 {
        return true;
    }
    if a0.len() == 1 && b0.starts_with(a0) {
        return true;
    }
    if b0.len() == 1 && a0.starts_with(b0) {
        return true;
    }
    false
}

fn parse_district_num(raw: &str) -> Option<u32> {
    let t = raw.trim();
    if t.is_empty() {
        return None;
    }
    // "003", "3", "District 3"
    let re = regex::Regex::new(r"(\d+)").ok()?;
    re.captures(t)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

/// Extract district number from ballot office text / OCD.
pub fn district_from_ballot_office(office: &str) -> Option<u32> {
    let re = regex::Regex::new(r"(?i)\b(?:district|circuit)\s+#?(\d+)\b").ok()?;
    re.captures(office)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

fn office_families_match(query_code: Option<&str>, hit_code: &str) -> bool {
    let Some(q) = query_code.map(|s| s.trim().to_ascii_uppercase()) else {
        // No office filter — allow any (name+district must carry disambiguation).
        return true;
    };
    let h = hit_code.trim().to_ascii_uppercase();
    if q.is_empty() || q == "ALL" {
        return true;
    }
    if q == h {
        return true;
    }
    // Soft family groups
    fn fam(c: &str) -> String {
        match c {
            "STS" => "senate".into(),
            "STR" => "house".into(),
            "USS" => "ussenate".into(),
            "USR" => "ushouse".into(),
            "GOV" | "ATG" | "CFO" | "AGR" | "SEC" => "statewide".into(),
            "CTJ" | "COJ" | "DCA" | "SCJ" | "STA" | "PUB" => "judicial".into(),
            "CCM" | "SB" | "MAY" => "local".into(),
            other => other.to_string(),
        }
    }
    fam(&q) == fam(&h)
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

/// Strict match rules for A2:
/// 1. Last name equal (normalized)
/// 2. First name compatible (equal / initial); missing first on either side OK
/// 3. Office family match when query has an office code
/// 4. District equal when both sides have a district number
/// 5. Major-party conflict (DEM vs REP) rejects
/// 6. 0 hits → None; 1 → Unique; >1 → Ambiguous (caller must skip)
pub fn match_fl_contrib_candidate(
    hits: &[FlContribCandidateHit],
    q: &FlNameSearchQuery,
) -> FlNameMatch {
    let q_office = office_code_from_ballot(&q.chamber, &q.office);
    let q_dist = q.district.or_else(|| district_from_ballot_office(&q.office));

    let matched: Vec<&FlContribCandidateHit> = hits
        .iter()
        .filter(|h| last_names_match(&q.name, &h.name))
        .filter(|h| first_names_compatible(&q.name, &h.name))
        .filter(|h| office_families_match(q_office.as_deref(), &h.office_code))
        .filter(|h| {
            match (q_dist, parse_district_num(&h.district)) {
                (Some(want), Some(got)) => want == got,
                (Some(_), None) => {
                    // Query has district but hit doesn't — only OK for non-districted offices.
                    matches!(
                        h.office_code.as_str(),
                        "GOV" | "ATG" | "CFO" | "AGR" | "SEC" | "SCJ" | "USS"
                    )
                }
                (None, _) => true,
            }
        })
        .filter(|h| !parties_conflict(&q.party, &h.party))
        .collect();

    match matched.len() {
        0 => FlNameMatch::None,
        1 => FlNameMatch::Unique {
            hit: matched[0].clone(),
        },
        n => FlNameMatch::Ambiguous { count: n },
    }
}

/// Same rules against CanList hits; returns unique account or nothing on ambiguity.
pub fn match_fl_can_list_account(hits: &[FlCanListHit], q: &FlNameSearchQuery) -> Option<String> {
    let q_office = office_code_from_ballot(&q.chamber, &q.office);
    let q_dist = q.district.or_else(|| district_from_ballot_office(&q.office));

    let matched: Vec<&FlCanListHit> = hits
        .iter()
        .filter(|h| last_names_match(&q.name, &h.name))
        .filter(|h| first_names_compatible(&q.name, &h.name))
        .filter(|h| office_families_match(q_office.as_deref(), &h.office_code))
        .filter(|h| match (q_dist, parse_district_num(&h.district)) {
            (Some(want), Some(got)) => want == got,
            (Some(_), None) => matches!(
                h.office_code.as_str(),
                "GOV" | "ATG" | "CFO" | "AGR" | "SEC" | "SCJ" | "USS"
            ),
            (None, _) => true,
        })
        .filter(|h| !parties_conflict(&q.party, &h.party))
        .collect();

    if matched.len() == 1 {
        Some(matched[0].account.clone())
    } else {
        None
    }
}

/// Convert a unique contrib hit + optional resolved account into a JSON-friendly result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FlNameSearchResult {
    pub status: String, // unique | none | ambiguous | no_account
    pub account: Option<String>,
    pub hit: Option<FlContribCandidateHit>,
    pub match_count: usize,
    pub note: String,
}

pub fn fl_name_search_result(
    match_out: &FlNameMatch,
    account: Option<String>,
) -> FlNameSearchResult {
    match match_out {
        FlNameMatch::None => FlNameSearchResult {
            status: "none".into(),
            account: None,
            hit: None,
            match_count: 0,
            note: "No FL DOS campaign-finance row matched name+office+district.".into(),
        },
        FlNameMatch::Ambiguous { count } => FlNameSearchResult {
            status: "ambiguous".into(),
            account: None,
            hit: None,
            match_count: *count,
            note: format!(
                "Ambiguous FL DOS name match ({count} rows) — skipped to avoid wrong person."
            ),
        },
        FlNameMatch::Unique { hit } => {
            if let Some(a) = account.filter(|a| !a.is_empty() && a != "0") {
                FlNameSearchResult {
                    status: "unique".into(),
                    account: Some(a),
                    hit: Some(hit.clone()),
                    match_count: 1,
                    note: "Unique FL DOS name match; account resolved for TreFin.".into(),
                }
            } else {
                FlNameSearchResult {
                    status: "no_account".into(),
                    account: None,
                    hit: Some(hit.clone()),
                    match_count: 1,
                    note: "Unique FL DOS name match but no CanDetail account resolved.".into(),
                }
            }
        }
    }
}

pub fn parse_fl_measure_summary_html(html: &str) -> Option<String> {
    // Prefer the Summary <dd> body text (skip PDF-only links).
    let re = regex::Regex::new(
        r#"(?is)<dt[^>]*>\s*Summary\s*</dt>\s*<dd[^>]*>(.*?)</dd>"#,
    )
    .ok()?;
    let block = re.captures(html)?.get(1)?.as_str();
    // Strip tags, keep text paragraphs.
    let text_re = regex::Regex::new(r"(?is)<p[^>]*>(.*?)</p>").ok()?;
    let mut parts = Vec::new();
    for cap in text_re.captures_iter(block) {
        let inner = cap.get(1)?.as_str();
        let stripped = regex::Regex::new(r"(?is)<[^>]+>")
            .ok()?
            .replace_all(inner, " ");
        let t = html_unescape(&stripped);
        let t = t.split_whitespace().collect::<Vec<_>>().join(" ");
        if t.len() < 40 {
            continue;
        }
        if t.to_ascii_lowercase().contains("view full text") {
            continue;
        }
        parts.push(t);
    }
    if parts.is_empty() {
        // Fallback: first long "Proposing…" sentence on the page.
        let prop = regex::Regex::new(r"(?i)(Proposing[^.<]{30,600}\.)").ok()?;
        let t = html_unescape(prop.captures(html)?.get(1)?.as_str());
        let t = t.split_whitespace().collect::<Vec<_>>().join(" ");
        if t.len() >= 40 {
            return Some(t);
        }
        return None;
    }
    Some(parts.join(" "))
}


pub fn parse_senate_roster(html: &str) -> HashMap<u32, Member> {
    let mut out = HashMap::new();
    let re = match regex::Regex::new(
        r#"(?is)href="(/Senators/2024-2026/S(\d+))"[^>]*>\s*<img[^>]*alt="Senator\s+([^"]+)"[^>]*>[\s\S]*?</a>[\s\S]*?<td[^>]*>\s*(\d+)\s*</td>\s*<td[^>]*>\s*(Republican|Democrat|Democratic|Independent)\s*</td>"#,
    ) {
        Ok(r) => r,
        Err(_) => return out,
    };

    for cap in re.captures_iter(html) {
        let path = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let name = cap
            .get(3)
            .map(|m| html_unescape(m.as_str().trim()))
            .unwrap_or_default();
        let district: u32 = cap
            .get(4)
            .and_then(|m| m.as_str().parse().ok())
            .or_else(|| cap.get(2).and_then(|m| m.as_str().parse().ok()))
            .unwrap_or(0);
        let party = cap
            .get(5)
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "Unknown".into());
        if district == 0 || name.is_empty() {
            continue;
        }
        out.entry(district).or_insert(Member {
            name: strip_senator_prefix(&name),
            party,
            district,
            profile_url: format!("https://www.flsenate.gov{path}"),
        });
    }

    if out.len() < 20 {
        let loose = regex::Regex::new(
            r#"(?is)href="(/Senators/2024-2026/S(\d+)(?:/[^"]*)?)"[^>]*>(?:<img[^>]*alt="Senator\s+([^"]+)"[^>]*>)?\s*([^<]{0,80})</a>\s*<span>\s*-\s*District\s+(\d+),\s*(Republican|Democrat|Democratic)"#,
        )
        .ok();
        if let Some(re) = loose {
            for cap in re.captures_iter(html) {
                let district: u32 = cap
                    .get(5)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(0);
                if district == 0 || out.contains_key(&district) {
                    continue;
                }
                let name = cap
                    .get(3)
                    .map(|m| m.as_str())
                    .filter(|s| !s.is_empty())
                    .or_else(|| cap.get(4).map(|m| m.as_str()))
                    .unwrap_or("")
                    .trim();
                if name.is_empty() {
                    continue;
                }
                let path = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                out.insert(
                    district,
                    Member {
                        name: strip_senator_prefix(&html_unescape(name)),
                        party: cap
                            .get(6)
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_else(|| "Unknown".into()),
                        district,
                        profile_url: format!("https://www.flsenate.gov{path}"),
                    },
                );
            }
        }
    }

    out
}

pub fn parse_house_roster(html: &str) -> HashMap<u32, Member> {
    let mut out = HashMap::new();
    let re = match regex::Regex::new(
        r#"(?is)href="([^"]*details\.aspx\?MemberId=(\d+)[^"]*)"[\s\S]{0,200}?aria-label="Click to Representative\s+([^"]+)"[\s\S]{0,900}?(Republican|Democrat|Democratic|Independent)\s*(?:&mdash;|—|-)\s*<span[^>]*>District:\s*(\d+)</span>"#,
    ) {
        Ok(r) => r,
        Err(_) => return out,
    };

    for cap in re.captures_iter(html) {
        let path = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let name = html_unescape(cap.get(3).map(|m| m.as_str().trim()).unwrap_or("").trim());
        let party = cap
            .get(4)
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "Unknown".into());
        let district: u32 = cap
            .get(5)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0);
        if district == 0 || name.is_empty() {
            continue;
        }
        let profile_url = if path.starts_with("http") {
            path.to_string()
        } else if path.starts_with('/') {
            format!("https://www.flhouse.gov{path}")
        } else {
            format!("https://www.flhouse.gov/{path}")
        };
        out.entry(district).or_insert(Member {
            name: format_fl_name(&name),
            party,
            district,
            profile_url,
        });
    }
    out
}

pub fn strip_senator_prefix(name: &str) -> String {
    let n = name.trim();
    let n = n
        .strip_prefix("Senator ")
        .or_else(|| n.strip_prefix("Sen. "))
        .unwrap_or(n);
    format_fl_name(n)
}

pub fn format_fl_name(raw: &str) -> String {
    let cleaned = html_unescape(raw);
    if let Some((last, rest)) = cleaned.split_once(',') {
        let first = rest.trim();
        let last = last.trim();
        title_case_words(&format!("{first} {last}"))
    } else {
        title_case_words(&cleaned)
    }
}

pub fn title_case_words(s: &str) -> String {
    s.split_whitespace()
        .map(|w| {
            let bare = w.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '\'');
            if bare.len() <= 3 && bare.chars().all(|c| c.is_ascii_alphabetic()) {
                let upper = bare.to_ascii_uppercase();
                if matches!(upper.as_str(), "II" | "III" | "IV" | "JR" | "SR" | "J.") {
                    return w.to_string();
                }
            }
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + &c.as_str().to_ascii_lowercase(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn html_unescape(s: &str) -> String {
    s.replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&mdash;", "—")
        .replace("&nbsp;", " ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const SAMPLE_TSV: &str = "\
AcctNum\tVoterID\tElectionID\tOfficeCode\tOfficeDesc\tJuris1num\tJuris2num\tStatusCode\tStatusDesc\tPartyCode\tPartyDesc\tNameLast\tNameFirst\tNameMiddle\tSuppressAddress\tAddr1\tAddr2\tCity\tState\tZip\tCounty\tPhone\tTrsNameLast\tTrsNameFirst\tTrsNameMiddle\tEmail
88799\t0\t20261103-GEN\tSTS\tState Senator\t008\t\tQUA\tQualified\tREP\tRepublican Party of Florida\tJohansson\tJake\t\tN\t\t\t\tFL\t\t\t\t\t\t\t
89602\t0\t20261103-GEN\tSTS\tState Senator\t008\t\tQUA\tQualified\tDEM\tFlorida Democratic Party\tNgying\tJudy\tM.\tN\t\t\t\tFL\t\t\t\t\t\t\t
89632\t0\t20261103-GEN\tSTS\tState Senator\t008\t\tQUA\tQualified\tLPF\tLibertarian Party of Florida\tWozniak\tGabriel\tJude\tN\t\t\t\tFL\t\t\t\t\t\t\t
90845\t0\t20261103-GEN\tSTR\tState Representative\t030\t\tQUA\tQualified\tDEM\tFlorida Democratic Party\tMunoz\tEdwin\t\tN\t\t\t\tFL\t\t\t\t\t\t\t
88777\t0\t20261103-GEN\tSTR\tState Representative\t030\t\tQUA\tQualified\tREP\tRepublican Party of Florida\tTramont\tChase\t\tN\t\t\t\tFL\t\t\t\t\t\t\t
90433\t0\t20261103-GEN\tGOV\tGovernor\t\t\tQUA\tQualified\tNPA\tNo Party Affiliation\tAbrams\tDean\tOcean\tN\t\t\t\tFL\t\t\t\t\t\t\t
89119\t0\t20261103-GEN\tUSS\tUnited States Senator\t\t\tQUA\tQualified\tREP\tRepublican\tMoody\tAshley\t\tN\t\t\t\tFL\t\t\t\t\t\t\t
99999\t0\t20261103-GEN\tSTS\tState Senator\t008\t\tDNQ\tDid Not Qualify\tREP\tRepublican\tVoelz\tJason\t\tN\t\t\t\tFL\t\t\t\t\t\t\t
90108\t0\t20261103-GEN\tCTJ\tCircuit Judge\t018\t013\tQUA\tQualified\tNOP\tNo Party\tExample\tJudge\tA\tN\t\t\t\tFL\t\t\t\t\t\t\t
90109\t0\t20261103-GEN\tCTJ\tCircuit Judge\t018\t002\tUNO\tUnopposed\tNOP\tNo Party\tChase\tMelanie\t\tN\t\t\t\tFL\t\tSeminole\t\t\t\t\t
90110\t0\t20261103-GEN\tCOJ\tCounty Judge\t\t003\tQUA\tQualified\tNOP\tNo Party\tEdwards\tRodney\t\tN\t\t\t\tFL\t\tBrevard\t\t\t\t\t
90111\t0\t20261103-GEN\tCOJ\tCounty Judge\t\t003\tQUA\tQualified\tNOP\tNo Party\tTucker\tTimi\t\tN\t\t\t\tFL\t\tBrevard\t\t\t\t\t
90112\t0\t20261103-GEN\tCOJ\tCounty Judge\t\t001\tUNO\tUnopposed\tNOP\tNo Party\tSkip\tMe\t\tN\t\t\t\tFL\t\tBrevard\t\t\t\t\t
90001\t0\t20261103-GEN\tDCA\tDistrict Court of Appeal\t005\t\tQUA\tQualified\tNOP\tNo Party\tAppeals\tPat\t\tN\t\t\t\tFL\t\t\t\t\t\t\t
91001\t0\t20261103-GEN\tCCM\tCounty Commissioner\t002\t\tQUA\tQualified\tREP\tRepublican Party of Florida\tSmith\tAlice\t\tN\t\t\t\tFL\t\tBrevard\t\t\t\t\t
91002\t0\t20261103-GEN\tSB\tSchool Board Member\t003\t\tQUA\tQualified\tNPA\tNo Party Affiliation\tJones\tBob\t\tN\t\t\t\tFL\t\tBrevard\t\t\t\t\t
91003\t0\t20261103-GEN\tMAY\tMayor\t\t\tQUA\tQualified\tDEM\tFlorida Democratic Party\tLee\tCasey\t\tN\t\t\t\tFL\t\tBrevard\t\t\t\t\t
91004\t0\t20261103-GEN\tSPE\tWater Control District Supervisor\t\t\tQUA\tQualified\tNPA\tNo Party\tRiver\tPat\t\tN\t\t\t\tFL\t\tBrevard\t\t\t\t\t
91005\t0\t20261103-GEN\tCCM\tCounty Commissioner\t001\t\tQUA\tQualified\tREP\tRepublican\tOther\tCounty\t\tN\t\t\t\tFL\t\tOrange\t\t\t\t\t
91006\t0\t20261103-GEN\tCCM\tCounty Commissioner\t004\t\tUNO\tUnopposed\tREP\tRepublican\tUnopposed\tLocal\t\tN\t\t\t\tFL\t\tBrevard\t\t\t\t\t
";

    #[test]
    fn fl_gen_elec_id_2026() {
        assert_eq!(fl_gen_elec_id(2026), "20261103-GEN");
    }

    #[test]
    fn parse_dos_filters_status_and_federal() {
        let filings = parse_dos_tsv(SAMPLE_TSV);
        let names: HashSet<_> = filings.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains("Jake Johansson"));
        assert!(names.contains("Judy M. Ngying"));
        assert!(names.contains("Dean Ocean Abrams"));
        assert!(names.contains("Judge A Example"));
        assert!(!names.iter().any(|n| n.contains("Moody")), "USS skipped");
        assert!(!names.iter().any(|n| n.contains("Voelz")), "DNQ skipped");
        assert_eq!(filings.iter().filter(|f| f.office_code == "STS").count(), 3);
    }

    #[test]
    fn map_filings_zip_style_geo() {
        let filings = parse_dos_tsv(SAMPLE_TSV);
        let geo = GeoResolution {
            state: "FL".into(),
            state_name: "Florida".into(),
            county: "Brevard County".into(),
            city: "Melbourne".into(),
            congressional_district: "FL-8".into(),
            state_senate_district: Some(8),
            state_house_district: Some(30),
            state_house_label: Some("30".into()),
            latitude: None,
            longitude: None,
            jurisdictions: vec![],
            source_url: String::new(),
            source_publisher: String::new(),
        };
        let mut extra = Vec::new();
        let mut senators = HashMap::new();
        senators.insert(
            8,
            Member {
                name: "Someone Else".into(),
                party: "Republican".into(),
                district: 8,
                profile_url: String::new(),
            },
        );
        let mut house = HashMap::new();
        house.insert(
            30,
            Member {
                name: "Chase Tramont".into(),
                party: "Republican".into(),
                district: 30,
                profile_url:
                    "https://www.flhouse.gov/Sections/Representatives/details.aspx?MemberId=999"
                        .into(),
            },
        );
        let cands = map_filings_for_geo(
            &filings,
            &geo,
            "ocd-division/country:us/state:fl",
            &senators,
            &house,
            &mut extra,
        );
        let senate: Vec<_> = cands
            .iter()
            .filter(|c| c.office.contains("Senate"))
            .collect();
        assert_eq!(senate.len(), 3);
        // F8: incumbent house row prefers chamber profile URL for dossier bio.
        let tramont = cands
            .iter()
            .find(|c| c.name.contains("Tramont"))
            .expect("Tramont");
        assert!(tramont.is_incumbent);
        assert!(tramont.source_url.contains("flhouse.gov"));
        let house_c: Vec<_> = cands
            .iter()
            .filter(|c| c.office.contains("House"))
            .collect();
        assert_eq!(house_c.len(), 2);
        assert!(house_c.iter().any(|c| c.is_incumbent && c.name.contains("Tramont")));
        assert!(cands.iter().any(|c| c.office == "Governor"));
        assert!(cands.iter().any(|c| c.office.contains("Circuit Judge") && c.office.contains("Group 13")));
        assert!(cands.iter().any(|c| c.office.contains("District Court of Appeal")));
        // UNO seats still listed (collapsed in UI) so voters see who takes the bench
        assert!(cands.iter().any(|c| c.name.contains("Melanie Chase") && c.office.contains("Group 2")));
        assert!(cands.iter().any(|c| c.name.contains("Me Skip") && c.office.contains("Group 1")));
        assert!(cands.iter().any(|c| c.name.contains("Local Unopposed")));
        // Contested county judge group (sample primary: Group 3) + UNO Group 1
        let coj: Vec<_> = cands
            .iter()
            .filter(|c| c.office.contains("County Judge"))
            .collect();
        assert_eq!(coj.len(), 3);
        assert!(coj.iter().all(|c| c.is_judge && c.chamber.as_deref() == Some("judicial")));
        assert!(coj.iter().any(|c| c.name.contains("Rodney Edwards") && c.office.contains("Group 3")));
        assert!(coj.iter().any(|c| c.name.contains("Timi Tucker")));
        assert!(extra.iter().any(|j| j.ocd_id.contains("circuit:18")));
        assert!(extra.iter().any(|j| j.ocd_id.contains("court_of_appeal:5")));
        // Local / special matched to Brevard only
        assert!(cands.iter().any(|c| c.name.contains("Alice Smith") && c.chamber.as_deref() == Some("county")));
        assert!(cands.iter().any(|c| c.name.contains("Bob Jones") && c.office.contains("School Board")));
        assert!(cands.iter().any(|c| c.name.contains("Casey Lee") && c.chamber.as_deref() == Some("municipal")));
        assert!(cands.iter().any(|c| c.name.contains("Pat River") && c.chamber.as_deref() == Some("special_district")));
        assert!(!cands.iter().any(|c| c.name.contains("County Other")), "other county excluded");
        // A1: DOS AcctNum → fl:acct:* for TreFin detail finance
        let jake = cands.iter().find(|c| c.name.contains("Johansson")).expect("jake");
        assert_eq!(jake.external_id.as_deref(), Some("fl:acct:88799"));
        let judge = cands.iter().find(|c| c.name.contains("Judge A")).expect("judge");
        assert_eq!(judge.external_id.as_deref(), Some("fl:acct:90108"));
        assert_eq!(judge.party, "Nonpartisan");
        assert!(judge.is_judge);
        let local = cands.iter().find(|c| c.name.contains("Alice Smith")).expect("local");
        assert_eq!(local.external_id.as_deref(), Some("fl:acct:91001"));
    }

    #[test]
    fn parse_dos_keeps_unopposed_for_bench_info() {
        let filings = parse_dos_tsv(SAMPLE_TSV);
        assert!(filings.iter().any(|f| f.name.contains("Melanie") && f.status_desc == "Unopposed"));
        assert!(filings.iter().any(|f| f.name.contains("Me Skip")));
        assert!(filings.iter().any(|f| f.office_code == "COJ" && f.name.contains("Edwards")));
        assert_eq!(filings.iter().filter(|f| f.office_code == "CTJ").count(), 2);
    }

    #[test]
    fn fl_acct_external_id_helpers() {
        assert_eq!(fl_acct_external_id("88799").as_deref(), Some("fl:acct:88799"));
        assert_eq!(fl_acct_external_id("").as_deref(), None);
        assert_eq!(fl_acct_external_id("0").as_deref(), None);
        assert_eq!(parse_fl_acct_external_id("fl:acct:90108").as_deref(), Some("90108"));
        assert_eq!(parse_fl_acct_external_id("H0FL00000").as_deref(), None);
    }

    #[test]
    fn parse_dos_keeps_local_office_codes() {
        let filings = parse_dos_tsv(SAMPLE_TSV);
        assert!(filings.iter().any(|f| f.office_code == "CCM" && f.county == "Brevard"));
        assert!(filings.iter().any(|f| f.office_code == "SPE"));
        assert!(!filings.iter().any(|f| f.office_code == "USS"));
    }

    #[test]
    fn classify_local_office_heuristics() {
        assert_eq!(classify_local_office("CCM", "County Commissioner"), ("county", "county"));
        assert_eq!(classify_local_office("MAY", "Mayor"), ("municipal", "municipal"));
        assert_eq!(
            classify_local_office("SPE", "Water Control District Supervisor"),
            ("special_district", "special_district")
        );
    }

    #[test]
    fn county_circuit_brevard() {
        assert_eq!(county_to_circuit("brevard"), Some(18));
        assert_eq!(circuit_to_dca(18), Some(5));
        assert_eq!(
            county_to_circuit(&normalize_county_key("Miami-Dade County")),
            Some(11)
        );
    }

    #[test]
    fn parse_senate_sample() {
        let html = r#"
            <a class="senatorLink" href="/Senators/2024-2026/S27"><img width="50" height="66" border="0" alt="Senator Ben Albritton" src="x.jpg" class="senatorThumb middle">Albritton, Ben</a>
            <br /><strong>President</strong>
            </th>
            <td class="middle">27</td>
            <td class="middle">Republican</td>
        "#;
        let map = parse_senate_roster(html);
        let m = map.get(&27).expect("district 27");
        assert!(m.name.contains("Albritton"));
        assert!(m.party.to_ascii_lowercase().contains("republic"));
    }

    #[test]
    fn parse_house_sample() {
        let html = r#"
            <a href="/Sections/Representatives/details.aspx?MemberId=4864&LegislativeTermId=91"
                role="link"
                aria-label="Click to Representative  Abbott, Shane G.">
                <div class="team-txt">
                    <h5>Abbott, Shane G.</h5>
                    <p>Republican &mdash;  <span class="text-nowrap">District: 5</span></p>
                </div>
            </a>
        "#;
        let map = parse_house_roster(html);
        let m = map.get(&5).expect("district 5");
        assert!(m.name.to_ascii_lowercase().contains("abbott"));
        assert_eq!(m.district, 5);
    }

    const SAMPLE_MEASURES_HTML: &str = r#"
    <table>
      <tr>
        <td>2026 GEN</td><td>Active</td><td>06/17/2025</td>
        <td><a href="https://constitutionalinitiatives.dos.fl.gov/Home/InitDetail?account=10&amp;seqnum=108">Budget Stabilization Fund</a></td>
        <td>(1)</td>
      </tr>
      <tr>
        <td>2026 GEN</td><td>Active</td><td>06/18/2025</td>
        <td><a href="https://constitutionalinitiatives.dos.fl.gov/Home/InitDetail?account=10&amp;seqnum=109">Exemption of Tangible Personal Property on Agricultural Land from Taxation</a></td>
        <td>(2)</td>
      </tr>
      <tr>
        <td>2026 GEN</td><td>Active</td><td>06/16/2026</td>
        <td><a href="/Home/InitDetail?account=10&amp;seqnum=110">SAVE OUR HOMES FROM EXCESSIVE PROPERTY TAXES</a></td>
        <td>(3)</td>
      </tr>
      <tr>
        <td>2028 GEN</td><td>Active</td><td>01/01/2027</td>
        <td><a href="https://constitutionalinitiatives.dos.fl.gov/Home/InitDetail?account=99&amp;seqnum=1">Future Petition Should Be Excluded</a></td>
        <td></td>
      </tr>
    </table>
    "#;

    #[test]
    fn parse_fl_measures_amendments_1_to_3() {
        let ms = parse_fl_measures_table(SAMPLE_MEASURES_HTML, 2026);
        assert_eq!(ms.len(), 3);
        assert_eq!(ms[0].measure_code.as_deref(), Some("Amendment 1"));
        assert_eq!(ms[0].title, "Budget Stabilization Fund");
        assert!(ms[0].source_url.contains("seqnum=108"));
        assert_eq!(ms[1].measure_code.as_deref(), Some("Amendment 2"));
        assert_eq!(ms[2].measure_code.as_deref(), Some("Amendment 3"));
        assert!(ms[2].source_url.starts_with("https://"));
        assert!(!ms.iter().any(|m| m.title.contains("Future Petition")));
    }

    #[test]
    fn parse_dos_account() {
        assert_eq!(
            parse_dos_account_from_url(
                "https://constitutionalinitiatives.dos.fl.gov/Home/InitDetail?account=83475&seqnum=1"
            )
            .as_deref(),
            Some("83475")
        );
    }

    #[test]
    fn parse_com_lkup_and_select_oppose() {
        let html = r##"
      <table>
    <tr valign="top" align="center" bgcolor="#D7DBFD">
      <td width="70%"><p><b>Committee</b></td>
      <td width="15%"><p><b>Type</b></td>
      <td width="15%"><p><b>Status</b></td>
    </tr>
    <tr valign="top" align="left" bgcolor="White">
      <td align=left>
           <a HREF="ComDetail.asp?account=41393"> Hands Off Florida -- No on 2 </a>
      </td>
      <td align="center">PAC</td>
      <td align="center">Closed</td>
    </tr>
    <tr valign="top" align="left" bgcolor="#D7DBFD">
      <td align=left>
           <a HREF="ComDetail.asp?account=93273"> No On 3. Protect Our Homes </a>
      </td>
      <td align="center">PAC</td>
      <td align="center">Active</td>
    </tr>
    <tr valign="top" align="left" bgcolor="White">
      <td align=left>
           <a HREF="ComDetail.asp?account=92021"> Vote No on 3, Inc. </a>
      </td>
      <td align="center">PAC</td>
      <td align="center">Active</td>
    </tr>
    <tr valign="top" align="left" bgcolor="#D7DBFD">
      <td align=left>
           <a HREF="ComDetail.asp?account=73884"> No on 10 </a>
      </td>
      <td align="center">PAC</td>
      <td align="center">Closed</td>
    </tr>
      </table>
        "##;
        let hits = parse_com_lkup_by_name_html(html);
        assert_eq!(hits.len(), 4);
        assert_eq!(hits[1].account, "93273");
        assert_eq!(oppose_amendment_number_from_name(&hits[1].name), Some(3));
        // Do not match amendment 3 against "No on 10"
        assert_eq!(oppose_amendment_number_from_name("No on 10"), Some(10));
        assert_eq!(amendment_number_from_code(Some("Amendment 3")), Some(3));

        let sel = select_oppose_committees(&hits, 3, 2);
        assert_eq!(sel.len(), 2);
        assert!(sel.iter().all(|h| h.status.eq_ignore_ascii_case("Active")));
        assert!(sel.iter().any(|h| h.account == "93273"));
        assert!(sel.iter().any(|h| h.account == "92021"));

        let a2 = select_oppose_committees(&hits, 2, 1);
        assert_eq!(a2.len(), 1);
        assert_eq!(a2[0].account, "41393");
    }

    #[test]
    fn parse_trefin_sample_lines() {
        let html = r#"
        <pre>
Rpt Yr Rpt Type Date           Amount   Contributor Name
------ -------- ---------- ------------ ------------------------------
2024   Q2       05/13/2024    30,000.00 FLORIDA RIGHT TO LIFE          19690 CROWSN LANE
2024   P7       08/12/2024    29,961.80 BROCK MARY                     731 SW 37TH AVE
2024   Q3       09/01/2024    10,000.00 FLORIDA RIGHT TO LIFE          19690 CROWSN LANE
        </pre>"#;
        let fin = parse_trefin_contributions_html(html, "83475", 10);
        assert_eq!(fin.line_count, 3);
        assert!((fin.contributions_sum - 69961.8).abs() < 0.1);
        assert_eq!(fin.top_contributors[0].name, "FLORIDA RIGHT TO LIFE");
        assert_eq!(fin.top_contributors[0].gift_count, Some(2));
        assert!(fin.committee_url.contains("83475"));
    }

    #[test]
    fn parse_trefin_live_dump_shape() {
        // Snippet from live TreFin (Vote No on 3 / sponsor-scale dumps): fixed-width pre.
        let html = r#"
<pre>
<b>Rpt Yr Rpt Type Date           Amount   Contributor Name               Address
------ -------- ---------- ------------ ------------------------------ ---------------------------------------- </b>
2026   P4       07/11/2026    10,000.00 DEAN RINGERS MORGAN AND LAWTON PO BOX 2928                              ORLANDO, FL 32802
2026   P4       07/16/2026    10,000.00 JOHNSON ANSELMO MURDOCH BURKE  2455 EAST SUNRISE BOULEVARD #1000        FORT LAUDERDALE, FL 33304
2024   D1       10/22/2024 22,000,000.00 TRULIEVE, INC                  24671 US HIGHWAY 19 N                    CLEARWATER, FL 33764
2026   P2       06/25/2026       521.15 MARAZZITO JOSH                 4801 W LEONA ST                          TAMPA, FL 33629
0 Contribution(s) Selected
</pre>"#;
        let fin = parse_trefin_contributions_html(html, "92021", 5);
        assert_eq!(fin.line_count, 4);
        assert!((fin.contributions_sum - 22_020_521.15).abs() < 0.1);
        assert!(fin
            .top_contributors
            .iter()
            .any(|c| c.name.to_ascii_uppercase().contains("TRULIEVE")));
    }

    #[test]
    fn parse_fl_measure_summary_from_detail() {
        let html = r#"
            <dt class="col-sm-1">Summary</dt>
            <dd class="col-sm-11">
                <p class="text-wrap"><a href="x.pdf">View Full Text</a></p>
                <p class="text-wrap">
                    Proposing an amendment to the State Constitution to increase the amount of funds
                    that may be retained in the budget stabilization fund from 10% to 25%.
                </p>
            </dd>
        "#;
        let s = parse_fl_measure_summary_html(html).expect("summary");
        assert!(s.contains("budget stabilization fund"));
        assert!(!s.to_ascii_lowercase().contains("view full text"));
    }

    #[test]
    fn extract_antiforgery_token() {
        let html = r#"<input name="__RequestVerificationToken" type="hidden" value="tok123" />"#;
        assert_eq!(extract_verification_token(html).as_deref(), Some("tok123"));
    }

    // --- A2 name-search ---

    #[test]
    fn parse_contrib_totals_tsv_sample() {
        let tsv = "\
Candidate Name\tParty\tOffice\tDistrict\tGroup\tTotal Amount
Carlos Guillermo Smith\tDEM\tSTS\t017\t \t348037.47
David Smith\tREP\tSTR\t038\t\t629067.18
Ronrico \"Rico\" Smith\tREP\tSTR\t067\t\t67611.78
";
        let hits = parse_contrib_candidate_totals_tsv(tsv);
        assert_eq!(hits.len(), 3);
        assert_eq!(hits[1].name, "David Smith");
        assert_eq!(hits[1].office_code, "STR");
        assert_eq!(hits[1].district, "038");
        assert!((hits[1].total_amount - 629067.18).abs() < 0.01);
    }

    #[test]
    fn match_contrib_accepts_unique_name_office_district() {
        let tsv = "\
Candidate Name\tParty\tOffice\tDistrict\tGroup\tTotal Amount
Carlos Guillermo Smith\tDEM\tSTS\t017\t\t348037.47
David Smith\tREP\tSTR\t038\t\t629067.18
Ronrico \"Rico\" Smith\tREP\tSTR\t067\t\t67611.78
";
        let hits = parse_contrib_candidate_totals_tsv(tsv);
        let q = FlNameSearchQuery {
            name: "David Smith".into(),
            office: "Florida House (District 38)".into(),
            chamber: "state_house".into(),
            party: "Republican".into(),
            district: Some(38),
            county: String::new(),
        };
        match match_fl_contrib_candidate(&hits, &q) {
            FlNameMatch::Unique { hit } => {
                assert_eq!(hit.name, "David Smith");
                assert_eq!(hit.office_code, "STR");
            }
            other => panic!("expected unique, got {other:?}"),
        }
    }

    #[test]
    fn match_contrib_skips_ambiguous_same_office_family() {
        let tsv = "\
Candidate Name\tParty\tOffice\tDistrict\tGroup\tTotal Amount
David Smith\tREP\tSTR\t038\t\t100.0
David Smith\tDEM\tSTR\t099\t\t200.0
";
        // Wait — different districts; with district in query should still unique.
        // Ambiguous: same district missing
        let hits = parse_contrib_candidate_totals_tsv(tsv);
        let q = FlNameSearchQuery {
            name: "David Smith".into(),
            office: "Florida House".into(),
            chamber: "state_house".into(),
            party: String::new(),
            district: None, // no district → both STR David Smith match
            county: String::new(),
        };
        match match_fl_contrib_candidate(&hits, &q) {
            FlNameMatch::Ambiguous { count } => assert_eq!(count, 2),
            other => panic!("expected ambiguous, got {other:?}"),
        }
    }

    #[test]
    fn match_contrib_rejects_party_conflict() {
        let tsv = "\
Candidate Name\tParty\tOffice\tDistrict\tGroup\tTotal Amount
David Smith\tDEM\tSTR\t038\t\t100.0
";
        let hits = parse_contrib_candidate_totals_tsv(tsv);
        let q = FlNameSearchQuery {
            name: "David Smith".into(),
            office: "Florida House (District 38)".into(),
            chamber: "state_house".into(),
            party: "Republican".into(),
            district: Some(38),
            county: String::new(),
        };
        assert!(matches!(
            match_fl_contrib_candidate(&hits, &q),
            FlNameMatch::None
        ));
    }

    #[test]
    fn match_contrib_rejects_wrong_district() {
        let tsv = "\
Candidate Name\tParty\tOffice\tDistrict\tGroup\tTotal Amount
David Smith\tREP\tSTR\t038\t\t100.0
";
        let hits = parse_contrib_candidate_totals_tsv(tsv);
        let q = FlNameSearchQuery {
            name: "David Smith".into(),
            office: "Florida House (District 67)".into(),
            chamber: "state_house".into(),
            party: "Republican".into(),
            district: Some(67),
            county: String::new(),
        };
        assert!(matches!(
            match_fl_contrib_candidate(&hits, &q),
            FlNameMatch::None
        ));
    }

    #[test]
    fn split_and_normalize_names() {
        let (f, l) = split_candidate_first_last(r#"Ronrico "Rico" Smith"#);
        assert_eq!(l, "Smith");
        assert!(f.to_ascii_lowercase().contains("ronrico"));
        assert!(last_names_match("David A. Smith Jr.", "David Smith"));
        assert!(first_names_compatible("J Smith", "John Smith"));
        assert!(!first_names_compatible("Alice Smith", "Bob Smith"));
    }

    #[test]
    fn office_code_from_ballot_chambers() {
        assert_eq!(
            office_code_from_ballot("state_senate", "Florida Senate (District 8)").as_deref(),
            Some("STS")
        );
        assert_eq!(
            office_code_from_ballot("state_house", "Florida House (District 30)").as_deref(),
            Some("STR")
        );
        assert_eq!(
            office_code_from_ballot("judicial", "Circuit Judge (Circuit 18, Group 13)").as_deref(),
            Some("CTJ")
        );
        assert_eq!(
            office_code_from_ballot("judicial", "County Judge (Group 3)").as_deref(),
            Some("COJ")
        );
    }

    #[test]
    fn parse_can_list_and_resolve_account() {
        let html = r##"
        <b>State Senator</b></font>
        <table class="results">
        <tr bgcolor="#D7DBFD"><td>District</td><th>Candidate</th><th>Status</th></tr>
        <tr>
          <td align=center><b>8</b></td>
          <td align=left>
            <a href="CanDetail.asp?account=88799">Johansson</a><a href="CanDetail.asp?account=88799">,</a>
            <a href="CanDetail.asp?account=88799">Jake</a>
            (REP)
          </td>
          <td>Qualified</td>
        </tr>
        <tr>
          <td align=center><b>&nbsp;</b></td>
          <td align=left>
            <a href="CanDetail.asp?account=89602">Ngying</a><a href="CanDetail.asp?account=89602">,</a>
            <a href="CanDetail.asp?account=89602">Judy</a>
            (DEM)
          </td>
          <td>Qualified</td>
        </tr>
        <tr>
          <td align=center><b>30</b></td>
          <td align=left>
            <a href="CanDetail.asp?account=99901">Other</a><a href="CanDetail.asp?account=99901">,</a>
            <a href="CanDetail.asp?account=99901">Person</a>
            (REP)
          </td>
          <td>Qualified</td>
        </tr>
        </table>
        "##;
        let hits = parse_can_list_html(html);
        assert!(hits.iter().any(|h| h.account == "88799" && h.name.contains("Jake")));
        // District carry-forward for second candidate in district 8
        let judy = hits.iter().find(|h| h.account == "89602").expect("judy");
        assert_eq!(judy.district, "8");
        assert_eq!(judy.office_code, "STS");

        let q = FlNameSearchQuery {
            name: "Jake Johansson".into(),
            office: "Florida Senate (District 8)".into(),
            chamber: "state_senate".into(),
            party: "Republican".into(),
            district: Some(8),
            county: String::new(),
        };
        assert_eq!(
            match_fl_can_list_account(&hits, &q).as_deref(),
            Some("88799")
        );
        // Ambiguous without district when two different people — only one Jake
        let q2 = FlNameSearchQuery {
            name: "Person Other".into(),
            office: "Florida Senate (District 8)".into(),
            chamber: "state_senate".into(),
            party: "Republican".into(),
            district: Some(8),
            county: String::new(),
        };
        assert_eq!(match_fl_can_list_account(&hits, &q2), None);
    }

    #[test]
    fn fl_contrib_form_sets_party_all() {
        let form = fl_contrib_candidate_totals_form("20241105-GEN", "Smith", "David", "STR", "38");
        assert!(form.contains("party=All"));
        assert!(form.contains("search_on=3"));
        assert!(form.contains("queryformat=2"));
        assert!(form.contains("CanLName=Smith"));
        assert!(form.contains("office=STR"));
    }

    #[test]
    fn fl_name_search_result_statuses() {
        let hit = FlContribCandidateHit {
            name: "David Smith".into(),
            party: "REP".into(),
            office_code: "STR".into(),
            district: "38".into(),
            group: String::new(),
            total_amount: 100.0,
            total_display: "$100.00".into(),
        };
        let u = FlNameMatch::Unique { hit: hit.clone() };
        let r = fl_name_search_result(&u, Some("12345".into()));
        assert_eq!(r.status, "unique");
        assert_eq!(r.account.as_deref(), Some("12345"));
        let r2 = fl_name_search_result(&u, None);
        assert_eq!(r2.status, "no_account");
        let r3 = fl_name_search_result(&FlNameMatch::Ambiguous { count: 3 }, None);
        assert_eq!(r3.status, "ambiguous");
        assert!(r3.note.contains("skipped"));
    }
}
