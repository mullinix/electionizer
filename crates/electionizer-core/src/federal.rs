//! Pure federal live pipeline: Zippo/Census geo + FEC candidate list → ballot report.
//! No HTTP — callers pass response bodies.

use crate::models::{
    build_ballot_sections, build_office_groups, election_year_from, federal_election_date,
    judicial_explainer_for, normalize_party_label, normalize_zip, voter_portal_for_state,
    BallotCandidateRow, BallotReport, BallotSnapshot, GeoResolution, MeasureSummary,
    ResolvedJurisdiction, SnapshotCandidate,
};
use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

// --- Zippopotam ---

#[derive(Debug, Clone, Serialize)]
pub struct ZippoPlace {
    pub city: String,
    pub state_name: String,
    pub state_abbr: String,
    pub longitude: f64,
    pub latitude: f64,
}

#[derive(Debug, Deserialize)]
struct ZippoResponse {
    places: Vec<ZippoPlaceRaw>,
}

#[derive(Debug, Deserialize)]
struct ZippoPlaceRaw {
    #[serde(rename = "place name")]
    place_name: String,
    #[serde(rename = "state abbreviation")]
    state_abbreviation: String,
    state: String,
    longitude: String,
    latitude: String,
}

pub fn parse_zippo_json(json: &str) -> Result<ZippoPlace, String> {
    let body: ZippoResponse =
        serde_json::from_str(json).map_err(|e| format!("zippopotam json: {e}"))?;
    let place = body
        .places
        .into_iter()
        .next()
        .ok_or_else(|| "zippopotam returned no places".to_string())?;
    let longitude: f64 = place
        .longitude
        .parse()
        .map_err(|e| format!("zippopotam longitude: {e}"))?;
    let latitude: f64 = place
        .latitude
        .parse()
        .map_err(|e| format!("zippopotam latitude: {e}"))?;
    Ok(ZippoPlace {
        city: place.place_name,
        state_name: place.state,
        state_abbr: place.state_abbreviation,
        longitude,
        latitude,
    })
}

// --- Census coordinates geocoder ---

#[derive(Debug, Clone)]
pub struct CensusGeo {
    pub state_name: Option<String>,
    pub county_name: Option<String>,
    pub place_name: Option<String>,
    pub cd_number: Option<u32>,
    pub state_senate_district: Option<u32>,
    pub state_house_district: Option<u32>,
    /// Alphanumeric lower-chamber code (MD `30A`, numeric `46`, etc.).
    pub state_house_label: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CensusCoordResponse {
    result: CensusCoordResult,
}

#[derive(Debug, Deserialize)]
struct CensusCoordResult {
    geographies: CensusCoordGeographies,
}

#[derive(Debug, Deserialize)]
struct CensusCoordGeographies {
    #[serde(rename = "States")]
    states: Option<Vec<CensusNamed>>,
    #[serde(rename = "Counties")]
    counties: Option<Vec<CensusNamed>>,
    #[serde(rename = "Incorporated Places")]
    places: Option<Vec<CensusNamed>>,
    #[serde(rename = "119th Congressional Districts")]
    congressional: Option<Vec<CensusCd>>,
    #[serde(rename = "2024 State Legislative Districts - Upper")]
    state_upper: Option<Vec<CensusCd>>,
    #[serde(rename = "2024 State Legislative Districts - Lower")]
    state_lower: Option<Vec<CensusCd>>,
}

#[derive(Debug, Deserialize)]
struct CensusNamed {
    #[serde(rename = "NAME")]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CensusCd {
    #[serde(rename = "NAME")]
    name: Option<String>,
    #[serde(rename = "BASENAME")]
    basename: Option<String>,
    #[serde(rename = "CD119")]
    cd119: Option<String>,
}

fn first_name(items: &Option<Vec<CensusNamed>>) -> Option<String> {
    items
        .as_ref()
        .and_then(|v| v.first())
        .and_then(|i| i.name.clone())
}

pub fn parse_census_coordinates_json(json: &str) -> Result<CensusGeo, String> {
    let body: CensusCoordResponse =
        serde_json::from_str(json).map_err(|e| format!("census json: {e}"))?;
    let geos = body.result.geographies;

    let state_name = first_name(&geos.states);
    let county_name = first_name(&geos.counties);
    let place_name = first_name(&geos.places).map(|n| {
        n.trim_end_matches(" city")
            .trim_end_matches(" town")
            .trim_end_matches(" village")
            .to_string()
    });

    let cd_number = geos
        .congressional
        .as_ref()
        .and_then(|v| v.first())
        .and_then(|c| {
            c.cd119
                .as_deref()
                .or(c.basename.as_deref())
                .map(|s| s.to_string())
                .or_else(|| {
                    c.name
                        .as_ref()
                        .and_then(|n| n.split_whitespace().last().map(|s| s.to_string()))
                })
                .and_then(|s| s.parse::<u32>().ok())
        });

    let state_senate_district = geos
        .state_upper
        .as_ref()
        .and_then(|v| v.first())
        .and_then(|c| {
            c.basename
                .as_deref()
                .or(c.name.as_ref().and_then(|n| n.split_whitespace().last()))
                .and_then(|s| s.parse::<u32>().ok())
        });
    let house_token = geos.state_lower.as_ref().and_then(|v| v.first()).and_then(|c| {
        c.basename
            .as_deref()
            .or(c.name.as_ref().and_then(|n| n.split_whitespace().last()))
            .map(|s| s.to_string())
    });
    let state_house_label = house_token.as_deref().and_then(parse_house_label);
    let state_house_district = house_token
        .as_deref()
        .and_then(parse_district_token)
        .or_else(|| state_house_label.as_deref().and_then(leading_district_num));

    Ok(CensusGeo {
        state_name,
        county_name,
        place_name,
        cd_number,
        state_senate_district,
        state_house_district,
        state_house_label,
    })
}

pub fn slugify(s: &str) -> String {
    s.to_ascii_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .split('_')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

pub fn ordinal(n: u32) -> String {
    let suffix = match n % 100 {
        11 | 12 | 13 => "th",
        _ => match n % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        },
    };
    format!("{n}{suffix}")
}

/// Merge Zippo + Census into a full geo resolution for `zip`.
pub fn geo_from_zippo_and_census(zip: &str, zippo: &ZippoPlace, census: &CensusGeo) -> GeoResolution {
    let state = zippo.state_abbr.to_ascii_uppercase();
    let state_l = state.to_ascii_lowercase();
    let state_name = census
        .state_name
        .clone()
        .unwrap_or_else(|| zippo.state_name.clone());
    let city = census
        .place_name
        .clone()
        .unwrap_or_else(|| zippo.city.clone());
    let county = census
        .county_name
        .clone()
        .unwrap_or_else(|| "Unknown County".into());

    let cd_num = census.cd_number.unwrap_or(0);
    let cd_label = if cd_num == 0 {
        format!("{state}-AL")
    } else {
        format!("{state}-{cd_num}")
    };
    let cd_name = if cd_num == 0 {
        format!("{state_name} At-Large Congressional District")
    } else {
        format!(
            "{state_name}'s {} Congressional District",
            ordinal(cd_num)
        )
    };
    let cd_ocd = if cd_num == 0 {
        format!("ocd-division/country:us/state:{state_l}/cd:0")
    } else {
        format!("ocd-division/country:us/state:{state_l}/cd:{cd_num}")
    };

    let mut jurisdictions = vec![
        ResolvedJurisdiction {
            ocd_id: "ocd-division/country:us".into(),
            name: "United States".into(),
            level: "federal".into(),
            state: None,
        },
        ResolvedJurisdiction {
            ocd_id: format!("ocd-division/country:us/state:{state_l}"),
            name: state_name.clone(),
            level: "state".into(),
            state: Some(state.clone()),
        },
        ResolvedJurisdiction {
            ocd_id: cd_ocd,
            name: cd_name,
            level: "congressional".into(),
            state: Some(state.clone()),
        },
    ];

    if let Some(ref cname) = census.county_name {
        let slug = slugify(cname.trim_end_matches(" County"));
        jurisdictions.push(ResolvedJurisdiction {
            ocd_id: format!("ocd-division/country:us/state:{state_l}/county:{slug}"),
            name: cname.clone(),
            level: "county".into(),
            state: Some(state.clone()),
        });
    }

    if let Some(ref place) = census.place_name {
        let slug = slugify(place);
        jurisdictions.push(ResolvedJurisdiction {
            ocd_id: format!("ocd-division/country:us/state:{state_l}/place:{slug}"),
            name: place.clone(),
            level: "municipal".into(),
            state: Some(state.clone()),
        });
    }

    if let Some(sd) = census.state_senate_district {
        jurisdictions.push(ResolvedJurisdiction {
            ocd_id: format!("ocd-division/country:us/state:{state_l}/sldu:{sd}"),
            name: format!("{state_name} State Senate District {sd}"),
            level: "state_senate".into(),
            state: Some(state.clone()),
        });
    }
    let house_label = census
        .state_house_label
        .clone()
        .or_else(|| census.state_house_district.map(|n| n.to_string()));
    if let Some(ref hl) = house_label {
        let ocd_code = hl.to_ascii_lowercase();
        jurisdictions.push(ResolvedJurisdiction {
            ocd_id: format!("ocd-division/country:us/state:{state_l}/sldl:{ocd_code}"),
            name: format!("{state_name} State House District {hl}"),
            level: "state_house".into(),
            state: Some(state.clone()),
        });
    }

    let source_url = format!(
        "https://api.zippopotam.us/us/{zip} + https://geocoding.geo.census.gov/geocoder/geographies/coordinates?x={}&y={}&benchmark=Public_AR_Current&vintage=Current_Current&format=json",
        zippo.longitude, zippo.latitude
    );

    GeoResolution {
        state,
        state_name,
        county,
        city,
        congressional_district: cd_label,
        state_senate_district: census.state_senate_district,
        state_house_district: census.state_house_district,
        state_house_label: house_label,
        latitude: Some(zippo.latitude),
        longitude: Some(zippo.longitude),
        jurisdictions,
        source_url,
        source_publisher: "Zippopotam.us + U.S. Census Bureau Geocoder".into(),
    }
}

fn parse_district_token(s: &str) -> Option<u32> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    if t.chars().all(|c| c == '0') {
        return Some(0);
    }
    t.trim_start_matches('0').parse().ok().or_else(|| t.parse().ok())
}

/// Alphanumeric lower-chamber code (`30A`, `1B`, `46`).
fn parse_house_label(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    let cleaned: String = t
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_uppercase();
    if cleaned.is_empty() || cleaned.chars().all(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    Some(cleaned)
}

fn leading_district_num(s: &str) -> Option<u32> {
    let digits: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        None
    } else {
        parse_district_token(&digits)
    }
}

/// Parse district geography from either Census coordinates geocoder or TIGERweb identify JSON.
pub fn parse_district_geo_json(json: &str) -> Result<CensusGeo, String> {
    if let Ok(g) = parse_census_coordinates_json(json) {
        return Ok(g);
    }
    parse_tigerweb_identify_json(json)
}

/// TIGERweb Legislative MapServer `/identify` response (CORS-friendly in browsers).
pub fn parse_tigerweb_identify_json(json: &str) -> Result<CensusGeo, String> {
    let body: TigerIdentifyResponse =
        serde_json::from_str(json).map_err(|e| format!("tigerweb json: {e}"))?;
    if body.results.is_empty() {
        return Err("tigerweb identify returned no results".into());
    }

    let mut cd_number = None;
    let mut state_senate_district = None;
    let mut state_house_district = None;
    let mut state_house_label = None;
    let mut state_name = None;

    for r in &body.results {
        let layer = r.layer_name.as_deref().unwrap_or("");
        let attrs = r.attributes.as_ref();
        let basename = attrs.and_then(|a| a.basename.as_deref().or(a.cd119.as_deref()));
        let name = attrs.and_then(|a| a.name.as_deref());
        let token = basename.or_else(|| name.and_then(|n| n.split_whitespace().last()));
        let num = token.and_then(parse_district_token);

        if layer.contains("119th Congressional") || layer.contains("Congressional District") {
            if cd_number.is_none() {
                cd_number = num.or_else(|| {
                    attrs
                        .and_then(|a| a.cd119.as_deref())
                        .and_then(|s| s.parse().ok())
                });
            }
            if state_name.is_none() {
                state_name = attrs.and_then(|a| a.state_name.clone());
            }
        } else if layer.contains("Upper") || layer.contains("State Senate") {
            if state_senate_district.is_none() {
                state_senate_district = num;
            }
        } else if layer.contains("Lower") || layer.contains("State House") {
            if state_house_label.is_none() {
                if let Some(lbl) = token.and_then(parse_house_label) {
                    state_house_district = state_house_district
                        .or_else(|| leading_district_num(&lbl))
                        .or(num);
                    state_house_label = Some(lbl);
                } else if state_house_district.is_none() {
                    state_house_district = num;
                }
            }
        }
    }

    Ok(CensusGeo {
        state_name,
        county_name: None,
        place_name: None,
        cd_number,
        state_senate_district,
        state_house_district,
        state_house_label,
    })
}

#[derive(Debug, Deserialize)]
struct TigerIdentifyResponse {
    #[serde(default)]
    results: Vec<TigerIdentifyResult>,
}

#[derive(Debug, Deserialize)]
struct TigerIdentifyResult {
    #[serde(rename = "layerName")]
    layer_name: Option<String>,
    attributes: Option<TigerAttrs>,
}

#[derive(Debug, Deserialize)]
struct TigerAttrs {
    #[serde(rename = "NAME")]
    name: Option<String>,
    #[serde(rename = "BASENAME")]
    basename: Option<String>,
    #[serde(rename = "CD119")]
    cd119: Option<String>,
    #[serde(rename = "STATE_NAME")]
    state_name: Option<String>,
}

/// FCC census area API (`geo.fcc.gov/api/census/area`) — county + state, CORS-friendly.
pub fn parse_fcc_area_json(json: &str) -> Result<(Option<String>, Option<String>, Option<String>), String> {
    let body: FccAreaResponse =
        serde_json::from_str(json).map_err(|e| format!("fcc area json: {e}"))?;
    let row = body.results.into_iter().next();
    Ok(match row {
        Some(r) => (r.county_name, r.state_name, r.state_code),
        None => (None, None, None),
    })
}

#[derive(Debug, Deserialize)]
struct FccAreaResponse {
    #[serde(default)]
    results: Vec<FccAreaRow>,
}

#[derive(Debug, Deserialize)]
struct FccAreaRow {
    county_name: Option<String>,
    state_name: Option<String>,
    state_code: Option<String>,
}

fn enrich_with_fcc(mut geo: CensusGeo, fcc_json: &str) -> CensusGeo {
    if fcc_json.trim().is_empty() {
        return geo;
    }
    if let Ok((county, state_name, _)) = parse_fcc_area_json(fcc_json) {
        if geo.county_name.is_none() {
            geo.county_name = county;
        }
        if geo.state_name.is_none() {
            geo.state_name = state_name;
        }
    }
    geo
}

/// One-shot: Zippo + district JSON (Census geocoder **or** TIGERweb identify) → geo.
/// Optional `fcc_json` fills county when using TIGERweb (browser CORS path).
pub fn parse_geo_from_jsons(
    zip: &str,
    zippo_json: &str,
    district_json: &str,
) -> Result<GeoResolution, String> {
    parse_geo_from_jsons_ex(zip, zippo_json, district_json, "")
}

pub fn parse_geo_from_jsons_ex(
    zip: &str,
    zippo_json: &str,
    district_json: &str,
    fcc_json: &str,
) -> Result<GeoResolution, String> {
    let zippo = parse_zippo_json(zippo_json)?;
    let mut district = parse_district_geo_json(district_json)?;
    district = enrich_with_fcc(district, fcc_json);
    Ok(geo_from_zippo_and_census(zip, &zippo, &district))
}

// --- FEC candidate list ---

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FecCandidateRow {
    pub name: String,
    pub party: Option<String>,
    pub party_full: Option<String>,
    pub candidate_id: Option<String>,
    pub incumbent_challenge: Option<String>,
    pub incumbent_challenge_full: Option<String>,
    pub election_years: Option<Vec<i32>>,
}

#[derive(Debug, Deserialize)]
struct FecResponse {
    results: Vec<FecCandidateRow>,
}

/// Parse OpenFEC candidates list JSON (full `{results:…}` or bare array).
pub fn parse_fec_candidates_json(json: &str) -> Result<Vec<FecCandidateRow>, String> {
    if let Ok(resp) = serde_json::from_str::<FecResponse>(json) {
        return Ok(resp.results);
    }
    serde_json::from_str(json).map_err(|e| format!("fec candidates json: {e}"))
}

pub fn filter_fec_candidates(rows: Vec<FecCandidateRow>, cycle: i32) -> Vec<FecCandidateRow> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for r in rows {
        if let Some(years) = &r.election_years {
            if !years.is_empty() && !years.contains(&cycle) {
                continue;
            }
        }
        let n = r.name.to_ascii_lowercase();
        if n.contains(" party") || n.ends_with(" party") || n == "none" {
            continue;
        }
        let id = r
            .candidate_id
            .clone()
            .unwrap_or_else(|| format!("{}-{}", r.name, r.party.clone().unwrap_or_default()));
        if !seen.insert(id) {
            continue;
        }
        out.push(r);
        if out.len() >= 40 {
            break;
        }
    }
    out
}

pub fn parse_cd_number(label: &str) -> u32 {
    if label.to_ascii_uppercase().contains("AL") {
        return 0;
    }
    label
        .rsplit(['-', ' '])
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

pub fn format_person_name(raw: &str) -> String {
    let cleaned = if let Some((last, rest)) = raw.split_once(',') {
        format!("{} {}", rest.trim(), last.trim())
    } else {
        raw.to_string()
    };
    cleaned
        .split_whitespace()
        .map(|w| {
            if w.len() <= 3 && w.chars().all(|c| c.is_ascii_alphabetic()) && w.ends_with('.') {
                return w.to_string();
            }
            let lower = w.to_ascii_lowercase();
            let mut chars = lower.chars();
            match chars.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn map_fec_candidates(
    rows: &[FecCandidateRow],
    office_label: &str,
    chamber: &str,
    jurisdiction_ocd: &str,
    source_url: &str,
) -> Vec<SnapshotCandidate> {
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        let party_raw = r
            .party_full
            .as_deref()
            .or(r.party.as_deref())
            .unwrap_or("Unknown");
        let party = normalize_party_label(party_raw);
        let is_incumbent = r
            .incumbent_challenge
            .as_deref()
            .is_some_and(|c| c.eq_ignore_ascii_case("I"));
        let name = format_person_name(&r.name);
        let cid = r.candidate_id.clone().unwrap_or_default();
        let summary = {
            let mut parts = vec![format!("FEC candidate for {office_label} ({party}).")];
            if is_incumbent {
                parts.push("Incumbent.".into());
            } else if let Some(ref ic) = r.incumbent_challenge_full {
                parts.push(format!("{ic}."));
            }
            if !cid.is_empty() {
                parts.push(format!("FEC id {cid}."));
            }
            Some(parts.join(" "))
        };

        let cand_url = if cid.is_empty() {
            source_url.to_string()
        } else {
            format!("https://www.fec.gov/data/candidate/{cid}/")
        };

        out.push(SnapshotCandidate {
            office: office_label.to_string(),
            chamber: Some(chamber.into()),
            jurisdiction_ocd: jurisdiction_ocd.to_string(),
            is_judicial: false,
            name,
            party,
            is_incumbent,
            is_judge: false,
            summary,
            source_url: cand_url,
            source_publisher: Some("Federal Election Commission".into()),
            external_id: if cid.is_empty() { None } else { Some(cid) },
        });
    }
    out
}

/// Build federal-only ballot snapshot from geo + FEC list JSON bodies.
pub fn federal_ballot_snapshot(
    geo: &GeoResolution,
    house_fec_json: &str,
    senate_fec_json: &str,
    cycle: i32,
) -> Result<BallotSnapshot, String> {
    let state_l = geo.state.to_ascii_lowercase();
    let state_ocd = format!("ocd-division/country:us/state:{state_l}");
    let cd_num = parse_cd_number(&geo.congressional_district);
    let cd_ocd = geo
        .jurisdictions
        .iter()
        .find(|j| j.level == "congressional")
        .map(|j| j.ocd_id.clone())
        .unwrap_or_else(|| {
            if cd_num == 0 {
                format!("ocd-division/country:us/state:{state_l}/cd:0")
            } else {
                format!("ocd-division/country:us/state:{state_l}/cd:{cd_num}")
            }
        });

    let house_office = if cd_num == 0 {
        format!("U.S. House ({}) At-Large", geo.state)
    } else {
        format!("U.S. House ({})", geo.congressional_district)
    };

    let house_url = crate::fec_source_url_public(&geo.state, "H", Some(cd_num), cycle);
    let senate_url = crate::fec_source_url_public(&geo.state, "S", None, cycle);

    let house_rows = filter_fec_candidates(parse_fec_candidates_json(house_fec_json)?, cycle);
    let senate_rows = filter_fec_candidates(parse_fec_candidates_json(senate_fec_json)?, cycle);

    let mut candidates = map_fec_candidates(
        &house_rows,
        &house_office,
        "house",
        &cd_ocd,
        &house_url,
    );
    candidates.extend(map_fec_candidates(
        &senate_rows,
        "U.S. Senate",
        "senate",
        &state_ocd,
        &senate_url,
    ));

    if candidates.is_empty() {
        return Err(format!(
            "FEC returned no active federal candidates for {} / {} cycle {}",
            geo.state, geo.congressional_district, cycle
        ));
    }

    Ok(BallotSnapshot {
        election_name: format!("{cycle} General Election"),
        election_date: federal_election_date(cycle),
        election_scope: "federal".into(),
        candidates,
        measures: vec![],
        source_url: "https://api.open.fec.gov/v1/candidates/".into(),
        source_publisher: "Federal Election Commission OpenFEC API".into(),
        coverage_note: Some(
            "Federal (live via FEC) · State/local pending · Measures pending".into(),
        ),
        extra_jurisdictions: vec![],
    })
}

/// Build a ballot report from geo + snapshot without a database.
pub fn ballot_report_from_snapshot(
    zip: &str,
    geo: &GeoResolution,
    snapshot: &BallotSnapshot,
) -> BallotReport {
    let zip = normalize_zip(zip).unwrap_or_else(|| zip.to_string());

    let jur_name: std::collections::HashMap<&str, &str> = geo
        .jurisdictions
        .iter()
        .chain(snapshot.extra_jurisdictions.iter())
        .map(|j| (j.ocd_id.as_str(), j.name.as_str()))
        .collect();

    let has_state_jurisdiction = geo.jurisdictions.iter().any(|j| j.level == "state");
    let state_label = geo
        .jurisdictions
        .iter()
        .find(|j| j.level == "state")
        .map(|j| j.name.as_str());

    let geo_summary = {
        let mut interesting = Vec::new();
        for j in &geo.jurisdictions {
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

    let year = Some(cycle_from_election(
        &snapshot.election_name,
        &snapshot.election_date,
    ));

    let rows: Vec<BallotCandidateRow> = snapshot
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

    let measures: Vec<MeasureSummary> = snapshot
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

    let state_code = Some(geo.state.to_ascii_uppercase());
    let voter_portal = Some(voter_portal_for_state(&geo.state));

    BallotReport {
        zip,
        status: "ready".into(),
        last_built_at: None,
        coverage_note: snapshot.coverage_note.clone(),
        election_name: Some(snapshot.election_name.clone()),
        election_date: Some(snapshot.election_date.clone()),
        geo_summary,
        state_code,
        voter_portal,
        office_groups,
        ballot_sections,
        judicial_explainer,
        measures,
    }
}

fn cycle_from_election(name: &str, date: &str) -> i32 {
    election_year_from(Some(name), Some(date)).unwrap_or_else(|| Utc::now().year())
}

/// Full live federal path: Zippo + district geo + House/Senate FEC JSON → BallotReport.
/// `district_json` may be Census coordinates geocoder **or** TIGERweb identify JSON.
pub fn ballot_report_from_federal_live(
    zip: &str,
    zippo_json: &str,
    district_json: &str,
    house_fec_json: &str,
    senate_fec_json: &str,
    cycle: i32,
) -> Result<BallotReport, String> {
    ballot_report_from_federal_live_ex(
        zip,
        zippo_json,
        district_json,
        "",
        house_fec_json,
        senate_fec_json,
        cycle,
    )
}

/// Same as [`ballot_report_from_federal_live`] with optional FCC area JSON for county.
pub fn ballot_report_from_federal_live_ex(
    zip: &str,
    zippo_json: &str,
    district_json: &str,
    fcc_json: &str,
    house_fec_json: &str,
    senate_fec_json: &str,
    cycle: i32,
) -> Result<BallotReport, String> {
    ballot_report_from_live_with_state(
        zip,
        zippo_json,
        district_json,
        fcc_json,
        house_fec_json,
        senate_fec_json,
        cycle,
        "",
    )
}

/// Federal live ballot plus optional state bodies.
///
/// `state_bodies_json`: JSON object map of named response bodies (empty/`{}` = federal only).
/// Keys: `fl:dos`, `fl:senate`, `fl:house`, `fl:measures`, `fl:soe`, `fl:sample_ballot`,
/// `az:senate`, `az:house`, `az:measures`, `nc:candidates`, `nc:measures`, `nc:measures_url`, `md:*`,
/// `civic:voterinfo`, `os:people.geo`.
/// Missing or empty values are skipped.
/// Open States is used when FL/AZ/NC/MD candidates are absent.
pub fn ballot_report_from_live_with_state(
    zip: &str,
    zippo_json: &str,
    district_json: &str,
    fcc_json: &str,
    house_fec_json: &str,
    senate_fec_json: &str,
    cycle: i32,
    state_bodies_json: &str,
) -> Result<BallotReport, String> {
    let zip = normalize_zip(zip).ok_or_else(|| format!("invalid zip: {zip}"))?;
    let geo = parse_geo_from_jsons_ex(&zip, zippo_json, district_json, fcc_json)?;
    let mut snapshot = federal_ballot_snapshot(&geo, house_fec_json, senate_fec_json, cycle)?;

    let bodies = crate::state_ballot::parse_state_bodies_json(state_bodies_json)?;
    let st = geo.state.to_ascii_uppercase();
    if let Some(ex) = crate::state_ballot::extras_from_state_bodies(&geo, cycle, &bodies) {
        crate::state_ballot::apply_state_extras(&mut snapshot, &ex, &st);
    }

    Ok(ballot_report_from_snapshot(&zip, &geo, &snapshot))
}

/// Compact geo summary for JS (state + CD number for FEC URL building).
#[derive(Debug, Clone, Serialize)]
pub struct GeoSummaryJs {
    pub state: String,
    pub state_name: String,
    pub city: String,
    pub county: String,
    pub congressional_district: String,
    pub cd_number: u32,
    pub longitude: f64,
    pub latitude: f64,
}

pub fn geo_summary_from_jsons(
    zip: &str,
    zippo_json: &str,
    district_json: &str,
) -> Result<GeoSummaryJs, String> {
    let zippo = parse_zippo_json(zippo_json)?;
    let census = parse_district_geo_json(district_json)?;
    let geo = geo_from_zippo_and_census(zip, &zippo, &census);
    Ok(GeoSummaryJs {
        state: geo.state,
        state_name: geo.state_name,
        city: geo.city,
        county: geo.county,
        congressional_district: geo.congressional_district.clone(),
        cd_number: parse_cd_number(&geo.congressional_district),
        longitude: zippo.longitude,
        latitude: zippo.latitude,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_fec_names() {
        assert_eq!(format_person_name("SHERMAN, BRAD"), "Brad Sherman");
        assert_eq!(format_person_name("DOE, JANE A"), "Jane A Doe");
    }

    #[test]
    fn parses_cd() {
        assert_eq!(parse_cd_number("CA-36"), 36);
        assert_eq!(parse_cd_number("NY-12"), 12);
        assert_eq!(parse_cd_number("AK-AL"), 0);
    }

    #[test]
    fn parse_zippo_minimal() {
        let json = r#"{
          "places": [{
            "place name": "Beverly Hills",
            "state abbreviation": "CA",
            "state": "California",
            "longitude": "-118.4065",
            "latitude": "34.0901"
          }]
        }"#;
        let p = parse_zippo_json(json).unwrap();
        assert_eq!(p.state_abbr, "CA");
        assert!((p.latitude - 34.0901).abs() < 0.001);
    }

    #[test]
    fn filter_and_map_fec() {
        let json = r#"{
          "results": [
            {
              "name": "CHEN, SAM",
              "party": "DEM",
              "party_full": "DEMOCRATIC PARTY",
              "candidate_id": "H8CA32001",
              "incumbent_challenge": "I",
              "incumbent_challenge_full": "Incumbent",
              "election_years": [2026]
            },
            {
              "name": "SOME PARTY",
              "party": "UNK",
              "candidate_id": "H8CA32999",
              "election_years": [2026]
            }
          ]
        }"#;
        let rows = filter_fec_candidates(parse_fec_candidates_json(json).unwrap(), 2026);
        assert_eq!(rows.len(), 1);
        let mapped = map_fec_candidates(&rows, "U.S. House (CA-32)", "house", "ocd:cd", "https://x");
        assert_eq!(mapped[0].name, "Sam Chen");
        assert!(mapped[0].is_incumbent);
        assert_eq!(mapped[0].party, "Democratic");
    }

    #[test]
    fn federal_report_from_bodies() {
        let zippo = r#"{
          "places": [{
            "place name": "Beverly Hills",
            "state abbreviation": "CA",
            "state": "California",
            "longitude": "-118.4",
            "latitude": "34.09"
          }]
        }"#;
        let census = r#"{
          "result": {
            "geographies": {
              "States": [{"NAME": "California"}],
              "Counties": [{"NAME": "Los Angeles County"}],
              "Incorporated Places": [{"NAME": "Beverly Hills city"}],
              "119th Congressional Districts": [{"CD119": "32", "NAME": "Congressional District 32"}],
              "2024 State Legislative Districts - Upper": [{"BASENAME": "24"}],
              "2024 State Legislative Districts - Lower": [{"BASENAME": "51"}]
            }
          }
        }"#;
        let house = r#"{
          "results": [{
            "name": "CHEN, SAM",
            "party_full": "DEMOCRATIC PARTY",
            "candidate_id": "H8CA32001",
            "incumbent_challenge": "I",
            "election_years": [2026]
          }]
        }"#;
        let senate = r#"{
          "results": [{
            "name": "RIVERA, ALEX",
            "party_full": "DEMOCRATIC PARTY",
            "candidate_id": "S6CA00123",
            "incumbent_challenge": "I",
            "election_years": [2026]
          }]
        }"#;
        let report =
            ballot_report_from_federal_live("90210", zippo, census, house, senate, 2026).unwrap();
        assert_eq!(report.zip, "90210");
        assert_eq!(report.state_code.as_deref(), Some("CA"));
        assert!(report.office_groups.len() >= 2);
        assert!(report
            .coverage_note
            .as_deref()
            .unwrap()
            .contains("Federal (live via FEC)"));
    }

    #[test]
    fn tigerweb_identify_parses_cd() {
        let json = r#"{
          "results": [
            {
              "layerName": "119th Congressional Districts",
              "attributes": {"CD119": "08", "BASENAME": "8", "NAME": "Congressional District 8"}
            },
            {
              "layerName": "2024 State Legislative Districts - Upper",
              "attributes": {"BASENAME": "8", "NAME": "State Senate District 8"}
            },
            {
              "layerName": "2024 State Legislative Districts - Lower",
              "attributes": {"BASENAME": "30", "NAME": "State House District 30"}
            }
          ]
        }"#;
        let g = parse_tigerweb_identify_json(json).unwrap();
        assert_eq!(g.cd_number, Some(8));
        assert_eq!(g.state_senate_district, Some(8));
        assert_eq!(g.state_house_district, Some(30));
        assert_eq!(g.state_house_label.as_deref(), Some("30"));
    }

    #[test]
    fn tigerweb_identify_parses_md_subdistrict() {
        let json = r#"{
          "results": [
            {
              "layerName": "119th Congressional Districts",
              "attributes": {"CD119": "03", "BASENAME": "3", "NAME": "Congressional District 3", "STATE_NAME": "Maryland"}
            },
            {
              "layerName": "2024 State Legislative Districts - Upper",
              "attributes": {"BASENAME": "30", "NAME": "State Senate District 30"}
            },
            {
              "layerName": "2024 State Legislative Districts - Lower",
              "attributes": {"BASENAME": "30A", "NAME": "State Legislative Subdistrict 30A"}
            }
          ]
        }"#;
        let g = parse_tigerweb_identify_json(json).unwrap();
        assert_eq!(g.cd_number, Some(3));
        assert_eq!(g.state_senate_district, Some(30));
        assert_eq!(g.state_house_label.as_deref(), Some("30A"));
        assert_eq!(g.state_house_district, Some(30));
    }

    #[test]
    fn fcc_area_county() {
        let json = r#"{
          "results": [{
            "county_name": "Brevard County",
            "state_name": "Florida",
            "state_code": "FL"
          }]
        }"#;
        let (c, s, code) = parse_fcc_area_json(json).unwrap();
        assert_eq!(c.as_deref(), Some("Brevard County"));
        assert_eq!(s.as_deref(), Some("Florida"));
        assert_eq!(code.as_deref(), Some("FL"));
    }
}
