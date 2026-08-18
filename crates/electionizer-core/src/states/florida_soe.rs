//! Florida county SOE via VoterFocus: ballot locals + campaign finance (A5).
//! Pure parse/map — no HTTP. Browser JS fetches HTML through Wisp.

use crate::models::{
    format_usd, GeoResolution, ResolvedJurisdiction, SnapshotCandidate,
};
use crate::states::florida::{
    classify_local_office, district_from_ballot_office, first_names_compatible,
    fl_judicial_party_label, fl_party_label, last_names_match, names_match, normalize_county_key,
    push_jur, slugify_simple, split_candidate_first_last, title_case_words,
};
use serde::{Deserialize, Serialize};

pub const VF_BASE: &str = "https://www.voterfocus.com/CampaignFinance";
pub const VF_PUBLISHER: &str = "County Supervisor of Elections (VoterFocus)";
pub const FL_SOE_DIRECTORY_URL: &str =
    "https://dos.myflorida.com/elections/contacts/supervisor-of-elections/";

/// VoterFocus `c=` county token for a geo county name, when supported.
/// Pilot documented: Brevard (ZIP 32901). Same HTML shape covers many FL SOEs.
pub fn voterfocus_county_param(county: &str) -> Option<&'static str> {
    let k = normalize_county_key(county);
    Some(match k.as_str() {
        "alachua" => "Alachua",
        "bay" => "Bay",
        "brevard" => "Brevard",
        "broward" => "Broward",
        "charlotte" => "Charlotte",
        "clay" => "Clay",
        "collier" => "Collier",
        "duval" => "Duval",
        "escambia" => "Escambia",
        "hernando" => "Hernando",
        "hillsborough" => "Hillsborough",
        "indian river" => "Indian River",
        "lake" => "Lake",
        "lee" => "Lee",
        "leon" => "Leon",
        "manatee" => "Manatee",
        "marion" => "Marion",
        "miami dade" => "Miami-Dade",
        "orange" => "Orange",
        "osceola" => "Osceola",
        "palm beach" => "Palm Beach",
        "pasco" => "Pasco",
        "pinellas" => "Pinellas",
        "polk" => "Polk",
        "santa rosa" => "Santa Rosa",
        "sarasota" => "Sarasota",
        "seminole" => "Seminole",
        "st johns" | "saint johns" => "St. Johns",
        "volusia" => "Volusia",
        _ => return None,
    })
}

/// Public candidate reports list (default election selected by portal).
pub fn fl_soe_candidate_list_url(county: &str) -> Option<String> {
    let c = voterfocus_county_param(county)?;
    Some(format!(
        "{VF_BASE}/candidate_pr.php?c={}",
        urlenc_component(c)
    ))
}

/// SOE public site for known pilot / major counties; else DOS SOE directory.
pub fn fl_soe_contact_url(county: &str) -> String {
    let k = normalize_county_key(county);
    match k.as_str() {
        "brevard" => "https://www.votebrevard.gov/Candidate-Information/Candidate-Contributions-and-Expenditures".into(),
        "orange" => "https://www.voteorangefl.gov/".into(),
        "pinellas" => "https://www.votepinellas.gov/".into(),
        "hillsborough" => "https://www.votehillsborough.gov/".into(),
        "miami dade" => "https://www.votemiamidade.gov/".into(),
        "broward" => "https://www.browardsoe.org/".into(),
        "palm beach" => "https://www.votepalmbeach.gov/".into(),
        "duval" => "https://www.duvalelections.com/".into(),
        "lee" => "https://www.lee.vote/".into(),
        "polk" => "https://www.polkelections.com/".into(),
        "volusia" => "https://www.volusiaelections.gov/".into(),
        "seminole" => "https://www.voteseminole.org/".into(),
        _ => FL_SOE_DIRECTORY_URL.into(),
    }
}

fn urlenc_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for b in s.as_bytes() {
        match *b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*b as char);
            }
            b' ' => out.push_str("%20"),
            _ => {
                out.push('%');
                out.push_str(&format!("{b:02X}"));
            }
        }
    }
    out
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlSoeHit {
    pub candidate_id: u32,
    pub name: String,
    pub party: String,
    pub office: String,
    pub status: String,
    pub monetary: Option<f64>,
    pub in_kind: Option<f64>,
    pub expenditures: Option<f64>,
    pub election_id: Option<u32>,
    pub county_slug: String,
    pub detail_path: String,
}

#[derive(Debug, Clone)]
pub struct FlSoeMatchQuery {
    pub name: String,
    pub office: String,
    pub chamber: String,
    pub party: String,
    pub district: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind")]
pub enum FlSoeMatch {
    #[serde(rename = "unique")]
    Unique { hit: FlSoeHit },
    #[serde(rename = "none")]
    None,
    #[serde(rename = "ambiguous")]
    Ambiguous { count: usize },
}

#[derive(Debug, Clone, Serialize)]
pub struct FlSoeFinance {
    pub source: String,
    pub cycle: String,
    pub account: String,
    pub match_name: String,
    pub match_office: String,
    pub receipts_display: String,
    pub disbursements_display: String,
    pub cash_on_hand_display: String,
    pub in_kind_display: String,
    pub source_label: String,
    pub profile_url: String,
    pub note: String,
}

fn parse_money(s: &str) -> Option<f64> {
    let t = s
        .replace('$', "")
        .replace(',', "")
        .replace('(', "-")
        .replace(')', "")
        .trim()
        .to_string();
    if t.is_empty() || t == "—" || t == "-" {
        return None;
    }
    t.parse().ok()
}

fn decode_basic_entities(s: &str) -> String {
    s.replace("&#34;", "\"")
        .replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
}

/// Strip trailing `(DEM)` / `(REP)` party tag from VoterFocus name cell.
pub fn split_vf_name_party(raw: &str) -> (String, String) {
    let t = decode_basic_entities(raw).trim().to_string();
    if let Some(open) = t.rfind('(') {
        if t.ends_with(')') {
            let name = t[..open].trim().to_string();
            let party = t[open + 1..t.len() - 1].trim().to_string();
            if !name.is_empty() && party.len() <= 8 {
                return (name, party);
            }
        }
    }
    (t, String::new())
}

fn strip_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    decode_basic_entities(&out)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Parse VoterFocus `candidate_pr.php` list HTML into hits.
pub fn parse_fl_soe_candidate_list_html(html: &str) -> Result<Vec<FlSoeHit>, String> {
    if html.trim().is_empty() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    // Split by office groups when present.
    let office_re = regex_lite(r#"(?is)<div class="col-xs-12 officename">\s*Office:\s*([^<]+)</div>"#);
    let row_re = regex_lite(
        r#"(?is)<a class="rowlink" href="(/CampaignFinance/candidate_pr\.php\?op=cv[^"]+)"[^>]*>\s*<div class="col-sm-6[^"]*">.*?<div class="col-xs-7[^"]*"[^>]*>\s*([^<]+?)\s*</div>.*?statustext[^>]*>([^<]*)</span>.*?<span class="for-screen-reader">monetary</span>\s*(\$[^<]*).*?<span class="for-screen-reader">in-kind</span>\s*(\$[^<]*).*?<span class="for-screen-reader">expenditures</span>\s*(\$[^<]*)"#,
    );

    // Walk office sections: find each officename then rows until next officename.
    let mut offices: Vec<(usize, String)> = Vec::new();
    if let Some(re) = &office_re {
        for cap in re.captures_iter(html) {
            let start = cap.get(0).map(|m| m.start()).unwrap_or(0);
            let name = strip_tags(cap.get(1).map(|m| m.as_str()).unwrap_or("")).trim().to_string();
            if !name.is_empty() {
                offices.push((start, name));
            }
        }
    }

    let office_at = |pos: usize| -> String {
        let mut cur = String::new();
        for (start, name) in &offices {
            if *start <= pos {
                cur = name.clone();
            } else {
                break;
            }
        }
        cur
    };

    if let Some(re) = &row_re {
        for cap in re.captures_iter(html) {
            let full = cap.get(0).map(|m| m.start()).unwrap_or(0);
            let path = decode_basic_entities(cap.get(1).map(|m| m.as_str()).unwrap_or(""));
            let name_raw = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            let status = strip_tags(cap.get(3).map(|m| m.as_str()).unwrap_or(""));
            let monetary = parse_money(&strip_tags(cap.get(4).map(|m| m.as_str()).unwrap_or("")));
            let in_kind = parse_money(&strip_tags(cap.get(5).map(|m| m.as_str()).unwrap_or("")));
            let expenditures =
                parse_money(&strip_tags(cap.get(6).map(|m| m.as_str()).unwrap_or("")));
            let (name, party) = split_vf_name_party(name_raw);
            if name.is_empty() {
                continue;
            }
            let ca = path
                .split("ca=")
                .nth(1)
                .and_then(|s| s.split('&').next())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let election_id = path
                .split("e=")
                .nth(1)
                .and_then(|s| s.split('&').next())
                .and_then(|s| s.parse().ok());
            let county_slug = path
                .split("c=")
                .nth(1)
                .and_then(|s| s.split('&').next())
                .unwrap_or("")
                .to_string();
            out.push(FlSoeHit {
                candidate_id: ca,
                name,
                party,
                office: office_at(full),
                status,
                monetary,
                in_kind,
                expenditures,
                election_id,
                county_slug,
                detail_path: path,
            });
        }
    }

    if out.is_empty() && html.contains("candidate_pr.php?op=cv") {
        return Err("VoterFocus list HTML present but no candidate rows parsed".into());
    }
    Ok(out)
}

fn regex_lite(pat: &str) -> Option<regex::Regex> {
    regex::Regex::new(pat).ok()
}

fn parties_conflict(a: &str, b: &str) -> bool {
    let norm = |p: &str| -> Option<&'static str> {
        let u = p.trim().to_ascii_uppercase();
        if u.is_empty() || u == "NPA" || u == "WRI" || u == "N" {
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

fn office_compatible(q: &FlSoeMatchQuery, hit: &FlSoeHit) -> bool {
    let ho = hit.office.to_ascii_lowercase();
    let qo = q.office.to_ascii_lowercase();
    let ch = q.chamber.to_ascii_lowercase();
    if ho.is_empty() {
        return true;
    }
    if ch == "state_senate" || (qo.contains("senate") && !qo.contains("us")) {
        return ho.contains("senator") || ho.contains("senate");
    }
    if ch == "state_house" || qo.contains("representative") || qo.contains("state house") {
        return ho.contains("representative") || ho.contains("house");
    }
    if ch == "judicial" || qo.contains("judge") || qo.contains("court") {
        return ho.contains("judge") || ho.contains("court") || ho.contains("judicial");
    }
    if ch == "county"
        || qo.contains("commission")
        || qo.contains("school")
        || qo.contains("clerk")
        || qo.contains("sheriff")
        || qo.contains("property appraiser")
        || qo.contains("tax collector")
        || qo.contains("supervisor of elections")
    {
        // Soft token overlap
        for t in [
            "commission",
            "school",
            "clerk",
            "sheriff",
            "appraiser",
            "tax collector",
            "supervisor",
            "mayor",
            "council",
            "commissioner",
        ] {
            if qo.contains(t) {
                return ho.contains(t);
            }
        }
        // county chamber without specific token — accept county-level offices
        return !ho.contains("united states") && !ho.contains("congress");
    }
    if !qo.is_empty() {
        // generic: share a significant word
        for w in qo.split(|c: char| !c.is_ascii_alphanumeric()) {
            if w.len() < 5 {
                continue;
            }
            if ho.contains(w) {
                return true;
            }
        }
    }
    true
}

fn district_compatible(q: &FlSoeMatchQuery, hit: &FlSoeHit) -> bool {
    let q_dist = q.district.or_else(|| district_from_ballot_office(&q.office));
    let h_dist = district_from_ballot_office(&hit.office);
    match (q_dist, h_dist) {
        (Some(a), Some(b)) => a == b,
        _ => true,
    }
}

pub fn match_fl_soe_candidate(hits: &[FlSoeHit], q: &FlSoeMatchQuery) -> FlSoeMatch {
    let matched: Vec<&FlSoeHit> = hits
        .iter()
        .filter(|h| last_names_match(&q.name, &h.name))
        .filter(|h| first_names_compatible(&q.name, &h.name))
        .filter(|h| office_compatible(q, h))
        .filter(|h| district_compatible(q, h))
        .filter(|h| !parties_conflict(&q.party, &h.party))
        .collect();
    match matched.len() {
        0 => FlSoeMatch::None,
        1 => FlSoeMatch::Unique {
            hit: matched[0].clone(),
        },
        n => FlSoeMatch::Ambiguous { count: n },
    }
}

pub fn fl_soe_profile_url(hit: &FlSoeHit) -> String {
    if hit.detail_path.starts_with("http") {
        return hit.detail_path.clone();
    }
    if hit.detail_path.starts_with('/') {
        return format!("https://www.voterfocus.com{}", hit.detail_path);
    }
    format!("{VF_BASE}/{}", hit.detail_path.trim_start_matches('/'))
}

pub fn fl_soe_finance_from_hit(hit: &FlSoeHit, cycle: i32, county_label: &str) -> FlSoeFinance {
    let receipts = match (hit.monetary, hit.in_kind) {
        (Some(m), Some(k)) => Some(m + k),
        (Some(m), None) => Some(m),
        (None, Some(k)) => Some(k),
        _ => None,
    };
    let county = if county_label.trim().is_empty() {
        hit.county_slug.clone()
    } else {
        county_label.trim().to_string()
    };
    FlSoeFinance {
        source: "fl_soe".into(),
        cycle: cycle.to_string(),
        account: hit.candidate_id.to_string(),
        match_name: hit.name.clone(),
        match_office: hit.office.clone(),
        receipts_display: receipts.map(format_usd).unwrap_or_else(|| "—".into()),
        disbursements_display: hit
            .expenditures
            .map(format_usd)
            .unwrap_or_else(|| "—".into()),
        cash_on_hand_display: "—".into(),
        in_kind_display: hit.in_kind.map(format_usd).unwrap_or_else(|| "—".into()),
        source_label: format!("{county} SOE via VoterFocus"),
        profile_url: fl_soe_profile_url(hit),
        note: "Monetary + in-kind contributions and expenditures from the county SOE VoterFocus candidate overview for the selected election. Filed portal totals — not a bank audit.".into(),
    }
}

/// VF / SOE status that belongs on a ballot (not DNQ / withdrawn / redesignated).
pub fn soe_status_on_ballot(status: &str) -> bool {
    let s = status.to_ascii_lowercase();
    if s.contains("did not")
        || s.contains("dnq")
        || s.contains("withdraw")
        || s.contains("redesignat")
    {
        return false;
    }
    s.contains("qualified") || s.contains("unopposed")
}

/// Offices DOS + FEC already cover — skip so we do not double-list.
pub fn soe_office_is_state_or_federal(office: &str) -> bool {
    let o = office.to_ascii_lowercase();
    o.contains("united states")
        || o.contains("u.s.")
        || o.contains("us senate")
        || o.contains("us house")
        || o.contains("congress")
        || o.contains("governor")
        || o.contains("attorney general")
        || o.contains("chief financial")
        || o.contains("commissioner of agriculture")
        || o.contains("state senator")
        || o.contains("state representative")
        || o.contains("supreme court")
        || o.contains("district court of appeal")
        || o.contains("circuit judge")
        || o.contains("circuit court judge")
}

fn soe_is_county_judge(office: &str) -> bool {
    let o = office.to_ascii_lowercase();
    (o.contains("county judge") || o.contains("county court judge")) && !o.contains("circuit")
}

/// Countywide / county-district races that belong on the ZIP ballot without a precinct PDF.
pub fn soe_office_is_default_local(office: &str) -> bool {
    if soe_office_is_state_or_federal(office) {
        return false;
    }
    let o = office.to_ascii_lowercase();
    o.contains("county commission")
        || o.contains("board of county")
        || o.contains("school board")
        || soe_is_county_judge(office)
        || o.contains("port authority")
        || o.contains("soil and water")
        || o.contains("sheriff")
        || o.contains("clerk of")
        || o.contains("property appraiser")
        || o.contains("tax collector")
        || o.contains("supervisor of elections")
}

fn group_from_office(office: &str) -> Option<u32> {
    let re = regex::Regex::new(r"(?i)\bgroup\s+#?(\d+)\b").ok()?;
    re.captures(office)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

fn collapse_ws_lower(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// True when extracted sample-ballot text looks usable for contest filtering.
pub fn sample_ballot_text_usable(text: &str) -> bool {
    let s = text.to_ascii_lowercase();
    let hits = [
        s.contains("ballot"),
        s.contains("precinct"),
        s.contains("commission"),
        s.contains("school"),
        s.contains("judge"),
        s.contains("port"),
    ]
    .into_iter()
    .filter(|b| *b)
    .count();
    text.len() >= 80 && hits >= 2
}

fn sample_has_district(sample: &str, n: u32) -> bool {
    let pats = [
        format!("district {n}"),
        format!("dist {n}"),
        format!("district{n}"),
    ];
    pats.iter().any(|p| sample.contains(p))
}

fn sample_has_group(sample: &str, n: u32) -> bool {
    sample.contains(&format!("group {n}")) || sample.contains(&format!("group{n}"))
}

fn sample_window_has(sample: &str, family: &str, n: Option<u32>) -> bool {
    let s = collapse_ws_lower(sample);
    let fam = family.trim().to_ascii_lowercase();
    if fam.is_empty() {
        return false;
    }
    let mut start = 0;
    while let Some(rel) = s[start..].find(&fam) {
        let abs = start + rel;
        let hi = (abs + fam.len() + 36).min(s.len());
        let win = &s[abs..hi];
        if let Some(d) = n {
            if sample_has_district(win, d) {
                return true;
            }
        } else {
            return true;
        }
        start = abs + fam.len();
        if start >= s.len() {
            break;
        }
    }
    false
}

/// Districted local: keep when the official sample names this contest.
/// Countywide (judge, soil & water, constitutional officers) always kept.
pub fn sample_ballot_covers_office(sample_text: &str, office: &str) -> bool {
    if !sample_ballot_text_usable(sample_text) {
        return soe_office_is_default_local(office);
    }
    let s = collapse_ws_lower(sample_text);
    let o = office.to_ascii_lowercase();
    if soe_is_county_judge(office) {
        return s.contains("county") && s.contains("judge");
    }
    if o.contains("soil") && o.contains("water") {
        return true;
    }
    if o.contains("sheriff")
        || o.contains("clerk of")
        || o.contains("property appraiser")
        || o.contains("tax collector")
        || o.contains("supervisor of elections")
    {
        return true;
    }
    let dist = district_from_ballot_office(office);
    if o.contains("port") {
        return sample_window_has(sample_text, "port", dist);
    }
    if o.contains("school") {
        return sample_window_has(sample_text, "school", dist);
    }
    if o.contains("commission") {
        return sample_window_has(sample_text, "commission", dist);
    }
    let key = collapse_ws_lower(office);
    if key.len() >= 8 && s.contains(&key) {
        return true;
    }
    if let Some(g) = group_from_office(office) {
        if sample_has_group(&s, g) {
            return true;
        }
    }
    false
}

fn soe_party_label(office: &str, party: &str, status: &str) -> String {
    let st = status.to_ascii_lowercase();
    if party.trim().is_empty() && st.contains("write") {
        return "Write-In".into();
    }
    if soe_is_county_judge(office)
        || office.to_ascii_lowercase().contains("school board")
        || office.to_ascii_lowercase().contains("soil and water")
    {
        return fl_judicial_party_label(party);
    }
    fl_party_label(party)
}

fn soe_external_id(hit: &FlSoeHit) -> Option<String> {
    if hit.candidate_id == 0 {
        None
    } else {
        Some(format!("fl:vf:{}", hit.candidate_id))
    }
}

/// Same person + office family already on the DOS ballot.
pub fn soe_duplicates_existing(existing: &SnapshotCandidate, soe: &SnapshotCandidate) -> bool {
    if !names_match(&existing.name, &soe.name) {
        return false;
    }
    let eo = existing.office.to_ascii_lowercase();
    let so = soe.office.to_ascii_lowercase();
    let same_fam = (eo.contains("judge") && so.contains("judge"))
        || (eo.contains("commission") && so.contains("commission"))
        || (eo.contains("school") && so.contains("school"))
        || (eo.contains("port") && so.contains("port"))
        || (eo.contains("sheriff") && so.contains("sheriff"))
        || (eo.contains("clerk") && so.contains("clerk"));
    if !same_fam {
        return false;
    }
    let ed = district_from_ballot_office(&existing.office).or_else(|| group_from_office(&existing.office));
    let sd = district_from_ballot_office(&soe.office).or_else(|| group_from_office(&soe.office));
    match (ed, sd) {
        (Some(a), Some(b)) => a == b,
        _ => true,
    }
}

/// Map VF candidate-list hits onto this county's ballot.
///
/// `sample_text`: official sample-ballot extract when precinct is set; empty = default county locals.
pub fn map_soe_hits_for_geo(
    hits: &[FlSoeHit],
    geo: &GeoResolution,
    sample_text: &str,
    extra_jurisdictions: &mut Vec<ResolvedJurisdiction>,
) -> Vec<SnapshotCandidate> {
    let county_key = normalize_county_key(&geo.county);
    if county_key.is_empty() {
        return Vec::new();
    }
    let state_l = "fl";
    let county_slug = slugify_simple(&county_key);
    let county_ocd = format!("ocd-division/country:us/state:{state_l}/county:{county_slug}");
    let county_name = if geo.county.trim().is_empty() {
        format!("{} County", title_case_words(&county_key))
    } else {
        geo.county.clone()
    };
    let city_l = geo.city.trim().to_ascii_lowercase();
    let filter_by_sample = sample_ballot_text_usable(sample_text);

    let mut out = Vec::new();
    for hit in hits {
        if !soe_status_on_ballot(&hit.status) {
            continue;
        }
        if soe_office_is_state_or_federal(&hit.office) {
            continue;
        }
        let office = hit.office.trim();
        if office.is_empty() {
            continue;
        }
        let is_judge = soe_is_county_judge(office);
        let (chamber, level) = if is_judge {
            ("judicial", "county")
        } else {
            classify_local_office("", office)
        };

        let include = if filter_by_sample {
            sample_ballot_covers_office(sample_text, office)
        } else if soe_office_is_default_local(office) {
            true
        } else if chamber == "municipal" {
            !city_l.is_empty() && office.to_ascii_lowercase().contains(&city_l)
        } else {
            false
        };
        if !include {
            continue;
        }

        let (ocd, jur_name, jur_level) = if chamber == "municipal" {
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
        } else if chamber == "special_district" {
            let slug = slugify_simple(office);
            let ocd = format!(
                "ocd-division/country:us/state:{state_l}/county:{county_slug}/special_district:{slug}"
            );
            (ocd, office.to_string(), "special_district")
        } else {
            (county_ocd.clone(), county_name.clone(), level)
        };
        push_jur(extra_jurisdictions, &ocd, &jur_name, jur_level);
        if chamber != "municipal" {
            push_jur(extra_jurisdictions, &county_ocd, &county_name, "county");
        }

        let party = soe_party_label(office, &hit.party, &hit.status);
        let source_url = fl_soe_profile_url(hit);
        let status_l = hit.status.trim();
        out.push(SnapshotCandidate {
            office: office.to_string(),
            chamber: Some(chamber.into()),
            jurisdiction_ocd: ocd,
            is_judicial: is_judge,
            name: hit.name.clone(),
            party,
            is_incumbent: false,
            is_judge,
            summary: Some(format!(
                "{status_l} · {office}. Source: {VF_PUBLISHER}."
            )),
            source_url,
            source_publisher: Some(VF_PUBLISHER.into()),
            external_id: soe_external_id(hit),
        });
    }
    out
}

pub fn fl_soe_search_name_fragment(ballot_name: &str) -> String {
    let (_f, last) = split_candidate_first_last(ballot_name);
    if !last.is_empty() {
        last
    } else {
        ballot_name.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brevard_list_parse_match() {
        let html = include_str!("../../../../testdata/fl_soe_brevard_candidate_pr.html");
        let hits = parse_fl_soe_candidate_list_html(html).expect("parse");
        assert!(hits.len() >= 3, "got {}", hits.len());
        let book = hits
            .iter()
            .find(|h| h.name.contains("Bookhardt"))
            .expect("Bookhardt");
        assert!(book.office.to_ascii_lowercase().contains("commissioner"));
        assert!(book.party.to_ascii_uppercase().contains('D') || book.party == "DEM");
        assert!(book.status.to_ascii_lowercase().contains("qualified"));
        assert!(book.monetary.unwrap_or(0.0) > 0.0);

        let q = FlSoeMatchQuery {
            name: "Aldon Bookhardt".into(),
            office: "County Commissioner, District 1".into(),
            chamber: "county".into(),
            party: "Democratic".into(),
            district: Some(1),
        };
        match match_fl_soe_candidate(&hits, &q) {
            FlSoeMatch::Unique { hit } => {
                assert!(hit.name.contains("Bookhardt"));
                let fin = fl_soe_finance_from_hit(&hit, 2026, "Brevard County");
                assert_eq!(fin.source, "fl_soe");
                assert!(fin.receipts_display.contains("$"));
                assert!(fin.profile_url.contains("candidate_pr.php"));
                assert!(fin.profile_url.contains("ca="));
            }
            other => panic!("expected unique {other:?}"),
        }

        assert_eq!(voterfocus_county_param("Brevard County"), Some("Brevard"));
        assert_eq!(voterfocus_county_param("Miami-Dade"), Some("Miami-Dade"));
        assert!(voterfocus_county_param("Nowhere").is_none());
        let url = fl_soe_candidate_list_url("Brevard").unwrap();
        assert!(url.contains("candidate_pr.php"));
        assert!(url.contains("Brevard"));
        assert!(fl_soe_contact_url("Brevard").contains("votebrevard"));
        assert!(fl_soe_contact_url("Mystery").contains("supervisor-of-elections"));
    }

    #[test]
    fn name_party_split() {
        let (n, p) = split_vf_name_party("Micah Loyd (REP)");
        assert_eq!(n, "Micah Loyd");
        assert_eq!(p, "REP");
    }

    fn brevard_geo() -> GeoResolution {
        GeoResolution {
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
        }
    }

    fn hit(name: &str, office: &str, party: &str, status: &str, id: u32) -> FlSoeHit {
        FlSoeHit {
            candidate_id: id,
            name: name.into(),
            party: party.into(),
            office: office.into(),
            status: status.into(),
            monetary: None,
            in_kind: None,
            expenditures: None,
            election_id: Some(30),
            county_slug: "brevard".into(),
            detail_path: format!(
                "/CampaignFinance/candidate_pr.php?op=cv&e=30&c=brevard&ca={id}&rellevel=4&committee=N"
            ),
        }
    }

    #[test]
    fn map_soe_default_includes_missing_county_races() {
        let hits = vec![
            hit("Pam Avery", "County Commissioner, District 1", "REP", "Active-Qualified", 1089),
            hit("Myron Horne", "County Commissioner, District 1", "REP", "Active-Did not qualify", 1092),
            hit("Tyler Sirois", "County Commissioner, District 2", "REP", "Active-Unopposed", 1046),
            hit("Vince Jackson", "Canaveral Port Authority, District 3", "REP", "Active-Qualified", 1095),
            hit("Carol Craig", "Canaveral Port Authority, District 5", "REP", "Active-Qualified", 1098),
            hit("Rodney Antonio Edwards", "County Court Judge, Group 3", "", "Active-Qualified", 1056),
            hit("Timi Tucker", "County Court Judge, Group 3", "", "Active-Qualified", 1054),
            hit("Tara Gibson", "School Board, District 1", "", "Active-Qualified", 1075),
            hit("Megan Wright", "School Board, District 1", "", "Active-Qualified", 1079),
            hit("Jan Creitz", "Barefoot Bay Trustee", "", "Active-Qualified", 1200),
            hit("Chase Tramont", "State Representative, District 30", "REP", "Active-Qualified", 1),
        ];
        let mut extra = Vec::new();
        let cands = map_soe_hits_for_geo(&hits, &brevard_geo(), "", &mut extra);
        assert!(cands.iter().any(|c| c.name.contains("Avery") && c.office.contains("District 1")));
        assert!(cands.iter().any(|c| c.name.contains("Sirois")));
        assert!(cands.iter().any(|c| c.name.contains("Jackson") && c.office.contains("Port")));
        assert!(cands.iter().any(|c| c.name.contains("Craig") && c.office.contains("District 5")));
        assert!(cands.iter().any(|c| c.name.contains("Edwards") && c.is_judge));
        assert!(cands.iter().any(|c| c.name.contains("Tucker") && c.office.contains("Group 3")));
        assert!(cands.iter().any(|c| c.name.contains("Gibson") && c.office.contains("School")));
        assert!(cands.iter().any(|c| c.name.contains("Wright")));
        assert!(!cands.iter().any(|c| c.name.contains("Horne")), "DNQ skipped");
        assert!(!cands.iter().any(|c| c.name.contains("Creitz")), "CDD/rec skipped without sample");
        assert!(!cands.iter().any(|c| c.name.contains("Tramont")), "state skipped");
        let judge = cands.iter().find(|c| c.name.contains("Edwards")).unwrap();
        assert_eq!(judge.party, "Nonpartisan");
        assert_eq!(judge.chamber.as_deref(), Some("judicial"));
        assert_eq!(judge.external_id.as_deref(), Some("fl:vf:1056"));
        assert!(judge.source_url.contains("ca=1056"));
    }

    #[test]
    fn map_soe_sample_keeps_precinct_districts() {
        let hits = vec![
            hit("Pam Avery", "County Commissioner, District 1", "REP", "Active-Qualified", 1089),
            hit("Tyler Sirois", "County Commissioner, District 2", "REP", "Active-Unopposed", 1046),
            hit("Vince Jackson", "Canaveral Port Authority, District 3", "REP", "Active-Qualified", 1095),
            hit("Stan Retz", "Canaveral Port Authority, District 1", "REP", "Active-Unopposed", 1090),
            hit("Carol Craig", "Canaveral Port Authority, District 5", "REP", "Active-Qualified", 1098),
            hit("Tara Gibson", "School Board, District 1", "", "Active-Qualified", 1075),
            hit("Kyle Savage", "School Board, District 2", "", "Active-Qualified", 1082),
            hit("Rodney Edwards", "County Court Judge, Group 3", "", "Active-Qualified", 1056),
            hit("Jan Creitz", "Barefoot Bay Trustee", "", "Active-Qualified", 1200),
        ];
        let sample = "\
Official Sample Ballot Precinct 104 Republican
Board of County Commissioners District 1
Canaveral Port Authority District 3
Canaveral Port Authority District 5
County Court Judge Group 3
School Board Member District 1
";
        let mut extra = Vec::new();
        let cands = map_soe_hits_for_geo(&hits, &brevard_geo(), sample, &mut extra);
        assert!(cands.iter().any(|c| c.name.contains("Avery")));
        assert!(!cands.iter().any(|c| c.name.contains("Sirois")), "wrong BCC district");
        assert!(cands.iter().any(|c| c.name.contains("Jackson")));
        assert!(cands.iter().any(|c| c.name.contains("Craig")));
        assert!(!cands.iter().any(|c| c.name.contains("Retz")), "wrong port district");
        assert!(cands.iter().any(|c| c.name.contains("Gibson")));
        assert!(!cands.iter().any(|c| c.name.contains("Savage")), "wrong school district");
        assert!(cands.iter().any(|c| c.name.contains("Edwards")));
        assert!(!cands.iter().any(|c| c.name.contains("Creitz")));
    }

    #[test]
    fn soe_status_and_dedup() {
        assert!(soe_status_on_ballot("Active-Qualified"));
        assert!(soe_status_on_ballot("Active-Qualified Write-In"));
        assert!(soe_status_on_ballot("Active-Unopposed"));
        assert!(!soe_status_on_ballot("Active-Did not qualify"));
        assert!(!soe_status_on_ballot("Inactive-Withdrawn"));
        assert!(!soe_status_on_ballot("Active-Redesignated"));
        let dos = SnapshotCandidate {
            office: "County Judge (Group 3)".into(),
            chamber: Some("judicial".into()),
            jurisdiction_ocd: "ocd-division/country:us/state:fl/county:brevard".into(),
            is_judicial: true,
            name: "Rodney Edwards".into(),
            party: "Nonpartisan".into(),
            is_incumbent: false,
            is_judge: true,
            summary: None,
            source_url: String::new(),
            source_publisher: None,
            external_id: Some("fl:acct:90110".into()),
        };
        let soe = SnapshotCandidate {
            office: "County Court Judge, Group 3".into(),
            chamber: Some("judicial".into()),
            jurisdiction_ocd: "ocd-division/country:us/state:fl/county:brevard".into(),
            is_judicial: true,
            name: "Rodney Antonio Edwards".into(),
            party: "Nonpartisan".into(),
            is_incumbent: false,
            is_judge: true,
            summary: None,
            source_url: String::new(),
            source_publisher: None,
            external_id: Some("fl:vf:1056".into()),
        };
        assert!(soe_duplicates_existing(&dos, &soe));
    }

    #[test]
    fn map_soe_from_brevard_list_fixture() {
        let html = include_str!("../../../../testdata/fl_soe_brevard_candidate_pr.html");
        let hits = parse_fl_soe_candidate_list_html(html).expect("parse");
        let mut extra = Vec::new();
        let cands = map_soe_hits_for_geo(&hits, &brevard_geo(), "", &mut extra);
        assert!(cands.iter().any(|c| c.name.contains("Bookhardt")));
        assert!(cands.iter().any(|c| c.office.contains("County Commissioner")));
        assert!(!cands.iter().any(|c| c.name.contains("Horne")));
    }
}
