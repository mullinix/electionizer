//! Track L — voter scrutiny: money signals, endorsements, news, stated claims,
//! keyword contrasts, FL Bar / Ethics / JQC, and ballot-measure Support/Oppose.
//! Pure parse/map. JS owns I/O. Signals, not verdicts.

use crate::bio::Endorsement;
use serde::{Deserialize, Serialize};

pub const BREVARD_VF_ELECTION_PRIMARY_2026: &str = "104";
pub const GDELT_DOC_API: &str = "https://api.gdeltproject.org/api/v2/doc/doc";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MoneySignal {
    pub id: String,
    pub label: String,
    pub value_display: String,
    pub note: String,
    pub trust: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MoneySignals {
    pub signals: Vec<MoneySignal>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub empty_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NewsHit {
    pub title: String,
    pub outlet: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    pub url: String,
    pub trust: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SampleBallotRef {
    pub county: String,
    pub precinct: String,
    pub party: String,
    pub election_id: String,
    pub url: String,
    #[serde(default)]
    pub contests: Vec<String>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NamedPortal {
    pub label: String,
    pub url: String,
    pub note: String,
}

#[derive(Debug, Deserialize)]
struct SizeIn {
    #[serde(default)]
    label: String,
    #[serde(default)]
    total_display: String,
}

#[derive(Debug, Deserialize)]
struct ContribIn {
    #[serde(default)]
    name: String,
    #[serde(default)]
    amount_display: String,
    #[serde(default)]
    location: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OutsideIn {
    #[serde(default)]
    committee: String,
    #[serde(default)]
    amount_display: String,
    #[serde(default)]
    support_oppose: String,
}

/// Normalize FL closed-primary party for VoterFocus sample ballots.
pub fn vf_sample_party(party: &str) -> Option<&'static str> {
    let u = party.trim().to_ascii_uppercase();
    if u.is_empty() {
        return None;
    }
    if u.starts_with("REP") || u == "R" || u == "GOP" {
        return Some("Rep");
    }
    if u.starts_with("DEM") || u == "D" {
        return Some("Dem");
    }
    if u.starts_with("NON") || u == "NPA" || u == "NP" || u.contains("NO PARTY") {
        return Some("Non");
    }
    None
}

/// VoterFocus precinct token: `104` → `104.1`.
pub fn vf_precinct_token(precinct: &str) -> Option<String> {
    let t = precinct.trim();
    if t.is_empty() {
        return None;
    }
    let digits: String = t.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() || digits.len() > 5 {
        return None;
    }
    if t.contains('.') {
        Some(t.to_string())
    } else {
        Some(format!("{digits}.1"))
    }
}

pub fn brevard_sample_ballot_url(precinct: &str, party: &str, election_id: &str) -> Option<String> {
    let prec = vf_precinct_token(precinct)?;
    let pty = vf_sample_party(party)?;
    let elec = election_id.trim();
    let elec = if elec.is_empty() {
        BREVARD_VF_ELECTION_PRIMARY_2026
    } else {
        elec
    };
    Some(format!(
        "https://www.voterfocus.com/SampleBallots/WHSampleBallot.php?county=brevard&precinct={prec}&party={pty}&election={elec}"
    ))
}

pub fn sample_ballot_ref(
    precinct: &str,
    party: &str,
    election_id: &str,
) -> Option<SampleBallotRef> {
    let url = brevard_sample_ballot_url(precinct, party, election_id)?;
    let pty = vf_sample_party(party)?.to_string();
    let prec = precinct.trim().to_string();
    Some(SampleBallotRef {
        county: "Brevard".into(),
        precinct: prec,
        party: pty,
        election_id: if election_id.trim().is_empty() {
            BREVARD_VF_ELECTION_PRIMARY_2026.into()
        } else {
            election_id.trim().into()
        },
        url,
        contests: Vec::new(),
        note: "Official VoteBrevard / VoterFocus sample ballot (PDF). ZIP centroid is not precinct-accurate."
            .into(),
    })
}

pub fn scrutiny_portals(name: &str, is_judge: bool) -> Vec<NamedPortal> {
    let q = urlencoding_minimal(name.trim());
    let mut out = vec![
        NamedPortal {
            label: "VoteBrevard 2026 candidates".into(),
            url: "https://www.votebrevard.gov/Candidate-Information/2026-Candidates".into(),
            note: "Official Brevard candidate listing".into(),
        },
        NamedPortal {
            label: "VoteBrevard sample ballots".into(),
            url: "https://www.votebrevard.gov/Election-Information/2026-Primary-Sample-Ballots"
                .into(),
            note: "Official precinct + party sample ballots".into(),
        },
        NamedPortal {
            label: "Florida Commission on Ethics filings".into(),
            url: "https://disclosure.floridaethics.gov/PublicSearch/Filings".into(),
            note: "Form 1 / Form 6 public search".into(),
        },
        NamedPortal {
            label: "Florida Bar lawyer search".into(),
            url: fl_bar_search_url(name)
                .unwrap_or_else(|| "https://www.floridabar.org/directories/find-mbr/".into()),
            note: "License + public standing".into(),
        },
        NamedPortal {
            label: "OpenFEC candidate search".into(),
            url: if q.is_empty() {
                "https://www.fec.gov/data/candidates/".into()
            } else {
                format!("https://www.fec.gov/data/candidates/?q={q}")
            },
            note: "Federal filings and enforcement".into(),
        },
    ];
    if is_judge {
        out.push(NamedPortal {
            label: "Florida JQC".into(),
            url: "https://www.floridajqc.com/".into(),
            note: "Judicial Qualifications Commission".into(),
        });
        out.push(NamedPortal {
            label: "CourtListener search".into(),
            url: if q.is_empty() {
                "https://www.courtlistener.com/".into()
            } else {
                format!("https://www.courtlistener.com/?type=p&q={q}")
            },
            note: "Opinions and judge directory".into(),
        });
    }
    out.push(NamedPortal {
        label: "Google News search".into(),
        url: if q.is_empty() {
            "https://news.google.com/".into()
        } else {
            format!("https://news.google.com/search?q=%22{q}%22%20Brevard%20OR%20Florida")
        },
        note: "Headlines — allegations, not findings".into(),
    });
    out
}

fn urlencoding_minimal(s: &str) -> String {
    let mut out = String::new();
    for b in s.as_bytes() {
        match *b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*b as char);
            }
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

pub fn parse_usd_display(s: &str) -> Option<f64> {
    let t = s.trim();
    if t.is_empty() || t == "—" || t == "-" {
        return None;
    }
    let mut buf = String::new();
    let mut neg = false;
    for c in t.chars() {
        if c == '(' {
            neg = true;
        } else if c.is_ascii_digit() || c == '.' {
            buf.push(c);
        }
    }
    if buf.is_empty() {
        return None;
    }
    let n: f64 = buf.parse().ok()?;
    Some(if neg || t.contains('-') { -n } else { n })
}

fn signal(id: &str, label: &str, value: String, note: &str) -> MoneySignal {
    MoneySignal {
        id: id.into(),
        label: label.into(),
        value_display: value,
        note: note.into(),
        trust: "inference".into(),
    }
}

fn pct(part: f64, whole: f64) -> Option<String> {
    if whole <= 0.0 || part < 0.0 {
        return None;
    }
    let p = 100.0 * part / whole;
    if !p.is_finite() {
        return None;
    }
    Some(format!("{p:.0}%"))
}

/// Compute money-signal cards from already-fetched finance JSON blobs.
pub fn money_signals_from_json(
    size_json: &str,
    individuals_json: &str,
    committees_json: &str,
    outside_json: &str,
    receipts_display: &str,
    pac_display: &str,
    individual_display: &str,
    candidate_name: &str,
    home_state: &str,
    home_county: &str,
) -> MoneySignals {
    let sizes: Vec<SizeIn> = serde_json::from_str(size_json).unwrap_or_default();
    let indiv: Vec<ContribIn> = serde_json::from_str(individuals_json).unwrap_or_default();
    let cmtes: Vec<ContribIn> = serde_json::from_str(committees_json).unwrap_or_default();
    let outside: Vec<OutsideIn> = serde_json::from_str(outside_json).unwrap_or_default();

    let mut signals = Vec::new();
    let receipts = parse_usd_display(receipts_display).unwrap_or(0.0);

    if !sizes.is_empty() {
        let mut large = 0.0;
        let mut total = 0.0;
        for s in &sizes {
            let amt = parse_usd_display(&s.total_display).unwrap_or(0.0);
            total += amt;
            let lab = s.label.to_ascii_lowercase();
            if lab.contains("1,000") || lab.contains("2,000") || lab.contains("$1000") {
                large += amt;
            }
        }
        if let Some(p) = pct(large, total) {
            signals.push(signal(
                "large_donor_share",
                "Share of itemized $ from $1k+ gifts",
                p,
                "FEC size buckets — not a completeness score.",
            ));
        }
    }

    let pac = parse_usd_display(pac_display);
    let indiv_tot = parse_usd_display(individual_display);
    if let (Some(p), Some(r)) = (pac, if receipts > 0.0 { Some(receipts) } else { None }) {
        if let Some(v) = pct(p, r) {
            signals.push(signal(
                "pac_share",
                "PAC / committee share of receipts",
                v,
                "From FEC cycle totals when reported. Not “bought.”",
            ));
        }
    } else if let (Some(i), Some(r)) = (
        indiv_tot,
        if receipts > 0.0 { Some(receipts) } else { None },
    ) {
        if r > i {
            if let Some(v) = pct(r - i, r) {
                signals.push(signal(
                    "non_individual_share",
                    "Non-individual share of receipts",
                    v,
                    "Receipts minus itemized+unitemized individuals, when both exist.",
                ));
            }
        }
    }

    let mut named: Vec<(String, f64, Option<String>)> = Vec::new();
    for r in indiv.iter().chain(cmtes.iter()) {
        if let Some(a) = parse_usd_display(&r.amount_display) {
            if a > 0.0 && !r.name.trim().is_empty() {
                named.push((r.name.clone(), a, r.location.clone()));
            }
        }
    }
    named.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let named_sum: f64 = named.iter().map(|x| x.1).sum();
    if named_sum > 0.0 && named.len() >= 2 {
        let top_n = named.iter().take(5).map(|x| x.1).sum::<f64>();
        if let Some(p) = pct(top_n, named_sum) {
            signals.push(signal(
                "top5_concentration",
                "Top 5 sources of listed itemized $",
                p,
                "Among loaded contributor rows only — not the full universe of donors.",
            ));
        }
    }

    let st = home_state.trim().to_ascii_uppercase();
    let county = home_county.trim().to_ascii_lowercase();
    if !st.is_empty() && named_sum > 0.0 {
        let mut in_st = 0.0;
        let mut known = 0.0;
        let mut in_county = 0.0;
        let mut county_known = 0.0;
        for (_, amt, loc) in &named {
            let loc_u = loc.as_deref().unwrap_or("").to_ascii_uppercase();
            if loc_u.is_empty() {
                continue;
            }
            known += *amt;
            let in_florida = loc_u.contains(" FL")
                || loc_u.ends_with("FL")
                || loc_u.contains("FLORIDA")
                || loc_u.contains(&format!(", {st}"))
                || loc_u.ends_with(&format!(" {st}"));
            if st == "FL" {
                if in_florida {
                    in_st += *amt;
                }
            } else if loc_u.contains(&st) {
                in_st += *amt;
            }
            if !county.is_empty() {
                let loc_l = loc.as_deref().unwrap_or("").to_ascii_lowercase();
                county_known += *amt;
                if loc_l.contains(&county) {
                    in_county += *amt;
                }
            }
        }
        if known > 0.0 {
            if let Some(p) = pct(known - in_st, known) {
                signals.push(signal(
                    "out_of_state_share",
                    "Out-of-state share of located itemized $",
                    p,
                    "Uses donor city/state on loaded rows. Unknown locations omitted.",
                ));
            }
        }
        if county_known > 0.0 {
            if let Some(p) = pct(county_known - in_county, county_known) {
                signals.push(signal(
                    "out_of_county_share",
                    "Outside-county share of located itemized $",
                    p,
                    "Soft string match on county name in donor location.",
                ));
            }
        }
    }

    let last = candidate_last_token(candidate_name);
    if !last.is_empty() && named_sum > 0.0 {
        let self_amt: f64 = named
            .iter()
            .filter(|(n, _, _)| {
                n.to_ascii_uppercase()
                    .split(|c: char| !c.is_ascii_alphanumeric())
                    .any(|t| t == last)
            })
            .map(|x| x.1)
            .sum();
        if self_amt > 0.0 {
            if let Some(p) = pct(self_amt, named_sum) {
                signals.push(signal(
                    "name_match_share",
                    "Itemized $ from donors sharing the candidate last name",
                    p,
                    "Soft name match — family or namesake, not proven self-funding.",
                ));
            }
        }
    }

    let mut ie_sup = 0.0;
    let mut ie_opp = 0.0;
    for o in &outside {
        let amt = parse_usd_display(&o.amount_display).unwrap_or(0.0);
        let stc = o.support_oppose.to_ascii_lowercase();
        if stc.contains("support") {
            ie_sup += amt;
        } else if stc.contains("oppose") {
            ie_opp += amt;
        }
        let _ = &o.committee;
    }
    if ie_sup + ie_opp > 0.0 {
        signals.push(signal(
            "outside_spend",
            "Independent expenditures (support / oppose)",
            format!(
                "{} / {}",
                format_usd_short(ie_sup),
                format_usd_short(ie_opp)
            ),
            "FEC Schedule E. Super PAC / IE attention — not candidate-controlled cash.",
        ));
    }

    if signals.is_empty() {
        MoneySignals {
            signals,
            empty_note: Some("Not enough itemized finance loaded to compute money signals.".into()),
        }
    } else {
        MoneySignals {
            signals,
            empty_note: None,
        }
    }
}

fn candidate_last_token(name: &str) -> String {
    name.split_whitespace()
        .rev()
        .find(|t| t.chars().any(|c| c.is_ascii_alphabetic()) && t.len() > 2)
        .map(|t| {
            t.chars()
                .filter(|c| c.is_ascii_alphanumeric())
                .collect::<String>()
                .to_ascii_uppercase()
        })
        .unwrap_or_default()
}

fn format_usd_short(n: f64) -> String {
    if n.abs() >= 1_000_000.0 {
        format!("${:.1}M", n / 1_000_000.0)
    } else if n.abs() >= 1_000.0 {
        format!("${:.0}k", n / 1_000.0)
    } else {
        format!("${n:.0}")
    }
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
    decode_entities(&out)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn decode_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&nbsp;", " ")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

fn looks_like_endorser(text: &str) -> bool {
    let t = text.trim();
    if t.len() < 3 || t.len() > 180 {
        return false;
    }
    let low = t.to_ascii_lowercase();
    if low.starts_with("see also")
        || low.starts_with("retrieved")
        || low.starts_with("external link")
        || low.starts_with("chip in")
        || low.starts_with("paid by")
        || low.starts_with("terms")
        || low.starts_with("privacy")
        || low.starts_with("skip to")
        || low.contains("click here")
        || low.contains("edit source")
        || low.contains("additional endorsement")
        || low.contains("received the following endorsement")
        || low.contains("to send us additional")
    {
        return false;
    }
    matches!(
        low.as_str(),
        "about"
            | "news"
            | "issues"
            | "volunteer"
            | "contact"
            | "donate"
            | "shop"
            | "media"
            | "home"
            | "meet byron"
    )
    .then_some(false)
    .unwrap_or(true)
}

fn heading_is_endorsement(text: &str) -> Option<&'static str> {
    let low = text.to_ascii_lowercase();
    if heading_is_outgoing_endorsements(text) {
        return None;
    }
    if low.contains("oppos") && (low.contains("endors") || low.contains("against")) {
        return Some("oppose");
    }
    if low.contains("endors") {
        return Some("support");
    }
    None
}

fn heading_is_outgoing_endorsements(text: &str) -> bool {
    let low = text.to_ascii_lowercase();
    low.contains("notable endorsement")
        || low.contains("endorsements made")
        || low.contains("candidates endorsed")
        || low.contains("endorsements by this")
}

fn clean_endorser_name(raw: &str) -> String {
    raw.split(" — ")
        .next()
        .unwrap_or(raw)
        .split(" – ")
        .next()
        .unwrap_or(raw)
        .trim()
        .trim_end_matches(['-', '–', '—'])
        .trim()
        .to_string()
}

fn endorsement_kind(name: &str) -> &'static str {
    let low = name.to_ascii_lowercase();
    if [
        "association",
        "assoc.",
        "club for",
        " pac",
        "union",
        "chamber",
        "coalition",
        "committee",
        "afl-cio",
        "federation",
        "industries",
        "voting group",
    ]
    .iter()
    .any(|h| low.contains(h))
    {
        return "org";
    }
    if [
        "president ",
        "senator ",
        "sen. ",
        "rep. ",
        "representative ",
        "speaker ",
        "governor ",
        "sheriff ",
        "mayor ",
        "majority ",
        "u.s. sen",
        "u.s.rep",
    ]
    .iter()
    .any(|h| low.contains(h))
    {
        return "person";
    }
    let caps = name
        .split_whitespace()
        .filter(|w| w.chars().next().is_some_and(|c| c.is_uppercase()))
        .count();
    if caps >= 2 {
        "person"
    } else {
        "org"
    }
}

fn campaign_name_is_section(text: &str) -> bool {
    let low = text.to_ascii_lowercase();
    if heading_is_endorsement(text).is_some() {
        return true;
    }
    low.starts_with("paid by")
        || low.ends_with(" figures")
        || low.starts_with("endorsements by")
        || matches!(
            low.as_str(),
            "national figures"
                | "associations"
                | "members of congress"
                | "coalitions"
                | "elected officials"
                | "organizations"
                | "individuals"
                | "community leaders"
                | "faith leaders"
        )
}

/// Ballotpedia (and similar wiki) endorsement lists under Endorsements headings.
pub fn endorsements_from_ballotpedia_html(html: &str, page_url: &str) -> Vec<Endorsement> {
    if html.len() < 80 {
        return Vec::new();
    }
    let url = if page_url.trim().is_empty() {
        None
    } else {
        Some(page_url.trim().to_string())
    };
    let mut out = Vec::new();
    let heading_re = regex::Regex::new(r#"(?is)<h[1-6][^>]*>\s*(?:<[^>]+>\s*)*([^<]{3,80})"#);
    let Ok(heading_re) = heading_re else {
        return out;
    };
    let li_re = regex::Regex::new(r"(?is)<li\b[^>]*>([\s\S]{3,2500}?)</li>");
    let Ok(li_re) = li_re else {
        return out;
    };

    let lowers = html.to_ascii_lowercase();
    if !lowers.contains("endors") {
        return out;
    }

    let headings: Vec<(usize, String)> = heading_re
        .captures_iter(html)
        .filter_map(|c| {
            let start = c.get(0)?.start();
            let text = strip_tags(c.get(1)?.as_str());
            if text.is_empty() {
                return None;
            }
            Some((start, text))
        })
        .collect();

    for (i, (start, title)) in headings.iter().enumerate() {
        let Some(stance) = heading_is_endorsement(title) else {
            continue;
        };
        let end = headings.get(i + 1).map(|(s, _)| *s).unwrap_or(html.len());
        let slice = &html[*start..end.min(html.len())];
        for cap in li_re.captures_iter(slice) {
            let org = clean_endorser_name(&strip_tags(cap.get(1).map(|m| m.as_str()).unwrap_or("")));
            if !looks_like_endorser(&org) {
                continue;
            }
            push_endorsement(
                &mut out,
                &org,
                stance,
                "Ballotpedia",
                url.clone(),
                endorsement_kind(&org),
                "reference",
            );
        }
    }

    // Infobox / widget "Endorsements" key.
    if let Ok(re) = regex::Regex::new(
        r#"(?is)<div\s+class="widget-key"[^>]*>\s*([^<]*[Ee]ndors[^<]*)\s*</div>\s*<div\s+class="widget-value"[^>]*>([\s\S]{0,2000}?)</div>"#,
    ) {
        for cap in re.captures_iter(html) {
            let val = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            for piece in val.split(|c| c == '<' || c == ',' || c == ';' || c == '\n') {
                let org = clean_endorser_name(&strip_tags(piece));
                if looks_like_endorser(&org) {
                    push_endorsement(
                        &mut out,
                        &org,
                        "support",
                        "Ballotpedia",
                        url.clone(),
                        endorsement_kind(&org),
                        "reference",
                    );
                }
            }
        }
    }
    out.truncate(120);
    out
}

pub fn campaign_endorsement_urls(site_url: &str) -> Vec<String> {
    let base = normalize_site(site_url);
    if base.len() < 12 {
        return Vec::new();
    }
    let mut out = vec![
        format!("{base}/endorsements"),
        format!("{base}/endorsements/"),
        format!("{base}/endorsement"),
        format!("{base}/endorsed-by"),
        format!("{base}/supporters"),
        format!("{base}/coalition"),
    ];
    let mut seen = std::collections::HashSet::new();
    out.retain(|u| seen.insert(u.clone()));
    out
}

fn normalize_site(site_url: &str) -> String {
    let mut t = site_url.trim().to_string();
    if t.starts_with("http://") {
        t = format!("https://{}", t.trim_start_matches("http://"));
    }
    if !t.starts_with("https://") {
        t = format!("https://{t}");
    }
    t.trim_end_matches('/').to_string()
}

/// Campaign-site endorsement lists (self-reported).
pub fn endorsements_from_campaign_html(html: &str, page_url: &str) -> Vec<Endorsement> {
    if html.len() < 80 {
        return Vec::new();
    }
    let url = if page_url.trim().is_empty() {
        None
    } else {
        Some(page_url.trim().to_string())
    };
    let mut out = Vec::new();
    let low = html.to_ascii_lowercase();
    if !(low.contains("endors") || low.contains("supporter")) {
        return out;
    }
    for pat in [
        r"(?is)<h2[^>]*>([\s\S]{2,200}?)</h2>",
        r"(?is)<h3[^>]*>([\s\S]{2,200}?)</h3>",
    ] {
        let Ok(h_re) = regex::Regex::new(pat) else {
            continue;
        };
        for cap in h_re.captures_iter(html) {
            let name = clean_endorser_name(&strip_tags(cap.get(1).map(|m| m.as_str()).unwrap_or("")));
            if campaign_name_is_section(&name) || !looks_like_endorser(&name) {
                continue;
            }
            push_endorsement(
                &mut out,
                &name,
                "support",
                "Campaign website",
                url.clone(),
                endorsement_kind(&name),
                "campaign",
            );
        }
    }
    if let Ok(li_re) = regex::Regex::new(r"(?is)<li\b[^>]*>([\s\S]{3,800}?)</li>") {
        for cap in li_re.captures_iter(html) {
            let org = clean_endorser_name(&strip_tags(cap.get(1).map(|m| m.as_str()).unwrap_or("")));
            if looks_like_endorser(&org) {
                push_endorsement(
                    &mut out,
                    &org,
                    "support",
                    "Campaign website",
                    url.clone(),
                    endorsement_kind(&org),
                    "campaign",
                );
            }
        }
    }
    out.truncate(120);
    out
}

fn push_endorsement(
    out: &mut Vec<Endorsement>,
    org: &str,
    stance: &str,
    source: &str,
    source_url: Option<String>,
    kind: &str,
    trust: &str,
) {
    let org = org.trim();
    if org.is_empty() {
        return;
    }
    if out
        .iter()
        .any(|e| e.org.eq_ignore_ascii_case(org) && e.stance.eq_ignore_ascii_case(stance))
    {
        return;
    }
    out.push(Endorsement {
        org: org.into(),
        stance: stance.into(),
        source: source.into(),
        source_url,
        kind: Some(kind.into()),
        trust: Some(trust.into()),
        date: None,
    });
}

/// GDELT DOC artlist JSON → news hits. Person last name must appear in title or domain.
pub fn news_hits_from_gdelt_json(json: &str, person_name: &str) -> Vec<NewsHit> {
    let v: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let arts = v
        .get("articles")
        .and_then(|a| a.as_array())
        .cloned()
        .unwrap_or_default();
    let last = candidate_last_token(person_name);
    let mut out = Vec::new();
    for a in arts {
        let title = a
            .get("title")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let url = a
            .get("url")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if title.is_empty() || url.is_empty() {
            continue;
        }
        let domain = a
            .get("domain")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        let blob = format!("{title} {domain} {url}").to_ascii_uppercase();
        if !last.is_empty() && !blob.contains(&last) {
            continue;
        }
        let date = a
            .get("seendate")
            .and_then(|x| x.as_str())
            .map(|s| gdelt_date(s));
        out.push(NewsHit {
            title,
            outlet: if domain.is_empty() {
                host_of(&url)
            } else {
                domain
            },
            date,
            url,
            trust: "news".into(),
        });
        if out.len() >= 25 {
            break;
        }
    }
    out
}

fn gdelt_date(s: &str) -> String {
    let t = s.trim();
    if t.len() >= 8 && t.bytes().take(8).all(|b| b.is_ascii_digit()) {
        format!("{}-{}-{}", &t[0..4], &t[4..6], &t[6..8])
    } else {
        t.into()
    }
}

fn host_of(url: &str) -> String {
    url.split("://")
        .nth(1)
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or("")
        .trim_start_matches("www.")
        .to_string()
}

/// Google News RSS items.
pub fn news_hits_from_google_rss(xml: &str, person_name: &str) -> Vec<NewsHit> {
    let last = candidate_last_token(person_name);
    let item_re = regex::Regex::new(r"(?is)<item>([\s\S]*?)</item>");
    let Ok(item_re) = item_re else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for cap in item_re.captures_iter(xml) {
        let block = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let title = xml_tag(block, "title");
        let link = xml_tag(block, "link");
        if title.is_empty() || link.is_empty() {
            continue;
        }
        let blob = format!("{title} {link}").to_ascii_uppercase();
        if !last.is_empty() && !blob.contains(&last) {
            continue;
        }
        let source = xml_tag(block, "source");
        let pub_date = xml_tag(block, "pubDate");
        out.push(NewsHit {
            title,
            outlet: if source.is_empty() {
                host_of(&link)
            } else {
                source
            },
            date: if pub_date.is_empty() {
                None
            } else {
                Some(pub_date)
            },
            url: link,
            trust: "news".into(),
        });
        if out.len() >= 25 {
            break;
        }
    }
    out
}

fn xml_tag(block: &str, tag: &str) -> String {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let Some(i) = block.find(&open) else {
        return String::new();
    };
    let rest = &block[i + open.len()..];
    let Some(gt) = rest.find('>') else {
        return String::new();
    };
    let inner = &rest[gt + 1..];
    let Some(end) = inner.find(&close) else {
        return String::new();
    };
    strip_tags(&inner[..end].replace("<![CDATA[", "").replace("]]>", ""))
}

/// GDELT artlist URL. `locale_hint` e.g. `Brevard` or `Florida`.
pub fn gdelt_artlist_url(person_name: &str, locale_hint: &str) -> Option<String> {
    let name = person_name.trim();
    if name.len() < 5 {
        return None;
    }
    let last = candidate_last_token(name);
    if last.len() < 3 {
        return None;
    }
    let mut q = format!("\"{name}\"");
    let loc = locale_hint.trim();
    if !loc.is_empty() {
        q.push_str(&format!(" {loc}"));
    }
    Some(format!(
        "{GDELT_DOC_API}?query={}&mode=artlist&maxrecords=25&timespan=3m&format=json&sort=HybridRel",
        urlencoding_minimal(&q)
    ))
}

pub fn google_news_rss_url(person_name: &str, locale_hint: &str) -> Option<String> {
    let name = person_name.trim();
    if name.len() < 5 {
        return None;
    }
    let loc = locale_hint.trim();
    let q = if loc.is_empty() {
        format!("\"{name}\"")
    } else {
        format!("\"{name}\" {loc}")
    };
    Some(format!(
        "https://news.google.com/rss/search?q={}&hl=en-US&gl=US&ceid=US:en",
        urlencoding_minimal(&q)
    ))
}

pub fn merge_news_hits(base: &[NewsHit], extra: &[NewsHit]) -> Vec<NewsHit> {
    let mut out = base.to_vec();
    for h in extra {
        if out
            .iter()
            .any(|x| x.url == h.url || (x.title == h.title && x.outlet == h.outlet))
        {
            continue;
        }
        out.push(h.clone());
        if out.len() >= 40 {
            break;
        }
    }
    out
}

/// Stated issue / platform line from campaign or Ballotpedia. Cite-or-omit; never paraphrase.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PublicClaim {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    pub trust: String,
    /// `position` | `promise` | `record_claim`
    pub kind: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
}

/// One related vote/opinion row (keyword overlap only — not a verdict).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContrastMatch {
    pub date: String,
    pub question: String,
    pub position: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    pub url: String,
    pub overlap: Vec<String>,
}

/// Claim plus any keyword-paired roll-calls / opinions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContrastCard {
    pub claim_text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    pub claim_source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claim_url: Option<String>,
    pub claim_trust: String,
    pub claim_kind: String,
    pub matches: Vec<ContrastMatch>,
    pub note: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub llm_note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub llm_trust: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub llm_model: Option<String>,
}

const CLAIM_SECTION_NEEDLES: &[&str] = &[
    "campaign theme",
    "political position",
    "key issue",
    "on the issues",
    "where i stand",
    "where we stand",
    "my position",
    "our position",
    "platform",
    "priorities",
    "agenda",
];

const CLAIM_SECTION_EXACT: &[&str] = &["issues", "issue", "positions"];

const CLAIM_STOP_HEADINGS: &[&str] = &[
    "see also",
    "references",
    "external link",
    "notes",
    "footnote",
    "endorsement",
    "committee assignment",
    "contact",
    "donate",
    "navigation",
    "privacy",
    "subscribe",
];

/// (needle in text, canonical keyword stored on the claim)
const CLAIM_LEXICON: &[(&str, &str)] = &[
    ("abortion", "abortion"),
    ("reproductive", "abortion"),
    ("planned parenthood", "abortion"),
    ("pro-life", "abortion"),
    ("pro-choice", "abortion"),
    ("tax", "taxes"),
    ("irs", "taxes"),
    ("immigration", "immigration"),
    ("border", "immigration"),
    ("migrant", "immigration"),
    ("crime", "crime"),
    ("police", "crime"),
    ("public safety", "crime"),
    ("criminal", "crime"),
    ("education", "education"),
    ("school", "education"),
    ("voucher", "education"),
    ("teacher", "education"),
    ("gun", "guns"),
    ("firearm", "guns"),
    ("second amendment", "guns"),
    ("climate", "climate"),
    ("environment", "climate"),
    ("renewable", "climate"),
    ("health care", "healthcare"),
    ("healthcare", "healthcare"),
    ("medicaid", "healthcare"),
    ("medicare", "healthcare"),
    ("insurance", "healthcare"),
    ("housing", "housing"),
    ("homeless", "housing"),
    ("veteran", "veterans"),
    ("israel", "foreign-policy"),
    ("ukraine", "foreign-policy"),
    ("china", "foreign-policy"),
    ("spending", "budget"),
    ("budget", "budget"),
    ("deficit", "budget"),
    ("election integrity", "elections"),
    ("voting", "elections"),
    ("ballot", "elections"),
    ("amendment", "constitution"),
    ("constitution", "constitution"),
    ("parental right", "education"),
    ("first amendment", "constitution"),
    ("energy", "energy"),
    ("oil", "energy"),
    ("gas", "energy"),
    ("social security", "social-security"),
    ("socialism", "economy"),
    ("economy", "economy"),
    ("job", "economy"),
    ("wage", "economy"),
    ("union", "labor"),
    ("labor", "labor"),
    ("lgbt", "civil-rights"),
    ("transgender", "civil-rights"),
    ("civil right", "civil-rights"),
    ("death penalty", "crime"),
    ("death-penalty", "crime"),
];

fn heading_is_claim_section(text: &str) -> bool {
    let low = text.to_ascii_lowercase();
    let t = low.trim();
    if CLAIM_SECTION_EXACT.iter().any(|x| t == *x) {
        return true;
    }
    CLAIM_SECTION_NEEDLES.iter().any(|n| t.contains(n))
}

fn heading_is_stop(text: &str) -> bool {
    let low = text.to_ascii_lowercase();
    CLAIM_STOP_HEADINGS.iter().any(|n| low.contains(n))
}

fn heading_is_year_only(text: &str) -> bool {
    let digits: String = text.chars().filter(|c| c.is_ascii_digit()).collect();
    let rest: String = text.chars().filter(|c| c.is_ascii_alphabetic()).collect();
    digits.len() == 4 && rest.is_empty()
}

fn looks_like_claim_prose(text: &str) -> bool {
    let t = text.trim();
    if t.len() < 40 || t.len() > 900 {
        return false;
    }
    let low = t.to_ascii_lowercase();
    if low.contains("click here")
        || low.contains("edit source")
        || low.contains("cookie")
        || low.starts_with("home ")
        || low.contains("subscribe")
        || low.contains("sign up")
        || low.contains("donate now")
    {
        return false;
    }
    let words = t.split_whitespace().count();
    words >= 8
}

fn claim_kind_of(text: &str) -> &'static str {
    let low = text.to_ascii_lowercase();
    if low.contains("i will")
        || low.contains("i'll ")
        || low.contains("we will")
        || low.contains("we'll ")
        || low.contains("i promise")
    {
        return "promise";
    }
    if low.contains("voted")
        || low.contains("sponsored")
        || low.contains("co-sponsored")
        || low.contains("authored")
    {
        return "record_claim";
    }
    "position"
}

fn lexicon_keywords(topic: Option<&str>, text: &str) -> Vec<String> {
    let blob = format!("{} {text}", topic.unwrap_or("")).to_ascii_lowercase();
    let mut out = Vec::new();
    for (needle, canon) in CLAIM_LEXICON {
        if blob.contains(needle) && !out.iter().any(|x: &String| x == *canon) {
            out.push((*canon).to_string());
        }
    }
    if let Some(t) = topic {
        let token = t
            .trim()
            .to_ascii_lowercase()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect::<String>();
        let token = token.trim_matches('-').to_string();
        if token.len() >= 4 && !out.iter().any(|x| x == &token) {
            out.push(token);
        }
    }
    out
}

fn collapse_ws(s: &str) -> String {
    let mut out = String::new();
    let mut prev_space = true;
    for c in s.chars() {
        if c.is_whitespace() {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(c);
            prev_space = false;
        }
    }
    out.trim().to_string()
}

fn truncate_claim(text: &str) -> String {
    let t = collapse_ws(text);
    if t.len() <= 420 {
        return t;
    }
    let cut = t
        .char_indices()
        .take_while(|(i, _)| *i < 400)
        .map(|(i, c)| (i, c))
        .collect::<Vec<_>>();
    if let Some((i, _)) = cut
        .iter()
        .rev()
        .find(|(_, c)| *c == '.' || *c == ';' || *c == '!')
    {
        return t[..=*i].to_string();
    }
    let last = cut.last().map(|(i, _)| *i).unwrap_or(400);
    format!("{}…", &t[..last])
}

fn first_prose_in(html: &str) -> Option<String> {
    for tag in ["blockquote", "p"] {
        let pat = format!(r"(?is)<{tag}\b[^>]*>([\s\S]{{20,1200}}?)</{tag}>");
        let Ok(re) = regex::Regex::new(&pat) else {
            continue;
        };
        for cap in re.captures_iter(html) {
            let raw = strip_tags(cap.get(1).map(|m| m.as_str()).unwrap_or(""));
            if looks_like_claim_prose(&raw) {
                return Some(truncate_claim(&raw));
            }
        }
    }
    None
}

fn collect_headings(html: &str) -> Vec<(usize, u8, String)> {
    let re = regex::Regex::new(r"(?is)<h([1-4])[^>]*>\s*(?:<[^>]+>\s*)*([^<]{2,80})");
    let Ok(re) = re else {
        return Vec::new();
    };
    re.captures_iter(html)
        .filter_map(|c| {
            let start = c.get(0)?.start();
            let level = c.get(1)?.as_str().parse::<u8>().ok()?;
            let text = collapse_ws(&strip_tags(c.get(2)?.as_str()));
            if text.is_empty() {
                return None;
            }
            Some((start, level, text))
        })
        .collect()
}

fn push_claim(
    out: &mut Vec<PublicClaim>,
    text: &str,
    topic: Option<&str>,
    source: &str,
    source_url: Option<String>,
    trust: &str,
) {
    let text = truncate_claim(text);
    if !looks_like_claim_prose(&text) {
        return;
    }
    let topic_s = topic
        .map(collapse_ws)
        .filter(|t| !t.is_empty() && !heading_is_year_only(t) && !heading_is_stop(t));
    let norm = text.to_ascii_lowercase();
    if out.iter().any(|c| c.text.to_ascii_lowercase() == norm) {
        return;
    }
    if out.len() >= 20 {
        return;
    }
    let keywords = lexicon_keywords(topic_s.as_deref(), &text);
    let kind = claim_kind_of(&text).to_string();
    out.push(PublicClaim {
        text,
        topic: topic_s,
        source: source.into(),
        source_url,
        trust: trust.into(),
        kind,
        keywords,
    });
}

fn claims_from_html(html: &str, page_url: &str, source: &str, trust: &str) -> Vec<PublicClaim> {
    if html.len() < 80 {
        return Vec::new();
    }
    let url = if page_url.trim().is_empty() {
        None
    } else {
        Some(page_url.trim().to_string())
    };
    let headings = collect_headings(html);
    let mut out = Vec::new();

    for (i, (start, level, title)) in headings.iter().enumerate() {
        if !heading_is_claim_section(title) {
            continue;
        }
        let end = headings
            .iter()
            .skip(i + 1)
            .find(|(_, lv, _)| *lv <= *level)
            .map(|(s, _, _)| *s)
            .unwrap_or(html.len());
        let slice = &html[*start..end.min(html.len())];
        let mut got_sub = false;
        for (j, (s2, lv2, sub)) in headings.iter().enumerate().skip(i + 1) {
            if *s2 >= end {
                break;
            }
            if *lv2 <= *level {
                break;
            }
            if heading_is_year_only(sub) || heading_is_stop(sub) || heading_is_claim_section(sub) {
                continue;
            }
            let sub_end = headings
                .get(j + 1)
                .map(|(s, _, _)| *s)
                .unwrap_or(end)
                .min(end);
            if let Some(prose) = first_prose_in(&html[*s2..sub_end]) {
                push_claim(&mut out, &prose, Some(sub), source, url.clone(), trust);
                got_sub = true;
            }
        }
        if !got_sub {
            if let Some(prose) = first_prose_in(slice) {
                push_claim(&mut out, &prose, Some(title), source, url.clone(), trust);
            }
        }
    }

    if out.is_empty() {
        let url_l = page_url.to_ascii_lowercase();
        let page_is_issues = url_l.contains("issue")
            || url_l.contains("platform")
            || url_l.contains("priorit")
            || url_l.contains("agenda")
            || url_l.contains("position");
        if page_is_issues {
            for (i, (start, level, title)) in headings.iter().enumerate() {
                if heading_is_stop(title)
                    || heading_is_year_only(title)
                    || heading_is_claim_section(title)
                {
                    continue;
                }
                if *level > 3 {
                    continue;
                }
                let end = headings
                    .get(i + 1)
                    .map(|(s, _, _)| *s)
                    .unwrap_or(html.len());
                if let Some(prose) = first_prose_in(&html[*start..end]) {
                    push_claim(&mut out, &prose, Some(title), source, url.clone(), trust);
                }
            }
        }
    }
    out
}

pub fn claims_from_ballotpedia_html(html: &str, page_url: &str) -> Vec<PublicClaim> {
    claims_from_html(html, page_url, "Ballotpedia", "reference")
}

pub fn campaign_claim_urls(site_url: &str) -> Vec<String> {
    let base = normalize_site(site_url);
    if base.len() < 12 {
        return Vec::new();
    }
    let mut out = vec![
        format!("{base}/issues"),
        format!("{base}/issues/"),
        format!("{base}/platform"),
        format!("{base}/priorities"),
        format!("{base}/agenda"),
        format!("{base}/on-the-issues"),
        format!("{base}/my-positions"),
        format!("{base}/positions"),
        format!("{base}/why-im-running"),
    ];
    let mut seen = std::collections::HashSet::new();
    out.retain(|u| seen.insert(u.clone()));
    out
}

pub fn claims_from_campaign_html(html: &str, page_url: &str) -> Vec<PublicClaim> {
    claims_from_html(html, page_url, "Campaign website", "campaign")
}

pub fn merge_claims(base: &[PublicClaim], extra: &[PublicClaim]) -> Vec<PublicClaim> {
    let mut out = base.to_vec();
    for c in extra {
        let norm = c.text.to_ascii_lowercase();
        if out.iter().any(|x| x.text.to_ascii_lowercase() == norm) {
            continue;
        }
        out.push(c.clone());
        if out.len() >= 24 {
            break;
        }
    }
    out
}

fn vote_blob(v: &crate::models::VoteRecord) -> String {
    format!("{} {}", v.question, v.result.as_deref().unwrap_or("")).to_ascii_lowercase()
}

/// Keyword-pair claims vs loaded votes/opinions. Overlap is not a lie/bought verdict.
pub fn pair_claims_with_votes(
    claims: &[PublicClaim],
    votes: &[crate::models::VoteRecord],
) -> Vec<ContrastCard> {
    const NOTE: &str =
        "Keyword overlap only — not a finding that the vote contradicts or confirms the claim.";
    let mut cards = Vec::new();
    for c in claims {
        let keys = if c.keywords.is_empty() {
            lexicon_keywords(c.topic.as_deref(), &c.text)
        } else {
            c.keywords.clone()
        };
        let mut scored: Vec<(usize, ContrastMatch)> = Vec::new();
        if !keys.is_empty() {
            for v in votes {
                let blob = vote_blob(v);
                let overlap: Vec<String> = keys
                    .iter()
                    .filter(|k| blob.contains(k.as_str()) || blob.contains(&k.replace('-', " ")))
                    .cloned()
                    .collect();
                if overlap.is_empty() {
                    continue;
                }
                scored.push((
                    overlap.len(),
                    ContrastMatch {
                        date: v.date.clone(),
                        question: v.question.clone(),
                        position: v.position.clone(),
                        result: v.result.clone(),
                        url: v.url.clone(),
                        overlap,
                    },
                ));
            }
        }
        scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.date.cmp(&a.1.date)));
        let matches: Vec<ContrastMatch> = scored.into_iter().take(4).map(|(_, m)| m).collect();
        cards.push(ContrastCard {
            claim_text: c.text.clone(),
            topic: c.topic.clone(),
            claim_source: c.source.clone(),
            claim_url: c.source_url.clone(),
            claim_trust: c.trust.clone(),
            claim_kind: c.kind.clone(),
            matches,
            note: NOTE.into(),
            llm_note: None,
            llm_trust: None,
            llm_model: None,
        });
        if cards.len() >= 20 {
            break;
        }
    }
    cards
}

pub const LLM_PROVIDER_XAI: &str = "xai";
pub const LLM_PROVIDER_OPENAI: &str = "openai";
pub const LLM_XAI_CHAT_URL: &str = "https://api.x.ai/v1/chat/completions";
pub const LLM_OPENAI_CHAT_URL: &str = "https://api.openai.com/v1/chat/completions";
pub const LLM_XAI_DEFAULT_MODEL: &str = "grok-4-fast";
pub const LLM_OPENAI_DEFAULT_MODEL: &str = "gpt-4o-mini";

const LLM_VERDICT_NEEDLES: &[&str] = &[
    "bought",
    "scammer",
    "liar",
    "liars",
    "hypocrite",
    "corrupt",
    "corruption",
    "fraud",
    "fraudulent",
    "crook",
    "on the take",
    "sold out",
    "paid-for",
];

const LLM_SYSTEM: &str = "You compare a candidate's stated position to related roll-call votes or judicial opinions.\n\
Write 1-2 factual sentences per card. Cite the vote or opinion date and the recorded position.\n\
Do not call anyone bought, a scammer, a liar, corrupt, or a hypocrite.\n\
Do not say a vote confirms, contradicts, or proves a campaign claim.\n\
Keyword overlap is not a finding. Describe what the statement says and what the related record shows.\n\
Reply with JSON only: {\"notes\":[{\"i\":0,\"note\":\"...\"}]}\n\
i is the 0-based card index from the user list. Only include cards that have related records.";

pub fn llm_chat_url(provider: &str) -> Option<&'static str> {
    match provider.trim().to_ascii_lowercase().as_str() {
        "xai" | "grok" | "x.ai" => Some(LLM_XAI_CHAT_URL),
        "openai" | "chatgpt" => Some(LLM_OPENAI_CHAT_URL),
        _ => None,
    }
}

pub fn llm_default_model(provider: &str) -> Option<&'static str> {
    match provider.trim().to_ascii_lowercase().as_str() {
        "xai" | "grok" | "x.ai" => Some(LLM_XAI_DEFAULT_MODEL),
        "openai" | "chatgpt" => Some(LLM_OPENAI_DEFAULT_MODEL),
        _ => None,
    }
}

pub fn llm_normalize_provider(provider: &str) -> Option<&'static str> {
    match provider.trim().to_ascii_lowercase().as_str() {
        "xai" | "grok" | "x.ai" => Some(LLM_PROVIDER_XAI),
        "openai" | "chatgpt" => Some(LLM_PROVIDER_OPENAI),
        _ => None,
    }
}

fn llm_user_prompt(cards: &[ContrastCard], name: &str, office: &str) -> String {
    let mut out = format!(
        "Candidate: {}\nOffice: {}\nCards:\n",
        collapse_ws(name),
        collapse_ws(office)
    );
    for (i, c) in cards.iter().enumerate() {
        if c.matches.is_empty() {
            continue;
        }
        out.push_str(&format!(
            "[{i}] ({}) {}\n",
            c.claim_kind,
            collapse_ws(&c.claim_text)
        ));
        for m in &c.matches {
            out.push_str(&format!(
                "  - {} {} · {}\n",
                m.date,
                m.position,
                collapse_ws(&m.question)
            ));
        }
    }
    out
}

pub fn llm_contrast_request_body(
    model: &str,
    cards: &[ContrastCard],
    name: &str,
    office: &str,
) -> Option<String> {
    let model = model.trim();
    if model.is_empty() {
        return None;
    }
    if !cards.iter().any(|c| !c.matches.is_empty()) {
        return None;
    }
    let body = serde_json::json!({
        "model": model,
        "temperature": 0.2,
        "max_tokens": 900,
        "messages": [
            { "role": "system", "content": LLM_SYSTEM },
            { "role": "user", "content": llm_user_prompt(cards, name, office) }
        ]
    });
    serde_json::to_string(&body).ok()
}

pub fn parse_llm_chat_content(response_json: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(response_json).ok()?;
    let content = v
        .pointer("/choices/0/message/content")
        .and_then(|x| x.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())?;
    Some(content.to_string())
}

fn extract_json_object(s: &str) -> Option<&str> {
    let t = s.trim();
    if let Some(start) = t.find('{') {
        if let Some(end) = t.rfind('}') {
            if end > start {
                return Some(&t[start..=end]);
            }
        }
    }
    None
}

#[derive(Debug, Deserialize)]
struct LlmNotesFile {
    #[serde(default)]
    notes: Vec<LlmNoteRow>,
}

#[derive(Debug, Deserialize)]
struct LlmNoteRow {
    #[serde(default)]
    i: usize,
    #[serde(default)]
    note: String,
}

pub fn parse_llm_contrast_notes(content: &str) -> Vec<(usize, String)> {
    let raw = extract_json_object(content).unwrap_or(content);
    let parsed: LlmNotesFile = match serde_json::from_str(raw) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    parsed
        .notes
        .into_iter()
        .filter_map(|row| {
            let note = collapse_ws(&row.note);
            if note.len() < 20 {
                return None;
            }
            Some((row.i, note))
        })
        .take(12)
        .collect()
}

fn note_has_verdict(note: &str) -> bool {
    let low = note.to_ascii_lowercase();
    LLM_VERDICT_NEEDLES.iter().any(|n| low.contains(n))
}

fn truncate_llm_note(s: &str) -> String {
    let t = collapse_ws(s);
    if t.chars().count() <= 420 {
        return t;
    }
    let mut out: String = t.chars().take(417).collect();
    out.push('…');
    out
}

pub fn apply_llm_contrast_notes(
    cards: &mut [ContrastCard],
    notes: &[(usize, String)],
    model: &str,
) -> usize {
    let model = collapse_ws(model);
    let mut n = 0;
    for (i, note) in notes {
        if *i >= cards.len() {
            continue;
        }
        if cards[*i].matches.is_empty() {
            continue;
        }
        if note_has_verdict(note) {
            continue;
        }
        let cleaned = truncate_llm_note(note);
        if cleaned.len() < 20 {
            continue;
        }
        cards[*i].llm_note = Some(cleaned);
        cards[*i].llm_trust = Some("inference".into());
        if !model.is_empty() {
            cards[*i].llm_model = Some(model.clone());
        }
        n += 1;
    }
    n
}

pub fn apply_llm_chat_response(
    cards: &mut [ContrastCard],
    response_json: &str,
    model: &str,
) -> usize {
    let content = match parse_llm_chat_content(response_json) {
        Some(c) => c,
        None => return 0,
    };
    let notes = parse_llm_contrast_notes(&content);
    apply_llm_contrast_notes(cards, &notes, model)
}

/// Public license / ethics / judicial-discipline card. Cited only.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecordHit {
    pub kind: String,
    pub title: String,
    pub detail: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    pub url: String,
    pub source: String,
    pub trust: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

pub const FL_ETHICS_ORDERS_URL: &str = "https://ethics.state.fl.us/Research/Orders.aspx";
pub const FL_ETHICS_FILINGS_API: &str =
    "https://disclosure.floridaethics.gov/api/PublicFiling/SearchPublicFilings";
pub const FL_JQC_NEWS_URL: &str = "https://floridajqc.com/news/";
pub const FL_JQC_POSTS_API: &str = "https://floridajqc.com/wp-json/wp/v2/posts";

fn name_token(raw: &str) -> String {
    raw.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_uppercase()
}

fn is_honorific_or_suffix(tok: &str) -> bool {
    matches!(
        tok,
        "HON"
            | "HONORABLE"
            | "JUDGE"
            | "JUSTICE"
            | "MR"
            | "MRS"
            | "MS"
            | "MISS"
            | "DR"
            | "REP"
            | "SEN"
            | "PROF"
            | "REV"
            | "JR"
            | "SR"
            | "II"
            | "III"
            | "IV"
            | "ESQ"
    )
}

/// First + last after dropping honorifics/suffixes. `None` if fewer than two tokens.
pub fn record_first_last(name: &str) -> Option<(String, String)> {
    let toks: Vec<String> = name
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '\'')
        .filter(|t| !t.is_empty())
        .map(name_token)
        .filter(|t| t.len() >= 2 && !is_honorific_or_suffix(t))
        .collect();
    if toks.len() < 2 {
        return None;
    }
    Some((toks[0].clone(), toks[toks.len() - 1].clone()))
}

fn first_names_compatible(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    if a.len() == 1 && b.starts_with(a) {
        return true;
    }
    if b.len() == 1 && a.starts_with(b) {
        return true;
    }
    false
}

fn names_match_person(found: &str, candidate: &str) -> bool {
    let Some((cf, cl)) = record_first_last(candidate) else {
        return false;
    };
    let Some((ff, fl)) = record_first_last(found) else {
        return false;
    };
    fl == cl && first_names_compatible(&cf, &ff)
}

fn blob_has_first_and_last(blob: &str, candidate: &str) -> bool {
    let Some((first, last)) = record_first_last(candidate) else {
        return false;
    };
    let u = blob.to_ascii_uppercase();
    u.contains(&last) && u.contains(&first)
}

fn record_hit(
    kind: &str,
    title: String,
    detail: String,
    date: Option<String>,
    url: String,
    source: &str,
    status: Option<String>,
) -> RecordHit {
    RecordHit {
        kind: kind.into(),
        title,
        detail,
        date,
        url,
        source: source.into(),
        trust: "official".into(),
        status,
    }
}

pub fn fl_bar_search_url(name: &str) -> Option<String> {
    let (first, last) = record_first_last(name)?;
    Some(format!(
        "https://www.floridabar.org/directories/find-mbr/?fName={}&lName={}&sdx=N",
        urlencoding_minimal(&first.to_ascii_lowercase()),
        urlencoding_minimal(&last.to_ascii_lowercase())
    ))
}

fn bar_search_blocked(html: &str) -> bool {
    let l = html.to_ascii_lowercase();
    l.contains("too many results")
        || l.contains("just a moment")
        || l.contains("cf-challenge")
        || l.contains("attention required")
}

/// Unique first+last Bar directory card → standing / eligibility. Ambiguous → empty.
pub fn parse_fl_bar_search_html(html: &str, person_name: &str) -> Vec<RecordHit> {
    if html.len() < 80 || bar_search_blocked(html) {
        return Vec::new();
    }
    let mut hits = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find("profile-compact") {
        let chunk = &rest[start..];
        let end = chunk.find("</li>").unwrap_or(chunk.len().min(2500));
        let card = &chunk[..end];
        rest = &chunk[end.min(chunk.len())..];
        let name = attr_or_class_text(card, "profile-name");
        if name.is_empty() || !names_match_person(&name, person_name) {
            continue;
        }
        let href = href_containing(card, "profile/?num=");
        let bar_no = attr_or_class_text(card, "profile-bar-number");
        let standing = attr_or_class_text(card, "member-status");
        let eligibility = attr_or_class_text(card, "eligibility");
        let status = if !eligibility.is_empty() {
            eligibility.clone()
        } else {
            standing.clone()
        };
        let mut detail_parts = Vec::new();
        if !bar_no.is_empty() {
            detail_parts.push(bar_no);
        }
        if !standing.is_empty() {
            detail_parts.push(standing);
        }
        if !eligibility.is_empty() && !detail_parts.iter().any(|p| p == &eligibility) {
            detail_parts.push(eligibility);
        }
        let url = if href.is_empty() {
            fl_bar_search_url(person_name).unwrap_or_default()
        } else {
            href
        };
        hits.push(record_hit(
            "bar_status",
            format!("Florida Bar — {name}"),
            if detail_parts.is_empty() {
                "Directory listing.".into()
            } else {
                detail_parts.join(" · ")
            },
            None,
            url,
            "Florida Bar",
            if status.is_empty() {
                None
            } else {
                Some(status)
            },
        ));
    }
    if hits.len() != 1 {
        return Vec::new();
    }
    hits
}

fn attr_or_class_text(html: &str, class: &str) -> String {
    let needle = format!("class=\"{class}");
    let Some(i) = html.find(&needle) else {
        return String::new();
    };
    let after = &html[i + needle.len()..];
    let Some(gt) = after.find('>') else {
        return String::new();
    };
    let inner = &after[gt + 1..];
    let end = inner.find("</").unwrap_or(inner.len().min(240));
    collapse_ws(&strip_tags(&inner[..end]))
}

fn href_containing(html: &str, needle: &str) -> String {
    let mut rest = html;
    while let Some(i) = rest.find("href=\"") {
        let after = &rest[i + 6..];
        let Some(end) = after.find('"') else {
            break;
        };
        let href = &after[..end];
        if href.contains(needle) {
            if href.starts_with("http") {
                return href.to_string();
            }
            if href.starts_with('/') {
                return format!("https://www.floridabar.org{href}");
            }
        }
        rest = &after[end + 1..];
    }
    String::new()
}

pub fn fl_ethics_filings_url(name: &str) -> Option<String> {
    let (first, last) = record_first_last(name)?;
    Some(format!(
        "{FL_ETHICS_FILINGS_API}?filterByFirstName={}&filterByLastName={}&pageSize=25&pageNumber=1",
        urlencoding_minimal(&first.to_ascii_lowercase()),
        urlencoding_minimal(&last.to_ascii_lowercase())
    ))
}

pub fn fl_ethics_orders_url() -> &'static str {
    FL_ETHICS_ORDERS_URL
}

/// EFDMS public filings JSON. Unique person only; cap recent rows.
pub fn parse_fl_ethics_filings_json(json: &str, person_name: &str) -> Vec<RecordHit> {
    let v: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let rows = v
        .get("data")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();
    let mut matched = Vec::new();
    for row in rows {
        let full = row
            .get("fullName")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if full.is_empty() || !names_match_person(&full, person_name) {
            continue;
        }
        matched.push(row);
    }
    let mut identities = matched
        .iter()
        .filter_map(|r| {
            r.get("fullName")
                .and_then(|x| x.as_str())
                .and_then(record_first_last)
        })
        .collect::<Vec<_>>();
    identities.sort();
    identities.dedup();
    if identities.len() != 1 {
        return Vec::new();
    }
    let mut out = Vec::new();
    for row in matched.into_iter().take(5) {
        let full = row
            .get("fullName")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .trim();
        let form = row
            .get("formType")
            .and_then(|x| x.as_str())
            .unwrap_or("disclosure")
            .trim();
        let year = row
            .get("formYear")
            .and_then(|x| x.as_i64())
            .map(|y| y.to_string())
            .unwrap_or_default();
        let org = row
            .get("delimitedOrganizationNames")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .trim();
        let submitted = row
            .get("submissionDate")
            .and_then(|x| x.as_str())
            .map(ethics_date)
            .filter(|s| !s.is_empty());
        let url = row
            .get("filingHistoryUrl")
            .and_then(|x| x.as_str())
            .unwrap_or("https://disclosure.floridaethics.gov/PublicSearch/Filings")
            .trim()
            .to_string();
        let mut detail = form.to_string();
        if !year.is_empty() {
            detail = format!("{detail} · {year}");
        }
        if !org.is_empty() {
            detail = format!("{detail} · {org}");
        }
        if let Some(d) = submitted.as_deref() {
            detail = format!("{detail} · filed {d}");
        }
        out.push(record_hit(
            "ethics_filing",
            format!("Ethics disclosure — {full}"),
            detail,
            submitted,
            url,
            "Florida Commission on Ethics",
            Some(form.to_string()),
        ));
    }
    out
}

fn ethics_date(s: &str) -> String {
    let t = s.trim();
    if t.len() >= 10 && t.as_bytes()[4] == b'-' {
        t[..10].to_string()
    } else {
        t.to_string()
    }
}

/// Final/recommended orders listing. Unique first+last only.
pub fn parse_fl_ethics_orders_html(html: &str, person_name: &str) -> Vec<RecordHit> {
    if html.len() < 80 {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(tr) = rest.find("<tr") {
        let chunk = &rest[tr..];
        let end = chunk.find("</tr>").unwrap_or(chunk.len().min(4000));
        let row = &chunk[..end];
        rest = &chunk[end.min(chunk.len())..];
        if row.to_ascii_lowercase().contains("<h3>") {
            continue;
        }
        let cells = table_cells_raw(row);
        if cells.len() < 2 {
            continue;
        }
        let complaint = collapse_ws(&strip_tags(&cells[0]));
        let who = collapse_ws(&strip_tags(&cells[1]));
        if who.is_empty() || !names_match_person(&who, person_name) {
            continue;
        }
        let docs = if cells.len() >= 3 {
            cells[2].clone()
        } else {
            String::new()
        };
        let href = first_http_or_path_href(&docs, "https://ethics.state.fl.us");
        let url = if href.is_empty() {
            FL_ETHICS_ORDERS_URL.into()
        } else {
            href
        };
        out.push(record_hit(
            "ethics_order",
            format!("Ethics order — {who}"),
            if complaint.is_empty() {
                "Final or recommended order on file.".into()
            } else {
                format!("Complaint {complaint}")
            },
            complaint_year(&complaint),
            url,
            "Florida Commission on Ethics",
            Some("order".into()),
        ));
    }
    if out.len() > 1 {
        let mut ids = out
            .iter()
            .filter_map(|h| record_first_last(&h.title))
            .collect::<Vec<_>>();
        ids.sort();
        ids.dedup();
        if ids.len() != 1 {
            return Vec::new();
        }
    }
    out.truncate(5);
    out
}

fn table_cells_raw(row: &str) -> Vec<String> {
    let mut cells = Vec::new();
    let mut rest = row;
    while let Some(i) = rest.find("<td") {
        let after = &rest[i..];
        let Some(gt) = after.find('>') else {
            break;
        };
        let inner = &after[gt + 1..];
        let end = inner.find("</td>").unwrap_or(inner.len().min(2000));
        cells.push(inner[..end].to_string());
        rest = &inner[end.min(inner.len())..];
    }
    cells
}

fn first_http_or_path_href(html: &str, origin: &str) -> String {
    let mut rest = html;
    while let Some(i) = rest.find("href=\"") {
        let after = &rest[i + 6..];
        let Some(end) = after.find('"') else {
            break;
        };
        let href = after[..end].trim();
        if href.starts_with("http") {
            return href.to_string();
        }
        if href.starts_with('/') {
            return format!("{origin}{href}");
        }
        rest = &after[end + 1..];
    }
    String::new()
}

fn complaint_year(complaint: &str) -> Option<String> {
    let digits: String = complaint
        .chars()
        .take(2)
        .filter(|c| c.is_ascii_digit())
        .collect();
    if digits.len() == 2 {
        let n: u32 = digits.parse().ok()?;
        let year = if n >= 80 { 1900 + n } else { 2000 + n };
        return Some(year.to_string());
    }
    None
}

pub fn fl_jqc_posts_url(name: &str) -> Option<String> {
    let (_, last) = record_first_last(name)?;
    Some(format!(
        "{FL_JQC_POSTS_API}?search={}&per_page=20",
        urlencoding_minimal(&last.to_ascii_lowercase())
    ))
}

pub fn fl_jqc_news_url() -> &'static str {
    FL_JQC_NEWS_URL
}

/// WordPress posts JSON. First+last must appear in title or excerpt.
pub fn parse_fl_jqc_posts_json(json: &str, person_name: &str) -> Vec<RecordHit> {
    let v: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let rows = match v.as_array() {
        Some(a) => a.clone(),
        None => return Vec::new(),
    };
    let mut out = Vec::new();
    for row in rows {
        let title = row
            .pointer("/title/rendered")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let excerpt = row
            .pointer("/excerpt/rendered")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        let blob = format!("{title} {}", strip_tags(&excerpt));
        if title.is_empty() || !blob_has_first_and_last(&blob, person_name) {
            continue;
        }
        let url = row
            .get("link")
            .and_then(|x| x.as_str())
            .unwrap_or(FL_JQC_NEWS_URL)
            .trim()
            .to_string();
        let date = row
            .get("date")
            .and_then(|x| x.as_str())
            .map(ethics_date)
            .filter(|s| !s.is_empty());
        out.push(record_hit(
            "jqc_notice",
            collapse_ws(&strip_tags(&title)),
            "JQC public notice — not a pending-complaint finding.".into(),
            date,
            url,
            "Florida JQC",
            Some("notice".into()),
        ));
        if out.len() >= 8 {
            break;
        }
    }
    out
}

/// JQC news / home HTML fallback. First+last in heading + nearby prose.
pub fn parse_fl_jqc_news_html(html: &str, person_name: &str) -> Vec<RecordHit> {
    if html.len() < 80 {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(i) = rest.find("<h3") {
        let after = &rest[i..];
        let Some(gt) = after.find('>') else {
            break;
        };
        let inner = &after[gt + 1..];
        let end = inner.find("</h3>").unwrap_or(inner.len().min(400));
        let heading = collapse_ws(&strip_tags(&inner[..end]));
        let following = inner.get(end..end.saturating_add(500)).unwrap_or("");
        let blob = format!("{heading} {}", strip_tags(following));
        rest = &inner[end.min(inner.len())..];
        if heading.len() < 12 || !blob_has_first_and_last(&blob, person_name) {
            continue;
        }
        let href = first_http_or_path_href(after, "https://floridajqc.com");
        let url = if href.is_empty() {
            FL_JQC_NEWS_URL.into()
        } else {
            href
        };
        if out
            .iter()
            .any(|h: &RecordHit| h.url == url || h.title == heading)
        {
            continue;
        }
        out.push(record_hit(
            "jqc_notice",
            heading,
            "JQC public notice — not a pending-complaint finding.".into(),
            None,
            url,
            "Florida JQC",
            Some("notice".into()),
        ));
        if out.len() >= 8 {
            break;
        }
    }
    out
}

pub fn merge_record_hits(base: &[RecordHit], extra: &[RecordHit]) -> Vec<RecordHit> {
    let mut out = base.to_vec();
    for e in extra {
        if out.iter().any(|b| {
            (!e.url.is_empty() && b.url == e.url)
                || (b.kind == e.kind && b.title == e.title && b.detail == e.detail)
        }) {
            continue;
        }
        out.push(e.clone());
    }
    out
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BallotpediaMeasureLink {
    pub title: String,
    pub url: String,
    #[serde(default)]
    pub slug_title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amendment: Option<u32>,
}

fn bp_state_slug(state: &str) -> Option<&'static str> {
    match state.trim().to_ascii_uppercase().as_str() {
        "FL" | "FLORIDA" => Some("Florida"),
        "AZ" | "ARIZONA" => Some("Arizona"),
        "NC" | "NORTH CAROLINA" => Some("North_Carolina"),
        "MD" | "MARYLAND" => Some("Maryland"),
        "CA" | "CALIFORNIA" => Some("California"),
        "NY" | "NEW YORK" => Some("New_York"),
        "TX" | "TEXAS" => Some("Texas"),
        "GA" | "GEORGIA" => Some("Georgia"),
        "OH" | "OHIO" => Some("Ohio"),
        "PA" | "PENNSYLVANIA" => Some("Pennsylvania"),
        "WA" | "WASHINGTON" => Some("Washington"),
        "OR" | "OREGON" => Some("Oregon"),
        "CO" | "COLORADO" => Some("Colorado"),
        "NV" | "NEVADA" => Some("Nevada"),
        "MI" | "MICHIGAN" => Some("Michigan"),
        "MO" | "MISSOURI" => Some("Missouri"),
        "MA" | "MASSACHUSETTS" => Some("Massachusetts"),
        _ => None,
    }
}

pub fn ballotpedia_state_measures_url(state: &str, year: i32) -> Option<String> {
    if !(2000..=2100).contains(&year) {
        return None;
    }
    let slug = bp_state_slug(state)?;
    Some(format!(
        "https://ballotpedia.org/{slug}_{year}_ballot_measures"
    ))
}

fn resolve_ballotpedia_href(href: &str) -> Option<String> {
    let h = href.trim().replace("&amp;", "&");
    if h.is_empty() || h.starts_with('#') {
        return None;
    }
    if h.starts_with("https://ballotpedia.org/") || h.starts_with("http://ballotpedia.org/") {
        return Some(h.replace("http://", "https://"));
    }
    if let Some(rest) = h.strip_prefix("//ballotpedia.org/") {
        return Some(format!("https://ballotpedia.org/{rest}"));
    }
    if h.starts_with('/') && !h.starts_with("//") {
        return Some(format!("https://ballotpedia.org{h}"));
    }
    None
}

fn is_state_measure_href(href: &str) -> bool {
    let h = href.to_ascii_lowercase();
    if h.contains("_on_the_ballot")
        || h.contains("category:")
        || h.contains("special:categories")
        || h.contains("legislatively_referred")
        || h.contains("initiated_constitutional")
    {
        return false;
    }
    let path = h.split('?').next().unwrap_or(&h);
    let year_paren = regex::Regex::new(r"_\(((?:19|20)\d{2})\)(?:#|$)").ok();
    let Some(re) = year_paren else {
        return false;
    };
    re.is_match(path)
        && (path.contains("amendment")
            || path.contains("initiative")
            || path.contains("measure")
            || path.contains("question")
            || path.contains("proposition")
            || path.contains("referendum"))
}

fn slug_title_from_url(url: &str) -> String {
    let path = url
        .rsplit('/')
        .next()
        .unwrap_or(url)
        .split('?')
        .next()
        .unwrap_or("");
    let decoded = decode_entities(&path.replace('_', " ").replace("%2C", ","));
    let stripped = regex::Regex::new(r"\s*\((?:19|20)\d{2}\)\s*$")
        .ok()
        .and_then(|re| {
            let s = re.replace(&decoded, "").to_string();
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        })
        .unwrap_or(decoded);
    collapse_ws(&stripped)
}

fn amendment_num_from_text(s: &str) -> Option<u32> {
    let re = regex::Regex::new(r"(?i)\bamendment[_\s]+#?(\d+)\b").ok()?;
    let n: u32 = re.captures(s)?.get(1)?.as_str().parse().ok()?;
    if (1..80).contains(&n) {
        Some(n)
    } else {
        None
    }
}

pub fn measure_amendment_number(code: Option<&str>, title: &str) -> Option<u32> {
    amendment_num_from_text(code.unwrap_or(""))
        .or_else(|| amendment_num_from_text(title))
}

fn heading_index_ci(html: &str, needle: &str) -> Option<usize> {
    let low = html.to_ascii_lowercase();
    let n = needle.to_ascii_lowercase();
    let re = regex::Regex::new(&format!(
        r#"(?is)<h[1-4][^>]*>\s*(?:<[^>]+>\s*)*[^<]*{}[^<]*"#,
        regex::escape(&n)
    ))
    .ok()?;
    re.find(&low).map(|m| m.start())
}

fn on_the_ballot_slice(html: &str) -> &str {
    let start = heading_index_ci(html, "On the ballot").unwrap_or(0);
    if start == 0 && !html.to_ascii_lowercase().contains("on the ballot") {
        return "";
    }
    let rest = &html[start.min(html.len())..];
    let stops = [
        "Getting measures on the ballot",
        "Not on the ballot",
        "Historical facts",
        "See also",
        "External links",
    ];
    let mut end = rest.len();
    for stop in stops {
        if let Some(i) = heading_index_ci(rest, stop) {
            if i > 40 && i < end {
                end = i;
            }
        }
    }
    &rest[..end.min(rest.len())]
}

pub fn ballotpedia_measure_links_from_index(html: &str) -> Vec<BallotpediaMeasureLink> {
    if html.len() < 80 {
        return Vec::new();
    }
    let slice = on_the_ballot_slice(html);
    if slice.len() < 40 {
        return Vec::new();
    }
    let re = match regex::Regex::new(r#"(?is)<a\b[^>]*href="([^"]+)"[^>]*>([\s\S]{1,200}?)</a>"#) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for cap in re.captures_iter(slice) {
        let href = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let Some(url) = resolve_ballotpedia_href(href) else {
            continue;
        };
        if !is_state_measure_href(&url) {
            continue;
        }
        if !seen.insert(url.clone()) {
            continue;
        }
        let title = collapse_ws(&strip_tags(cap.get(2).map(|m| m.as_str()).unwrap_or("")));
        if title.is_empty() || title.len() > 180 {
            continue;
        }
        let slug_title = slug_title_from_url(&url);
        let amendment = amendment_num_from_text(&title)
            .or_else(|| amendment_num_from_text(&slug_title))
            .or_else(|| amendment_num_from_text(&url));
        out.push(BallotpediaMeasureLink {
            title,
            url,
            slug_title,
            amendment,
        });
    }
    out
}

const MEASURE_STOP: &[&str] = &[
    "the", "and", "for", "from", "with", "that", "this", "into", "used",
    "our", "other", "among", "such", "than", "amendment", "florida", "arizona",
    "maryland", "carolina", "north", "south", "ballot", "measure", "initiative",
    "constitutional", "changes", "question", "proposition",
];

fn stem_measure_token(s: &str) -> String {
    let t = s.to_ascii_lowercase();
    if t.len() > 4 && t.ends_with("es") && !t.ends_with("ss") {
        return t[..t.len() - 2].to_string();
    }
    if t.len() > 4 && t.ends_with('s') && !t.ends_with("ss") {
        return t[..t.len() - 1].to_string();
    }
    t
}

fn significant_measure_tokens(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_ascii_alphanumeric())
        .map(stem_measure_token)
        .filter(|t| t.len() >= 3 && !MEASURE_STOP.contains(&t.as_str()))
        .collect()
}

fn measure_tokens_compat(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    let (x, y) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    if x.len() >= 3 && y.starts_with(x) {
        return true;
    }
    a.chars().zip(b.chars()).take_while(|(c, d)| c == d).count() >= 8
}

fn measure_token_overlap(a: &[String], b: &[String]) -> u32 {
    a.iter()
        .filter(|x| b.iter().any(|y| measure_tokens_compat(x, y)))
        .count() as u32
}

fn score_measure_link(link: &BallotpediaMeasureLink, title: &str, code: Option<&str>) -> u32 {
    let blob = format!(
        "{} {} {}",
        link.title, link.slug_title, link.url.replace('_', " ")
    );
    let left = significant_measure_tokens(&format!("{} {}", code.unwrap_or(""), title));
    let right = significant_measure_tokens(&blob);
    let mut score = measure_token_overlap(&left, &right);
    if let Some(n) = measure_amendment_number(code, title) {
        if link.amendment == Some(n) {
            score = score.saturating_add(5);
        }
    }
    let low_title = title.to_ascii_lowercase();
    let low_blob = blob.to_ascii_lowercase();
    if (low_title.contains("save our homes") || low_title.contains("homestead"))
        && (low_blob.contains("homestead") || low_blob.contains("save our homes"))
    {
        score = score.saturating_add(3);
    }
    score
}

pub fn match_ballotpedia_measure(
    links: &[BallotpediaMeasureLink],
    title: &str,
    code: Option<&str>,
) -> Option<BallotpediaMeasureLink> {
    if title.trim().is_empty() || links.is_empty() {
        return None;
    }
    let mut scored: Vec<(u32, &BallotpediaMeasureLink)> = links
        .iter()
        .map(|l| (score_measure_link(l, title, code), l))
        .filter(|(s, _)| *s >= 2)
        .collect();
    if scored.is_empty() {
        return None;
    }
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    let best = scored[0].0;
    if scored.iter().filter(|(s, _)| *s == best).count() > 1 {
        return None;
    }
    if scored.len() > 1 && scored[1].0 + 1 >= best && best < 6 {
        return None;
    }
    Some(scored[0].1.clone())
}

pub fn ballotpedia_html_matches_measure(html: &str, title: &str, code: Option<&str>) -> bool {
    if html.len() < 80 || title.trim().is_empty() {
        return false;
    }
    let head = html.get(..html.len().min(12_000)).unwrap_or(html);
    let h1 = regex::Regex::new(r"(?is)<h1[^>]*>([\s\S]{3,400}?)</h1>")
        .ok()
        .and_then(|re| {
            re.captures(head)
                .map(|c| collapse_ws(&strip_tags(c.get(1).map(|m| m.as_str()).unwrap_or(""))))
        })
        .unwrap_or_default();
    let title_tag = regex::Regex::new(r"(?is)<title[^>]*>([\s\S]{3,300}?)</title>")
        .ok()
        .and_then(|re| {
            re.captures(head)
                .map(|c| collapse_ws(&strip_tags(c.get(1).map(|m| m.as_str()).unwrap_or(""))))
        })
        .unwrap_or_default();
    let blob = format!("{} {} {}", h1, title_tag, strip_tags(head).chars().take(2500).collect::<String>());
    let dummy = BallotpediaMeasureLink {
        title: h1,
        url: String::new(),
        slug_title: title_tag,
        amendment: amendment_num_from_text(&blob),
    };
    score_measure_link(&dummy, title, code) >= 2
}

fn heading_is_measure_stance(text: &str) -> Option<&'static str> {
    let low = collapse_ws(&text.to_ascii_lowercase());
    if low.contains("argument") || low.contains("campaign finance") {
        return None;
    }
    if low == "support" || low == "supporters" || low.starts_with("supporters") {
        return Some("support");
    }
    if low == "opposition" || low == "opponents" || low.starts_with("opponents") {
        return Some("oppose");
    }
    None
}

fn heading_stops_measure_endorsements(text: &str) -> bool {
    let low = collapse_ws(&text.to_ascii_lowercase());
    low.contains("argument")
        || low.contains("campaign finance")
        || low == "polls"
        || low.starts_with("background")
        || low.starts_with("path to the ballot")
        || low.starts_with("how to cast")
        || low.starts_with("see also")
        || low.starts_with("external")
        || low.starts_with("footnote")
        || low.starts_with("text of measure")
        || low.starts_with("measure design")
        || low == "overview"
}

fn looks_like_argument_item(text: &str) -> bool {
    let t = text.trim();
    if t.len() > 160 {
        return true;
    }
    if t.contains('"') && t.len() > 70 {
        return true;
    }
    if t.contains(": \"") || t.contains(":“") || t.contains(": “") {
        return true;
    }
    false
}

fn measure_endorser_kind(org: &str) -> &'static str {
    let t = org.trim();
    let low = t.to_ascii_lowercase();
    if low.starts_with("sen.")
        || low.starts_with("sen ")
        || low.starts_with("rep.")
        || low.starts_with("rep ")
        || low.starts_with("gov.")
        || low.starts_with("gov ")
        || low.starts_with("mayor")
        || low.starts_with("state sen")
        || low.starts_with("state rep")
        || low.starts_with("state chief")
        || low.contains("chief financial")
        || low.contains("commissioner")
    {
        "official"
    } else {
        "org"
    }
}

/// Ballotpedia measure Support / Opposition lists (not Arguments).
pub fn endorsements_from_ballotpedia_measure_html(html: &str, page_url: &str) -> Vec<Endorsement> {
    if html.len() < 80 {
        return Vec::new();
    }
    let url = if page_url.trim().is_empty() {
        None
    } else {
        Some(page_url.trim().to_string())
    };
    let headings = collect_headings(html);
    if headings.is_empty() {
        return Vec::new();
    }
    let li_re = match regex::Regex::new(r"(?is)<li\b[^>]*>([\s\S]{3,400}?)</li>") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    for (i, (start, _level, title)) in headings.iter().enumerate() {
        let Some(stance) = heading_is_measure_stance(title) else {
            continue;
        };
        let end = headings
            .iter()
            .skip(i + 1)
            .find(|(_, _, t)| {
                heading_is_measure_stance(t).is_some() || heading_stops_measure_endorsements(t)
            })
            .map(|(s, _, _)| *s)
            .unwrap_or(html.len());
        let slice = &html[*start..end.min(html.len())];
        for cap in li_re.captures_iter(slice) {
            let raw = collapse_ws(&strip_tags(cap.get(1).map(|m| m.as_str()).unwrap_or("")));
            let org = raw
                .split(" — ")
                .next()
                .unwrap_or(&raw)
                .split(" – ")
                .next()
                .unwrap_or(&raw)
                .trim();
            if !looks_like_endorser(org) || looks_like_argument_item(org) {
                continue;
            }
            push_endorsement(
                &mut out,
                org,
                stance,
                "Ballotpedia",
                url.clone(),
                measure_endorser_kind(org),
                "reference",
            );
        }
    }
    out.truncate(60);
    out
}

pub fn merge_endorsement_lists(base: &[Endorsement], extra: &[Endorsement]) -> Vec<Endorsement> {
    let mut out = base.to_vec();
    for e in extra {
        push_endorsement(
            &mut out,
            &e.org,
            &e.stance,
            &e.source,
            e.source_url.clone(),
            e.kind.as_deref().unwrap_or("org"),
            e.trust.as_deref().unwrap_or("reference"),
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_ballot_url_precinct_and_party() {
        let u = brevard_sample_ballot_url("201", "Republican", "").unwrap();
        assert!(u.contains("precinct=201.1"));
        assert!(u.contains("party=Rep"));
        assert!(u.contains("election=104"));
        assert!(u.contains("county=brevard"));
    }

    #[test]
    fn money_signals_large_donor_and_top5() {
        let sizes = r#"[{"label":"$200 and under","total_display":"$100"},{"label":"$1,000 – $1,999","total_display":"$900"}]"#;
        let indiv = r#"[
            {"name":"Alice Big","amount_display":"$500","location":"Orlando, FL"},
            {"name":"Bob Far","amount_display":"$300","location":"Austin, TX"},
            {"name":"Cara Near","amount_display":"$200","location":"Melbourne, FL"}
        ]"#;
        let m = money_signals_from_json(
            sizes,
            indiv,
            "[]",
            "[]",
            "$1,000",
            "",
            "",
            "Cara Near",
            "FL",
            "Brevard",
        );
        assert!(m.signals.iter().any(|s| s.id == "large_donor_share"));
        assert!(m.signals.iter().any(|s| s.id == "top5_concentration"));
        assert!(m.signals.iter().any(|s| s.id == "out_of_state_share"));
        assert!(m.signals.iter().any(|s| s.id == "name_match_share"));
    }

    #[test]
    fn bp_endorsements_from_heading_list() {
        let html = r#"<h2><span class="mw-headline" id="Endorsements">Endorsements</span></h2>
        <ul><li>Florida AFL-CIO</li><li>Brevard County Classroom Teachers Association</li></ul>
        <h2>See also</h2>"#;
        let e = endorsements_from_ballotpedia_html(html, "https://ballotpedia.org/Example");
        assert!(e.iter().any(|x| x.org.contains("AFL-CIO")));
        assert!(e.iter().any(|x| x.org.contains("Classroom")));
        assert!(e.iter().all(|x| x.trust.as_deref() == Some("reference")));
    }

    #[test]
    fn bp_endorsements_h5_and_long_li() {
        let pad = " ".repeat(480);
        let html = format!(
            r#"<h5><span class="mw-headline" id="Endorsements">Endorsements</span></h5>
            <ul class="H350">
              <li>President {pad}<a href="https://ballotpedia.org/Donald_Trump">Donald Trump</a> (R)</li>
              <li>U.S. Sen. {pad}<a href="https://ballotpedia.org/Rick_Scott">Rick Scott</a> (R)</li>
            </ul>
            <h2><span class="mw-headline" id="Notable_endorsements">Notable endorsements</span></h2>
            <table><tr><td><a href="/Chip_Roy">Chip Roy</a></td></tr></table>"#
        );
        let e = endorsements_from_ballotpedia_html(&html, "https://ballotpedia.org/Byron_Donalds");
        assert!(e.iter().any(|x| x.org.contains("Donald Trump")));
        assert!(e.iter().any(|x| x.org.contains("Rick Scott")));
        assert!(e.iter().any(|x| x.kind.as_deref() == Some("person")));
        assert!(!e.iter().any(|x| x.org.contains("Chip Roy")));
    }

    #[test]
    fn bp_endorsements_live_h350_widget() {
        let html = r#"<h5><span class="mw-headline" id="Endorsements">Endorsements</span></h5>
        <p>Byron Donalds received the following endorsements. To send us additional endorsements, <a href="https://form.jotform.com/x">click here</a>.</p>
        <ul class="H350">
          <li>
            President
            <a href="https://ballotpedia.org/Donald_Trump">Donald Trump</a>
            (R)
            <sup><a href="https://truthsocial.com/@realDonaldTrump/posts/1"><i></i></a></sup>
          </li>
          <li>
            U.S. Sen.
            <a href="https://ballotpedia.org/Rick_Scott">Rick Scott</a>
            (R)
          </li>
          <li>
            <a href="https://ballotpedia.org/Club_for_Growth">Club for Growth</a>
          </li>
        </ul>
        <div class="widget-key">Endorsements</div>
        <div class="widget-value">Veterans for America First</div>
        <h2><span class="mw-headline" id="Notable_endorsements">Notable endorsements</span></h2>
        <table><tr><td><a href="/Chip_Roy">Chip Roy</a></td></tr></table>"#;
        let e = endorsements_from_ballotpedia_html(html, "https://ballotpedia.org/Byron_Donalds");
        assert!(e.iter().any(|x| x.org.contains("Donald Trump")));
        assert!(e.iter().any(|x| x.org.contains("Rick Scott")));
        assert!(e.iter().any(|x| x.org.contains("Club for Growth")));
        assert!(e.iter().any(|x| x.org.contains("Veterans for America First")));
        assert!(!e.iter().any(|x| x.org.contains("Chip Roy")));
        assert!(!e.iter().any(|x| x.org.to_ascii_lowercase().contains("click here")));
        assert!(!e.iter().any(|x| x.org.to_ascii_lowercase().contains("received the following")));
    }

    #[test]
    fn campaign_endorsements_from_heading_cards() {
        let html = r#"<title>Endorsements</title>
        <h1>National Figures</h1>
        <h2>President Donald J. Trump</h2>
        <h2>Elon Musk</h2>
        <h2>Charlie Kirk</h2>
        <h1>Associations</h1>
        <h2>The Club for Growth</h2>
        <nav><ul><li>About</li><li>News</li><li>Donate</li></ul></nav>
        <h2>Florida State Senate Endorsements</h2>
        <ul><li>Senator Bryan Avila</li><li>Senator Debbie Mayfield</li></ul>"#;
        let e = endorsements_from_campaign_html(html, "https://byrondonalds.com/endorsements/");
        assert!(e.iter().any(|x| x.org.contains("Trump")));
        assert!(e.iter().any(|x| x.org.contains("Elon Musk")));
        assert!(e.iter().any(|x| x.org.contains("Club for Growth")));
        assert!(e.iter().any(|x| x.org.contains("Bryan Avila")));
        assert!(!e.iter().any(|x| x.org == "About" || x.org == "News"));
        assert!(!e.iter().any(|x| x.org.contains("National Figures")));
    }

    #[test]
    fn gdelt_filters_wrong_last_name() {
        let json = r#"{"articles":[
            {"title":"Jane Doe wins Brevard race","url":"https://ex.com/a","domain":"ex.com","seendate":"20260801T120000Z"},
            {"title":"Unrelated Smith story","url":"https://ex.com/b","domain":"ex.com","seendate":"20260802T120000Z"}
        ]}"#;
        let hits = news_hits_from_gdelt_json(json, "Jane Doe");
        assert_eq!(hits.len(), 1);
        assert!(hits[0].title.contains("Jane Doe"));
        assert_eq!(hits[0].date.as_deref(), Some("2026-08-01"));
    }

    #[test]
    fn google_rss_parse() {
        let xml = r#"<?xml version="1.0"?><rss><channel>
        <item><title>Timi Tucker for judge</title><link>https://news.example/t</link>
        <source>Space Coast Daily</source><pubDate>Wed, 12 Aug 2026</pubDate></item>
        </channel></rss>"#;
        let hits = news_hits_from_google_rss(xml, "Timi Tucker");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].outlet, "Space Coast Daily");
    }

    #[test]
    fn usd_parse() {
        assert_eq!(parse_usd_display("$1,234.50"), Some(1234.50));
        assert_eq!(parse_usd_display("—"), None);
    }

    #[test]
    fn bp_claims_from_campaign_themes() {
        let html = r#"<h2><span class="mw-headline" id="Campaign_themes">Campaign themes</span></h2>
        <h3>2026</h3>
        <h4>Abortion</h4>
        <blockquote>I will always defend the right of parents and doctors to make medical decisions without Tallahassee interference.</blockquote>
        <h4>Taxes</h4>
        <p>Our community cannot afford another round of property tax hikes on working families this cycle.</p>
        <h2>See also</h2>"#;
        let c = claims_from_ballotpedia_html(html, "https://ballotpedia.org/Example");
        assert!(c.iter().any(|x| x.topic.as_deref() == Some("Abortion")));
        assert!(c.iter().any(|x| x.keywords.iter().any(|k| k == "abortion")));
        assert!(c.iter().any(|x| x.kind == "promise"));
        assert!(c.iter().any(|x| x.topic.as_deref() == Some("Taxes")));
        assert!(c.iter().all(|x| x.trust == "reference"));
    }

    #[test]
    fn campaign_claims_from_issues_page() {
        let html = r#"<h1>Issues</h1>
        <h2>Public safety</h2>
        <p>I will hire more deputies and keep violent offenders in jail instead of cycling them back onto Brevard streets.</p>
        <h2>Education</h2>
        <p>Parents deserve to know what is taught in the classroom and to choose the school that fits their child.</p>"#;
        let c = claims_from_campaign_html(html, "https://janeexample.com/issues");
        assert!(c.len() >= 2);
        assert!(c.iter().any(|x| x.keywords.iter().any(|k| k == "crime")));
        assert!(c.iter().all(|x| x.trust == "campaign"));
    }

    #[test]
    fn pair_claim_to_vote_by_keyword() {
        let claims = claims_from_campaign_html(
            r#"<h2>Issues</h2><h3>Abortion</h3>
            <p>I will always defend reproductive rights and oppose a statewide abortion ban.</p>"#,
            "https://ex.com/issues",
        );
        let votes = vec![
            crate::models::VoteRecord {
                date: "2024-03-01".into(),
                question: "H.R. 7 — No Taxpayer Funding for Abortion Act".into(),
                position: "Nay".into(),
                result: Some("Passed".into()),
                url: "https://www.govtrack.us/congress/votes/1".into(),
            },
            crate::models::VoteRecord {
                date: "2024-04-01".into(),
                question: "Nomination of a U.S. Marshal".into(),
                position: "Yea".into(),
                result: Some("Confirmed".into()),
                url: "https://www.govtrack.us/congress/votes/2".into(),
            },
        ];
        let cards = pair_claims_with_votes(&claims, &votes);
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].matches.len(), 1);
        assert!(cards[0].matches[0].question.contains("Abortion"));
        assert!(cards[0].note.contains("not a finding"));
    }

    #[test]
    fn pair_empty_when_no_overlap() {
        let claims = vec![PublicClaim {
            text: "I will always defend the right of parents to review classroom materials in our district.".into(),
            topic: Some("Education".into()),
            source: "Campaign website".into(),
            source_url: None,
            trust: "campaign".into(),
            kind: "promise".into(),
            keywords: vec!["education".into()],
        }];
        let votes = vec![crate::models::VoteRecord {
            date: "2024-01-01".into(),
            question: "Nomination of Ambassador to Sweden".into(),
            position: "Yea".into(),
            result: None,
            url: "https://ex.com/v".into(),
        }];
        let cards = pair_claims_with_votes(&claims, &votes);
        assert_eq!(cards.len(), 1);
        assert!(cards[0].matches.is_empty());
    }

    #[test]
    fn llm_urls_and_models() {
        assert_eq!(llm_chat_url("xai"), Some(LLM_XAI_CHAT_URL));
        assert_eq!(llm_chat_url("OpenAI"), Some(LLM_OPENAI_CHAT_URL));
        assert!(llm_chat_url("newsapi").is_none());
        assert_eq!(llm_default_model("grok"), Some(LLM_XAI_DEFAULT_MODEL));
        assert_eq!(llm_normalize_provider("x.ai"), Some("xai"));
    }

    fn sample_contrast_card() -> ContrastCard {
        ContrastCard {
            claim_text:
                "I will always defend reproductive rights and oppose a statewide abortion ban."
                    .into(),
            topic: Some("Abortion".into()),
            claim_source: "Campaign website".into(),
            claim_url: Some("https://ex.com/issues".into()),
            claim_trust: "campaign".into(),
            claim_kind: "position".into(),
            matches: vec![ContrastMatch {
                date: "2024-03-01".into(),
                question: "H.R. 7 — No Taxpayer Funding for Abortion Act".into(),
                position: "Nay".into(),
                result: Some("Passed".into()),
                url: "https://www.govtrack.us/congress/votes/1".into(),
                overlap: vec!["abortion".into()],
            }],
            note: "Keyword overlap only.".into(),
            llm_note: None,
            llm_trust: None,
            llm_model: None,
        }
    }

    #[test]
    fn llm_request_body_includes_claim_and_rules() {
        let cards = vec![sample_contrast_card()];
        let body =
            llm_contrast_request_body("grok-4-fast", &cards, "Jane Doe", "U.S. House").unwrap();
        assert!(body.contains("reproductive rights"));
        assert!(body.contains("H.R. 7"));
        assert!(body.contains("not a finding") || body.contains("Do not call anyone bought"));
        assert!(llm_contrast_request_body("grok-4-fast", &[], "Jane", "House").is_none());
    }

    #[test]
    fn llm_parse_and_apply_chat_response() {
        let mut cards = vec![sample_contrast_card()];
        let resp = r#"{
            "choices":[{"message":{"content":"```json\n{\"notes\":[{\"i\":0,\"note\":\"The campaign page says the candidate will defend reproductive rights. A related 2024-03-01 roll-call shows a Nay on H.R. 7.\"}]}\n```"}}]
        }"#;
        let n = apply_llm_chat_response(&mut cards, resp, "grok-4-fast");
        assert_eq!(n, 1);
        assert!(cards[0].llm_note.as_ref().unwrap().contains("2024-03-01"));
        assert_eq!(cards[0].llm_trust.as_deref(), Some("inference"));
        assert_eq!(cards[0].llm_model.as_deref(), Some("grok-4-fast"));
    }

    #[test]
    fn llm_rejects_verdict_words() {
        let mut cards = vec![sample_contrast_card()];
        let notes = vec![(
            0usize,
            "This vote proves the candidate is a liar and bought.".into(),
        )];
        assert_eq!(
            apply_llm_contrast_notes(&mut cards, &notes, "gpt-4o-mini"),
            0
        );
        assert!(cards[0].llm_note.is_none());
    }

    #[test]
    fn llm_skips_card_without_matches() {
        let mut cards = vec![ContrastCard {
            claim_text: "Parents deserve to review classroom materials.".into(),
            topic: Some("Education".into()),
            claim_source: "Campaign".into(),
            claim_url: None,
            claim_trust: "campaign".into(),
            claim_kind: "position".into(),
            matches: vec![],
            note: "".into(),
            llm_note: None,
            llm_trust: None,
            llm_model: None,
        }];
        let notes = vec![(
            0usize,
            "The campaign discusses classroom materials and curriculum review.".into(),
        )];
        assert_eq!(apply_llm_contrast_notes(&mut cards, &notes, "x"), 0);
    }

    #[test]
    fn bar_search_unique_standing() {
        let html = r#"<p class="result-message">Showing 1 of 1 results.</p>
        <ul class="profiles-compact">
        <li class="profile-compact">
            <p class="profile-name"><a href="https://www.floridabar.org/directories/find-mbr/profile/?num=283495">Charles T Canady</a></p>
            <p class="profile-bar-number">Bar #283495</p>
            <div class="member-status status-good">Member in Good Standing</div>
            <div class="eligibility eligibility-eligible">Eligible to Practice Law in Florida</div>
        </li>
        </ul>"#;
        let hits = parse_fl_bar_search_html(html, "Charles Canady");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].kind, "bar_status");
        assert!(hits[0].detail.contains("283495"));
        assert!(hits[0].status.as_deref().unwrap().contains("Eligible"));
        assert_eq!(hits[0].trust, "official");
    }

    #[test]
    fn bar_search_wrong_last_name_skips() {
        let html = r#"<li class="profile-compact">
            <p class="profile-name"><a href="/directories/find-mbr/profile/?num=363901">Carlos Alberto Canet</a></p>
            <div class="eligibility eligibility-eligible">Eligible to Practice Law in Florida</div>
        </li>"#;
        assert!(parse_fl_bar_search_html(html, "Carlos Canady").is_empty());
    }

    #[test]
    fn bar_search_ambiguous_skips() {
        let html = r#"<ul>
        <li class="profile-compact"><p class="profile-name"><a href="/directories/find-mbr/profile/?num=1">Jane Q Smith</a></p></li>
        <li class="profile-compact"><p class="profile-name"><a href="/directories/find-mbr/profile/?num=2">Jane R Smith</a></p></li>
        </ul>"#;
        assert!(parse_fl_bar_search_html(html, "Jane Smith").is_empty());
    }

    #[test]
    fn bar_search_too_many_skips() {
        let html = r#"<p>Too many results. Please enter additional search parameters and try again.</p>
        <li class="profile-compact"><p class="profile-name">Charles T Canady</p></li>"#;
        assert!(parse_fl_bar_search_html(html, "Charles Canady").is_empty());
    }

    #[test]
    fn ethics_filings_unique_form6() {
        let json = r#"{
            "data":[{
                "filingId":1072177,
                "formYear":2025,
                "fullName":"Charles T. Canady",
                "formType":"Form 6",
                "submissionDate":"2026-05-27T11:59:04",
                "delimitedOrganizationNames":"Supreme Court",
                "filingHistoryUrl":"https://disclosure.floridaethics.gov/PublicSearch/FilingHistory/66806"
            }],
            "totalRowCount":1
        }"#;
        let hits = parse_fl_ethics_filings_json(json, "Charles Canady");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].kind, "ethics_filing");
        assert!(hits[0].detail.contains("Form 6"));
        assert!(hits[0].detail.contains("Supreme Court"));
        assert_eq!(hits[0].date.as_deref(), Some("2026-05-27"));
    }

    #[test]
    fn ethics_filings_wrong_person_skips() {
        let json = r#"{"data":[{"fullName":"Charles Smith","formType":"Form 1","formYear":2025}],"totalRowCount":1}"#;
        assert!(parse_fl_ethics_filings_json(json, "Charles Canady").is_empty());
    }

    #[test]
    fn ethics_orders_unique_row() {
        let html = r#"<table class="ordersTable">
        <tr><td><h3>Complaint Number</h3></td><td class="secondColumn"><h3>Name</h3></td></tr>
        <tr>
            <td>20-060, 20-073, 20-103 (cons.)</td>
            <td>Douglas Underhill</td>
            <td><a href="/Documents/Orders/2020/20-060fo.pdf">Final Order</a></td>
        </tr>
        </table>"#;
        let hits = parse_fl_ethics_orders_html(html, "Douglas Underhill");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].kind, "ethics_order");
        assert!(hits[0].detail.contains("20-060"));
        assert!(hits[0].url.contains("20-060fo.pdf"));
        assert!(parse_fl_ethics_orders_html(html, "Jane Doe").is_empty());
    }

    #[test]
    fn jqc_posts_require_first_and_last() {
        let json = r#"[{
            "title":{"rendered":"JUDGE ERNEST KOLLRA DISCIPLINED FOR PARTISAN CAMPAIGN CONDUCT"},
            "excerpt":{"rendered":"<p>Judge Ernest Kollra of the 17th Circuit</p>"},
            "link":"https://floridajqc.com/judge-ernest-kollra-disciplined-for-partisan-campaign-conduct/",
            "date":"2019-09-20T00:00:00"
        },{
            "title":{"rendered":"UNRELATED HALLWAY REPRIMAND"},
            "excerpt":{"rendered":"<p>Someone else</p>"},
            "link":"https://floridajqc.com/other/",
            "date":"2021-01-15T00:00:00"
        }]"#;
        let hits = parse_fl_jqc_posts_json(json, "Ernest Kollra");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].kind, "jqc_notice");
        assert_eq!(hits[0].date.as_deref(), Some("2019-09-20"));
        assert!(parse_fl_jqc_posts_json(json, "David Miller").is_empty());
    }

    #[test]
    fn jqc_news_html_heading() {
        let html = r#"<h3><a href="https://floridajqc.com/judge-ernest-kollra-disciplined-for-partisan-campaign-conduct/">JUDGE ERNEST KOLLRA DISCIPLINED FOR PARTISAN CAMPAIGN CONDUCT</a></h3>
        <p>TALLAHASSEE- Judge Ernest Kollra of the 17th Judicial Circuit was publicly reprimanded.</p>"#;
        let hits = parse_fl_jqc_news_html(html, "Ernest Kollra");
        assert_eq!(hits.len(), 1);
        assert!(hits[0].url.contains("kollra"));
    }

    #[test]
    fn bar_search_url_uses_fname_lname() {
        let u = fl_bar_search_url("Charles T. Canady").unwrap();
        assert!(u.contains("fName=charles"));
        assert!(u.contains("lName=canady"));
        assert!(u.contains("sdx=N"));
    }

    const FL_2026_MEASURES_INDEX: &str = r#"
        <h2><span class="mw-headline" id="On_the_ballot">On the ballot</span></h2>
        <table>
        <tr><td>LRCA</td>
        <td><a href="/Florida_Amendment_3,_Homestead_Tax_Exemptions,_Property_Assessments,_and_Spending_Restrictions_Amendment_(2026)" title="Amendment 3">Amendment 3</a></td>
        <td><a href="/Homestead_tax_exemptions_on_the_ballot">Homestead tax</a></td>
        <td>Increase the homestead tax exemption</td></tr>
        <tr><td>LRCA</td>
        <td><a href="/Florida_Changes_to_Budget_Stabilization_Fund_Amendment_(2026)">Changes to Budget Stabilization Fund Amendment</a></td>
        <td>Budget</td><td>Increase the budget stabilization fund cap</td></tr>
        <tr><td>LRCA</td>
        <td><a href="/Florida_Exempt_Tangible_Personal_Property_Used_for_Agriculture_or_Agritourism_from_Property_Taxes_Amendment_(2026)">Exempt Tangible Personal Property Used for Agriculture or Agritourism from Property Taxes Amendment</a></td>
        <td>Agriculture</td><td>Exempt farm equipment</td></tr>
        </table>
        <h2><span class="mw-headline" id="Getting_measures_on_the_ballot">Getting measures on the ballot</span></h2>
        <h2><span class="mw-headline" id="Not_on_the_ballot">Not on the ballot</span></h2>
        <p><a href="/Florida_Marijuana_Legalization_Initiative_(2026)">Marijuana Legalization Initiative</a></p>
    "#;

    #[test]
    fn bp_state_measures_url_florida_2026() {
        assert_eq!(
            ballotpedia_state_measures_url("FL", 2026).as_deref(),
            Some("https://ballotpedia.org/Florida_2026_ballot_measures")
        );
        assert!(ballotpedia_state_measures_url("XX", 2026).is_none());
    }

    #[test]
    fn bp_index_on_the_ballot_skips_failed_initiatives() {
        let links = ballotpedia_measure_links_from_index(FL_2026_MEASURES_INDEX);
        assert_eq!(links.len(), 3);
        assert!(links.iter().any(|l| l.amendment == Some(3)));
        assert!(links
            .iter()
            .any(|l| l.slug_title.to_ascii_lowercase().contains("budget")));
        assert!(!links.iter().any(|l| l.url.to_ascii_lowercase().contains("marijuana")));
        assert!(!links
            .iter()
            .any(|l| l.url.contains("Homestead_tax_exemptions_on_the_ballot")));
    }

    #[test]
    fn bp_match_nov_amendments_unique() {
        let links = ballotpedia_measure_links_from_index(FL_2026_MEASURES_INDEX);
        let a3 = match_ballotpedia_measure(
            &links,
            "SAVE OUR HOMES FROM EXCESSIVE PROPERTY TAXES",
            Some("Amendment 3"),
        )
        .expect("homestead");
        assert_eq!(a3.amendment, Some(3));
        assert!(a3.url.contains("Homestead"));

        let budget = match_ballotpedia_measure(&links, "Budget Stabilization Fund", Some("Amendment 1"))
            .expect("budget");
        assert!(budget.url.contains("Budget_Stabilization"));

        let ag = match_ballotpedia_measure(
            &links,
            "Exemption of Tangible Personal Property on Agricultural Land from Taxation",
            Some("Amendment 2"),
        )
        .expect("ag");
        assert!(ag.url.contains("Agriculture"));
    }

    #[test]
    fn bp_match_ambiguous_skips() {
        let links = vec![
            BallotpediaMeasureLink {
                title: "Amendment 3".into(),
                url: "https://ballotpedia.org/Florida_Amendment_3,_Homestead_Tax_A_(2026)".into(),
                slug_title: "Florida Amendment 3 Homestead Tax".into(),
                amendment: Some(3),
            },
            BallotpediaMeasureLink {
                title: "Amendment 3".into(),
                url: "https://ballotpedia.org/Florida_Amendment_3,_Homestead_Tax_B_(2026)".into(),
                slug_title: "Florida Amendment 3 Homestead Tax".into(),
                amendment: Some(3),
            },
        ];
        assert!(match_ballotpedia_measure(&links, "Homestead Tax", Some("Amendment 3")).is_none());
    }

    #[test]
    fn bp_measure_support_oppose_skips_arguments() {
        let html = r#"<h1>Florida Amendment 3, Homestead Tax Exemptions Amendment (2026)</h1>
        <h2><span class="mw-headline" id="Support">Support</span></h2>
        <h3>Supporters</h3>
        <h4>Officials</h4>
        <ul>
          <li>Sen. <a href="/Rick_Scott">Rick Scott</a> (R)</li>
          <li>Gov. <a href="/Ron_DeSantis">Ron DeSantis</a> (R)</li>
          <li>State Chief Financial Officer <a href="/Blaise_Ingoglia">Blaise Ingoglia</a> (R)</li>
        </ul>
        <h3>Arguments</h3>
        <ul><li><b>Chief Financial Officer Blaise Ingoglia (R):</b> "The only people who are complaining about it are the people who actually have to cut back."</li></ul>
        <h2><span class="mw-headline" id="Opposition">Opposition</span></h2>
        <h3>Opponents</h3>
        <ul>
          <li>State Sen. <a>Lori Berman</a> (D)</li>
          <li>Mayor <a>Donna Deegan</a> (D)</li>
        </ul>
        <h3>Arguments</h3>
        <ul><li>This proposed reduction will inevitably result in roads deteriorating and libraries closing across the state.</li></ul>
        <h2>Campaign finance</h2>"#;
        let page = "https://ballotpedia.org/Florida_Amendment_3,_Homestead_(2026)";
        let e = endorsements_from_ballotpedia_measure_html(html, page);
        assert!(e.iter().any(|x| x.org.contains("Rick Scott") && x.stance == "support"));
        assert!(e.iter().any(|x| x.org.contains("Ron DeSantis") && x.kind.as_deref() == Some("official")));
        assert!(e.iter().any(|x| x.org.contains("Ingoglia") && x.kind.as_deref() == Some("official")));
        assert!(e.iter().any(|x| x.org.contains("Lori Berman") && x.stance == "oppose"));
        assert!(e.iter().any(|x| x.org.contains("Donna Deegan")));
        assert!(!e.iter().any(|x| x.org.contains("complaining") || x.org.contains("deteriorating")));
        assert!(e.iter().all(|x| x.trust.as_deref() == Some("reference")));
        assert!(ballotpedia_html_matches_measure(
            html,
            "SAVE OUR HOMES FROM EXCESSIVE PROPERTY TAXES",
            Some("Amendment 3")
        ));
    }

    #[test]
    fn merge_endorsement_lists_dedupes() {
        let a = vec![Endorsement {
            org: "Vote No on 3".into(),
            stance: "oppose".into(),
            source: "FL TreFin".into(),
            source_url: None,
            kind: Some("committee".into()),
            trust: Some("filing".into()),
            date: None,
        }];
        let b = vec![Endorsement {
            org: "Vote No on 3".into(),
            stance: "oppose".into(),
            source: "Ballotpedia".into(),
            source_url: None,
            kind: Some("org".into()),
            trust: Some("reference".into()),
            date: None,
        }];
        let m = merge_endorsement_lists(&a, &b);
        assert_eq!(m.len(), 1);
    }
}
