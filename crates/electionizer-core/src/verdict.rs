//! Track M — AI verdict card. Pack filings, build Responses-API prompts,
//! parse/validate scored cards. JS owns HTTP.

use crate::bio::Endorsement;
use crate::models::party_bucket;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

pub const VERDICT_XAI_RESPONSES_URL: &str = "https://api.x.ai/v1/responses";
pub const VERDICT_OPENAI_RESPONSES_URL: &str = "https://api.openai.com/v1/responses";
pub const VERDICT_XAI_DEFAULT_MODEL: &str = "grok-4.6";
pub const VERDICT_XAI_SEARCH_MODEL: &str = "grok-4.6";
pub const VERDICT_OPENAI_DEFAULT_MODEL: &str = "gpt-4o-mini";

const MAX_VOTES: usize = 40;
const MAX_NEWS: usize = 12;
const MAX_DONORS: usize = 15;
const MAX_FACTS: usize = 16;
const MAX_CLAIMS: usize = 16;
const MAX_HEADLINE: usize = 160;
const MAX_VERDICT_LINE: usize = 280;
const MAX_SUMMARY: usize = 8;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AxisDef {
    pub id: String,
    pub label: String,
    pub definition: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VerdictCite {
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tab: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trust: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CitedSentence {
    pub text: String,
    #[serde(default)]
    pub cites: Vec<VerdictCite>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AxisScore {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verdict: Option<String>,
    #[serde(default)]
    pub evidence: Vec<CitedSentence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_score: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inverted: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FoundItem {
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stance: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trust: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TabCite {
    pub tab: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OverallScore {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verdict: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_score: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profiled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VerdictCard {
    pub kind: String,
    #[serde(default)]
    pub headline: String,
    #[serde(default)]
    pub summary: Vec<CitedSentence>,
    #[serde(default)]
    pub overall: OverallScore,
    #[serde(default)]
    pub axes: Vec<AxisScore>,
    #[serde(default)]
    pub found: Vec<FoundItem>,
    #[serde(default)]
    pub tab_cites: Vec<TabCite>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packed_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackedEndorsement {
    pub id: String,
    pub org: String,
    pub stance: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trust: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackedNews {
    pub id: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outlet: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackedClaim {
    pub id: String,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackedVote {
    pub id: String,
    pub date: String,
    pub question: String,
    pub position: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackedMoney {
    pub id: String,
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackedDonor {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackedFact {
    pub id: String,
    pub kind: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackedFinance {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipts: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackedContext {
    pub kind: String,
    pub name: String,
    pub office: String,
    pub party: String,
    pub party_bucket: String,
    pub is_judge: bool,
    pub is_incumbent: bool,
    pub jurisdiction: String,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub rubric: Vec<AxisDef>,
    #[serde(default)]
    pub endorsements: Vec<PackedEndorsement>,
    #[serde(default)]
    pub news: Vec<PackedNews>,
    #[serde(default)]
    pub claims: Vec<PackedClaim>,
    #[serde(default)]
    pub votes: Vec<PackedVote>,
    #[serde(default)]
    pub money: Vec<PackedMoney>,
    #[serde(default)]
    pub donors: Vec<PackedDonor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finance: Option<PackedFinance>,
    #[serde(default)]
    pub facts: Vec<PackedFact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub measure_code: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub voter_profile: Vec<VoterPref>,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct VoterPref {
    pub id: String,
    pub likert: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileAxis {
    pub id: String,
    pub label: String,
    pub definition: String,
    pub group: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub low_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub high_label: Option<String>,
    #[serde(default)]
    pub signed: bool,
}

const REPUBLICAN_AXES: &[(&str, &str, &str)] = &[
    ("maga", "MAGA", "Make America Great Again movement: America-first rhetoric, Trump loyalty, anti-establishment posture, border/immigration hard line."),
    ("america_first", "America First", "Non-interventionist or nationalist foreign policy, tariffs, reduced alliances, domestic-industry priority."),
    ("trump", "Trump", "Personal/political alignment with Donald Trump: endorsements, staff, votes to shield or advance his agenda."),
    ("neocon", "NeoCon", "Interventionist foreign policy, democracy-promotion wars, strong NATO/Ukraine aid, Bush/Cheney lineage."),
    ("zionist", "Zionist", "Pro-Israel policy alignment (AIPAC/CUFI, aid votes, statements). Not ethnicity or religion."),
    ("bush_era", "Bush-era", "2001–2009 GOP: compassionate conservatism, Iraq/Afghanistan, No Child Left Behind, Medicare Part D."),
    ("reagan_era", "Reagan-era", "Fusionist conservatism: tax cuts, anti-communism, fusion of evangelicals and free markets."),
    ("nixon_era", "Nixon-era", "Law-and-order, silent-majority, EPA-era mixed government, realpolitik foreign policy."),
    ("tea_party", "Tea Party", "2009–2014 Tea Party: spending/debt opposition, TARP backlash, town halls, House Freedom Caucus lineage."),
];

const DEMOCRAT_AXES: &[(&str, &str, &str)] = &[
    ("communism", "Communism", "Explicit communist program, party membership, or nationalization of major industries. Not 'voted for a tax'."),
    ("socialism", "Socialism", "Democratic-socialist program: DSA, public ownership, Medicare for All as single-payer, wealth cap."),
    ("green_new_deal", "Green New Deal", "GND-style climate industrial policy, fossil phase-out, job guarantee tied to decarbonization."),
    ("aoc", "AOC", "Squad / AOC-aligned: justice Democrats, abolish-ICE adjacent, progressive insurgency vs party leadership."),
    ("classical_liberal", "Classical liberal", "Free speech, markets, limited government, civil liberties; often at odds with modern progressive identitarianism."),
    ("clinton_era", "Clinton-era", "1990s New Democrat: NAFTA, welfare reform, triangulation, tough-on-crime, Wall Street friendly."),
    ("jfk_era", "JFK-era", "Cold War liberal: tax cuts + anti-communism + civil rights beginnings, New Frontier growth."),
    ("lbj_era", "LBJ-era", "Great Society: Medicare/Medicaid, Voting Rights, War on Poverty, expansive federal programs."),
    ("obama_era", "Obama-era", "2009–2017: ACA, DACA, stimulus, Iran deal, coalition foreign policy, identity-inclusive rhetoric."),
    ("fdr_era", "FDR-era", "1933–1945 New Deal: Social Security, public works, wartime industrial policy, four-term Democratic coalition."),
    ("occupy", "Occupy Wall Street", "Occupy Wall Street: 99%, anti-bank, debt/foreclosure, camp protests; precursor to left populism."),
];

const INDEPENDENT_AXES: &[(&str, &str, &str)] = &[
    ("left_lean", "Left-leaning", "Progressive or social-democratic policy pattern without a major-party label."),
    ("right_lean", "Right-leaning", "Conservative or nationalist policy pattern without a major-party label."),
    ("libertarian", "Libertarian", "Small government, civil liberties, non-intervention, drug/speech decriminalization."),
    ("populist", "Populist", "Anti-elite, anti-institution rhetoric from either left or right."),
    ("ron_paul", "Ron Paul movement", "Ron Paul movement: audit the Fed, non-intervention, hard money, 2008/2012 GOP insurgency."),
];

const ISSUE_AXES: &[(&str, &str, &str)] = &[
    ("lgbtq", "LGBTQ", "Policy record on LGBTQ rights (marriage, sports, youth medicine, religious liberty). Not the subject's orientation."),
    ("cannabis", "Cannabis", "Legalization, decriminalization, or medical-cannabis expansion vs prohibition."),
    ("voter_id", "Voter ID", "Photo ID or proof-of-citizenship requirements to vote."),
    ("medical_freedom", "Medical freedom", "Oppose vaccine/mask mandates; support off-label use, informed consent, medical-choice framing."),
    ("remigration", "Remigration", "Support for returning illegal or recently arrived migrants to origin countries; deportation-first policy."),
    ("abortion", "Abortion", "-100 reduce abortions (limits, funding bans); +100 increase access (codify Roe, public funding)."),
    ("health_insurance", "Health insurance", "-100 reduce fraud/waste and keep private coverage; +100 single-payer / Medicare for All."),
    ("h1b", "H-1B visas", "-100 cut H-1B/guest-worker visas; +100 expand H-1B and skilled immigration."),
    ("border", "Border", "-100 open/expansionist immigration; +100 closed border, remain-in-Mexico, interior enforcement."),
];

const JUDGE_AXES: &[(&str, &str, &str)] = &[
    ("left_lean", "Left-leaning", "Pattern of outcomes, clerks, orgs, or statements associated with the legal left."),
    ("right_lean", "Right-leaning", "Pattern of outcomes, clerks, orgs, or statements associated with the legal right."),
    ("originalism", "Originalism", "Text/history as binding; cites original public meaning; rejects living-constitution."),
    ("living_constitution", "Living constitution", "Constitution evolves with contemporary values; purposivism; opposite of originalism."),
    ("party_line", "Party-line", "Rulings that track the appointing party's preferred outcome on politicized questions."),
    ("tds", "Trump-derangement", "Trump treated unlike similarly situated parties. Requires a compared case or quote."),
    ("constitutional_applicability", "Constitutional applicability", "Willingness to apply constitutional text as a limit on statute, agency, or emergency power."),
    ("gun_rights", "Gun rights", "2A expansion vs restriction in rulings, amicus, or public statements."),
    ("adl_aligned", "ADL-aligned", "Support, citation, or partnership with ADL or similar groups in rulings or public life."),
];

const MEASURE_AXES: &[(&str, &str, &str)] = &[
    ("tax_direction", "Tax direction", "-100 cuts taxes; 0 shift/unclear; +100 raises taxes. Net fiscal effect on typical voter."),
    ("restriction_direction", "Restriction direction", "-100 removes law/limits government; +100 adds restrictions or new crimes/mandates."),
    ("constitutional_tension", "Constitutional tension", "0 fits existing constitution/statute; 100 likely conflicts or rewrites a constitutional rule."),
    ("incumbent_class_benefit", "Incumbent/class benefit", "How much the measure advantages sitting officials, a donor class, or a narrow industry."),
];

/// Flat voter-profile catalog — all rubric axes, no party split.
/// (id, label, definition, group, low_label, high_label)
const PROFILE_AXES: &[(&str, &str, &str, &str, &str, &str)] = &[
    ("maga", "MAGA", "Make America Great Again movement: America-first rhetoric, Trump loyalty, anti-establishment posture, border/immigration hard line.", "Movements", "Disagree", "Agree"),
    ("america_first", "America First", "Non-interventionist or nationalist foreign policy, tariffs, reduced alliances, domestic-industry priority.", "Movements", "Disagree", "Agree"),
    ("trump", "Trump", "Personal/political alignment with Donald Trump: endorsements, staff, votes to shield or advance his agenda.", "Movements", "Disagree", "Agree"),
    ("tea_party", "Tea Party", "2009–2014 Tea Party: spending/debt opposition, TARP backlash, town halls, House Freedom Caucus lineage.", "Movements", "Disagree", "Agree"),
    ("ron_paul", "Ron Paul movement", "Ron Paul movement: audit the Fed, non-intervention, hard money, 2008/2012 GOP insurgency.", "Movements", "Disagree", "Agree"),
    ("occupy", "Occupy Wall Street", "Occupy Wall Street: 99%, anti-bank, debt/foreclosure, camp protests; precursor to left populism.", "Movements", "Disagree", "Agree"),
    ("aoc", "AOC", "Squad / AOC-aligned: justice Democrats, abolish-ICE adjacent, progressive insurgency vs party leadership.", "Movements", "Disagree", "Agree"),
    ("populist", "Populist", "Anti-elite, anti-institution rhetoric from either left or right.", "Movements", "Disagree", "Agree"),
    ("libertarian", "Libertarian", "Small government, civil liberties, non-intervention, drug/speech decriminalization.", "Movements", "Disagree", "Agree"),
    ("communism", "Communism", "Explicit communist program, party membership, or nationalization of major industries. Not 'voted for a tax'.", "Programs", "Disagree", "Agree"),
    ("socialism", "Socialism", "Democratic-socialist program: DSA, public ownership, Medicare for All as single-payer, wealth cap.", "Programs", "Disagree", "Agree"),
    ("green_new_deal", "Green New Deal", "GND-style climate industrial policy, fossil phase-out, job guarantee tied to decarbonization.", "Programs", "Disagree", "Agree"),
    ("classical_liberal", "Classical liberal", "Free speech, markets, limited government, civil liberties; often at odds with modern progressive identitarianism.", "Programs", "Disagree", "Agree"),
    ("neocon", "NeoCon", "Interventionist foreign policy, democracy-promotion wars, strong NATO/Ukraine aid, Bush/Cheney lineage.", "Programs", "Disagree", "Agree"),
    ("zionist", "Zionist", "Pro-Israel policy alignment (AIPAC/CUFI, aid votes, statements). Not ethnicity or religion.", "Programs", "Disagree", "Agree"),
    ("fdr_era", "FDR-era", "1933–1945 New Deal: Social Security, public works, wartime industrial policy, four-term Democratic coalition.", "Eras", "Disagree", "Agree"),
    ("jfk_era", "JFK-era", "Cold War liberal: tax cuts + anti-communism + civil rights beginnings, New Frontier growth.", "Eras", "Disagree", "Agree"),
    ("lbj_era", "LBJ-era", "Great Society: Medicare/Medicaid, Voting Rights, War on Poverty, expansive federal programs.", "Eras", "Disagree", "Agree"),
    ("nixon_era", "Nixon-era", "Law-and-order, silent-majority, EPA-era mixed government, realpolitik foreign policy.", "Eras", "Disagree", "Agree"),
    ("reagan_era", "Reagan-era", "Fusionist conservatism: tax cuts, anti-communism, fusion of evangelicals and free markets.", "Eras", "Disagree", "Agree"),
    ("clinton_era", "Clinton-era", "1990s New Democrat: NAFTA, welfare reform, triangulation, tough-on-crime, Wall Street friendly.", "Eras", "Disagree", "Agree"),
    ("bush_era", "Bush-era", "2001–2009 GOP: compassionate conservatism, Iraq/Afghanistan, No Child Left Behind, Medicare Part D.", "Eras", "Disagree", "Agree"),
    ("obama_era", "Obama-era", "2009–2017: ACA, DACA, stimulus, Iran deal, coalition foreign policy, identity-inclusive rhetoric.", "Eras", "Disagree", "Agree"),
    ("left_lean", "Left-leaning", "Progressive or social-democratic policy pattern (or legal-left pattern for judges).", "Lean", "Disagree", "Agree"),
    ("right_lean", "Right-leaning", "Conservative or nationalist policy pattern (or legal-right pattern for judges).", "Lean", "Disagree", "Agree"),
    ("originalism", "Originalism", "Text/history as binding; cites original public meaning; rejects living-constitution.", "Courts", "Disagree", "Agree"),
    ("living_constitution", "Living constitution", "Constitution evolves with contemporary values; purposivism; opposite of originalism.", "Courts", "Disagree", "Agree"),
    ("party_line", "Party-line", "Rulings that track the appointing party's preferred outcome on politicized questions.", "Courts", "Disagree", "Agree"),
    ("tds", "Trump-derangement", "Trump treated unlike similarly situated parties. Requires a compared case or quote.", "Courts", "Disagree", "Agree"),
    ("constitutional_applicability", "Constitutional applicability", "Willingness to apply constitutional text as a limit on statute, agency, or emergency power.", "Courts", "Disagree", "Agree"),
    ("gun_rights", "Gun rights", "2A expansion vs restriction in rulings, amicus, or public statements.", "Courts", "Disagree", "Agree"),
    ("adl_aligned", "ADL-aligned", "Support, citation, or partnership with ADL or similar groups in rulings or public life.", "Courts", "Disagree", "Agree"),
    ("lgbtq", "LGBTQ", "Policy record on LGBTQ rights (marriage, sports, youth medicine, religious liberty). Not the subject's orientation.", "Issues", "Disagree", "Agree"),
    ("cannabis", "Cannabis", "Legalization, decriminalization, or medical-cannabis expansion vs prohibition.", "Issues", "Disagree", "Agree"),
    ("voter_id", "Voter ID", "Photo ID or proof-of-citizenship requirements to vote.", "Issues", "Disagree", "Agree"),
    ("medical_freedom", "Medical freedom", "Oppose vaccine/mask mandates; support off-label use, informed consent, medical-choice framing.", "Issues", "Disagree", "Agree"),
    ("remigration", "Remigration", "Support for returning illegal or recently arrived migrants to origin countries; deportation-first policy.", "Issues", "Disagree", "Agree"),
    ("abortion", "Abortion", "Reduce abortions vs increase access.", "Issues", "Reduce", "Access"),
    ("health_insurance", "Health insurance", "Reduce fraud/waste vs single-payer.", "Issues", "Cut fraud", "Single-payer"),
    ("h1b", "H-1B visas", "Cut vs expand H-1B and skilled guest-worker visas.", "Issues", "Reduce", "Increase"),
    ("border", "Border", "Open immigration vs closed border.", "Issues", "Open", "Closed"),
    ("tax_direction", "Tax direction", "Net fiscal effect. 1 = want cuts; 5 = want raises.", "Measures", "Cuts", "Raises"),
    ("restriction_direction", "Restriction direction", "1 = want fewer laws/limits; 5 = want more restrictions or mandates.", "Measures", "Fewer limits", "More limits"),
    ("constitutional_tension", "Constitutional tension", "How much you accept a measure that conflicts with or rewrites constitutional rules.", "Measures", "Oppose", "Favor"),
    ("incumbent_class_benefit", "Incumbent/class benefit", "How much you accept a measure that advantages sitting officials, donors, or a narrow industry.", "Measures", "Oppose", "Favor"),
];

fn prefs_from_value(v: Option<&Value>) -> Vec<VoterPref> {
    let Some(v) = v else {
        return Vec::new();
    };
    let mut out: Vec<VoterPref> = parse_voter_profile(&v.to_string())
        .into_iter()
        .map(|(id, likert)| VoterPref { id, likert })
        .collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

fn owned_axes(rows: &[(&str, &str, &str)]) -> Vec<AxisDef> {
    rows.iter()
        .map(|(id, label, def)| AxisDef {
            id: (*id).into(),
            label: (*label).into(),
            definition: (*def).into(),
        })
        .collect()
}

pub fn subject_kind(kind: &str, is_judge: bool) -> &'static str {
    let k = kind.trim().to_ascii_lowercase();
    if k == "measure" || k == "amendment" || k == "question" {
        return "measure";
    }
    if is_judge || k == "judge" || k == "judicial" {
        return "judge";
    }
    "candidate"
}

pub fn rubric_for(kind: &str, party: &str, is_judge: bool) -> Vec<AxisDef> {
    match subject_kind(kind, is_judge) {
        "measure" => owned_axes(MEASURE_AXES),
        "judge" => owned_axes(JUDGE_AXES),
        _ => {
            let mut axes = match party_bucket(party) {
                "republican" => owned_axes(REPUBLICAN_AXES),
                "democrat" => owned_axes(DEMOCRAT_AXES),
                _ => owned_axes(INDEPENDENT_AXES),
            };
            axes.extend(owned_axes(ISSUE_AXES));
            axes
        }
    }
}

pub fn voter_profile_axes() -> Vec<ProfileAxis> {
    PROFILE_AXES
        .iter()
        .map(|(id, label, def, group, low, high)| ProfileAxis {
            id: (*id).into(),
            label: (*label).into(),
            definition: (*def).into(),
            group: (*group).into(),
            low_label: Some((*low).into()),
            high_label: Some((*high).into()),
            signed: is_signed_axis(id),
        })
        .collect()
}

const SIGNED_AXES: &[&str] = &[
    "tax_direction",
    "restriction_direction",
    "abortion",
    "health_insurance",
    "h1b",
    "border",
];

pub fn is_signed_axis(id: &str) -> bool {
    SIGNED_AXES.contains(&id)
}

pub fn parse_voter_profile(profile_json: &str) -> HashMap<String, u8> {
    let v: Value = serde_json::from_str(profile_json).unwrap_or_else(|_| json!({}));
    let mut map = HashMap::new();
    let obj = match v.as_object() {
        Some(o) => o,
        None => return map,
    };
    for (k, val) in obj {
        let n = if let Some(u) = val.as_u64() {
            u
        } else if let Some(i) = val.as_i64() {
            i.max(0) as u64
        } else if let Some(s) = val.as_str() {
            s.parse().unwrap_or(0)
        } else {
            0
        };
        let id = k.trim().to_ascii_lowercase();
        if (1..=5).contains(&n) && !id.is_empty() {
            map.insert(id, n as u8);
        }
    }
    map
}

pub fn likert_weight(likert: u8) -> u8 {
    likert.abs_diff(3)
}

/// Remap one alignment score into a 0–100 voter-fit.
/// Likert 1–2 invert 0–100 axes (dislike 80 → fit 20). Neutral (3) skipped.
/// Signed measure axes: 1 wants −100, 5 wants +100; closer is better.
pub fn axis_fit(id: &str, score: i32, likert: u8) -> Option<(i32, u8)> {
    if !(1..=5).contains(&likert) {
        return None;
    }
    let weight = likert_weight(likert);
    if weight == 0 {
        return None;
    }
    let fit = if is_signed_axis(id) {
        let desired = (i32::from(likert) - 3) * 50;
        let dist = (score - desired).unsigned_abs() as i32;
        (100 - dist / 2).clamp(0, 100)
    } else if likert <= 2 {
        (100 - score).clamp(0, 100)
    } else {
        score.clamp(0, 100)
    };
    Some((fit, weight))
}

pub fn fit_label(n: i32) -> &'static str {
    match n {
        ..=24 => "Poor fit",
        25..=49 => "Weak fit",
        50..=64 => "Mixed fit",
        65..=79 => "Good fit",
        _ => "Strong fit",
    }
}

/// Remap axis scores + overall to voter-fit. Alignment stays in `raw_score`.
pub fn apply_voter_profile(card: &VerdictCard, profile_json: &str) -> VerdictCard {
    let profile = parse_voter_profile(profile_json);
    let mut out = card.clone();
    if profile.is_empty() {
        return out;
    }
    let mut wsum: u32 = 0;
    let mut fsum: i32 = 0;
    for axis in &mut out.axes {
        let raw = axis.score;
        if axis.raw_score.is_none() {
            axis.raw_score = raw;
        }
        let Some(score) = raw else { continue };
        let Some(&likert) = profile.get(&axis.id) else {
            continue;
        };
        let Some((fit, w)) = axis_fit(&axis.id, score, likert) else {
            continue;
        };
        axis.inverted = Some(likert <= 2 && !is_signed_axis(&axis.id));
        axis.score = Some(fit);
        wsum += u32::from(w);
        fsum += fit * i32::from(w);
    }
    if wsum == 0 {
        return out;
    }
    let overall_fit = ((f64::from(fsum) / f64::from(wsum)).round() as i32).clamp(0, 100);
    if out.overall.raw_score.is_none() {
        out.overall.raw_score = out.overall.score;
    }
    out.overall.score = Some(overall_fit);
    out.overall.profiled = Some(true);
    out.overall.label = Some(fit_label(overall_fit).to_string());
    out
}

pub fn verdict_normalize_provider(provider: &str) -> Option<&'static str> {
    match provider.trim().to_ascii_lowercase().as_str() {
        "xai" | "grok" | "x.ai" => Some("xai"),
        "openai" | "chatgpt" => Some("openai"),
        _ => None,
    }
}

pub fn verdict_responses_url(provider: &str) -> Option<&'static str> {
    match verdict_normalize_provider(provider)? {
        "xai" => Some(VERDICT_XAI_RESPONSES_URL),
        "openai" => Some(VERDICT_OPENAI_RESPONSES_URL),
        _ => None,
    }
}

pub fn verdict_default_model(provider: &str) -> Option<&'static str> {
    match verdict_normalize_provider(provider)? {
        "xai" => Some(VERDICT_XAI_DEFAULT_MODEL),
        "openai" => Some(VERDICT_OPENAI_DEFAULT_MODEL),
        _ => None,
    }
}

fn js_str(v: &Value, keys: &[&str]) -> String {
    for k in keys {
        if let Some(s) = v.get(*k).and_then(|x| x.as_str()) {
            let t = collapse_ws(s);
            if !t.is_empty() {
                return t;
            }
        }
    }
    String::new()
}

fn js_bool(v: &Value, keys: &[&str]) -> bool {
    for k in keys {
        match v.get(*k) {
            Some(Value::Bool(b)) => return *b,
            Some(Value::String(s)) => {
                let l = s.trim().to_ascii_lowercase();
                if l == "true" || l == "1" || l == "yes" {
                    return true;
                }
            }
            Some(Value::Number(n)) => {
                if n.as_i64() == Some(1) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

fn as_array(v: Option<&Value>) -> Vec<Value> {
    match v {
        Some(Value::Array(a)) => a.clone(),
        Some(Value::Object(map)) => map
            .values()
            .filter(|x| x.is_object())
            .cloned()
            .collect(),
        _ => Vec::new(),
    }
}

fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_chars(s: &str, max: usize) -> String {
    let t = collapse_ws(s);
    if t.chars().count() <= max {
        return t;
    }
    let mut out: String = t.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

fn fingerprint(s: &str) -> String {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    format!("{:016x}", h.finish())
}

fn is_http_url(s: &str) -> bool {
    let t = s.trim();
    t.starts_with("https://") || t.starts_with("http://")
}

fn take_url(v: &Value, keys: &[&str]) -> Option<String> {
    let s = js_str(v, keys);
    if is_http_url(&s) {
        Some(s)
    } else {
        None
    }
}

/// Pack a subject + enrich blob into a compact model context.
pub fn pack_verdict_context(subject_json: &str, enrich_json: &str) -> Option<PackedContext> {
    let subject: Value = serde_json::from_str(subject_json).ok()?;
    let enrich: Value = if enrich_json.trim().is_empty() {
        json!({})
    } else {
        serde_json::from_str(enrich_json).unwrap_or_else(|_| json!({}))
    };

    let is_judge = js_bool(&subject, &["is_judge"])
        || js_str(&subject, &["chamber"]).eq_ignore_ascii_case("judicial");
    let kind_hint = js_str(&subject, &["kind"]);
    let kind = subject_kind(&kind_hint, is_judge).to_string();
    let party = js_str(&subject, &["party"]);
    let name = js_str(&subject, &["name", "title"]);
    if name.is_empty() {
        return None;
    }

    let mut ctx = PackedContext {
        kind: kind.clone(),
        name,
        office: js_str(&subject, &["office"]),
        party: party.clone(),
        party_bucket: party_bucket(&party).to_string(),
        is_judge: kind == "judge",
        is_incumbent: js_bool(&subject, &["is_incumbent"]),
        jurisdiction: js_str(&subject, &["jurisdiction"]),
        state: js_str(&subject, &["state_code", "state"]).to_ascii_uppercase(),
        rubric: rubric_for(&kind, &party, kind == "judge"),
        summary: {
            let s = js_str(&subject, &["summary"]);
            if s.is_empty() {
                None
            } else {
                Some(truncate_chars(&s, 400))
            }
        },
        source_url: take_url(&subject, &["source_url", "ballotpedia_url"]),
        measure_code: {
            let s = js_str(&subject, &["measure_code"]);
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        },
        ..PackedContext::default()
    };
    if ctx.office.is_empty() && kind == "measure" {
        ctx.office = "Ballot measure".into();
    }

    ctx.voter_profile = prefs_from_value(enrich.get("voter_profile"));

    let dossier = enrich.get("dossier");
    let scrutiny = enrich.get("scrutiny");

    let mut ends = as_array(dossier.and_then(|d| d.get("endorsements")));
    if ends.is_empty() {
        ends = as_array(scrutiny.and_then(|s| s.get("endorsements")));
    }
    if ends.is_empty() {
        ends = as_array(subject.get("endorsements"));
    }
    for (i, e) in ends.into_iter().take(40).enumerate() {
        let org = js_str(&e, &["org"]);
        if org.is_empty() {
            continue;
        }
        ctx.endorsements.push(PackedEndorsement {
            id: format!("e{i}"),
            org,
            stance: {
                let s = js_str(&e, &["stance"]);
                if s.is_empty() {
                    "support".into()
                } else {
                    s
                }
            },
            source: nonempty(js_str(&e, &["source"])),
            url: take_url(&e, &["source_url", "url"]),
            trust: nonempty(js_str(&e, &["trust"])),
        });
    }

    let news = as_array(scrutiny.and_then(|s| s.get("news")));
    for (i, n) in news.into_iter().take(MAX_NEWS).enumerate() {
        let title = js_str(&n, &["title"]);
        if title.is_empty() {
            continue;
        }
        ctx.news.push(PackedNews {
            id: format!("n{i}"),
            title: truncate_chars(&title, 180),
            outlet: nonempty(js_str(&n, &["outlet"])),
            url: take_url(&n, &["url"]),
            date: nonempty(js_str(&n, &["date"])),
        });
    }

    let claims = as_array(scrutiny.and_then(|s| s.get("claims")));
    for (i, c) in claims.into_iter().take(MAX_CLAIMS).enumerate() {
        let text = js_str(&c, &["text", "claim_text"]);
        if text.is_empty() {
            continue;
        }
        ctx.claims.push(PackedClaim {
            id: format!("c{i}"),
            text: truncate_chars(&text, 240),
            kind: nonempty(js_str(&c, &["kind", "claim_kind"])),
            source: nonempty(js_str(&c, &["source"])),
        });
    }

    let votes = as_array(enrich.get("votes"));
    for (i, v) in votes.into_iter().take(MAX_VOTES).enumerate() {
        let question = js_str(&v, &["question"]);
        if question.is_empty() {
            continue;
        }
        ctx.votes.push(PackedVote {
            id: format!("v{i}"),
            date: js_str(&v, &["date"]),
            question: truncate_chars(&question, 200),
            position: js_str(&v, &["position"]),
            url: take_url(&v, &["url"]),
        });
    }

    let signals = as_array(
        scrutiny
            .and_then(|s| s.get("money"))
            .and_then(|m| m.get("signals"))
            .or_else(|| enrich.get("money_signals").and_then(|m| m.get("signals"))),
    );
    for (i, s) in signals.into_iter().take(12).enumerate() {
        let label = js_str(&s, &["label"]);
        if label.is_empty() {
            continue;
        }
        ctx.money.push(PackedMoney {
            id: format!("m{i}"),
            label,
            value: js_str(&s, &["value_display", "value"]),
        });
    }

    let donors = as_array(enrich.get("top_individuals"));
    for (i, d) in donors.into_iter().take(MAX_DONORS).enumerate() {
        let name = js_str(&d, &["name", "contributor_name"]);
        if name.is_empty() {
            continue;
        }
        ctx.donors.push(PackedDonor {
            id: format!("d{i}"),
            name,
            amount: nonempty(js_str(&d, &["amount_display", "amount"])),
        });
    }

    if let Some(fin) = enrich.get("finance").or_else(|| subject.get("finance")) {
        if fin.is_object() {
            ctx.finance = Some(PackedFinance {
                receipts: nonempty(js_str(
                    fin,
                    &["receipts_display", "contributions_sum_display"],
                )),
                source: nonempty(js_str(fin, &["source_label", "source"])),
                note: nonempty(js_str(fin, &["note"])),
            });
        }
    }

    let facts = as_array(dossier.and_then(|d| d.get("facts")));
    for (i, f) in facts.into_iter().take(MAX_FACTS).enumerate() {
        let text = js_str(&f, &["text"]);
        if text.is_empty() {
            continue;
        }
        ctx.facts.push(PackedFact {
            id: format!("f{i}"),
            kind: js_str(&f, &["kind"]),
            text: truncate_chars(&text, 200),
        });
    }

    let packed_json = serde_json::to_string(&ctx).unwrap_or_default();
    ctx.hash = fingerprint(&packed_json);
    Some(ctx)
}

fn nonempty(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

pub fn packed_fingerprint(packed_json: &str) -> String {
    if let Ok(ctx) = serde_json::from_str::<PackedContext>(packed_json) {
        if !ctx.hash.is_empty() {
            return ctx.hash;
        }
    }
    fingerprint(packed_json)
}

fn system_prompt(kind: &str) -> String {
    let signed = SIGNED_AXES.join(", ");
    format!(
        "You write a voter-facing verdict card for one ballot {kind}.\n\
Score only the locked rubric axes provided. Unsigned axes are 0-100. \
Signed axes ({signed}) are -100 to +100.\n\
Every scored axis needs at least one cite: a packed cite id (e0, v3, n1, …) or an https URL you actually retrieved.\n\
If you cannot cite, set score to null.\n\
Give an overall verdict and per-axis verdict lines. Be direct.\n\
If voter_profile is present (axis id → 1-5 Likert: 1 strongly disagree, 5 strongly agree; some axes use custom poles), write the overall headline and verdict relative to that voter's likes and dislikes. Axis scores stay alignment (how much the subject is that thing) — do not invert numbers; the client remaps fit.\n\
Do not invent family, citizenship, assets, sexual orientation, or quotations.\n\
News and X posts are reported-by, trust news. Endorsements found via search are trust news, not filing.\n\
Return one JSON object (no markdown). Types: headline string; overall object; \
summary array of objects; axes array of objects; found array; tab_cites array.\n\
{{\"headline\":\"...\",\"overall\":{{\"score\":0,\"label\":\"...\",\"verdict\":\"...\"}},\
\"summary\":[{{\"text\":\"...\",\"cites\":[{{\"id\":\"e0\"}}]}}],\
\"axes\":[{{\"id\":\"maga\",\"score\":70,\"verdict\":\"...\",\"cites\":[{{\"url\":\"https://...\"}}]}}],\
\"found\":[{{\"kind\":\"endorsement\",\"org\":\"...\",\"stance\":\"support\",\"url\":\"https://...\",\"trust\":\"news\"}}],\
\"tab_cites\":[{{\"tab\":\"scrutiny\",\"label\":\"Endorsements\"}}]}}\n\
cites may use packed id or url. tab is one of dossier,scrutiny,votes,finance,personal,timeline,party."
    )
}

fn user_prompt(ctx: &PackedContext) -> String {
    let compact = json!({
        "kind": ctx.kind,
        "name": ctx.name,
        "office": ctx.office,
        "party": ctx.party,
        "party_bucket": ctx.party_bucket,
        "is_judge": ctx.is_judge,
        "is_incumbent": ctx.is_incumbent,
        "jurisdiction": ctx.jurisdiction,
        "state": ctx.state,
        "measure_code": ctx.measure_code,
        "summary": ctx.summary,
        "source_url": ctx.source_url,
        "voter_profile": ctx.voter_profile,
        "rubric": ctx.rubric,
        "endorsements": ctx.endorsements,
        "news": ctx.news,
        "claims": ctx.claims,
        "votes": ctx.votes,
        "money": ctx.money,
        "donors": ctx.donors,
        "finance": ctx.finance,
        "facts": ctx.facts,
    });
    format!(
        "Packed filings (cite ids e# n# c# v# m# d# f#). Search the live web and X for gaps.\n{}",
        compact
    )
}

fn agent_tools(provider: &str, with_search: bool) -> Value {
    if !with_search {
        return json!([]);
    }
    if provider == "xai" {
        json!([{ "type": "web_search" }, { "type": "x_search" }])
    } else {
        json!([{ "type": "web_search" }])
    }
}

/// Responses API + Agent Tools. xAI Live Search (`search_parameters`) is
/// deprecated (HTTP 410). Use built-in `web_search` / `x_search` tools.
pub fn verdict_request_body(
    provider: &str,
    model: &str,
    packed_json: &str,
    with_search: bool,
) -> Option<String> {
    let model = model.trim();
    if model.is_empty() {
        return None;
    }
    let ctx: PackedContext = serde_json::from_str(packed_json).ok()?;
    if ctx.name.trim().is_empty() {
        return None;
    }
    let prov = verdict_normalize_provider(provider)?;
    let body = json!({
        "model": model,
        "instructions": system_prompt(&ctx.kind),
        "input": [{ "role": "user", "content": user_prompt(&ctx) }],
        "tools": agent_tools(prov, with_search),
        "text": { "format": { "type": "json_object" } },
    });
    serde_json::to_string(&body).ok()
}

/// Chat Completions fallback — packed filings only. Do not send
/// `search_parameters` (deprecated Live Search).
pub fn verdict_chat_request_body(
    provider: &str,
    model: &str,
    packed_json: &str,
    _with_search: bool,
) -> Option<String> {
    let model = model.trim();
    if model.is_empty() {
        return None;
    }
    let ctx: PackedContext = serde_json::from_str(packed_json).ok()?;
    if ctx.name.trim().is_empty() {
        return None;
    }
    let _ = verdict_normalize_provider(provider)?;
    let body = json!({
        "model": model,
        "temperature": 0.2,
        "response_format": { "type": "json_object" },
        "messages": [
            { "role": "system", "content": system_prompt(&ctx.kind) },
            { "role": "user", "content": user_prompt(&ctx) }
        ],
    });
    serde_json::to_string(&body).ok()
}

fn push_text_part(buf: &mut String, part: &Value) {
    if let Some(t) = part.as_str() {
        buf.push_str(t);
        return;
    }
    if let Some(t) = part.get("text").and_then(|t| t.as_str()) {
        buf.push_str(t);
    } else if let Some(t) = part.get("output_text").and_then(|t| t.as_str()) {
        buf.push_str(t);
    } else if let Some(t) = part.get("content").and_then(|t| t.as_str()) {
        buf.push_str(t);
    }
}

fn collect_choice_content(v: &Value, buf: &mut String) {
    let content = v
        .pointer("/choices/0/message/content")
        .or_else(|| v.pointer("/choices/0/delta/content"));
    match content {
        Some(Value::String(s)) => buf.push_str(s),
        Some(Value::Array(parts)) => {
            for part in parts {
                push_text_part(buf, part);
            }
        }
        _ => {}
    }
}

pub fn extract_model_text(response_json: &str) -> Option<String> {
    let v: Value = serde_json::from_str(response_json).ok()?;
    let mut buf = String::new();
    collect_choice_content(&v, &mut buf);
    if buf.trim().is_empty() {
        if let Some(s) = v
            .pointer("/choices/0/message/reasoning_content")
            .and_then(|x| x.as_str())
        {
            buf.push_str(s);
        }
    }
    if buf.trim().is_empty() {
        if let Some(s) = v.get("output_text").and_then(|x| x.as_str()) {
            buf.push_str(s);
        }
    }
    if buf.trim().is_empty() {
        if let Some(arr) = v.get("output").and_then(|x| x.as_array()) {
            for item in arr {
                let kind = item.get("type").and_then(|t| t.as_str()).unwrap_or("");
                if kind == "reasoning"
                    || kind == "web_search_call"
                    || kind == "x_search_call"
                    || kind == "function_call"
                    || kind == "custom_tool_call"
                {
                    continue;
                }
                match item.get("content") {
                    Some(Value::Array(parts)) => {
                        for part in parts {
                            let pkind = part.get("type").and_then(|t| t.as_str()).unwrap_or("");
                            if pkind == "refusal" {
                                continue;
                            }
                            push_text_part(&mut buf, part);
                        }
                    }
                    Some(Value::String(s)) => buf.push_str(s),
                    _ => {}
                }
                if let Some(t) = item.get("text").and_then(|t| t.as_str()) {
                    buf.push_str(t);
                }
            }
        }
    }
    let t = buf.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

pub fn extract_response_urls(response_json: &str) -> Vec<String> {
    let Ok(v) = serde_json::from_str::<Value>(response_json) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut push = |s: &str| {
        let t = s.trim();
        if is_http_url(t) && !out.iter().any(|u: &String| u == t) {
            out.push(t.to_string());
        }
    };
    match v.get("citations") {
        Some(Value::Array(a)) => {
            for item in a {
                if let Some(s) = item.as_str() {
                    push(s);
                } else if let Some(s) = item.get("url").and_then(|u| u.as_str()) {
                    push(s);
                }
            }
        }
        Some(Value::Object(map)) => {
            for val in map.values() {
                if let Some(s) = val.as_str() {
                    push(s);
                }
            }
        }
        _ => {}
    }
    if let Some(arr) = v.get("output").and_then(|x| x.as_array()) {
        for item in arr {
            let parts = match item.get("content") {
                Some(Value::Array(p)) => p.clone(),
                _ => continue,
            };
            for part in parts {
                let ann = match part.get("annotations").and_then(|a| a.as_array()) {
                    Some(a) => a,
                    None => continue,
                };
                for a in ann {
                    if let Some(s) = a.get("url").and_then(|u| u.as_str()) {
                        push(s);
                    }
                }
            }
        }
    }
    out
}

fn looks_like_card(v: &Value) -> bool {
    let Some(obj) = v.as_object() else {
        return false;
    };
    let axes = obj.contains_key("axes") || obj.contains_key("scores");
    let headline = obj.contains_key("headline");
    let overall = obj.contains_key("overall");
    axes || (headline && overall)
}

fn value_string(v: Option<&Value>) -> String {
    match v {
        Some(Value::String(s)) => s.trim().to_string(),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::Bool(b)) => b.to_string(),
        _ => String::new(),
    }
}

fn value_i32(v: Option<&Value>) -> Option<i32> {
    match v {
        Some(Value::Null) | None => None,
        Some(Value::Number(n)) => n
            .as_i64()
            .map(|i| i as i32)
            .or_else(|| n.as_f64().map(|f| f.round() as i32)),
        Some(Value::String(s)) => s.trim().parse::<f64>().ok().map(|f| f.round() as i32),
        Some(Value::Object(m)) => value_i32(m.get("score").or_else(|| m.get("value"))),
        _ => None,
    }
}

fn brace_slice(s: &str) -> Option<&str> {
    let mut depth = 0i32;
    let mut in_str = false;
    let mut esc = false;
    for (i, c) in s.char_indices() {
        if in_str {
            if esc {
                esc = false;
                continue;
            }
            if c == '\\' {
                esc = true;
                continue;
            }
            if c == '"' {
                in_str = false;
            }
            continue;
        }
        match c {
            '"' => in_str = true,
            '{' | '[' => depth += 1,
            '}' | ']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&s[..=i]);
                }
            }
            _ => {}
        }
    }
    None
}

fn repair_json(s: &str) -> Option<Value> {
    let mut buf = s.trim().to_string();
    if buf.is_empty() {
        return None;
    }
    let mut in_str = false;
    let mut esc = false;
    for c in buf.chars() {
        if in_str {
            if esc {
                esc = false;
                continue;
            }
            if c == '\\' {
                esc = true;
                continue;
            }
            if c == '"' {
                in_str = false;
            }
            continue;
        }
        if c == '"' {
            in_str = true;
        }
    }
    if in_str {
        buf.push('"');
    }
    let mut stack: Vec<char> = Vec::new();
    in_str = false;
    esc = false;
    for c in buf.chars() {
        if in_str {
            if esc {
                esc = false;
                continue;
            }
            if c == '\\' {
                esc = true;
                continue;
            }
            if c == '"' {
                in_str = false;
            }
            continue;
        }
        match c {
            '"' => in_str = true,
            '{' => stack.push('}'),
            '[' => stack.push(']'),
            '}' | ']' => {
                stack.pop();
            }
            _ => {}
        }
    }
    while let Some(c) = stack.pop() {
        buf.push(c);
    }
    serde_json::from_str(&buf).ok()
}

fn find_card_object(s: &str) -> Option<Value> {
    let mut best = None;
    for needle in ["\"headline\"", "\"axes\"", "\"overall\""] {
        let mut from = 0;
        while let Some(rel) = s[from..].find(needle) {
            let at = from + rel;
            if let Some(start) = s[..at].rfind('{') {
                if let Some(slice) = brace_slice(&s[start..]) {
                    if let Ok(v) = serde_json::from_str::<Value>(slice) {
                        if looks_like_card(&v) {
                            return Some(v);
                        }
                    }
                } else if let Some(v) = repair_json(&s[start..]) {
                    if looks_like_card(&v) {
                        best = Some(v);
                    }
                }
            }
            from = at + needle.len();
        }
    }
    best
}

fn card_from_wrapper(v: &Value) -> Option<Value> {
    let obj = v.as_object()?;
    for key in ["card", "verdict_card", "output_parsed"] {
        if let Some(inner) = obj.get(key) {
            if looks_like_card(inner) {
                return Some(inner.clone());
            }
            if let Some(c) = card_from_wrapper(inner) {
                return Some(c);
            }
        }
    }
    if let Some(inner) = obj.get("verdict") {
        if looks_like_card(inner) {
            return Some(inner.clone());
        }
    }
    None
}

fn card_json_from_response(response_json: &str) -> Option<Value> {
    let t = response_json.trim();
    if t.is_empty() {
        return None;
    }
    if let Some(text) = extract_model_text(t) {
        if let Some(c) = find_card_object(&text) {
            return Some(c);
        }
        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            if looks_like_card(&v) {
                return Some(v);
            }
            if let Some(c) = card_from_wrapper(&v) {
                return Some(c);
            }
        }
    }
    if let Ok(v) = serde_json::from_str::<Value>(t) {
        if looks_like_card(&v) {
            return Some(v);
        }
        if let Some(c) = card_from_wrapper(&v) {
            return Some(c);
        }
    }
    find_card_object(t)
}

fn card_from_prose(text: &str) -> Option<RawCard> {
    let t = collapse_ws(text);
    if t.chars().count() < 20 {
        return None;
    }
    if t.starts_with('{') && (t.contains("\"output\"") || t.contains("\"choices\"")) {
        return None;
    }
    Some(RawCard {
        headline: truncate_chars(&t, MAX_HEADLINE),
        summary: vec![RawSentence {
            text: truncate_chars(&t, 800),
            cites: Vec::new(),
        }],
        ..RawCard::default()
    })
}

fn cite_from_value(v: &Value) -> Option<RawCite> {
    match v {
        Value::String(s) => {
            let t = s.trim();
            if t.is_empty() {
                return None;
            }
            if is_http_url(t) {
                Some(RawCite {
                    url: t.to_string(),
                    ..RawCite::default()
                })
            } else {
                Some(RawCite {
                    id: t.to_string(),
                    ..RawCite::default()
                })
            }
        }
        Value::Object(_) => Some(RawCite {
            id: value_string(v.get("id")),
            url: value_string(v.get("url").or_else(|| v.get("href"))),
            tab: value_string(v.get("tab")),
            label: value_string(v.get("label").or_else(|| v.get("source"))),
            trust: value_string(v.get("trust")),
            kind: value_string(v.get("kind")),
        }),
        _ => None,
    }
}

fn cites_from_value(v: Option<&Value>) -> Vec<RawCite> {
    match v {
        Some(Value::Array(a)) => a.iter().filter_map(cite_from_value).collect(),
        Some(Value::String(_)) | Some(Value::Object(_)) => {
            cite_from_value(v.unwrap()).into_iter().collect()
        }
        _ => Vec::new(),
    }
}

fn sentence_from_value(v: &Value) -> Option<RawSentence> {
    match v {
        Value::String(s) => {
            let text = s.trim().to_string();
            if text.is_empty() {
                None
            } else {
                Some(RawSentence {
                    text,
                    cites: Vec::new(),
                })
            }
        }
        Value::Object(_) => {
            let text = value_string(
                v.get("text")
                    .or_else(|| v.get("sentence"))
                    .or_else(|| v.get("claim")),
            );
            if text.is_empty() {
                None
            } else {
                Some(RawSentence {
                    text,
                    cites: cites_from_value(v.get("cites").or_else(|| v.get("citations"))),
                })
            }
        }
        _ => None,
    }
}

fn sentences_from_value(v: Option<&Value>) -> Vec<RawSentence> {
    match v {
        Some(Value::String(s)) => sentence_from_value(&Value::String(s.clone()))
            .into_iter()
            .collect(),
        Some(Value::Array(a)) => a.iter().filter_map(sentence_from_value).collect(),
        Some(Value::Object(_)) => sentence_from_value(v.unwrap()).into_iter().collect(),
        _ => Vec::new(),
    }
}

fn axis_from_value(id_hint: Option<&str>, v: &Value) -> Option<RawAxis> {
    match v {
        Value::Number(_) | Value::String(_) => {
            let id = id_hint.unwrap_or("").trim().to_string();
            if id.is_empty() {
                return None;
            }
            Some(RawAxis {
                id,
                score: value_i32(Some(v)),
                ..RawAxis::default()
            })
        }
        Value::Object(_) => {
            let id = {
                let own = value_string(v.get("id").or_else(|| v.get("axis")));
                if !own.is_empty() {
                    own
                } else {
                    id_hint.unwrap_or("").trim().to_string()
                }
            };
            if id.is_empty() {
                return None;
            }
            Some(RawAxis {
                id,
                score: value_i32(v.get("score")),
                verdict: value_string(v.get("verdict").or_else(|| v.get("note"))),
                cites: cites_from_value(v.get("cites").or_else(|| v.get("citations"))),
                evidence: sentences_from_value(v.get("evidence")),
            })
        }
        _ => None,
    }
}

fn axes_from_value(v: Option<&Value>) -> Vec<RawAxis> {
    match v {
        Some(Value::Array(a)) => a.iter().filter_map(|x| axis_from_value(None, x)).collect(),
        Some(Value::Object(m)) => m
            .iter()
            .filter_map(|(k, val)| axis_from_value(Some(k), val))
            .collect(),
        _ => Vec::new(),
    }
}

fn overall_from_value(v: Option<&Value>) -> Option<RawOverall> {
    match v {
        Some(Value::Number(_)) => Some(RawOverall {
            score: value_i32(v),
            ..RawOverall::default()
        }),
        Some(Value::String(s)) => Some(RawOverall {
            verdict: s.trim().to_string(),
            ..RawOverall::default()
        }),
        Some(Value::Object(_)) => Some(RawOverall {
            score: value_i32(v.and_then(|x| x.get("score"))),
            label: value_string(v.and_then(|x| x.get("label"))),
            verdict: value_string(
                v.and_then(|x| x.get("verdict").or_else(|| x.get("text"))),
            ),
        }),
        _ => None,
    }
}

fn found_from_value(v: Option<&Value>) -> Vec<RawFound> {
    let arr = match v {
        Some(Value::Array(a)) => a.clone(),
        Some(Value::Object(m)) => m.values().cloned().collect(),
        _ => return Vec::new(),
    };
    arr.into_iter()
        .filter_map(|item| match item {
            Value::String(s) => {
                let text = s.trim().to_string();
                if text.is_empty() {
                    None
                } else {
                    Some(RawFound {
                        text,
                        kind: "news".into(),
                        ..RawFound::default()
                    })
                }
            }
            Value::Object(_) => Some(RawFound {
                kind: value_string(item.get("kind").or_else(|| item.get("type"))),
                org: value_string(item.get("org").or_else(|| item.get("name"))),
                stance: value_string(item.get("stance")),
                text: value_string(item.get("text").or_else(|| item.get("title"))),
                url: value_string(item.get("url").or_else(|| item.get("href"))),
                trust: value_string(item.get("trust")),
                date: value_string(item.get("date")),
                source: value_string(item.get("source").or_else(|| item.get("outlet"))),
            }),
            _ => None,
        })
        .collect()
}

fn tabs_from_value(v: Option<&Value>) -> Vec<RawTab> {
    match v {
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(|item| match item {
                Value::String(s) => {
                    let tab = s.trim().to_string();
                    if tab.is_empty() {
                        None
                    } else {
                        Some(RawTab {
                            tab,
                            label: String::new(),
                        })
                    }
                }
                Value::Object(_) => {
                    let tab = value_string(item.get("tab"));
                    if tab.is_empty() {
                        None
                    } else {
                        Some(RawTab {
                            tab,
                            label: value_string(item.get("label")),
                        })
                    }
                }
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn raw_card_from_value(v: &Value) -> RawCard {
    RawCard {
        headline: value_string(v.get("headline").or_else(|| v.get("title"))),
        overall: overall_from_value(v.get("overall")),
        summary: sentences_from_value(v.get("summary")),
        axes: axes_from_value(v.get("axes").or_else(|| v.get("scores"))),
        found: found_from_value(v.get("found").or_else(|| v.get("findings"))),
        tab_cites: tabs_from_value(v.get("tab_cites").or_else(|| v.get("tabs"))),
    }
}

fn de_opt_i32<'de, D: Deserializer<'de>>(d: D) -> Result<Option<i32>, D::Error> {
    let v = Option::<Value>::deserialize(d)?;
    Ok(v.and_then(|x| match x {
        Value::Null => None,
        Value::Number(n) => n
            .as_i64()
            .map(|i| i as i32)
            .or_else(|| n.as_f64().map(|f| f.round() as i32)),
        Value::String(s) => s.trim().parse::<f64>().ok().map(|f| f.round() as i32),
        _ => None,
    }))
}

#[derive(Debug, Default, Deserialize)]
struct RawCite {
    #[serde(default)]
    id: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    tab: String,
    #[serde(default)]
    label: String,
    #[serde(default)]
    trust: String,
    #[serde(default)]
    kind: String,
}

#[derive(Debug, Default, Deserialize)]
struct RawSentence {
    #[serde(default)]
    text: String,
    #[serde(default)]
    cites: Vec<RawCite>,
}

#[derive(Debug, Default, Deserialize)]
struct RawAxis {
    #[serde(default)]
    id: String,
    #[serde(default, deserialize_with = "de_opt_i32")]
    score: Option<i32>,
    #[serde(default)]
    verdict: String,
    #[serde(default)]
    cites: Vec<RawCite>,
    #[serde(default)]
    evidence: Vec<RawSentence>,
}

#[derive(Debug, Default, Deserialize)]
struct RawOverall {
    #[serde(default, deserialize_with = "de_opt_i32")]
    score: Option<i32>,
    #[serde(default)]
    label: String,
    #[serde(default)]
    verdict: String,
}

#[derive(Debug, Default, Deserialize)]
struct RawFound {
    #[serde(default)]
    kind: String,
    #[serde(default)]
    org: String,
    #[serde(default)]
    stance: String,
    #[serde(default)]
    text: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    trust: String,
    #[serde(default)]
    date: String,
    #[serde(default)]
    source: String,
}

#[derive(Debug, Default, Deserialize)]
struct RawTab {
    #[serde(default)]
    tab: String,
    #[serde(default)]
    label: String,
}

#[derive(Debug, Default, Deserialize)]
struct RawCard {
    #[serde(default)]
    headline: String,
    #[serde(default)]
    overall: Option<RawOverall>,
    #[serde(default)]
    summary: Vec<RawSentence>,
    #[serde(default)]
    axes: Vec<RawAxis>,
    #[serde(default)]
    found: Vec<RawFound>,
    #[serde(default)]
    tab_cites: Vec<RawTab>,
}

fn cite_index(ctx: &PackedContext) -> HashMap<String, VerdictCite> {
    let mut map = HashMap::new();
    for e in &ctx.endorsements {
        map.insert(
            e.id.clone(),
            VerdictCite {
                kind: "url".into(),
                url: e.url.clone(),
                tab: Some("scrutiny".into()),
                label: Some(e.org.clone()),
                trust: e.trust.clone().or_else(|| Some("filing".into())),
            },
        );
    }
    for n in &ctx.news {
        map.insert(
            n.id.clone(),
            VerdictCite {
                kind: "url".into(),
                url: n.url.clone(),
                tab: Some("scrutiny".into()),
                label: Some(n.title.clone()),
                trust: Some("news".into()),
            },
        );
    }
    for c in &ctx.claims {
        map.insert(
            c.id.clone(),
            VerdictCite {
                kind: "tab".into(),
                url: None,
                tab: Some("scrutiny".into()),
                label: Some(truncate_chars(&c.text, 48)),
                trust: Some("campaign".into()),
            },
        );
    }
    for v in &ctx.votes {
        map.insert(
            v.id.clone(),
            VerdictCite {
                kind: if v.url.is_some() {
                    "url".into()
                } else {
                    "tab".into()
                },
                url: v.url.clone(),
                tab: Some("votes".into()),
                label: Some(truncate_chars(&v.question, 48)),
                trust: Some("official".into()),
            },
        );
    }
    for m in &ctx.money {
        map.insert(
            m.id.clone(),
            VerdictCite {
                kind: "tab".into(),
                url: None,
                tab: Some("finance".into()),
                label: Some(m.label.clone()),
                trust: Some("filing".into()),
            },
        );
    }
    for d in &ctx.donors {
        map.insert(
            d.id.clone(),
            VerdictCite {
                kind: "tab".into(),
                url: None,
                tab: Some("finance".into()),
                label: Some(d.name.clone()),
                trust: Some("filing".into()),
            },
        );
    }
    for f in &ctx.facts {
        map.insert(
            f.id.clone(),
            VerdictCite {
                kind: "tab".into(),
                url: None,
                tab: Some("dossier".into()),
                label: Some(truncate_chars(&f.text, 48)),
                trust: Some("reference".into()),
            },
        );
    }
    map
}

const ALLOWED_TABS: &[&str] = &[
    "dossier",
    "scrutiny",
    "votes",
    "finance",
    "personal",
    "timeline",
    "party",
    "more",
];

fn resolve_cites(raw: &[RawCite], index: &HashMap<String, VerdictCite>) -> Vec<VerdictCite> {
    let mut out = Vec::new();
    for c in raw {
        if !c.id.is_empty() {
            if let Some(hit) = index.get(c.id.trim()) {
                out.push(hit.clone());
                continue;
            }
        }
        if is_http_url(&c.url) {
            let tab = c.tab.trim().to_ascii_lowercase();
            out.push(VerdictCite {
                kind: "url".into(),
                url: Some(c.url.trim().to_string()),
                tab: if ALLOWED_TABS.contains(&tab.as_str()) {
                    Some(tab)
                } else {
                    None
                },
                label: nonempty(collapse_ws(&c.label)),
                trust: nonempty(c.trust.trim().to_string()).or_else(|| Some("news".into())),
            });
            continue;
        }
        let tab = c.tab.trim().to_ascii_lowercase();
        if ALLOWED_TABS.contains(&tab.as_str()) {
            out.push(VerdictCite {
                kind: "tab".into(),
                url: None,
                tab: Some(tab),
                label: nonempty(collapse_ws(&c.label)),
                trust: nonempty(c.trust.trim().to_string()),
            });
        }
        let _ = &c.kind;
    }
    out
}

fn sentence_from_raw(s: RawSentence, index: &HashMap<String, VerdictCite>) -> Option<CitedSentence> {
    let text = truncate_chars(&s.text, 420);
    if text.len() < 8 {
        return None;
    }
    Some(CitedSentence {
        text,
        cites: resolve_cites(&s.cites, index),
    })
}

fn clamp_axis_score(_kind: &str, axis_id: &str, n: i32) -> i32 {
    if is_signed_axis(axis_id) {
        n.clamp(-100, 100)
    } else {
        n.clamp(0, 100)
    }
}

pub fn parse_verdict_card(
    response_json: &str,
    packed_json: &str,
    model: &str,
    provider: &str,
) -> Option<VerdictCard> {
    let ctx: PackedContext = serde_json::from_str(packed_json).unwrap_or_default();
    let parsed = if let Some(raw_val) = card_json_from_response(response_json) {
        raw_card_from_value(&raw_val)
    } else {
        card_from_prose(&extract_model_text(response_json)?)?
    };
    let index = cite_index(&ctx);
    let extra_urls = extract_response_urls(response_json);

    let mut summary = Vec::new();
    for s in parsed.summary.into_iter().take(MAX_SUMMARY) {
        if let Some(row) = sentence_from_raw(s, &index) {
            summary.push(row);
        }
    }

    let allowed: HashSet<&str> = ctx.rubric.iter().map(|a| a.id.as_str()).collect();
    let label_of: HashMap<&str, &str> = ctx
        .rubric
        .iter()
        .map(|a| (a.id.as_str(), a.label.as_str()))
        .collect();

    let mut axes = Vec::new();
    for a in parsed.axes {
        let id = a.id.trim().to_ascii_lowercase();
        if !allowed.is_empty() && !allowed.contains(id.as_str()) {
            continue;
        }
        let mut evidence: Vec<CitedSentence> = a
            .evidence
            .into_iter()
            .filter_map(|s| sentence_from_raw(s, &index))
            .take(4)
            .collect();
        let loose = resolve_cites(&a.cites, &index);
        if evidence.is_empty() && !loose.is_empty() {
            if let Some(v) = nonempty(truncate_chars(&a.verdict, MAX_VERDICT_LINE)) {
                evidence.push(CitedSentence {
                    text: v,
                    cites: loose.clone(),
                });
            } else {
                evidence.push(CitedSentence {
                    text: format!("{} score", label_of.get(id.as_str()).copied().unwrap_or(&id)),
                    cites: loose,
                });
            }
        }
        let has_cite = evidence.iter().any(|e| !e.cites.is_empty());
        let score = if has_cite {
            a.score.map(|n| clamp_axis_score(&ctx.kind, &id, n))
        } else {
            None
        };
        axes.push(AxisScore {
            label: label_of
                .get(id.as_str())
                .unwrap_or(&id.as_str())
                .to_string(),
            id,
            score,
            verdict: nonempty(truncate_chars(&a.verdict, MAX_VERDICT_LINE)),
            evidence,
            raw_score: None,
            inverted: None,
        });
    }
    for def in &ctx.rubric {
        if !axes.iter().any(|a| a.id == def.id) {
            axes.push(AxisScore {
                label: def.label.clone(),
                id: def.id.clone(),
                score: None,
                verdict: None,
                evidence: Vec::new(),
                raw_score: None,
                inverted: None,
            });
        }
    }
    axes.sort_by_key(|a| {
        ctx.rubric
            .iter()
            .position(|d| d.id == a.id)
            .unwrap_or(usize::MAX)
    });

    let mut found = Vec::new();
    for f in parsed.found.into_iter().take(20) {
        let url = if is_http_url(&f.url) {
            Some(f.url.trim().to_string())
        } else {
            None
        };
        let org = collapse_ws(&f.org);
        let text = collapse_ws(&f.text);
        if org.is_empty() && text.is_empty() && url.is_none() {
            continue;
        }
        let trust = {
            let t = f.trust.trim().to_ascii_lowercase();
            if t.is_empty() {
                "news".into()
            } else if t == "filing" || t == "official" {
                "news".into()
            } else {
                t
            }
        };
        found.push(FoundItem {
            kind: {
                let k = f.kind.trim().to_ascii_lowercase();
                if k.is_empty() {
                    "news".into()
                } else {
                    k
                }
            },
            org: nonempty(org),
            stance: nonempty(f.stance.trim().to_ascii_lowercase()),
            text: nonempty(truncate_chars(&text, 280)),
            url,
            trust: Some(trust),
            date: nonempty(collapse_ws(&f.date)),
            source: nonempty(collapse_ws(&f.source)),
        });
    }

    let mut tab_cites = Vec::new();
    for t in parsed.tab_cites {
        let tab = t.tab.trim().to_ascii_lowercase();
        if !ALLOWED_TABS.contains(&tab.as_str()) {
            continue;
        }
        tab_cites.push(TabCite {
            tab,
            label: {
                let l = collapse_ws(&t.label);
                if l.is_empty() {
                    "Details".into()
                } else {
                    l
                }
            },
        });
    }
    if tab_cites.is_empty() {
        if !ctx.endorsements.is_empty() || !ctx.news.is_empty() {
            tab_cites.push(TabCite {
                tab: "scrutiny".into(),
                label: "Scrutiny".into(),
            });
        }
        if !ctx.votes.is_empty() {
            tab_cites.push(TabCite {
                tab: "votes".into(),
                label: if ctx.is_judge {
                    "Decisions".into()
                } else {
                    "Votes".into()
                },
            });
        }
        if ctx.finance.is_some() || !ctx.donors.is_empty() {
            tab_cites.push(TabCite {
                tab: "finance".into(),
                label: "Finance".into(),
            });
        }
    }

    let overall_raw = parsed.overall.unwrap_or(RawOverall {
        score: None,
        label: String::new(),
        verdict: String::new(),
    });
    let overall_has_cite = !summary.is_empty() && summary.iter().any(|s| !s.cites.is_empty());
    let overall = OverallScore {
        score: if overall_has_cite || axes.iter().any(|a| a.score.is_some()) {
            overall_raw.score.map(|n| n.clamp(0, 100))
        } else {
            None
        },
        label: nonempty(truncate_chars(&overall_raw.label, 80)),
        verdict: nonempty(truncate_chars(&overall_raw.verdict, MAX_VERDICT_LINE)),
        raw_score: None,
        profiled: None,
    };

    let tools = if extra_urls.is_empty() {
        Vec::new()
    } else {
        vec!["web_search".into()]
    };

    let mut headline = truncate_chars(&parsed.headline, MAX_HEADLINE);
    if headline.trim().is_empty() {
        if let Some(v) = overall.verdict.as_ref() {
            headline = truncate_chars(v, MAX_HEADLINE);
        } else if let Some(s) = summary.first() {
            headline = truncate_chars(&s.text, MAX_HEADLINE);
        } else if let Some(l) = overall.label.as_ref() {
            headline = truncate_chars(l, MAX_HEADLINE);
        }
    }
    if headline.trim().is_empty()
        && summary.is_empty()
        && found.is_empty()
        && axes.iter().all(|a| a.score.is_none() && a.verdict.is_none())
    {
        return None;
    }
    Some(VerdictCard {
        kind: ctx.kind,
        headline,
        summary,
        overall,
        axes,
        found,
        tab_cites,
        model: nonempty(model.trim().to_string()),
        provider: verdict_normalize_provider(provider).map(|s| s.to_string()),
        tools,
        generated_at: Some(chrono::Utc::now().date_naive().to_string()),
        packed_hash: nonempty(ctx.hash),
    })
}

pub fn found_endorsements_from_verdict(card: &VerdictCard) -> Vec<Endorsement> {
    let mut out = Vec::new();
    for f in &card.found {
        if f.kind != "endorsement" && f.kind != "opposition" {
            continue;
        }
        let org = f.org.clone().unwrap_or_default();
        if org.is_empty() {
            continue;
        }
        let stance = f
            .stance
            .clone()
            .unwrap_or_else(|| {
                if f.kind == "opposition" {
                    "oppose".into()
                } else {
                    "support".into()
                }
            });
        out.push(Endorsement {
            org,
            stance,
            source: f
                .source
                .clone()
                .unwrap_or_else(|| "Live search".into()),
            source_url: f.url.clone(),
            kind: Some("org".into()),
            trust: Some(f.trust.clone().unwrap_or_else(|| "news".into())),
            date: f.date.clone(),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rubric_picks_party_and_judge_and_measure() {
        let r = rubric_for("candidate", "Republican", false);
        assert!(r.iter().any(|a| a.id == "maga"));
        assert!(r.iter().any(|a| a.id == "zionist"));
        assert!(r.iter().any(|a| a.id == "tea_party"));
        assert!(r.iter().any(|a| a.id == "abortion"));
        let d = rubric_for("candidate", "Democratic", false);
        assert!(d.iter().any(|a| a.id == "aoc"));
        assert!(d.iter().any(|a| a.id == "communism"));
        assert!(d.iter().any(|a| a.id == "obama_era"));
        assert!(d.iter().any(|a| a.id == "occupy"));
        let j = rubric_for("candidate", "NPA", true);
        assert!(j.iter().any(|a| a.id == "originalism"));
        assert!(j.iter().any(|a| a.id == "tds"));
        let m = rubric_for("measure", "", false);
        assert!(m.iter().any(|a| a.id == "tax_direction"));
        let i = rubric_for("candidate", "Libertarian", false);
        assert!(i.iter().any(|a| a.id == "libertarian"));
        assert!(i.iter().any(|a| a.id == "ron_paul"));
        assert!(i.iter().any(|a| a.id == "border"));
    }

    #[test]
    fn packer_pulls_endorsements_and_votes() {
        let subject = r#"{"name":"Byron Donalds","party":"Republican","office":"U.S. House","jurisdiction":"FL-19","is_incumbent":true}"#;
        let enrich = r#"{
            "dossier":{"endorsements":[{"org":"House Freedom Fund","stance":"support","source":"Ballotpedia","source_url":"https://ballotpedia.org/Byron_Donalds","trust":"reference"}]},
            "votes":[{"date":"2024-03-01","question":"HR 1","position":"Yea","url":"https://www.govtrack.us/congress/votes/1"}],
            "top_individuals":[{"name":"Jane Donor","amount_display":"$3,300"}]
        }"#;
        let ctx = pack_verdict_context(subject, enrich).unwrap();
        assert_eq!(ctx.kind, "candidate");
        assert_eq!(ctx.party_bucket, "republican");
        assert_eq!(ctx.endorsements.len(), 1);
        assert_eq!(ctx.endorsements[0].id, "e0");
        assert_eq!(ctx.votes.len(), 1);
        assert_eq!(ctx.donors.len(), 1);
        assert!(!ctx.hash.is_empty());
        assert!(ctx.rubric.iter().any(|a| a.id == "maga"));
    }

    #[test]
    fn packer_requires_name() {
        assert!(pack_verdict_context(r#"{"party":"Republican"}"#, "{}").is_none());
    }

    #[test]
    fn request_body_includes_tools_for_xai() {
        let ctx = pack_verdict_context(
            r#"{"name":"Jane Doe","party":"Republican","office":"U.S. House"}"#,
            "{}",
        )
        .unwrap();
        let packed = serde_json::to_string(&ctx).unwrap();
        let body = verdict_request_body("xai", "grok-4.6", &packed, true).unwrap();
        assert!(body.contains("web_search"));
        assert!(body.contains("x_search"));
        assert!(!body.contains("search_parameters"));
        assert!(body.contains("instructions"));
        assert!(body.contains("Jane Doe"));
        let no = verdict_request_body("xai", "grok-4.6", &packed, false).unwrap();
        assert!(no.contains("\"tools\":[]") || no.contains("\"tools\": []"));
        let oa = verdict_request_body("openai", "gpt-4o-mini", &packed, true).unwrap();
        assert!(oa.contains("web_search"));
        assert!(!oa.contains("x_search"));
        let chat = verdict_chat_request_body("xai", "grok-4.6", &packed, true).unwrap();
        assert!(chat.contains("messages"));
        assert!(!chat.contains("search_parameters"));
        assert!(body.contains("json_object"));
        assert!(chat.contains("json_object"));
    }

    #[test]
    fn parse_accepts_messy_grok_shapes() {
        let ctx = pack_verdict_context(
            r#"{"name":"Byron Donalds","party":"Republican","office":"Governor"}"#,
            r#"{"dossier":{"endorsements":[{"org":"Donald Trump","stance":"support","source_url":"https://ballotpedia.org/x"}]}}"#,
        )
        .unwrap();
        let packed = serde_json::to_string(&ctx).unwrap();

        let fenced = r#"{"choices":[{"message":{"content":[{"type":"text","text":"```json\n{\"headline\":\"MAGA standard-bearer\",\"overall\":82,\"summary\":\"Trump-endorsed Florida Republican running for governor.\",\"axes\":{\"maga\":{\"score\":90,\"verdict\":\"Trump-aligned House record.\",\"cites\":[\"e0\"]},\"trump\":88},\"found\":[{\"kind\":\"endorsement\",\"org\":\"Trump\",\"url\":\"https://x.com/realdonaldtrump\"}]}\n```"}]}}]}"#;
        let card = parse_verdict_card(fenced, &packed, "grok-4.6", "xai").unwrap();
        assert!(card.headline.to_lowercase().contains("maga"));
        assert_eq!(card.overall.score, Some(82));
        assert!(card.axes.iter().any(|a| a.id == "maga" && a.score == Some(90)));
        assert!(card.axes.iter().any(|a| a.id == "trump" && a.score.is_none()));
        assert!(card.axes.iter().any(|a| a.id == "neocon" && a.score.is_none()));
        assert_eq!(card.found.len(), 1);

        let hijack = r#"{"object":"response","output":[{"type":"web_search_call","result":{"headline":"Donalds wins Trump nod","url":"https://example.com/news","summary":"Wire story."}},{"type":"x_search_call","data":{"headline":"Post about Donalds"}},{"type":"message","role":"assistant","content":[{"type":"output_text","text":"{\"headline\":\"MAGA standard-bearer\",\"overall\":{\"score\":81,\"verdict\":\"Trump-aligned.\",\"label\":\"MAGA\"},\"axes\":[{\"id\":\"maga\",\"score\":91,\"cites\":[{\"id\":\"e0\"}]}]}"}]}]}"#;
        let card_h = parse_verdict_card(hijack, &packed, "grok-4.6", "xai").unwrap();
        assert!(card_h.headline.to_lowercase().contains("maga"));
        assert!(!card_h.headline.to_lowercase().contains("wins trump nod"));
        assert!(card_h.axes.iter().any(|a| a.id == "maga" && a.score == Some(91)));

        let wrapped = r#"{"output":[{"type":"web_search_call"},{"type":"message","content":[{"type":"output_text","text":"{\"card\":{\"headline\":\"America First governor bid\",\"overall\":{\"score\":75,\"verdict\":\"Trump-aligned.\",\"label\":\"MAGA\"},\"summary\":[\"Cited Trump endorsement from packed filings.\"],\"axes\":[{\"id\":\"america_first\",\"score\":80,\"cites\":[{\"id\":\"e0\"}]}]}}"}]}]}"#;
        let card2 = parse_verdict_card(wrapped, &packed, "grok-4.6", "xai").unwrap();
        assert!(card2.headline.contains("America First"));
        assert!(card2
            .axes
            .iter()
            .any(|a| a.id == "america_first" && a.score == Some(80)));
    }

    #[test]
    fn extract_xai_responses_output_and_float_scores() {
        let ctx = pack_verdict_context(
            r#"{"name":"Jane Doe","party":"Republican","office":"House"}"#,
            r#"{"votes":[{"date":"2024-01-01","question":"Border bill","position":"Yea","url":"https://www.govtrack.us/x"}]}"#,
        )
        .unwrap();
        let packed = serde_json::to_string(&ctx).unwrap();
        let resp = r#"{"object":"response","output":[{"type":"message","role":"assistant","content":[{"type":"output_text","text":"{\"headline\":\"MAGA-aligned\",\"overall\":{\"score\":80.4},\"summary\":[{\"text\":\"Voted Yea on the border bill in 2024.\",\"cites\":[{\"id\":\"v0\"}]}],\"axes\":[{\"id\":\"maga\",\"score\":88.7,\"verdict\":\"Strong MAGA record.\",\"cites\":[{\"id\":\"v0\"}]}]}"}]}],"citations":["https://example.com/a"]}"#;
        let card = parse_verdict_card(resp, &packed, "grok-4.6", "xai").unwrap();
        assert!(card.headline.contains("MAGA"));
        assert_eq!(card.overall.score, Some(80));
        assert!(card.axes.iter().any(|a| a.id == "maga" && a.score == Some(89)));
    }

    #[test]
    fn parse_drops_unknown_axis_and_unsourced_score() {
        let ctx = pack_verdict_context(
            r#"{"name":"Jane Doe","party":"Republican","office":"House"}"#,
            r#"{"votes":[{"date":"2024-01-01","question":"Border bill","position":"Yea","url":"https://www.govtrack.us/x"}]}"#,
        )
        .unwrap();
        let packed = serde_json::to_string(&ctx).unwrap();
        let resp = r#"{"choices":[{"message":{"content":"{\"headline\":\"MAGA-aligned incumbent\",\"overall\":{\"score\":80,\"label\":\"MAGA\",\"verdict\":\"Trump-aligned House member.\"},\"summary\":[{\"text\":\"Voted Yea on the border bill in 2024.\",\"cites\":[{\"id\":\"v0\"}]}],\"axes\":[{\"id\":\"maga\",\"score\":88,\"verdict\":\"Strong MAGA record.\",\"cites\":[{\"id\":\"v0\"}]},{\"id\":\"not_an_axis\",\"score\":99,\"cites\":[{\"id\":\"v0\"}]},{\"id\":\"neocon\",\"score\":40,\"verdict\":\"No cite so drop score.\"}],\"found\":[{\"kind\":\"endorsement\",\"org\":\"Trump\",\"stance\":\"support\",\"url\":\"https://example.com/trump\",\"trust\":\"news\"}]}"}}]}"#;
        let card = parse_verdict_card(resp, &packed, "grok-4-fast", "xai").unwrap();
        assert!(card.headline.contains("MAGA"));
        assert_eq!(card.overall.score, Some(80));
        assert!(card.axes.iter().any(|a| a.id == "maga" && a.score == Some(88)));
        assert!(card.axes.iter().all(|a| a.id != "not_an_axis"));
        let neo = card.axes.iter().find(|a| a.id == "neocon").unwrap();
        assert_eq!(neo.score, None);
        assert_eq!(card.found.len(), 1);
        let ends = found_endorsements_from_verdict(&card);
        assert_eq!(ends[0].org, "Trump");
        assert_eq!(ends[0].trust.as_deref(), Some("news"));
    }

    #[test]
    fn parse_rejects_bad_urls_and_forces_news_trust_on_found() {
        let ctx = pack_verdict_context(
            r#"{"name":"Pat Smith","party":"Democratic","office":"Senate"}"#,
            "{}",
        )
        .unwrap();
        let packed = serde_json::to_string(&ctx).unwrap();
        let resp = r#"{"output_text":"{\"headline\":\"Test\",\"axes\":[{\"id\":\"socialism\",\"score\":10,\"cites\":[{\"url\":\"javascript:alert(1)\"}]}],\"found\":[{\"kind\":\"endorsement\",\"org\":\"DSA\",\"url\":\"https://dsausa.org/\",\"trust\":\"filing\"}]}"}"#;
        let card = parse_verdict_card(resp, &packed, "grok-4.6", "xai").unwrap();
        let soc = card.axes.iter().find(|a| a.id == "socialism");
        assert!(soc.is_none() || soc.unwrap().score.is_none());
        assert_eq!(card.found[0].trust.as_deref(), Some("news"));
    }

    #[test]
    fn urls_and_models() {
        assert_eq!(
            verdict_responses_url("grok"),
            Some(VERDICT_XAI_RESPONSES_URL)
        );
        assert_eq!(
            verdict_default_model("xai"),
            Some(VERDICT_XAI_DEFAULT_MODEL)
        );
        assert_eq!(
            verdict_responses_url("openai"),
            Some(VERDICT_OPENAI_RESPONSES_URL)
        );
        assert!(verdict_responses_url("nope").is_none());
    }

    #[test]
    fn voter_profile_axes_cover_every_pack() {
        let axes = voter_profile_axes();
        let ids: HashSet<&str> = axes.iter().map(|a| a.id.as_str()).collect();
        for pack in [
            REPUBLICAN_AXES,
            DEMOCRAT_AXES,
            INDEPENDENT_AXES,
            JUDGE_AXES,
            MEASURE_AXES,
            ISSUE_AXES,
        ] {
            for (id, _, _) in pack {
                assert!(ids.contains(id), "missing profile axis {id}");
            }
        }
        assert!(!ids.iter().any(|id| id.eq_ignore_ascii_case("republican")));
        assert_eq!(ids.len(), voter_profile_axes().len());
    }

    #[test]
    fn invert_disliked_axis_and_aggregate_fit() {
        let card = VerdictCard {
            kind: "candidate".into(),
            headline: "Test".into(),
            overall: OverallScore {
                score: Some(80),
                label: Some("MAGA".into()),
                ..Default::default()
            },
            axes: vec![
                AxisScore {
                    id: "zionist".into(),
                    label: "Zionist".into(),
                    score: Some(80),
                    ..Default::default()
                },
                AxisScore {
                    id: "maga".into(),
                    label: "MAGA".into(),
                    score: Some(70),
                    ..Default::default()
                },
                AxisScore {
                    id: "trump".into(),
                    label: "Trump".into(),
                    score: Some(90),
                    ..Default::default()
                },
                AxisScore {
                    id: "neocon".into(),
                    label: "NeoCon".into(),
                    score: Some(40),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let fitted = apply_voter_profile(
            &card,
            r#"{"zionist":1,"maga":5,"trump":5,"neocon":1,"bush_era":1}"#,
        );
        let z = fitted.axes.iter().find(|a| a.id == "zionist").unwrap();
        assert_eq!(z.raw_score, Some(80));
        assert_eq!(z.score, Some(20));
        assert_eq!(z.inverted, Some(true));
        let maga = fitted.axes.iter().find(|a| a.id == "maga").unwrap();
        assert_eq!(maga.score, Some(70));
        assert_eq!(maga.inverted, Some(false));
        let neo = fitted.axes.iter().find(|a| a.id == "neocon").unwrap();
        assert_eq!(neo.score, Some(60));
        assert_eq!(fitted.overall.raw_score, Some(80));
        assert_eq!(fitted.overall.score, Some(60));
        assert_eq!(fitted.overall.profiled, Some(true));
        assert_eq!(fitted.overall.label.as_deref(), Some("Mixed fit"));
    }

    #[test]
    fn signed_tax_fit_and_neutral_skip() {
        assert_eq!(axis_fit("tax_direction", -70, 1), Some((85, 2)));
        assert_eq!(axis_fit("tax_direction", 80, 1), Some((10, 2)));
        assert_eq!(axis_fit("abortion", -80, 1), Some((90, 2)));
        assert_eq!(axis_fit("border", 80, 5), Some((90, 2)));
        assert_eq!(axis_fit("maga", 80, 3), None);
        let card = VerdictCard {
            kind: "measure".into(),
            axes: vec![AxisScore {
                id: "tax_direction".into(),
                label: "Tax".into(),
                score: Some(-70),
                ..Default::default()
            }],
            ..Default::default()
        };
        let none = apply_voter_profile(&card, r#"{"tax_direction":3}"#);
        assert_eq!(none.overall.profiled, None);
        assert_eq!(none.axes[0].score, Some(-70));
        let empty = apply_voter_profile(&card, "{}");
        assert_eq!(empty.axes[0].score, Some(-70));
    }

    #[test]
    fn packer_includes_voter_profile() {
        let enrich = r#"{"voter_profile":{"maga":5,"zionist":1}}"#;
        let ctx = pack_verdict_context(
            r#"{"name":"Byron Donalds","party":"Republican","office":"U.S. House"}"#,
            enrich,
        )
        .unwrap();
        assert!(ctx.voter_profile.iter().any(|p| p.id == "maga" && p.likert == 5));
        assert!(ctx
            .voter_profile
            .iter()
            .any(|p| p.id == "zionist" && p.likert == 1));
        let packed = serde_json::to_string(&ctx).unwrap();
        let body = verdict_request_body("xai", "grok-4.6", &packed, true).unwrap();
        assert!(body.contains("voter_profile"));
        assert!(body.contains("do not invert"));
    }

    #[test]
    fn measure_pack_and_signed_score() {
        let ctx = pack_verdict_context(
            r#"{"kind":"measure","name":"Amendment 3","title":"Amendment 3","measure_code":"A3","jurisdiction":"Florida","summary":"Homestead"}"#,
            r#"{"endorsements":[]}"#,
        )
        .unwrap();
        assert_eq!(ctx.kind, "measure");
        assert!(ctx.rubric.iter().any(|a| a.id == "tax_direction"));
        let packed = serde_json::to_string(&ctx).unwrap();
        let resp = r#"{"choices":[{"message":{"content":"{\"headline\":\"Tax cut\",\"overall\":{\"score\":20},\"summary\":[{\"text\":\"Lowers homestead tax burden.\",\"cites\":[{\"url\":\"https://dos.fl.gov/a3\"}]}],\"axes\":[{\"id\":\"tax_direction\",\"score\":-70,\"verdict\":\"Cuts property tax.\",\"cites\":[{\"url\":\"https://dos.fl.gov/a3\"}]}]}"}}]}"#;
        let card = parse_verdict_card(resp, &packed, "grok-4-fast", "xai").unwrap();
        let tax = card.axes.iter().find(|a| a.id == "tax_direction").unwrap();
        assert_eq!(tax.score, Some(-70));
    }
}
