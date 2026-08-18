//! Pure CourtListener (Free Law Project) JSON helpers — no HTTP.
//! People match, career positions → spans/facts, opinion search → VoteRecord.

use crate::bio::{BioFact, CareerSpan, LifeCategory};
use crate::models::VoteRecord;
use crate::openstates::{last_name, normalize_name_key};
use crate::states::florida::first_names_compatible;
use serde::Serialize;
use serde_json::Value;

pub const CL_SOURCE: &str = "CourtListener";
pub const CL_API_ROOT: &str = "https://www.courtlistener.com/api/rest/v4";
pub const CL_HOME: &str = "https://www.courtlistener.com/";
/// Cap opinions loaded into detail (search page size).
pub const CL_OPINIONS_CAP: usize = 100;

#[derive(Debug, Clone, Serialize)]
pub struct CourtListenerPersonMatch {
    pub person_id: i64,
    pub name: String,
    pub slug: Option<String>,
    pub profile_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub birth_year: Option<i32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub court_ids: Vec<String>,
}

/// Map ballot office + state → preferred CourtListener court id(s).
/// Empty when unknown — people search is name-only then.
pub fn courtlistener_court_ids(state: &str, office: &str) -> Vec<String> {
    let st = state.trim().to_ascii_lowercase();
    let o = office.to_ascii_lowercase();
    let mut out = Vec::new();

    if o.contains("supreme") {
        if let Some(id) = state_supreme_court_id(&st) {
            out.push(id.into());
        }
        return out;
    }

    if st == "fl" || st == "florida" {
        if o.contains("district court of appeal")
            || o.contains("dca")
            || o.contains("dist. ct. app")
            || o.contains("district court")
        {
            out.push("fladistctapp".into());
            return out;
        }
        // Circuit / county trial courts — CL coverage thin; no stable id required.
        return out;
    }

    if st == "md" || st == "maryland" {
        if o.contains("court of appeals") || o.contains("supreme") {
            out.push("md".into());
        } else if o.contains("appellate") || o.contains("special appeals") {
            out.push("mdctspecapp".into());
        }
        return out;
    }

    if st == "nc" || st == "north carolina" {
        if o.contains("supreme") {
            out.push("nc".into());
        } else if o.contains("court of appeals") || o.contains("appellate") {
            out.push("ncctapp".into());
        }
        return out;
    }

    if st == "az" || st == "arizona" {
        if o.contains("supreme") {
            out.push("ariz".into());
        } else if o.contains("court of appeals") || o.contains("appellate") {
            out.push("arizctapp".into());
        }
        return out;
    }

    out
}

fn state_supreme_court_id(st: &str) -> Option<&'static str> {
    match st {
        "fl" | "florida" => Some("fla"),
        "md" | "maryland" => Some("md"),
        "nc" | "north carolina" => Some("nc"),
        "az" | "arizona" => Some("ariz"),
        "al" | "alabama" => Some("ala"),
        "ak" | "alaska" => Some("alaska"),
        "ar" | "arkansas" => Some("ark"),
        "ca" | "california" => Some("cal"),
        "co" | "colorado" => Some("colo"),
        "ct" | "connecticut" => Some("conn"),
        "de" | "delaware" => Some("del"),
        "ga" | "georgia" => Some("ga"),
        "hi" | "hawaii" => Some("haw"),
        "id" | "idaho" => Some("idaho"),
        "il" | "illinois" => Some("ill"),
        "in" | "indiana" => Some("ind"),
        "ia" | "iowa" => Some("iowa"),
        "ks" | "kansas" => Some("kan"),
        "ky" | "kentucky" => Some("ky"),
        "la" | "louisiana" => Some("la"),
        "me" | "maine" => Some("me"),
        "ma" | "massachusetts" => Some("mass"),
        "mi" | "michigan" => Some("mich"),
        "mn" | "minnesota" => Some("minn"),
        "ms" | "mississippi" => Some("miss"),
        "mo" | "missouri" => Some("mo"),
        "mt" | "montana" => Some("mont"),
        "ne" | "nebraska" => Some("neb"),
        "nv" | "nevada" => Some("nev"),
        "nh" | "new hampshire" => Some("nh"),
        "nj" | "new jersey" => Some("nj"),
        "nm" | "new mexico" => Some("nm"),
        "ny" | "new york" => Some("ny"),
        "nd" | "north dakota" => Some("nd"),
        "oh" | "ohio" => Some("ohio"),
        "ok" | "oklahoma" => Some("okla"),
        "or" | "oregon" => Some("or"),
        "pa" | "pennsylvania" => Some("pa"),
        "ri" | "rhode island" => Some("ri"),
        "sc" | "south carolina" => Some("sc"),
        "sd" | "south dakota" => Some("sd"),
        "tn" | "tennessee" => Some("tenn"),
        "tx" | "texas" => Some("tex"),
        "ut" | "utah" => Some("utah"),
        "vt" | "vermont" => Some("vt"),
        "va" | "virginia" => Some("va"),
        "wa" | "washington" => Some("wash"),
        "wv" | "west virginia" => Some("wva"),
        "wi" | "wisconsin" => Some("wis"),
        "wy" | "wyoming" => Some("wyo"),
        "dc" | "district of columbia" => Some("dc"),
        _ => None,
    }
}

/// Build people search URL (caller fetches). Optional first name tightens match.
pub fn courtlistener_people_search_url(name: &str) -> Option<String> {
    let (first, last) = split_person_name(name);
    if last.len() < 2 {
        return None;
    }
    let mut u = format!(
        "{CL_API_ROOT}/people/?name_last={}",
        urlencoding_minimal(&last)
    );
    if first.len() >= 2 {
        u.push_str(&format!("&name_first={}", urlencoding_minimal(&first)));
    }
    u.push_str("&page_size=20");
    Some(u)
}

/// Positions list URL for a person id.
pub fn courtlistener_positions_url(person_id: i64) -> String {
    format!("{CL_API_ROOT}/positions/?person={person_id}&page_size=100")
}

/// Opinion search URL (authored) ordered by file date desc.
pub fn courtlistener_opinions_search_url(person_id: i64, page_size: usize) -> String {
    let ps = page_size.clamp(1, CL_OPINIONS_CAP);
    format!(
        "{CL_API_ROOT}/search/?type=o&q=author_id%3A{person_id}&order_by=dateFiled+desc&page_size={ps}"
    )
}

/// Public person profile on CourtListener.
pub fn courtlistener_person_profile_url(person_id: i64, slug: Option<&str>) -> String {
    if let Some(s) = slug.filter(|s| !s.is_empty()) {
        format!("https://www.courtlistener.com/person/{person_id}/{s}/")
    } else {
        format!("https://www.courtlistener.com/person/{person_id}/")
    }
}

/// Search UI for a judge name (portal when no opinions loaded).
pub fn courtlistener_search_portal_url(name: &str) -> Option<String> {
    let q = name.trim();
    if q.len() < 3 {
        return None;
    }
    Some(format!(
        "https://www.courtlistener.com/?type=p&q={}",
        urlencoding_minimal(q)
    ))
}

fn urlencoding_minimal(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn split_person_name(name: &str) -> (String, String) {
    let cleaned = name
        .split("--")
        .next()
        .unwrap_or(name)
        .trim()
        .trim_matches(|c: char| c == '"' || c == '\'');
    if cleaned.contains(',') {
        let mut parts = cleaned.splitn(2, ',');
        let last = parts.next().unwrap_or("").trim().to_string();
        let rest = parts.next().unwrap_or("").trim();
        let first = rest
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_matches(|c: char| !c.is_alphabetic() && c != '-')
            .to_string();
        return (first, last);
    }
    let tokens: Vec<&str> = cleaned.split_whitespace().collect();
    if tokens.is_empty() {
        return (String::new(), String::new());
    }
    let last = tokens[tokens.len() - 1]
        .trim_matches(|c: char| !c.is_alphabetic() && c != '-')
        .to_string();
    let first = tokens[0]
        .trim_matches(|c: char| !c.is_alphabetic() && c != '-')
        .to_string();
    (first, last)
}

fn year_from_iso(s: &str) -> Option<i32> {
    let t = s.trim();
    if t.len() >= 4 {
        t[..4].parse().ok()
    } else {
        None
    }
}

fn person_display_name(p: &Value) -> String {
    let first = p
        .get("name_first")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    let middle = p
        .get("name_middle")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    let last = p
        .get("name_last")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    let suffix = p
        .get("name_suffix")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    let mut parts = Vec::new();
    if !first.is_empty() {
        parts.push(first.to_string());
    }
    if !middle.is_empty() {
        parts.push(middle.to_string());
    }
    if !last.is_empty() {
        parts.push(last.to_string());
    }
    let mut s = parts.join(" ");
    if !suffix.is_empty() {
        s.push(' ');
        s.push_str(suffix);
    }
    if s.is_empty() {
        "Judge".into()
    } else {
        s
    }
}

fn court_id_from_position(pos: &Value) -> Option<String> {
    let court = pos.get("court")?;
    if let Some(id) = court.get("id").and_then(|v| v.as_str()) {
        return Some(id.to_string());
    }
    // Nested resource_uri …/courts/{id}/
    if let Some(uri) = court
        .get("resource_uri")
        .and_then(|v| v.as_str())
        .or_else(|| court.as_str())
    {
        if let Some(i) = uri.find("/courts/") {
            let rest = &uri[i + "/courts/".len()..];
            let id = rest.split(['/', '?', '#']).next().unwrap_or("").trim();
            if !id.is_empty() {
                return Some(id.to_string());
            }
        }
    }
    None
}

/// Match a people list JSON payload to a ballot candidate.
/// `prefer_court_ids` — when non-empty, require a position on one of those courts
/// if we can observe courts from nested position objects; otherwise name-only.
/// Ambiguous multi-hit without court disambiguation → None.
pub fn pick_courtlistener_person(
    people_json: &str,
    candidate_name: &str,
    prefer_court_ids: &[String],
) -> Option<CourtListenerPersonMatch> {
    let root: Value = serde_json::from_str(people_json).ok()?;
    let results = root
        .get("results")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if results.is_empty() {
        return None;
    }

    let want_last = normalize_name_key(&last_name(candidate_name));
    if want_last.len() < 2 {
        return None;
    }
    let (want_first, _) = split_person_name(candidate_name);

    let mut hits: Vec<(CourtListenerPersonMatch, bool)> = Vec::new();
    for p in &results {
        let plast = p
            .get("name_last")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        if normalize_name_key(plast) != want_last {
            continue;
        }
        let pfirst = p
            .get("name_first")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        if !want_first.is_empty()
            && !pfirst.is_empty()
            && !first_names_compatible(candidate_name, &format!("{pfirst} {plast}"))
            && !first_names_compatible(&want_first, pfirst)
        {
            // Soft: initial match "C" vs "Charles"
            let wi = want_first.chars().next().map(|c| c.to_ascii_lowercase());
            let pi = pfirst.chars().next().map(|c| c.to_ascii_lowercase());
            if wi != pi {
                continue;
            }
        }

        let id = p.get("id").and_then(|v| v.as_i64())?;
        let slug = p
            .get("slug")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty());
        let name = person_display_name(p);
        let birth_year = p
            .get("date_dob")
            .and_then(|v| v.as_str())
            .and_then(year_from_iso);
        let mut court_ids = Vec::new();
        if let Some(positions) = p.get("positions").and_then(|v| v.as_array()) {
            for pos in positions {
                if let Some(cid) = court_id_from_position(pos) {
                    if !court_ids.iter().any(|x| x == &cid) {
                        court_ids.push(cid);
                    }
                }
            }
        }
        let court_hit = if prefer_court_ids.is_empty() {
            false
        } else if court_ids.is_empty() {
            // Cannot confirm from list payload — keep as soft candidate.
            false
        } else {
            prefer_court_ids.iter().any(|w| court_ids.iter().any(|c| c == w))
        };

        hits.push((
            CourtListenerPersonMatch {
                person_id: id,
                name,
                slug: slug.clone(),
                profile_url: courtlistener_person_profile_url(id, slug.as_deref()),
                birth_year,
                court_ids,
            },
            court_hit,
        ));
    }

    if hits.is_empty() {
        return None;
    }

    // Prefer court-confirmed hits when prefer list non-empty.
    if !prefer_court_ids.is_empty() {
        let confirmed: Vec<_> = hits
            .iter()
            .filter(|(_, ch)| *ch)
            .map(|(m, _)| m.clone())
            .collect();
        if confirmed.len() == 1 {
            return Some(confirmed.into_iter().next().unwrap());
        }
        if confirmed.len() > 1 {
            // Still ambiguous among same court — skip.
            return None;
        }
        // No confirmed court on list payload: allow unique name match only.
    }

    if hits.len() == 1 {
        return Some(hits.into_iter().next().unwrap().0);
    }

    // Multiple name hits without court disambiguation — skip (cite-or-omit).
    None
}

/// After positions fetch, re-check person has a preferred court (soft filter).
pub fn person_positions_match_courts(positions_json: &str, prefer_court_ids: &[String]) -> bool {
    if prefer_court_ids.is_empty() {
        return true;
    }
    let ids = court_ids_from_positions_json(positions_json);
    if ids.is_empty() {
        // Unknown — do not reject.
        return true;
    }
    prefer_court_ids.iter().any(|w| ids.iter().any(|c| c == w))
}

fn court_ids_from_positions_json(positions_json: &str) -> Vec<String> {
    let Ok(root) = serde_json::from_str::<Value>(positions_json) else {
        return Vec::new();
    };
    let results = root
        .get("results")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut out = Vec::new();
    for pos in results {
        if let Some(cid) = court_id_from_position(&pos) {
            if !out.iter().any(|x| x == &cid) {
                out.push(cid);
            }
        }
    }
    out
}

fn position_type_category(position_type: &str) -> Option<LifeCategory> {
    let t = position_type.trim().to_ascii_lowercase();
    match t.as_str() {
        "jud" | "c-jud" | "jus" | "c-jus" | "scjus" | "act-jud" | "ret-jud" | "mag" | "c-mag"
        | "ref" | "spec-m" | "com" | "pres" | "c-pres" | "vice-pres" | "chief" => {
            Some(LifeCategory::Political)
        }
        "prac" | "atty" | "pros" | "pub-def" | "gov-atty" | "priv" | "law-clerk" | "clerk" => {
            Some(LifeCategory::Legal)
        }
        "legis" | "leg" | "sen" | "rep" | "del" | "mayor" | "gov" | "lt-gov" | "ag" | "sec"
        | "treas" | "comp" | "comm" => Some(LifeCategory::Political),
        "prof" | "teach" | "aca" => Some(LifeCategory::Education),
        "" => None,
        _ => {
            // Unknown codes: treat attorney-ish labels via job_title later.
            None
        }
    }
}

fn position_label(pos: &Value) -> String {
    let job = pos
        .get("job_title")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    let org = pos
        .get("organization_name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    let court_name = pos
        .pointer("/court/full_name")
        .or_else(|| pos.pointer("/court/short_name"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    let ptype = pos
        .get("position_type")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();

    if !job.is_empty() && !org.is_empty() {
        return format!("{job}, {org}");
    }
    if !job.is_empty() && !court_name.is_empty() {
        return format!("{job}, {court_name}");
    }
    if !org.is_empty() {
        return org.to_string();
    }
    if !court_name.is_empty() {
        let role = match ptype {
            "jud" | "c-jud" => "Judge",
            "jus" | "c-jus" | "scjus" => "Justice",
            "prac" => "Private practice",
            "legis" | "leg" => "Legislator",
            "atty" | "pros" | "pub-def" | "gov-atty" => "Attorney",
            _ if !job.is_empty() => job,
            _ => "Position",
        };
        return format!("{role}, {court_name}");
    }
    if !job.is_empty() {
        return job.to_string();
    }
    match ptype {
        "prac" => "Private law practice".into(),
        "jud" | "c-jud" => "Judge".into(),
        "jus" | "c-jus" | "scjus" => "Justice".into(),
        "legis" | "leg" => "Legislator".into(),
        "atty" => "Attorney".into(),
        other if !other.is_empty() => format!("Position ({other})"),
        _ => "Position".into(),
    }
}

/// Positions JSON → career spans + bio facts (work/legal).
pub fn spans_and_facts_from_positions(
    positions_json: &str,
    profile_url: Option<&str>,
) -> (Vec<CareerSpan>, Vec<BioFact>, Option<i32>) {
    let Ok(root) = serde_json::from_str::<Value>(positions_json) else {
        return (Vec::new(), Vec::new(), None);
    };
    let results = root
        .get("results")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let src_url = profile_url.map(|s| s.to_string());
    let mut spans = Vec::new();
    let mut facts = Vec::new();
    let mut birth_year = None;

    for pos in results {
        // Birth from nested person if present
        if birth_year.is_none() {
            birth_year = pos
                .pointer("/person/date_dob")
                .and_then(|v| v.as_str())
                .and_then(year_from_iso);
        }

        let ptype = pos
            .get("position_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let label = position_label(&pos);
        let start = pos
            .get("date_start")
            .and_then(|v| v.as_str())
            .and_then(year_from_iso);
        let end = pos
            .get("date_termination")
            .and_then(|v| v.as_str())
            .and_then(year_from_iso)
            .or_else(|| {
                pos.get("date_retirement")
                    .and_then(|v| v.as_str())
                    .and_then(year_from_iso)
            });

        let mut cat = position_type_category(ptype);
        if cat.is_none() {
            let low = label.to_ascii_lowercase();
            if low.contains("judge")
                || low.contains("justice")
                || low.contains("magistrate")
                || low.contains("legislat")
                || low.contains("senator")
                || low.contains("representative")
            {
                cat = Some(LifeCategory::Political);
            } else if low.contains("attorney")
                || low.contains("counsel")
                || low.contains("law")
                || low.contains("practice")
                || low.contains("partner")
                || low.contains("associate")
                || low.contains("prosecutor")
                || low.contains("public defender")
            {
                cat = Some(LifeCategory::Legal);
            } else if start.is_some() || end.is_some() {
                cat = Some(LifeCategory::Work);
            }
        }
        let Some(category) = cat else {
            continue;
        };

        // Skip empty undated practice stubs with no org.
        if category == LifeCategory::Legal
            && start.is_none()
            && label.eq_ignore_ascii_case("Private law practice")
        {
            // Still record as undated fact if we want — skip pure noise.
            continue;
        }

        spans.push(CareerSpan::new(
            category,
            label.clone(),
            start,
            end,
            CL_SOURCE,
            src_url.clone(),
        ));

        let kind = match category {
            LifeCategory::Legal => "legal",
            LifeCategory::Political => "office",
            LifeCategory::Education => "education",
            _ => "work",
        };
        let mut text = label.clone();
        if let Some(s) = start {
            if let Some(e) = end {
                text = format!("{label} ({s}–{e})");
            } else {
                text = format!("{label} ({s}–present)");
            }
        }
        facts.push(BioFact::new(
            kind,
            text,
            CL_SOURCE,
            src_url.clone(),
        ));
    }

    (spans, facts, birth_year)
}

fn opinion_type_position(op_type: &str, author_id: Option<i64>, person_id: i64) -> &'static str {
    let t = op_type.to_ascii_lowercase();
    if t.contains("dissent") {
        return "Dissent";
    }
    if t.contains("concur") {
        return "Concur";
    }
    if author_id == Some(person_id) {
        return "Author";
    }
    if t.contains("order") {
        return "Order";
    }
    "Opinion"
}

/// Map CourtListener search (type=o) JSON → VoteRecord rows for Decisions tab.
pub fn opinions_from_search_json(search_json: &str, person_id: i64) -> Vec<VoteRecord> {
    let Ok(root) = serde_json::from_str::<Value>(search_json) else {
        return Vec::new();
    };
    let results = root
        .get("results")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut out = Vec::new();
    for r in results {
        if out.len() >= CL_OPINIONS_CAP {
            break;
        }
        let date = r
            .get("dateFiled")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if date.is_empty() {
            continue;
        }
        let case = r
            .get("caseName")
            .and_then(|v| v.as_str())
            .or_else(|| r.get("caseNameFull").and_then(|v| v.as_str()))
            .unwrap_or("Opinion")
            .trim();
        let cite = r
            .get("citation")
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        let docket = r
            .get("docketNumber")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();

        let mut question = case.to_string();
        if !cite.is_empty() {
            question = format!("{case}, {cite}");
        } else if !docket.is_empty() {
            question = format!("{case} ({docket})");
        }

        let abs = r
            .get("absolute_url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        let url = if abs.starts_with("http") {
            abs.to_string()
        } else if abs.starts_with('/') {
            format!("https://www.courtlistener.com{abs}")
        } else if let Some(cid) = r.get("cluster_id").and_then(|v| v.as_i64()) {
            format!("https://www.courtlistener.com/opinion/{cid}/")
        } else {
            continue;
        };

        // Role from nested opinions array
        let mut position = "Author".to_string();
        if let Some(ops) = r.get("opinions").and_then(|v| v.as_array()) {
            for op in ops {
                let aid = op.get("author_id").and_then(|v| v.as_i64());
                let joined = op
                    .get("joined_by_ids")
                    .and_then(|v| v.as_array())
                    .map(|a| a.iter().any(|x| x.as_i64() == Some(person_id)))
                    .unwrap_or(false);
                let otype = op.get("type").and_then(|v| v.as_str()).unwrap_or("");
                if aid == Some(person_id) {
                    position = opinion_type_position(otype, aid, person_id).into();
                    break;
                }
                if joined {
                    position = "Join".into();
                    break;
                }
            }
        }

        let status = r
            .get("status")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        out.push(VoteRecord {
            date,
            question,
            position,
            result: status,
            url,
        });
    }
    out
}

/// Total available count from search JSON when present.
pub fn opinions_search_total(search_json: &str) -> Option<u64> {
    let root: Value = serde_json::from_str(search_json).ok()?;
    root.get("count")
        .and_then(|v| {
            v.as_u64()
                .or_else(|| v.as_i64().map(|n| n as u64))
                .or_else(|| v.as_str()?.parse().ok())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn court_ids_fl_supreme_and_dca() {
        assert_eq!(
            courtlistener_court_ids("FL", "Justice of the Supreme Court"),
            vec!["fla".to_string()]
        );
        assert_eq!(
            courtlistener_court_ids("fl", "District Court of Appeal, 4th DCA"),
            vec!["fladistctapp".to_string()]
        );
        assert!(courtlistener_court_ids("FL", "Circuit Judge, Circuit 18").is_empty());
    }

    #[test]
    fn pick_canady_unique() {
        let json = r#"{
          "count": 1,
          "results": [{
            "id": 4029,
            "slug": "charles-t-canady",
            "name_first": "Charles",
            "name_middle": "T.",
            "name_last": "Canady",
            "name_suffix": "",
            "date_dob": "1954-06-22",
            "positions": []
          }]
        }"#;
        let m = pick_courtlistener_person(json, "Charles T. Canady", &["fla".into()]).unwrap();
        assert_eq!(m.person_id, 4029);
        assert!(m.profile_url.contains("4029"));
        assert_eq!(m.birth_year, Some(1954));
    }

    #[test]
    fn pick_ambiguous_skips() {
        let json = r#"{
          "results": [
            {"id": 1, "name_first": "John", "name_last": "Smith", "slug": "a", "positions": []},
            {"id": 2, "name_first": "John", "name_last": "Smith", "slug": "b", "positions": []}
          ]
        }"#;
        assert!(pick_courtlistener_person(json, "John Smith", &[]).is_none());
    }

    #[test]
    fn positions_to_spans() {
        let json = r#"{
          "results": [
            {
              "position_type": "jud",
              "job_title": "",
              "organization_name": null,
              "date_start": "2008-08-28",
              "date_termination": null,
              "court": {"id": "fla", "full_name": "Supreme Court of Florida", "short_name": "Supreme Court of Florida"}
            },
            {
              "position_type": "prac",
              "job_title": "Partner",
              "organization_name": "Smith & Jones LLP",
              "date_start": "1990-01-01",
              "date_termination": "2001-01-01",
              "court": null
            },
            {
              "position_type": "legis",
              "job_title": "",
              "organization_name": null,
              "date_start": "2001-01-01",
              "date_termination": "2001-01-01",
              "court": null
            }
          ]
        }"#;
        let (spans, facts, _) =
            spans_and_facts_from_positions(json, Some("https://www.courtlistener.com/person/1/"));
        assert!(spans.iter().any(|s| s.category == "political"));
        assert!(spans.iter().any(|s| s.category == "legal"
            && s.label.to_ascii_lowercase().contains("smith")));
        assert!(facts.iter().any(|f| f.kind == "legal"));
    }

    #[test]
    fn opinions_map_author() {
        let json = r#"{
          "count": 2,
          "results": [
            {
              "caseName": "Brown v. NAGELHOUT",
              "citation": ["84 So. 3d 304"],
              "dateFiled": "2012-03-15",
              "absolute_url": "/opinion/2550869/brown-v-nagelhout/",
              "cluster_id": 2550869,
              "status": "Published",
              "docketNumber": "SC10-868",
              "opinions": [{"author_id": 4029, "type": "combined-opinion", "joined_by_ids": []}]
            },
            {
              "caseName": "Other v. Case",
              "citation": [],
              "dateFiled": "2011-01-01",
              "absolute_url": "/opinion/1/other/",
              "status": "Published",
              "opinions": [{"author_id": 99, "type": "dissent", "joined_by_ids": [4029]}]
            }
          ]
        }"#;
        let rows = opinions_from_search_json(json, 4029);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].position, "Author");
        assert!(rows[0].question.contains("Brown"));
        assert!(rows[0].url.contains("courtlistener.com"));
        assert_eq!(rows[1].position, "Join");
        assert_eq!(opinions_search_total(json), Some(2));
    }

    #[test]
    fn people_search_url_encodes() {
        let u = courtlistener_people_search_url("Charles Canady").unwrap();
        assert!(u.contains("name_last=Canady"));
        assert!(u.contains("name_first=Charles"));
    }
}
