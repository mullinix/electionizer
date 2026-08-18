//! Browser bindings: JS owns I/O; this crate only exposes pure core functions.

use electionizer_core::bio::{
    apply_holdings_to_dossier, apply_member_bio_fill_gaps, apply_member_bio_to_dossier,
    apply_photo_to_dossier, assess_career, assess_from_congress_legislators,
    assess_from_openstates_person_json, ballotpedia_campaign_website,
    ballotpedia_html_matches_person, ballotpedia_title_candidates, campaign_about_urls,
    dbpedia_describe_ntriples_url, dbpedia_ntriples_url, dbpedia_page_url, dossier_from_career,
    efd_abs_url, efd_split_person_name, empty_dossier, empty_not_found_copy,
    endorsements_from_ie_support, endorsements_from_measure_sides, federal_disclosure_portals,
    fl_bar_member_search_url, fl_courts_index_url, fl_judge_decision_portals,
    fl_judicial_opinion_portals, grokipedia_page_url, grokipedia_typeahead_url,
    house_clerk_abs_url, is_campaign_site_url, match_fl_courts_judge_link,
    match_grokipedia_typeahead, merge_career_spans, merge_endorsements, note_source_checked,
    parse_ballotpedia_member_html, parse_campaign_about_html, parse_dbpedia_ntriples,
    parse_efd_search_data_json, parse_fl_chamber_member_html, parse_fl_circuit_directory_links,
    parse_fl_circuit_wp_bio_html, parse_fl_courts_judge_html, parse_fl_courts_next_index,
    parse_fl_senate_member_html, parse_grokipedia_page_html, parse_house_clerk_fd_pdf,
    parse_house_clerk_fd_text, parse_house_clerk_search_html, parse_official_member_about_html,
    parse_senate_efd_annual_html, parse_wikidata_entity_bio, parse_wikipedia_extract_json,
    parse_wikipedia_summary_photo, pick_efd_annual_report, pick_house_clerk_fd_report,
    polish_dossier_empty_notes, unitedstates_congress_photo_url, wikidata_entity_url,
    wikidata_label_ids_needed, wikipedia_article_url, wikipedia_extract_api_url,
    wikipedia_summary_api_url, wikipedia_summary_match_person, CareerSpan, Endorsement,
    PersonDossier, PersonalHolding,
};
use electionizer_core::courtlistener::{
    courtlistener_court_ids, courtlistener_opinions_search_url, courtlistener_people_search_url,
    courtlistener_person_profile_url, courtlistener_positions_url, courtlistener_search_portal_url,
    opinions_from_search_json, opinions_search_total, person_positions_match_courts,
    pick_courtlistener_person, spans_and_facts_from_positions, CL_OPINIONS_CAP, CL_SOURCE,
};
use electionizer_core::fec::{
    bio_facts_from_sched_a_candidate, parse_ie_json, parse_principal_committee,
    parse_sched_a_aggregated, parse_sched_a_json, parse_size_json, parse_totals_json,
};
use electionizer_core::govtrack::{
    assemble_govtrack_votes, ballot_affiliations, ballotpedia_page_url, bioguide_profile_url,
    campaign_committee_affiliation, congress_gov_member_url, current_party_affiliation,
    match_legislator_by_fec, official_about_urls, parse_vote_voter_list, vote_ids_needing_detail,
    vote_voter_total_count,
};
use electionizer_core::openstates::{
    affiliations_from_openstates_people_json, district_from_office, extract_votes_for_person,
    looks_like_fec_id, merge_affiliation_spans, pick_person, state_code_from_jurisdiction,
    state_legislator_from_person_json, vote_sessions,
};
use electionizer_core::{
    ballot_report_from_federal_live_ex, ballot_report_from_fixture,
    ballot_report_from_live_with_state, normalize_zip, parse_zippo_json,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    #[cfg(all(feature = "console_error_panic_hook", target_arch = "wasm32"))]
    console_error_panic_hook::set_once();
}

fn to_js<T: Serialize>(v: &T) -> Result<JsValue, JsError> {
    // Maps/json! objects must be plain JS objects — default serde_wasm_bindgen
    // emits ES6 Map, which breaks `obj.field` access throughout enrich.js.
    let ser = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    v.serialize(&ser).map_err(|e| JsError::new(&e.to_string()))
}

/// Normalize a ZIP input to 5 digits, or throw if invalid.
#[wasm_bindgen]
pub fn normalize_zip_js(raw: &str) -> Result<String, JsError> {
    normalize_zip(raw).ok_or_else(|| JsError::new(&format!("invalid zip: {raw}")))
}

/// Parse fixture JSON and build a ballot report object (for JS).
/// `zip_override` may be empty to keep the fixture's zip.
#[wasm_bindgen]
pub fn build_report_from_fixture(
    fixture_json: &str,
    zip_override: &str,
) -> Result<JsValue, JsError> {
    let zip = zip_override.trim();
    let override_opt = if zip.is_empty() { None } else { Some(zip) };
    let report =
        ballot_report_from_fixture(fixture_json, override_opt).map_err(|e| JsError::new(&e))?;
    to_js(&report)
}

/// Same as `build_report_from_fixture` but returns JSON text.
#[wasm_bindgen]
pub fn build_report_from_fixture_json(
    fixture_json: &str,
    zip_override: &str,
) -> Result<String, JsError> {
    let zip = zip_override.trim();
    let override_opt = if zip.is_empty() { None } else { Some(zip) };
    let report =
        ballot_report_from_fixture(fixture_json, override_opt).map_err(|e| JsError::new(&e))?;
    serde_json::to_string(&report).map_err(|e| JsError::new(&e.to_string()))
}

/// Parse Zippopotam JSON → `{ city, state_abbr, state_name, longitude, latitude }`.
#[wasm_bindgen]
pub fn parse_zippo_js(zippo_json: &str) -> Result<JsValue, JsError> {
    let place = parse_zippo_json(zippo_json).map_err(|e| JsError::new(&e))?;
    to_js(&place)
}

/// Live federal ballot from response bodies.
/// `district_json`: Census geocoder **or** TIGERweb identify JSON.
/// `fcc_json`: optional FCC area JSON for county (pass `""` if unused).
#[wasm_bindgen]
pub fn build_federal_ballot_report(
    zip: &str,
    zippo_json: &str,
    district_json: &str,
    house_fec_json: &str,
    senate_fec_json: &str,
    cycle: i32,
) -> Result<JsValue, JsError> {
    build_federal_ballot_report_ex(
        zip,
        zippo_json,
        district_json,
        "",
        house_fec_json,
        senate_fec_json,
        cycle,
    )
}

/// Like `build_federal_ballot_report` with FCC area JSON for county name.
#[wasm_bindgen]
pub fn build_federal_ballot_report_ex(
    zip: &str,
    zippo_json: &str,
    district_json: &str,
    fcc_json: &str,
    house_fec_json: &str,
    senate_fec_json: &str,
    cycle: i32,
) -> Result<JsValue, JsError> {
    let report = ballot_report_from_federal_live_ex(
        zip,
        zippo_json,
        district_json,
        fcc_json,
        house_fec_json,
        senate_fec_json,
        cycle,
    )
    .map_err(|e| JsError::new(&e))?;
    to_js(&report)
}

/// Federal + optional state bodies.
/// `state_bodies_json`: JSON map — keys `fl:dos`, `fl:senate`, `fl:house`, `fl:measures`,
/// `fl:soe`, `fl:sample_ballot`, `az:senate`, `az:house`, `nc:candidates`, `md:statewide`, `md:local`,
/// `civic:voterinfo`, `os:people.geo`
/// (pass `""` or `"{}"` for federal only).
#[wasm_bindgen]
pub fn build_live_ballot_report(
    zip: &str,
    zippo_json: &str,
    district_json: &str,
    fcc_json: &str,
    house_fec_json: &str,
    senate_fec_json: &str,
    cycle: i32,
    state_bodies_json: &str,
) -> Result<JsValue, JsError> {
    let report = ballot_report_from_live_with_state(
        zip,
        zippo_json,
        district_json,
        fcc_json,
        house_fec_json,
        senate_fec_json,
        cycle,
        state_bodies_json,
    )
    .map_err(|e| JsError::new(&e))?;
    to_js(&report)
}

/// Extract ASP.NET antiforgery token from FL measures index HTML.
#[wasm_bindgen]
pub fn extract_verification_token_js(html: &str) -> Option<String> {
    electionizer_core::states::florida::extract_verification_token(html)
}

/// Parse FL DOS InitDetail HTML → ballot summary text (or null).
#[wasm_bindgen]
pub fn parse_fl_measure_summary_js(html: &str) -> Option<String> {
    electionizer_core::states::florida::parse_fl_measure_summary_html(html)
}

/// Stable IndexedDB key for a FL measure detail URL.
#[wasm_bindgen]
pub fn fl_measure_detail_cache_key_js(detail_url: &str) -> String {
    electionizer_core::states::florida::fl_measure_detail_cache_key(detail_url)
}

/// DOS committee account from InitDetail/ComDetail URL.
#[wasm_bindgen]
pub fn parse_dos_account_js(url: &str) -> Option<String> {
    electionizer_core::states::florida::parse_dos_account_from_url(url)
}

/// `fl:acct:{n}` external_id from DOS AcctNum (or null).
#[wasm_bindgen]
pub fn fl_acct_external_id_js(acct: &str) -> Option<String> {
    electionizer_core::states::florida::fl_acct_external_id(acct)
}

/// Account number from `fl:acct:{n}` (or null).
#[wasm_bindgen]
pub fn parse_fl_acct_external_id_js(id: &str) -> Option<String> {
    electionizer_core::states::florida::parse_fl_acct_external_id(id)
}

/// TreFin contributions URL for a DOS account.
#[wasm_bindgen]
pub fn fl_trefin_contrib_url_js(account: &str) -> String {
    electionizer_core::states::florida::fl_trefin_contrib_url(account)
}

/// Parse TreFin HTML → committee finance JSON (top donors aggregated).
#[wasm_bindgen]
pub fn parse_trefin_finance_js(
    html: &str,
    account: &str,
    limit: usize,
) -> Result<JsValue, JsError> {
    to_js(
        &electionizer_core::states::florida::parse_trefin_contributions_html(html, account, limit),
    )
}

/// DOS committee name-search POST URL.
#[wasm_bindgen]
pub fn fl_com_lkup_by_name_url_js() -> String {
    electionizer_core::states::florida::fl_com_lkup_by_name_url().to_string()
}

/// Form body for committee name search (containing).
#[wasm_bindgen]
pub fn fl_com_lkup_by_name_form_js(name: &str) -> String {
    electionizer_core::states::florida::fl_com_lkup_by_name_form(name)
}

/// Parse ComLkupByName HTML → committee hit list JSON.
#[wasm_bindgen]
pub fn parse_com_lkup_by_name_js(html: &str) -> Result<JsValue, JsError> {
    to_js(&electionizer_core::states::florida::parse_com_lkup_by_name_html(html))
}

/// Amendment number from measure_code (`Amendment 3` → 3), or null.
#[wasm_bindgen]
pub fn amendment_number_from_code_js(code: Option<String>) -> Option<u32> {
    electionizer_core::states::florida::amendment_number_from_code(code.as_deref())
}

// --- A2: FL contrib.exe name-search fallback ---

#[wasm_bindgen]
pub fn fl_contrib_url_js() -> String {
    electionizer_core::states::florida::fl_contrib_url().to_string()
}

#[wasm_bindgen]
pub fn fl_can_list_url_js() -> String {
    electionizer_core::states::florida::fl_can_list_url().to_string()
}

#[wasm_bindgen]
pub fn fl_can_list_form_js(elec_id: &str) -> String {
    electionizer_core::states::florida::fl_can_list_form(elec_id)
}

/// `application/x-www-form-urlencoded` body for candidate contribution totals (TSV).
#[wasm_bindgen]
pub fn fl_contrib_candidate_totals_form_js(
    election: &str,
    last_name: &str,
    first_name: &str,
    office_code: &str,
    district: &str,
) -> String {
    electionizer_core::states::florida::fl_contrib_candidate_totals_form(
        election,
        last_name,
        first_name,
        office_code,
        district,
    )
}

#[wasm_bindgen]
pub fn fl_gen_elec_id_js(cycle: i32) -> String {
    electionizer_core::states::florida::fl_gen_elec_id(cycle)
}

#[wasm_bindgen]
pub fn split_candidate_first_last_js(name: &str) -> Result<JsValue, JsError> {
    let (first, last) = electionizer_core::states::florida::split_candidate_first_last(name);
    to_js(&serde_json::json!({ "first": first, "last": last }))
}

#[wasm_bindgen]
pub fn office_code_from_ballot_js(chamber: &str, office: &str) -> Option<String> {
    electionizer_core::states::florida::office_code_from_ballot(chamber, office)
}

#[wasm_bindgen]
pub fn district_from_ballot_office_js(office: &str) -> Option<u32> {
    electionizer_core::states::florida::district_from_ballot_office(office)
}

#[wasm_bindgen]
pub fn parse_contrib_candidate_totals_js(body: &str) -> Result<JsValue, JsError> {
    let hits = if body.contains('\t') && body.to_ascii_lowercase().contains("candidate name") {
        electionizer_core::states::florida::parse_contrib_candidate_totals_tsv(body)
    } else {
        electionizer_core::states::florida::parse_contrib_candidate_totals_html(body)
    };
    to_js(&hits)
}

#[wasm_bindgen]
pub fn parse_can_list_js(html: &str) -> Result<JsValue, JsError> {
    to_js(&electionizer_core::states::florida::parse_can_list_html(
        html,
    ))
}

/// Strict name+office+district match against contrib totals hits.
/// `query` JSON: `{ name, office, chamber, party, district?, county? }`.
#[wasm_bindgen]
pub fn match_fl_contrib_candidate_js(
    hits_json: &str,
    query_json: &str,
) -> Result<JsValue, JsError> {
    let hits: Vec<electionizer_core::states::florida::FlContribCandidateHit> =
        serde_json::from_str(hits_json).map_err(|e| JsError::new(&e.to_string()))?;
    let qv: serde_json::Value =
        serde_json::from_str(query_json).map_err(|e| JsError::new(&e.to_string()))?;
    let q = electionizer_core::states::florida::FlNameSearchQuery {
        name: qv
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        office: qv
            .get("office")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        chamber: qv
            .get("chamber")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        party: qv
            .get("party")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        district: qv.get("district").and_then(|v| {
            v.as_u64()
                .map(|n| n as u32)
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        }),
        county: qv
            .get("county")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    };
    let m = electionizer_core::states::florida::match_fl_contrib_candidate(&hits, &q);
    to_js(&m)
}

/// Resolve unique CanDetail account from CanList hits + same query shape as contrib match.
#[wasm_bindgen]
pub fn match_fl_can_list_account_js(hits_json: &str, query_json: &str) -> Result<JsValue, JsError> {
    let hits: Vec<electionizer_core::states::florida::FlCanListHit> =
        serde_json::from_str(hits_json).map_err(|e| JsError::new(&e.to_string()))?;
    let qv: serde_json::Value =
        serde_json::from_str(query_json).map_err(|e| JsError::new(&e.to_string()))?;
    let q = electionizer_core::states::florida::FlNameSearchQuery {
        name: qv
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        office: qv
            .get("office")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        chamber: qv
            .get("chamber")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        party: qv
            .get("party")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        district: qv.get("district").and_then(|v| {
            v.as_u64()
                .map(|n| n as u32)
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        }),
        county: qv
            .get("county")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    };
    let acct = electionizer_core::states::florida::match_fl_can_list_account(&hits, &q);
    to_js(&acct)
}

/// Combine match + optional account into status object.
#[wasm_bindgen]
pub fn fl_name_search_result_js(
    match_json: &str,
    account: Option<String>,
) -> Result<JsValue, JsError> {
    let m: electionizer_core::states::florida::FlNameMatch =
        serde_json::from_str(match_json).map_err(|e| JsError::new(&e.to_string()))?;
    to_js(&electionizer_core::states::florida::fl_name_search_result(
        &m, account,
    ))
}

/// Select oppose PACs for an amendment number from a hits JSON array.
#[wasm_bindgen]
pub fn select_oppose_committees_js(
    hits_json: &str,
    amendment: u32,
    limit: usize,
) -> Result<JsValue, JsError> {
    let hits: Vec<electionizer_core::states::florida::FlCommitteeHit> =
        serde_json::from_str(hits_json).map_err(|e| JsError::new(&e.to_string()))?;
    to_js(&electionizer_core::states::florida::select_oppose_committees(&hits, amendment, limit))
}

// --- FollowTheMoney (A3) ---

#[wasm_bindgen]
pub fn ftm_data_year_js(cycle: i32) -> i32 {
    electionizer_core::ftm::ftm_data_year(cycle)
}

#[wasm_bindgen]
pub fn ftm_office_type_code_js(chamber: &str, office: &str) -> Option<String> {
    electionizer_core::ftm::ftm_office_type_code(chamber, office).map(|s| s.to_string())
}

#[wasm_bindgen]
pub fn ftm_candidates_url_js(
    api_key: &str,
    state: &str,
    year: i32,
    office_type: Option<String>,
) -> String {
    electionizer_core::ftm::ftm_candidates_url(api_key, state, year, office_type.as_deref())
}

#[wasm_bindgen]
pub fn ftm_top_donors_url_js(api_key: &str, candidate_id: &str, state: &str, year: i32) -> String {
    electionizer_core::ftm::ftm_top_donors_url(api_key, candidate_id, state, year)
}

#[wasm_bindgen]
pub fn ftm_profile_url_js(eid: &str) -> String {
    electionizer_core::ftm::ftm_profile_url(eid)
}

#[wasm_bindgen]
pub fn ftm_show_me_url_js(state: &str, year: i32, office_type: Option<String>) -> String {
    electionizer_core::ftm::ftm_show_me_url(state, year, office_type.as_deref())
}

#[wasm_bindgen]
pub fn ftm_error_message_js(json: &str) -> Option<String> {
    electionizer_core::ftm::ftm_error_message(json)
}

#[wasm_bindgen]
pub fn parse_ftm_candidate_records_js(json: &str) -> Result<JsValue, JsError> {
    to_js(&electionizer_core::ftm::parse_ftm_candidate_records(json).map_err(|e| JsError::new(&e))?)
}

#[wasm_bindgen]
pub fn parse_ftm_donor_records_js(
    json: &str,
    limit: usize,
    profile_url: &str,
) -> Result<JsValue, JsError> {
    to_js(
        &electionizer_core::ftm::parse_ftm_donor_records(json, limit, profile_url)
            .map_err(|e| JsError::new(&e))?,
    )
}

/// Match FTM candidate hits. query JSON: { name, office, chamber, party, district?, state }
#[wasm_bindgen]
pub fn match_ftm_candidate_js(hits_json: &str, query_json: &str) -> Result<JsValue, JsError> {
    let hits: Vec<electionizer_core::ftm::FtmCandidateHit> =
        serde_json::from_str(hits_json).map_err(|e| JsError::new(&e.to_string()))?;
    let qv: serde_json::Value =
        serde_json::from_str(query_json).map_err(|e| JsError::new(&e.to_string()))?;
    let q = electionizer_core::ftm::FtmMatchQuery {
        name: qv
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        office: qv
            .get("office")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        chamber: qv
            .get("chamber")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        party: qv
            .get("party")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        district: qv.get("district").and_then(|v| {
            v.as_u64()
                .map(|n| n as u32)
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        }),
        state: qv
            .get("state")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    };
    to_js(&electionizer_core::ftm::match_ftm_candidate(&hits, &q))
}

// --- FollowTheMoney ballot measures (E1) ---

#[wasm_bindgen]
pub fn ftm_measures_list_url_js(state: &str, year: i32) -> String {
    electionizer_core::ftm::ftm_measures_list_url(state, year)
}

#[wasm_bindgen]
pub fn ftm_measure_overview_url_js(eid: &str) -> String {
    electionizer_core::ftm::ftm_measure_overview_url(eid)
}

#[wasm_bindgen]
pub fn ftm_measure_committees_url_js(eid: &str, support: bool) -> String {
    electionizer_core::ftm::ftm_measure_committees_url(eid, support)
}

#[wasm_bindgen]
pub fn ftm_measure_donors_url_js(eid: &str, support: bool) -> String {
    electionizer_core::ftm::ftm_measure_donors_url(eid, support)
}

#[wasm_bindgen]
pub fn ftm_measure_show_me_url_js(eid: &str) -> String {
    electionizer_core::ftm::ftm_measure_show_me_url(eid)
}

#[wasm_bindgen]
pub fn parse_ftm_measures_list_html_js(html: &str) -> Result<JsValue, JsError> {
    to_js(&electionizer_core::ftm::parse_ftm_measures_list_html(html))
}

#[wasm_bindgen]
pub fn parse_ftm_measure_overview_html_js(html: &str) -> Result<JsValue, JsError> {
    to_js(
        &electionizer_core::ftm::parse_ftm_measure_overview_html(html)
            .ok_or_else(|| JsError::new("FTM measure overview parse failed"))?,
    )
}

#[wasm_bindgen]
pub fn parse_ftm_measure_entity_table_html_js(
    html: &str,
    limit: usize,
) -> Result<JsValue, JsError> {
    to_js(&electionizer_core::ftm::parse_ftm_measure_entity_table_html(html, limit))
}

/// Match ballot measure to FTM hits. Returns `{kind:"unique",hit}|{kind:"none"}|{kind:"ambiguous",count}`.
#[wasm_bindgen]
pub fn match_ftm_measure_js(
    hits_json: &str,
    measure_code: &str,
    title: &str,
) -> Result<JsValue, JsError> {
    let hits: Vec<electionizer_core::ftm::FtmMeasureHit> =
        serde_json::from_str(hits_json).map_err(|e| JsError::new(&e.to_string()))?;
    to_js(&electionizer_core::ftm::match_ftm_measure(
        &hits,
        measure_code,
        title,
    ))
}

/// Build measure finance from hit + overview + optional support donors / oppose committees JSON arrays.
#[wasm_bindgen]
pub fn ftm_measure_finance_from_parts_js(
    hit_json: &str,
    overview_json: &str,
    support_donors_json: &str,
    oppose_committees_json: &str,
    top_limit: usize,
) -> Result<JsValue, JsError> {
    let hit: electionizer_core::ftm::FtmMeasureHit =
        serde_json::from_str(hit_json).map_err(|e| JsError::new(&e.to_string()))?;
    let overview: electionizer_core::ftm::FtmMeasureOverview =
        serde_json::from_str(overview_json).map_err(|e| JsError::new(&e.to_string()))?;
    let support: Vec<electionizer_core::ftm::FtmMeasureCommittee> =
        if support_donors_json.trim().is_empty() {
            vec![]
        } else {
            serde_json::from_str(support_donors_json).map_err(|e| JsError::new(&e.to_string()))?
        };
    let oppose: Vec<electionizer_core::ftm::FtmMeasureCommittee> =
        if oppose_committees_json.trim().is_empty() {
            vec![]
        } else {
            serde_json::from_str(oppose_committees_json)
                .map_err(|e| JsError::new(&e.to_string()))?
        };
    to_js(&electionizer_core::ftm::ftm_measure_finance_from_parts(
        &hit, &overview, &support, &oppose, top_limit,
    ))
}

/// Build FTM finance JSON from a unique candidate hit + optional raw donors API body.
#[wasm_bindgen]
pub fn ftm_finance_from_hit_js(
    hit_json: &str,
    state: &str,
    year: i32,
    office_type: Option<String>,
    donors_api_json: &str,
) -> Result<JsValue, JsError> {
    let hit: electionizer_core::ftm::FtmCandidateHit =
        serde_json::from_str(hit_json).map_err(|e| JsError::new(&e.to_string()))?;
    let profile = electionizer_core::ftm::ftm_profile_url(&hit.id);
    let top = if donors_api_json.trim().is_empty() {
        vec![]
    } else {
        electionizer_core::ftm::parse_ftm_donor_records(donors_api_json, 50, &profile)
            .unwrap_or_default()
    };
    to_js(&electionizer_core::ftm::ftm_finance_from_hit(
        &hit,
        state,
        year,
        office_type.as_deref(),
        top,
    ))
}

// --- AZ SeeTheMoney campaign finance (A4) ---

#[wasm_bindgen]
pub fn az_cf_candidates_url_js(
    start_year: i32,
    end_year: i32,
    name: &str,
    office_id: Option<u32>,
) -> String {
    electionizer_core::states::arizona::az_cf_candidates_url(start_year, end_year, name, office_id)
}

#[wasm_bindgen]
pub fn az_cf_datatables_body_js() -> String {
    electionizer_core::states::arizona::az_cf_datatables_body()
}

#[wasm_bindgen]
pub fn az_cf_search_name_fragment_js(ballot_name: &str) -> String {
    electionizer_core::states::arizona::az_cf_search_name_fragment(ballot_name)
}

#[wasm_bindgen]
pub fn az_cf_office_id_js(chamber: &str, district: Option<u32>) -> Option<u32> {
    electionizer_core::states::arizona::az_cf_office_id(chamber, district)
}

#[wasm_bindgen]
pub fn parse_az_cf_table_json_js(body: &str) -> Result<JsValue, JsError> {
    to_js(
        &electionizer_core::states::arizona::parse_az_cf_table_json(body)
            .map_err(|e| JsError::new(&e))?,
    )
}

/// Match AZ CF hits. query JSON: { name, office, chamber, party, district? }
#[wasm_bindgen]
pub fn match_az_cf_candidate_js(hits_json: &str, query_json: &str) -> Result<JsValue, JsError> {
    let hits: Vec<electionizer_core::states::arizona::AzCfHit> =
        serde_json::from_str(hits_json).map_err(|e| JsError::new(&e.to_string()))?;
    let qv: serde_json::Value =
        serde_json::from_str(query_json).map_err(|e| JsError::new(&e.to_string()))?;
    let q = electionizer_core::states::arizona::AzCfMatchQuery {
        name: qv
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        office: qv
            .get("office")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        chamber: qv
            .get("chamber")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        party: qv
            .get("party")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        district: qv.get("district").and_then(|v| {
            v.as_u64()
                .map(|n| n as u32)
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        }),
    };
    to_js(&electionizer_core::states::arizona::match_az_cf_candidate(
        &hits, &q,
    ))
}

#[wasm_bindgen]
pub fn az_cf_finance_from_hit_js(hit_json: &str, cycle: i32) -> Result<JsValue, JsError> {
    let hit: electionizer_core::states::arizona::AzCfHit =
        serde_json::from_str(hit_json).map_err(|e| JsError::new(&e.to_string()))?;
    to_js(&electionizer_core::states::arizona::az_cf_finance_from_hit(
        &hit, cycle,
    ))
}

// --- NC referendum list PDF (C2) ---

/// Extract plain text from an NCSBE referendum PDF (bytes → text for `nc:measures`).
#[wasm_bindgen]
pub fn extract_pdf_text_js(data: &[u8]) -> Result<String, JsError> {
    electionizer_core::states::north_carolina::extract_pdf_text(data).map_err(|e| JsError::new(&e))
}

// --- NC campaign finance (A4) ---

#[wasm_bindgen]
pub fn nc_cf_search_url_js() -> String {
    electionizer_core::states::north_carolina::NC_CF_SEARCH_URL.to_string()
}

#[wasm_bindgen]
pub fn nc_cf_committee_search_form_js(name: &str) -> String {
    electionizer_core::states::north_carolina::nc_cf_committee_search_form(name)
}

#[wasm_bindgen]
pub fn nc_cf_search_name_fragment_js(ballot_name: &str) -> String {
    electionizer_core::states::north_carolina::nc_cf_search_name_fragment(ballot_name)
}

#[wasm_bindgen]
pub fn nc_cf_documents_url_js(sboe_id: &str, org_group_id: u32) -> String {
    electionizer_core::states::north_carolina::nc_cf_documents_url(sboe_id, org_group_id)
}

#[wasm_bindgen]
pub fn nc_cf_summary_csv_url_js(report_id: &str) -> String {
    electionizer_core::states::north_carolina::nc_cf_summary_csv_url(report_id)
}

#[wasm_bindgen]
pub fn parse_nc_cf_committee_search_html_js(html: &str) -> Result<JsValue, JsError> {
    to_js(
        &electionizer_core::states::north_carolina::parse_nc_cf_committee_search_html(html)
            .map_err(|e| JsError::new(&e))?,
    )
}

#[wasm_bindgen]
pub fn parse_nc_cf_documents_html_js(html: &str) -> Result<JsValue, JsError> {
    to_js(
        &electionizer_core::states::north_carolina::parse_nc_cf_documents_html(html)
            .map_err(|e| JsError::new(&e))?,
    )
}

#[wasm_bindgen]
pub fn pick_latest_nc_disclosure_js(docs_json: &str, cycle: i32) -> Result<JsValue, JsError> {
    let docs: Vec<electionizer_core::states::north_carolina::NcDisclosureReport> =
        serde_json::from_str(docs_json).map_err(|e| JsError::new(&e.to_string()))?;
    match electionizer_core::states::north_carolina::pick_latest_disclosure_report(&docs, cycle) {
        Some(r) => to_js(r),
        None => Ok(JsValue::NULL),
    }
}

#[wasm_bindgen]
pub fn parse_nc_cf_summary_csv_js(csv: &str) -> Result<JsValue, JsError> {
    let sum = electionizer_core::states::north_carolina::parse_nc_cf_summary_csv(csv)
        .map_err(|e| JsError::new(&e))?;
    // Serialize fields JS expects
    to_js(&serde_json::json!({
        "receipts_cycle": sum.receipts_cycle,
        "expenditures_cycle": sum.expenditures_cycle,
        "cash_on_hand": sum.cash_on_hand,
        "cash_beginning_cycle": sum.cash_beginning_cycle,
    }))
}

/// Match NC CF committees. query JSON: { name, office, chamber, party }
#[wasm_bindgen]
pub fn match_nc_cf_committee_js(hits_json: &str, query_json: &str) -> Result<JsValue, JsError> {
    let hits: Vec<electionizer_core::states::north_carolina::NcCommitteeHit> =
        serde_json::from_str(hits_json).map_err(|e| JsError::new(&e.to_string()))?;
    let qv: serde_json::Value =
        serde_json::from_str(query_json).map_err(|e| JsError::new(&e.to_string()))?;
    let q = electionizer_core::states::north_carolina::NcCfMatchQuery {
        name: qv
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        office: qv
            .get("office")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        chamber: qv
            .get("chamber")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        party: qv
            .get("party")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    };
    to_js(&electionizer_core::states::north_carolina::match_nc_cf_committee(&hits, &q))
}

#[wasm_bindgen]
pub fn nc_cf_finance_from_parts_js(
    hit_json: &str,
    summary_json: &str,
    report_json: &str,
    cycle: i32,
) -> Result<JsValue, JsError> {
    let hit: electionizer_core::states::north_carolina::NcCommitteeHit =
        serde_json::from_str(hit_json).map_err(|e| JsError::new(&e.to_string()))?;
    let sv: serde_json::Value =
        serde_json::from_str(summary_json).map_err(|e| JsError::new(&e.to_string()))?;
    let sum = electionizer_core::states::north_carolina::NcCfSummary {
        receipts_cycle: sv.get("receipts_cycle").and_then(|v| v.as_f64()),
        expenditures_cycle: sv.get("expenditures_cycle").and_then(|v| v.as_f64()),
        cash_on_hand: sv.get("cash_on_hand").and_then(|v| v.as_f64()),
        cash_beginning_cycle: sv.get("cash_beginning_cycle").and_then(|v| v.as_f64()),
    };
    let report: electionizer_core::states::north_carolina::NcDisclosureReport =
        serde_json::from_str(report_json).map_err(|e| JsError::new(&e.to_string()))?;
    to_js(
        &electionizer_core::states::north_carolina::nc_cf_finance_from_hit(
            &hit, &sum, &report, cycle,
        ),
    )
}

// --- FL county SOE / VoterFocus (A5) ---

#[wasm_bindgen]
pub fn fl_soe_candidate_list_url_js(county: &str) -> Option<String> {
    electionizer_core::states::florida_soe::fl_soe_candidate_list_url(county)
}

#[wasm_bindgen]
pub fn fl_soe_contact_url_js(county: &str) -> String {
    electionizer_core::states::florida_soe::fl_soe_contact_url(county)
}

#[wasm_bindgen]
pub fn voterfocus_county_param_js(county: &str) -> Option<String> {
    electionizer_core::states::florida_soe::voterfocus_county_param(county).map(|s| s.to_string())
}

#[wasm_bindgen]
pub fn parse_fl_soe_candidate_list_html_js(html: &str) -> Result<JsValue, JsError> {
    to_js(
        &electionizer_core::states::florida_soe::parse_fl_soe_candidate_list_html(html)
            .map_err(|e| JsError::new(&e))?,
    )
}

/// Match FL SOE hits. query JSON: { name, office, chamber, party, district? }
#[wasm_bindgen]
pub fn match_fl_soe_candidate_js(hits_json: &str, query_json: &str) -> Result<JsValue, JsError> {
    let hits: Vec<electionizer_core::states::florida_soe::FlSoeHit> =
        serde_json::from_str(hits_json).map_err(|e| JsError::new(&e.to_string()))?;
    let qv: serde_json::Value =
        serde_json::from_str(query_json).map_err(|e| JsError::new(&e.to_string()))?;
    let q = electionizer_core::states::florida_soe::FlSoeMatchQuery {
        name: qv
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        office: qv
            .get("office")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        chamber: qv
            .get("chamber")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        party: qv
            .get("party")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        district: qv.get("district").and_then(|v| {
            v.as_u64()
                .map(|n| n as u32)
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        }),
    };
    to_js(&electionizer_core::states::florida_soe::match_fl_soe_candidate(&hits, &q))
}

#[wasm_bindgen]
pub fn fl_soe_finance_from_hit_js(
    hit_json: &str,
    cycle: i32,
    county_label: &str,
) -> Result<JsValue, JsError> {
    let hit: electionizer_core::states::florida_soe::FlSoeHit =
        serde_json::from_str(hit_json).map_err(|e| JsError::new(&e.to_string()))?;
    to_js(
        &electionizer_core::states::florida_soe::fl_soe_finance_from_hit(&hit, cycle, county_label),
    )
}

// --- MD MDCRIS campaign finance (A4) ---

#[wasm_bindgen]
pub fn md_cf_committee_list_url_js() -> String {
    electionizer_core::states::maryland::md_cf_committee_list_url()
}

#[wasm_bindgen]
pub fn md_cf_financial_summary_url_js() -> String {
    electionizer_core::states::maryland::md_cf_financial_summary_url()
}

#[wasm_bindgen]
pub fn md_cf_committee_list_body_js(filer_name: &str, page_size: u32) -> String {
    electionizer_core::states::maryland::md_cf_committee_list_body(filer_name, page_size)
}

#[wasm_bindgen]
pub fn md_cf_financial_summary_body_js(filer_registration_guid: &str) -> String {
    electionizer_core::states::maryland::md_cf_financial_summary_body(filer_registration_guid)
}

#[wasm_bindgen]
pub fn md_cf_search_name_fragment_js(ballot_name: &str) -> String {
    electionizer_core::states::maryland::md_cf_search_name_fragment(ballot_name)
}

#[wasm_bindgen]
pub fn md_cf_profile_url_js(filer_registration_guid: &str) -> String {
    electionizer_core::states::maryland::md_cf_profile_url(filer_registration_guid)
}

#[wasm_bindgen]
pub fn parse_md_cf_committee_list_json_js(body: &str) -> Result<JsValue, JsError> {
    to_js(
        &electionizer_core::states::maryland::parse_md_cf_committee_list_json(body)
            .map_err(|e| JsError::new(&e))?,
    )
}

#[wasm_bindgen]
pub fn parse_md_cf_financial_summary_json_js(body: &str) -> Result<JsValue, JsError> {
    to_js(
        &electionizer_core::states::maryland::parse_md_cf_financial_summary_json(body)
            .map_err(|e| JsError::new(&e))?,
    )
}

/// Match MD CF hits. query JSON: { name, office, chamber, party, district?, county? }
#[wasm_bindgen]
pub fn match_md_cf_candidate_js(hits_json: &str, query_json: &str) -> Result<JsValue, JsError> {
    let hits: Vec<electionizer_core::states::maryland::MdCfHit> =
        serde_json::from_str(hits_json).map_err(|e| JsError::new(&e.to_string()))?;
    let qv: serde_json::Value =
        serde_json::from_str(query_json).map_err(|e| JsError::new(&e.to_string()))?;
    let q = electionizer_core::states::maryland::MdCfMatchQuery {
        name: qv
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        office: qv
            .get("office")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        chamber: qv
            .get("chamber")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        party: qv
            .get("party")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        district: qv.get("district").and_then(|v| {
            v.as_u64()
                .map(|n| n as u32)
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        }),
        county: qv
            .get("county")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    };
    to_js(&electionizer_core::states::maryland::match_md_cf_candidate(
        &hits, &q,
    ))
}

#[wasm_bindgen]
pub fn md_cf_finance_from_hit_js(
    hit_json: &str,
    cycle: i32,
    summary_json: Option<String>,
) -> Result<JsValue, JsError> {
    let hit: electionizer_core::states::maryland::MdCfHit =
        serde_json::from_str(hit_json).map_err(|e| JsError::new(&e.to_string()))?;
    let summary = if let Some(s) = summary_json.as_deref() {
        if s.trim().is_empty() {
            None
        } else {
            Some(
                electionizer_core::states::maryland::parse_md_cf_financial_summary_json(s)
                    .map_err(|e| JsError::new(&e))?,
            )
        }
    } else {
        None
    };
    to_js(
        &electionizer_core::states::maryland::md_cf_finance_from_hit(&hit, cycle, summary.as_ref()),
    )
}

/// Search terms for MDCRIS ballot-issue committee harvest.
#[wasm_bindgen]
pub fn md_measure_search_terms_js(
    measure_code: &str,
    title: &str,
    cycle: i32,
) -> Result<JsValue, JsError> {
    to_js(&electionizer_core::states::maryland::md_measure_search_terms(measure_code, title, cycle))
}

/// Build measure finance from MDCRIS committee hits JSON for one ballot row.
#[wasm_bindgen]
pub fn md_measure_finance_from_hits_js(
    hits_json: &str,
    measure_code: &str,
    title: &str,
    cycle: i32,
) -> Result<JsValue, JsError> {
    let hits: Vec<electionizer_core::states::maryland::MdCfHit> =
        serde_json::from_str(hits_json).map_err(|e| JsError::new(&e.to_string()))?;
    match electionizer_core::states::maryland::md_measure_finance_from_hits(
        &hits,
        measure_code,
        title,
        cycle,
    ) {
        Some(fin) => to_js(&fin),
        None => Ok(JsValue::NULL),
    }
}

// --- Detail enrich parsers ---

#[wasm_bindgen]
pub fn looks_like_fec_id_js(id: &str) -> bool {
    looks_like_fec_id(id)
}

#[wasm_bindgen]
pub fn parse_fec_totals_js(json: &str, candidate_id: &str, cycle: i32) -> Result<JsValue, JsError> {
    to_js(&parse_totals_json(json, candidate_id, cycle))
}

#[wasm_bindgen]
pub fn parse_fec_ie_js(json: &str) -> Result<JsValue, JsError> {
    to_js(&parse_ie_json(json))
}

/// Schedule A lines matching the candidate name → occupation/employer bio facts.
#[wasm_bindgen]
pub fn fec_occupation_facts_from_sched_a_js(
    json: &str,
    candidate_name: &str,
) -> Result<JsValue, JsError> {
    to_js(&bio_facts_from_sched_a_candidate(json, candidate_name))
}

#[wasm_bindgen]
pub fn parse_fec_sched_a_js(json: &str, limit: usize) -> Result<JsValue, JsError> {
    // Aggregate by contributor name (unique-donor totals from itemized lines).
    to_js(&parse_sched_a_aggregated(json, limit))
}

/// Single Schedule A lines (no aggregation) — for debugging / tests.
#[wasm_bindgen]
pub fn parse_fec_sched_a_lines_js(json: &str, limit: usize) -> Result<JsValue, JsError> {
    to_js(&parse_sched_a_json(json, limit))
}

#[wasm_bindgen]
pub fn parse_fec_size_js(json: &str) -> Result<JsValue, JsError> {
    to_js(&parse_size_json(json))
}

#[wasm_bindgen]
pub fn parse_fec_principal_js(json: &str) -> Result<JsValue, JsError> {
    to_js(&parse_principal_committee(json))
}

#[wasm_bindgen]
pub fn match_legislator_by_fec_js(
    legislators_json: &str,
    fec_id: &str,
) -> Result<JsValue, JsError> {
    to_js(&match_legislator_by_fec(legislators_json, fec_id))
}

/// Empty person dossier shell (honest empty-states).
#[wasm_bindgen]
pub fn empty_dossier_js(as_of_year: i32) -> Result<JsValue, JsError> {
    to_js(&empty_dossier(as_of_year))
}

/// Career assessment from congress-legislators JSON + FEC id.
#[wasm_bindgen]
pub fn assess_career_from_cl_js(
    legislators_json: &str,
    fec_id: &str,
    as_of_year: i32,
) -> Result<JsValue, JsError> {
    to_js(&assess_from_congress_legislators(
        legislators_json,
        fec_id,
        as_of_year,
    ))
}

/// Career assessment + optional photo from Open States person JSON object.
#[wasm_bindgen]
pub fn assess_career_from_os_person_js(
    person_json: &str,
    as_of_year: i32,
) -> Result<JsValue, JsError> {
    to_js(&assess_from_openstates_person_json(person_json, as_of_year))
}

/// Build dossier from a CareerAssessment JSON (+ optional photo).
#[wasm_bindgen]
pub fn dossier_from_career_js(
    career_json: &str,
    photo_url: Option<String>,
    photo_source: Option<String>,
    photo_source_url: Option<String>,
) -> Result<JsValue, JsError> {
    let career: electionizer_core::bio::CareerAssessment =
        serde_json::from_str(career_json).map_err(|e| JsError::new(&e.to_string()))?;
    to_js(&dossier_from_career(
        career,
        photo_url,
        photo_source,
        photo_source_url,
    ))
}

/// Merge two CareerSpan JSON arrays.
#[wasm_bindgen]
pub fn merge_career_spans_js(base_json: &str, extra_json: &str) -> Result<JsValue, JsError> {
    let base: Vec<CareerSpan> =
        serde_json::from_str(base_json).map_err(|e| JsError::new(&e.to_string()))?;
    let extra: Vec<CareerSpan> =
        serde_json::from_str(extra_json).map_err(|e| JsError::new(&e.to_string()))?;
    to_js(&merge_career_spans(&base, &extra))
}

/// Re-assess from merged spans JSON + optional birth year.
#[wasm_bindgen]
pub fn assess_career_js(
    spans_json: &str,
    birth_year: Option<i32>,
    as_of_year: i32,
) -> Result<JsValue, JsError> {
    let spans: Vec<CareerSpan> =
        serde_json::from_str(spans_json).map_err(|e| JsError::new(&e.to_string()))?;
    to_js(&assess_career(&spans, birth_year, as_of_year))
}

/// FEC IE support/oppose row → endorsement stub (or null).
#[wasm_bindgen]
pub fn endorsement_from_ie_js(
    committee: &str,
    support_oppose: &str,
    url: &str,
) -> Result<JsValue, JsError> {
    to_js(&endorsements_from_ie_support(
        committee,
        support_oppose,
        url,
    ))
}

/// unitedstates/images congress headshot URL for a Bioguide id (no fetch).
#[wasm_bindgen]
pub fn unitedstates_congress_photo_url_js(bioguide: &str) -> Result<JsValue, JsError> {
    to_js(&unitedstates_congress_photo_url(bioguide))
}

/// Wikidata entity page URL for a Q-id.
#[wasm_bindgen]
pub fn wikidata_entity_url_js(qid: &str) -> Result<JsValue, JsError> {
    to_js(&wikidata_entity_url(qid))
}

/// Q-ids referenced by a Wikidata entity body (for label fetch).
#[wasm_bindgen]
pub fn wikidata_label_ids_needed_js(entity_json: &str) -> Result<JsValue, JsError> {
    to_js(&wikidata_label_ids_needed(entity_json))
}

/// Parse Wikidata entity JSON + labels map → MemberBioParse.
#[wasm_bindgen]
pub fn parse_wikidata_entity_bio_js(
    entity_json: &str,
    labels_json: &str,
) -> Result<JsValue, JsError> {
    to_js(&parse_wikidata_entity_bio(entity_json, labels_json))
}

/// Ballotpedia person page URL from CL `id.ballotpedia` title.
#[wasm_bindgen]
pub fn ballotpedia_page_url_js(title: &str) -> Result<JsValue, JsError> {
    to_js(&ballotpedia_page_url(title))
}

/// Parse Ballotpedia person HTML → MemberBioParse (photo + facts + spans).
#[wasm_bindgen]
pub fn parse_ballotpedia_member_html_js(html: &str, page_url: &str) -> Result<JsValue, JsError> {
    to_js(&parse_ballotpedia_member_html(html, page_url))
}

/// Challenger Ballotpedia title guesses from name + state + chamber/office.
#[wasm_bindgen]
pub fn ballotpedia_title_candidates_js(
    name: &str,
    state_code: &str,
    chamber: &str,
    office: &str,
) -> Result<JsValue, JsError> {
    let st = state_code.trim();
    let ch = chamber.trim();
    let off = office.trim();
    to_js(&ballotpedia_title_candidates(
        name,
        if st.is_empty() { None } else { Some(st) },
        if ch.is_empty() { None } else { Some(ch) },
        if off.is_empty() { None } else { Some(off) },
    ))
}

/// High-precision check that BP HTML is this person (optional state).
#[wasm_bindgen]
pub fn ballotpedia_html_matches_person_js(
    html: &str,
    person_name: &str,
    state_code: &str,
) -> Result<bool, JsError> {
    let st = state_code.trim();
    Ok(ballotpedia_html_matches_person(
        html,
        person_name,
        if st.is_empty() { None } else { Some(st) },
    ))
}

/// Campaign website URL from Ballotpedia Contact row, if any.
#[wasm_bindgen]
pub fn ballotpedia_campaign_website_js(html: &str) -> Result<JsValue, JsError> {
    to_js(&ballotpedia_campaign_website(html))
}

/// True when URL looks like a candidate campaign site.
#[wasm_bindgen]
pub fn is_campaign_site_url_js(url: &str) -> bool {
    is_campaign_site_url(url)
}

/// About path candidates on a campaign site.
#[wasm_bindgen]
pub fn campaign_about_urls_js(site_url: &str) -> Result<JsValue, JsError> {
    to_js(&campaign_about_urls(site_url))
}

/// Parse campaign About/homepage HTML → MemberBioParse.
#[wasm_bindgen]
pub fn parse_campaign_about_html_js(html: &str, page_url: &str) -> Result<JsValue, JsError> {
    to_js(&parse_campaign_about_html(html, page_url))
}

/// Candidate About URLs for a member house.gov / senate.gov site.
#[wasm_bindgen]
pub fn official_about_urls_js(site_url: &str) -> Result<JsValue, JsError> {
    to_js(&official_about_urls(site_url))
}

/// Parse official House/Senate About HTML → MemberBioParse.
#[wasm_bindgen]
pub fn parse_official_member_about_html_js(html: &str, page_url: &str) -> Result<JsValue, JsError> {
    to_js(&parse_official_member_about_html(html, page_url))
}

/// DBpedia `/data/{Title}.ntriples` URL.
#[wasm_bindgen]
pub fn dbpedia_ntriples_url_js(title: &str) -> Result<JsValue, JsError> {
    to_js(&dbpedia_ntriples_url(title))
}

/// DBpedia SPARQL DESCRIBE N-Triples fallback URL.
#[wasm_bindgen]
pub fn dbpedia_describe_ntriples_url_js(title: &str) -> Result<JsValue, JsError> {
    to_js(&dbpedia_describe_ntriples_url(title))
}

/// DBpedia human page URL.
#[wasm_bindgen]
pub fn dbpedia_page_url_js(title: &str) -> Result<JsValue, JsError> {
    to_js(&dbpedia_page_url(title))
}

/// Parse DBpedia N-Triples → MemberBioParse (spouse/education/birth gaps).
#[wasm_bindgen]
pub fn parse_dbpedia_ntriples_js(nt: &str, title: &str) -> Result<JsValue, JsError> {
    to_js(&parse_dbpedia_ntriples(nt, title))
}

/// Grokipedia typeahead API URL (needs Wisp — no CORS).
#[wasm_bindgen]
pub fn grokipedia_typeahead_url_js(query: &str) -> Result<JsValue, JsError> {
    to_js(&grokipedia_typeahead_url(query))
}

/// Grokipedia page URL from slug.
#[wasm_bindgen]
pub fn grokipedia_page_url_js(slug: &str) -> Result<JsValue, JsError> {
    to_js(&grokipedia_page_url(slug))
}

/// Unique high-precision typeahead hit, or null if ambiguous/none.
#[wasm_bindgen]
pub fn match_grokipedia_typeahead_js(json: &str, person_name: &str) -> Result<JsValue, JsError> {
    match match_grokipedia_typeahead(json, person_name) {
        Some(hit) => to_js(&hit),
        None => Ok(JsValue::NULL),
    }
}

/// Parse Grokipedia page HTML → MemberBioParse (no family/citizenship).
#[wasm_bindgen]
pub fn parse_grokipedia_page_html_js(html: &str, page_url: &str) -> Result<JsValue, JsError> {
    to_js(&parse_grokipedia_page_html(html, page_url))
}

/// Wikipedia REST summary API URL for a page title.
#[wasm_bindgen]
pub fn wikipedia_summary_api_url_js(title: &str) -> Result<JsValue, JsError> {
    to_js(&wikipedia_summary_api_url(title))
}

/// MediaWiki plain-extract API URL for a page title.
#[wasm_bindgen]
pub fn wikipedia_extract_api_url_js(title: &str) -> Result<JsValue, JsError> {
    to_js(&wikipedia_extract_api_url(title))
}

/// Wikipedia article URL for a page title.
#[wasm_bindgen]
pub fn wikipedia_article_url_js(title: &str) -> Result<JsValue, JsError> {
    to_js(&wikipedia_article_url(title))
}

/// Parse Wikipedia REST summary JSON → `{ photo_url, page_url }` or null.
#[wasm_bindgen]
pub fn parse_wikipedia_summary_photo_js(json: &str) -> Result<JsValue, JsError> {
    match parse_wikipedia_summary_photo(json) {
        Some((photo, page)) => to_js(&serde_json::json!({
            "photo_url": photo,
            "page_url": page,
        })),
        None => Ok(JsValue::NULL),
    }
}

/// Soft person match on Wikipedia REST summary → page title or null (J3 name guess).
#[wasm_bindgen]
pub fn wikipedia_summary_match_person_js(
    json: &str,
    person_name: &str,
    state_code: &str,
    office: &str,
) -> Result<JsValue, JsError> {
    let st = state_code.trim();
    let off = office.trim();
    to_js(&wikipedia_summary_match_person(
        json,
        person_name,
        if st.is_empty() { None } else { Some(st) },
        if off.is_empty() { None } else { Some(off) },
    ))
}

/// Parse MediaWiki extracts JSON → MemberBioParse.
#[wasm_bindgen]
pub fn parse_wikipedia_extract_json_js(json: &str) -> Result<JsValue, JsError> {
    to_js(&parse_wikipedia_extract_json(json))
}

/// Merge bio into dossier gaps only (no clobber of stronger sources).
#[wasm_bindgen]
pub fn apply_member_bio_fill_gaps_js(
    dossier_json: &str,
    bio_json: &str,
    as_of_year: i32,
) -> Result<JsValue, JsError> {
    let mut d: PersonDossier =
        serde_json::from_str(dossier_json).map_err(|e| JsError::new(&e.to_string()))?;
    let bio: electionizer_core::bio::MemberBioParse =
        serde_json::from_str(bio_json).map_err(|e| JsError::new(&e.to_string()))?;
    apply_member_bio_fill_gaps(&mut d, &bio, as_of_year);
    to_js(&d)
}

/// Record a bio host consulted (I4 empty-state).
#[wasm_bindgen]
pub fn note_source_checked_js(dossier_json: &str, label: &str) -> Result<JsValue, JsError> {
    let mut d: PersonDossier =
        serde_json::from_str(dossier_json).map_err(|e| JsError::new(&e.to_string()))?;
    note_source_checked(&mut d, label);
    to_js(&d)
}

/// “Checked X / Y — not found.” from sources_checked JSON string array.
#[wasm_bindgen]
pub fn empty_not_found_copy_js(sources_json: &str) -> String {
    let list: Vec<String> = serde_json::from_str(sources_json).unwrap_or_default();
    empty_not_found_copy(&list)
}

/// Polish empty_notes using sources_checked (I4).
#[wasm_bindgen]
pub fn polish_dossier_empty_notes_js(dossier_json: &str) -> Result<JsValue, JsError> {
    let mut d: PersonDossier =
        serde_json::from_str(dossier_json).map_err(|e| JsError::new(&e.to_string()))?;
    polish_dossier_empty_notes(&mut d);
    to_js(&d)
}

/// Parse FL Senate member page HTML → photo + facts + career spans.
#[wasm_bindgen]
pub fn parse_fl_senate_member_html_js(html: &str, page_url: &str) -> Result<JsValue, JsError> {
    to_js(&parse_fl_senate_member_html(html, page_url))
}

/// Parse FL Senate or House member page by URL host/path.
#[wasm_bindgen]
pub fn parse_fl_chamber_member_html_js(html: &str, page_url: &str) -> Result<JsValue, JsError> {
    to_js(&parse_fl_chamber_member_html(html, page_url))
}

/// FL courts judges index URL for ballot office (SC / DCA / known circuits).
#[wasm_bindgen]
pub fn fl_courts_index_url_js(office: &str) -> Result<JsValue, JsError> {
    to_js(&fl_courts_index_url(office))
}

/// Florida Bar member search portal URL (name → fn/ln query).
#[wasm_bindgen]
pub fn fl_bar_member_search_url_js(name: &str) -> Result<JsValue, JsError> {
    to_js(&fl_bar_member_search_url(name))
}

/// FL judge Decisions-tab portals (directory + Bar + opinions) — link-out only.
#[wasm_bindgen]
pub fn fl_judge_decision_portals_js(office: &str, person_name: &str) -> Result<JsValue, JsError> {
    to_js(&fl_judge_decision_portals(office, person_name))
}

/// FL judicial opinion / court-records portals for a ballot office.
#[wasm_bindgen]
pub fn fl_judicial_opinion_portals_js(office: &str) -> Result<JsValue, JsError> {
    to_js(&fl_judicial_opinion_portals(office))
}

/// Parse flcourts Next.js judges/justices index HTML.
#[wasm_bindgen]
pub fn parse_fl_courts_next_index_js(html: &str, index_url: &str) -> Result<JsValue, JsError> {
    to_js(&parse_fl_courts_next_index(html, index_url))
}

/// Parse circuit WP directory biography links.
#[wasm_bindgen]
pub fn parse_fl_circuit_directory_links_js(
    html: &str,
    index_url: &str,
) -> Result<JsValue, JsError> {
    to_js(&parse_fl_circuit_directory_links(html, index_url))
}

/// Match person name to a unique court index link.
#[wasm_bindgen]
pub fn match_fl_courts_judge_link_js(
    links_json: &str,
    person_name: &str,
) -> Result<JsValue, JsError> {
    let links: Vec<electionizer_core::bio::FlCourtsJudgeLink> =
        serde_json::from_str(links_json).map_err(|e| JsError::new(&e.to_string()))?;
    match match_fl_courts_judge_link(&links, person_name) {
        Some(l) => to_js(l),
        None => Ok(JsValue::NULL),
    }
}

/// Parse flcourts judge/justice page HTML → MemberBioParse.
#[wasm_bindgen]
pub fn parse_fl_courts_judge_html_js(html: &str, page_url: &str) -> Result<JsValue, JsError> {
    to_js(&parse_fl_courts_judge_html(html, page_url))
}

/// Parse circuit WP judge biography page.
#[wasm_bindgen]
pub fn parse_fl_circuit_wp_bio_html_js(html: &str, page_url: &str) -> Result<JsValue, JsError> {
    to_js(&parse_fl_circuit_wp_bio_html(html, page_url))
}

/// Merge chamber/bio parse into dossier JSON.
#[wasm_bindgen]
pub fn apply_member_bio_to_dossier_js(
    dossier_json: &str,
    bio_json: &str,
    as_of_year: i32,
) -> Result<JsValue, JsError> {
    let mut d: PersonDossier =
        serde_json::from_str(dossier_json).map_err(|e| JsError::new(&e.to_string()))?;
    let bio: electionizer_core::bio::MemberBioParse =
        serde_json::from_str(bio_json).map_err(|e| JsError::new(&e.to_string()))?;
    apply_member_bio_to_dossier(&mut d, &bio, as_of_year);
    to_js(&d)
}

/// Set photo when dossier has none.
#[wasm_bindgen]
pub fn apply_photo_to_dossier_js(
    dossier_json: &str,
    photo_url: Option<String>,
    photo_source: Option<String>,
    photo_source_url: Option<String>,
) -> Result<JsValue, JsError> {
    let mut d: PersonDossier =
        serde_json::from_str(dossier_json).map_err(|e| JsError::new(&e.to_string()))?;
    apply_photo_to_dossier(&mut d, photo_url, photo_source, photo_source_url);
    to_js(&d)
}

/// Official federal personal financial disclosure search portals.
#[wasm_bindgen]
pub fn federal_disclosure_portals_js(chamber: Option<String>) -> Result<JsValue, JsError> {
    to_js(&federal_disclosure_portals(chamber.as_deref()))
}

/// Split person name → `{ first, last }` for eFD search form.
#[wasm_bindgen]
pub fn efd_split_person_name_js(name: &str) -> Result<JsValue, JsError> {
    let (first, last) = efd_split_person_name(name);
    to_js(&serde_json::json!({ "first": first, "last": last }))
}

/// Absolute eFD URL helper.
#[wasm_bindgen]
pub fn efd_abs_url_js(path_or_url: &str) -> String {
    efd_abs_url(path_or_url)
}

/// Parse eFD DataTables `/search/report/data/` JSON → hit list.
#[wasm_bindgen]
pub fn parse_efd_search_data_json_js(json: &str) -> Result<JsValue, JsError> {
    to_js(&parse_efd_search_data_json(json).map_err(|e| JsError::new(&e))?)
}

/// Pick latest Annual report for candidate name (+ optional state code).
#[wasm_bindgen]
pub fn pick_efd_annual_report_js(
    hits_json: &str,
    candidate_name: &str,
    state: Option<String>,
) -> Result<JsValue, JsError> {
    let hits: Vec<electionizer_core::bio::EfdReportHit> =
        serde_json::from_str(hits_json).map_err(|e| JsError::new(&e.to_string()))?;
    to_js(&pick_efd_annual_report(
        &hits,
        candidate_name,
        state.as_deref(),
    ))
}

/// Parse Senate eFD annual HTML Part 3 → holdings JSON.
#[wasm_bindgen]
pub fn parse_senate_efd_annual_html_js(html: &str, source_url: &str) -> Result<JsValue, JsError> {
    to_js(&parse_senate_efd_annual_html(html, source_url))
}

/// Absolute House Clerk URL helper.
#[wasm_bindgen]
pub fn house_clerk_abs_url_js(path_or_url: &str) -> String {
    house_clerk_abs_url(path_or_url)
}

/// Parse House Clerk member/candidate search result HTML → filing hits.
#[wasm_bindgen]
pub fn parse_house_clerk_search_html_js(html: &str) -> Result<JsValue, JsError> {
    to_js(&parse_house_clerk_search_html(html))
}

/// Pick latest FD Original for candidate (+ optional state + district).
#[wasm_bindgen]
pub fn pick_house_clerk_fd_report_js(
    hits_json: &str,
    candidate_name: &str,
    state: Option<String>,
    district: Option<u32>,
) -> Result<JsValue, JsError> {
    let hits: Vec<electionizer_core::bio::HouseClerkFilingHit> =
        serde_json::from_str(hits_json).map_err(|e| JsError::new(&e.to_string()))?;
    to_js(&pick_house_clerk_fd_report(
        &hits,
        candidate_name,
        state.as_deref(),
        district,
    ))
}

/// Parse House Clerk FD PDF text → holdings.
#[wasm_bindgen]
pub fn parse_house_clerk_fd_text_js(text: &str, source_url: &str) -> Result<JsValue, JsError> {
    to_js(&parse_house_clerk_fd_text(text, source_url))
}

/// Parse House Clerk FD PDF bytes → holdings (pdf-extract + Schedule A).
#[wasm_bindgen]
pub fn parse_house_clerk_fd_pdf_js(data: &[u8], source_url: &str) -> Result<JsValue, JsError> {
    to_js(&parse_house_clerk_fd_pdf(data, source_url))
}

/// Apply holdings into dossier JSON (keeps other fields).
#[wasm_bindgen]
pub fn apply_holdings_to_dossier_js(
    dossier_json: &str,
    holdings_json: &str,
) -> Result<JsValue, JsError> {
    let mut d: PersonDossier =
        serde_json::from_str(dossier_json).map_err(|e| JsError::new(&e.to_string()))?;
    let holdings: Vec<PersonalHolding> = if holdings_json.trim().is_empty() {
        Vec::new()
    } else {
        serde_json::from_str(holdings_json).map_err(|e| JsError::new(&e.to_string()))?
    };
    apply_holdings_to_dossier(&mut d, holdings);
    to_js(&d)
}

/// Measure sponsor + oppose sides → endorsement list JSON.
/// `oppose_json`: `[["Name","url?"], ...]`
#[wasm_bindgen]
pub fn endorsements_from_measure_sides_js(
    sponsor_name: &str,
    sponsor_url: Option<String>,
    source: &str,
    oppose_json: &str,
) -> Result<JsValue, JsError> {
    let oppose_raw: Vec<serde_json::Value> = if oppose_json.trim().is_empty() {
        Vec::new()
    } else {
        serde_json::from_str(oppose_json).map_err(|e| JsError::new(&e.to_string()))?
    };
    let mut oppose: Vec<(String, Option<String>)> = Vec::new();
    for v in oppose_raw {
        if let Some(arr) = v.as_array() {
            let name = arr
                .first()
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let url = arr
                .get(1)
                .and_then(|x| x.as_str())
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty());
            if !name.is_empty() {
                oppose.push((name, url));
            }
        } else if let Some(obj) = v.as_object() {
            let name = obj
                .get("committee_name")
                .or_else(|| obj.get("name"))
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let url = obj
                .get("committee_url")
                .or_else(|| obj.get("url"))
                .and_then(|x| x.as_str())
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty());
            if !name.is_empty() {
                oppose.push((name, url));
            }
        }
    }
    to_js(&endorsements_from_measure_sides(
        sponsor_name,
        sponsor_url.as_deref(),
        source,
        &oppose,
    ))
}

/// Merge endorsement array JSON into dossier.
#[wasm_bindgen]
pub fn merge_endorsements_js(
    dossier_json: &str,
    endorsements_json: &str,
) -> Result<JsValue, JsError> {
    let mut d: PersonDossier =
        serde_json::from_str(dossier_json).map_err(|e| JsError::new(&e.to_string()))?;
    let extra: Vec<Endorsement> =
        serde_json::from_str(endorsements_json).map_err(|e| JsError::new(&e.to_string()))?;
    merge_endorsements(&mut d, &extra);
    to_js(&d)
}

/// Apply career into existing dossier JSON (keeps facts/endorsements/holdings when present).
#[wasm_bindgen]
pub fn apply_career_to_dossier_js(
    dossier_json: &str,
    career_json: &str,
    photo_url: Option<String>,
    photo_source: Option<String>,
    photo_source_url: Option<String>,
) -> Result<JsValue, JsError> {
    let mut d: PersonDossier =
        serde_json::from_str(dossier_json).map_err(|e| JsError::new(&e.to_string()))?;
    let career: electionizer_core::bio::CareerAssessment =
        serde_json::from_str(career_json).map_err(|e| JsError::new(&e.to_string()))?;
    d.career = career;
    if let Some(u) = photo_url {
        if !u.is_empty() {
            d.photo_url = Some(u);
            d.photo_source = photo_source;
            d.photo_source_url = photo_source_url;
            d.empty_notes
                .retain(|n| !n.to_ascii_lowercase().starts_with("photo:"));
        }
    }
    // Refresh empty notes that depend on career fractions
    let has_non_pol = d
        .career
        .fractions
        .iter()
        .any(|f| f.category != "political" && f.years > 0.0);
    if has_non_pol {
        d.empty_notes.retain(|n| {
            !n.to_ascii_lowercase()
                .starts_with("education / work / business / legal")
        });
    }
    to_js(&d)
}

#[wasm_bindgen]
pub fn current_party_affiliation_js(party: &str, office: &str) -> Result<JsValue, JsError> {
    to_js(&current_party_affiliation(party, office))
}

/// Soft CF context span: campaign committee ≠ voter affiliation.
#[wasm_bindgen]
pub fn campaign_committee_affiliation_js(
    committee_name: &str,
    designation: &str,
    source: Option<String>,
    source_url: Option<String>,
) -> Result<JsValue, JsError> {
    to_js(&campaign_committee_affiliation(
        committee_name,
        designation,
        source.as_deref(),
        source_url.as_deref(),
    ))
}

/// Ballot-derived affiliation spans (filing party + optional incumbent), each cited.
/// Pass `is_judge` for judicial ballot designation (merit retention / nonpartisan / seat).
#[wasm_bindgen]
pub fn ballot_affiliations_js(
    party: &str,
    office: &str,
    is_incumbent: bool,
    is_judge: bool,
    source: Option<String>,
    source_url: Option<String>,
) -> Result<JsValue, JsError> {
    to_js(&ballot_affiliations(
        party,
        office,
        is_incumbent,
        is_judge,
        source.as_deref(),
        source_url.as_deref(),
    ))
}

#[wasm_bindgen]
pub fn parse_vote_voter_list_js(json: &str) -> Result<JsValue, JsError> {
    to_js(&parse_vote_voter_list(json))
}

#[wasm_bindgen]
pub fn vote_voter_total_count_js(json: &str) -> Result<JsValue, JsError> {
    to_js(&vote_voter_total_count(json))
}

#[wasm_bindgen]
pub fn vote_ids_needing_detail_js(json: &str) -> Result<JsValue, JsError> {
    to_js(&vote_ids_needing_detail(json))
}

/// `details_map_json`: JSON object `{ "voteId": "<detail body string>", ... }`.
#[wasm_bindgen]
pub fn assemble_govtrack_votes_js(
    vote_voter_json: &str,
    details_map_json: &str,
) -> Result<JsValue, JsError> {
    to_js(&assemble_govtrack_votes(vote_voter_json, details_map_json))
}

#[wasm_bindgen]
pub fn bioguide_profile_url_js(bioguide: &str) -> Result<JsValue, JsError> {
    to_js(&bioguide_profile_url(bioguide))
}

#[wasm_bindgen]
pub fn congress_gov_member_url_js(bioguide: &str) -> Result<JsValue, JsError> {
    to_js(&congress_gov_member_url(bioguide))
}

#[wasm_bindgen]
pub fn pick_openstates_person_js(
    json: &str,
    name: &str,
    want_org: &str,
    district: Option<u32>,
    state: &str,
) -> Result<JsValue, JsError> {
    to_js(&pick_person(json, name, want_org, district, state))
}

/// Single Open States person object → match fields (detail fetch).
#[wasm_bindgen]
pub fn openstates_person_detail_js(person_json: &str, state: &str) -> Result<JsValue, JsError> {
    to_js(&state_legislator_from_person_json(person_json, state))
}

/// Open States person → cited affiliation spans (party + roles / current_role).
#[wasm_bindgen]
pub fn openstates_affiliations_js(people_json: &str, person_id: &str) -> Result<JsValue, JsError> {
    to_js(&affiliations_from_openstates_people_json(
        people_json,
        person_id,
    ))
}

/// Append Open States (or other) spans after ballot spans without dropping either.
#[wasm_bindgen]
pub fn merge_affiliation_spans_js(base_json: &str, extra_json: &str) -> Result<JsValue, JsError> {
    let base: Vec<electionizer_core::models::AffiliationSpan> =
        serde_json::from_str(base_json).unwrap_or_default();
    let extra: Vec<electionizer_core::models::AffiliationSpan> =
        serde_json::from_str(extra_json).unwrap_or_default();
    to_js(&merge_affiliation_spans(&base, &extra))
}

#[wasm_bindgen]
pub fn extract_openstates_votes_js(
    json: &str,
    person_id: &str,
    limit: usize,
) -> Result<JsValue, JsError> {
    to_js(&extract_votes_for_person(json, person_id, limit))
}

#[wasm_bindgen]
pub fn vote_sessions_js(cycle: i32) -> Result<JsValue, JsError> {
    to_js(&vote_sessions(cycle))
}

#[wasm_bindgen]
pub fn district_from_office_js(office: &str) -> Result<JsValue, JsError> {
    to_js(&district_from_office(office))
}

#[wasm_bindgen]
pub fn state_code_from_jurisdiction_js(
    jurisdiction: &str,
    office: &str,
) -> Result<JsValue, JsError> {
    to_js(&state_code_from_jurisdiction(jurisdiction, office))
}

// --- CourtListener (judicial decisions + practice) ---

#[wasm_bindgen]
pub fn courtlistener_court_ids_js(state: &str, office: &str) -> Result<JsValue, JsError> {
    to_js(&courtlistener_court_ids(state, office))
}

#[wasm_bindgen]
pub fn courtlistener_people_search_url_js(name: &str) -> Result<JsValue, JsError> {
    to_js(&courtlistener_people_search_url(name))
}

#[wasm_bindgen]
pub fn courtlistener_positions_url_js(person_id: f64) -> Result<JsValue, JsError> {
    to_js(&courtlistener_positions_url(person_id as i64))
}

#[wasm_bindgen]
pub fn courtlistener_opinions_search_url_js(
    person_id: f64,
    page_size: usize,
) -> Result<JsValue, JsError> {
    to_js(&courtlistener_opinions_search_url(
        person_id as i64,
        page_size,
    ))
}

#[wasm_bindgen]
pub fn courtlistener_person_profile_url_js(person_id: f64, slug: &str) -> Result<JsValue, JsError> {
    let s = slug.trim();
    to_js(&courtlistener_person_profile_url(
        person_id as i64,
        if s.is_empty() { None } else { Some(s) },
    ))
}

#[wasm_bindgen]
pub fn courtlistener_search_portal_url_js(name: &str) -> Result<JsValue, JsError> {
    to_js(&courtlistener_search_portal_url(name))
}

#[wasm_bindgen]
pub fn pick_courtlistener_person_js(
    people_json: &str,
    candidate_name: &str,
    prefer_courts_json: &str,
) -> Result<JsValue, JsError> {
    let prefer: Vec<String> = serde_json::from_str(prefer_courts_json).unwrap_or_default();
    to_js(&pick_courtlistener_person(
        people_json,
        candidate_name,
        &prefer,
    ))
}

#[wasm_bindgen]
pub fn person_positions_match_courts_js(
    positions_json: &str,
    prefer_courts_json: &str,
) -> Result<bool, JsError> {
    let prefer: Vec<String> = serde_json::from_str(prefer_courts_json).unwrap_or_default();
    Ok(person_positions_match_courts(positions_json, &prefer))
}

#[wasm_bindgen]
pub fn courtlistener_positions_bio_js(
    positions_json: &str,
    profile_url: &str,
) -> Result<JsValue, JsError> {
    let url = profile_url.trim();
    let (spans, facts, birth_year) = spans_and_facts_from_positions(
        positions_json,
        if url.is_empty() { None } else { Some(url) },
    );
    #[derive(serde::Serialize)]
    struct Out {
        spans: Vec<electionizer_core::bio::CareerSpan>,
        facts: Vec<electionizer_core::bio::BioFact>,
        birth_year: Option<i32>,
        source: &'static str,
    }
    to_js(&Out {
        spans,
        facts,
        birth_year,
        source: CL_SOURCE,
    })
}

#[wasm_bindgen]
pub fn courtlistener_opinions_from_search_js(
    search_json: &str,
    person_id: f64,
) -> Result<JsValue, JsError> {
    #[derive(serde::Serialize)]
    struct Out {
        votes: Vec<electionizer_core::models::VoteRecord>,
        total_available: Option<u64>,
        fetch_cap: usize,
        source: &'static str,
    }
    let votes = opinions_from_search_json(search_json, person_id as i64);
    let total_available = opinions_search_total(search_json);
    to_js(&Out {
        votes,
        total_available,
        fetch_cap: CL_OPINIONS_CAP,
        source: CL_SOURCE,
    })
}

#[wasm_bindgen]
pub fn courtlistener_opinions_cap_js() -> usize {
    CL_OPINIONS_CAP
}

#[wasm_bindgen]
pub fn brevard_sample_ballot_url_js(
    precinct: &str,
    party: &str,
    election_id: &str,
) -> Option<String> {
    electionizer_core::scrutiny::brevard_sample_ballot_url(precinct, party, election_id)
}

#[wasm_bindgen]
pub fn sample_ballot_ref_js(
    precinct: &str,
    party: &str,
    election_id: &str,
) -> Result<JsValue, JsError> {
    to_js(&electionizer_core::scrutiny::sample_ballot_ref(
        precinct,
        party,
        election_id,
    ))
}

#[wasm_bindgen]
pub fn scrutiny_portals_js(name: &str, is_judge: bool) -> Result<JsValue, JsError> {
    to_js(&electionizer_core::scrutiny::scrutiny_portals(
        name, is_judge,
    ))
}

#[wasm_bindgen]
pub fn money_signals_from_json_js(
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
) -> Result<JsValue, JsError> {
    to_js(&electionizer_core::scrutiny::money_signals_from_json(
        size_json,
        individuals_json,
        committees_json,
        outside_json,
        receipts_display,
        pac_display,
        individual_display,
        candidate_name,
        home_state,
        home_county,
    ))
}

#[wasm_bindgen]
pub fn endorsements_from_ballotpedia_html_js(
    html: &str,
    page_url: &str,
) -> Result<JsValue, JsError> {
    to_js(&electionizer_core::scrutiny::endorsements_from_ballotpedia_html(html, page_url))
}

#[wasm_bindgen]
pub fn campaign_endorsement_urls_js(site_url: &str) -> Result<JsValue, JsError> {
    to_js(&electionizer_core::scrutiny::campaign_endorsement_urls(
        site_url,
    ))
}

#[wasm_bindgen]
pub fn endorsements_from_campaign_html_js(html: &str, page_url: &str) -> Result<JsValue, JsError> {
    to_js(&electionizer_core::scrutiny::endorsements_from_campaign_html(html, page_url))
}

#[wasm_bindgen]
pub fn gdelt_artlist_url_js(person_name: &str, locale_hint: &str) -> Option<String> {
    electionizer_core::scrutiny::gdelt_artlist_url(person_name, locale_hint)
}

#[wasm_bindgen]
pub fn google_news_rss_url_js(person_name: &str, locale_hint: &str) -> Option<String> {
    electionizer_core::scrutiny::google_news_rss_url(person_name, locale_hint)
}

#[wasm_bindgen]
pub fn news_hits_from_gdelt_json_js(json: &str, person_name: &str) -> Result<JsValue, JsError> {
    to_js(&electionizer_core::scrutiny::news_hits_from_gdelt_json(
        json,
        person_name,
    ))
}

#[wasm_bindgen]
pub fn news_hits_from_google_rss_js(xml: &str, person_name: &str) -> Result<JsValue, JsError> {
    to_js(&electionizer_core::scrutiny::news_hits_from_google_rss(
        xml,
        person_name,
    ))
}

#[wasm_bindgen]
pub fn merge_news_hits_js(base_json: &str, extra_json: &str) -> Result<JsValue, JsError> {
    let base: Vec<electionizer_core::scrutiny::NewsHit> =
        serde_json::from_str(base_json).map_err(|e| JsError::new(&e.to_string()))?;
    let extra: Vec<electionizer_core::scrutiny::NewsHit> =
        serde_json::from_str(extra_json).map_err(|e| JsError::new(&e.to_string()))?;
    to_js(&electionizer_core::scrutiny::merge_news_hits(&base, &extra))
}

#[wasm_bindgen]
pub fn claims_from_ballotpedia_html_js(html: &str, page_url: &str) -> Result<JsValue, JsError> {
    to_js(&electionizer_core::scrutiny::claims_from_ballotpedia_html(
        html, page_url,
    ))
}

#[wasm_bindgen]
pub fn campaign_claim_urls_js(site_url: &str) -> Result<JsValue, JsError> {
    to_js(&electionizer_core::scrutiny::campaign_claim_urls(site_url))
}

#[wasm_bindgen]
pub fn claims_from_campaign_html_js(html: &str, page_url: &str) -> Result<JsValue, JsError> {
    to_js(&electionizer_core::scrutiny::claims_from_campaign_html(
        html, page_url,
    ))
}

#[wasm_bindgen]
pub fn merge_claims_js(base_json: &str, extra_json: &str) -> Result<JsValue, JsError> {
    let base: Vec<electionizer_core::scrutiny::PublicClaim> =
        serde_json::from_str(base_json).map_err(|e| JsError::new(&e.to_string()))?;
    let extra: Vec<electionizer_core::scrutiny::PublicClaim> =
        serde_json::from_str(extra_json).map_err(|e| JsError::new(&e.to_string()))?;
    to_js(&electionizer_core::scrutiny::merge_claims(&base, &extra))
}

#[wasm_bindgen]
pub fn pair_claims_with_votes_js(claims_json: &str, votes_json: &str) -> Result<JsValue, JsError> {
    let claims: Vec<electionizer_core::scrutiny::PublicClaim> =
        serde_json::from_str(claims_json).map_err(|e| JsError::new(&e.to_string()))?;
    let votes: Vec<electionizer_core::models::VoteRecord> =
        serde_json::from_str(votes_json).map_err(|e| JsError::new(&e.to_string()))?;
    to_js(&electionizer_core::scrutiny::pair_claims_with_votes(
        &claims, &votes,
    ))
}

#[wasm_bindgen]
pub fn llm_chat_url_js(provider: &str) -> Option<String> {
    electionizer_core::scrutiny::llm_chat_url(provider).map(str::to_string)
}

#[wasm_bindgen]
pub fn llm_default_model_js(provider: &str) -> Option<String> {
    electionizer_core::scrutiny::llm_default_model(provider).map(str::to_string)
}

#[wasm_bindgen]
pub fn llm_normalize_provider_js(provider: &str) -> Option<String> {
    electionizer_core::scrutiny::llm_normalize_provider(provider).map(str::to_string)
}

#[wasm_bindgen]
pub fn llm_contrast_request_body_js(
    model: &str,
    cards_json: &str,
    name: &str,
    office: &str,
) -> Result<Option<String>, JsError> {
    let cards: Vec<electionizer_core::scrutiny::ContrastCard> =
        serde_json::from_str(cards_json).map_err(|e| JsError::new(&e.to_string()))?;
    Ok(electionizer_core::scrutiny::llm_contrast_request_body(
        model, &cards, name, office,
    ))
}

#[wasm_bindgen]
pub fn apply_llm_chat_response_js(
    cards_json: &str,
    response_json: &str,
    model: &str,
) -> Result<JsValue, JsError> {
    let mut cards: Vec<electionizer_core::scrutiny::ContrastCard> =
        serde_json::from_str(cards_json).map_err(|e| JsError::new(&e.to_string()))?;
    electionizer_core::scrutiny::apply_llm_chat_response(&mut cards, response_json, model);
    to_js(&cards)
}

#[wasm_bindgen]
pub fn fl_bar_search_url_js(name: &str) -> Option<String> {
    electionizer_core::scrutiny::fl_bar_search_url(name)
}

#[wasm_bindgen]
pub fn parse_fl_bar_search_html_js(html: &str, person_name: &str) -> Result<JsValue, JsError> {
    to_js(&electionizer_core::scrutiny::parse_fl_bar_search_html(
        html,
        person_name,
    ))
}

#[wasm_bindgen]
pub fn fl_ethics_filings_url_js(name: &str) -> Option<String> {
    electionizer_core::scrutiny::fl_ethics_filings_url(name)
}

#[wasm_bindgen]
pub fn fl_ethics_orders_url_js() -> String {
    electionizer_core::scrutiny::fl_ethics_orders_url().to_string()
}

#[wasm_bindgen]
pub fn parse_fl_ethics_filings_json_js(json: &str, person_name: &str) -> Result<JsValue, JsError> {
    to_js(&electionizer_core::scrutiny::parse_fl_ethics_filings_json(
        json,
        person_name,
    ))
}

#[wasm_bindgen]
pub fn parse_fl_ethics_orders_html_js(html: &str, person_name: &str) -> Result<JsValue, JsError> {
    to_js(&electionizer_core::scrutiny::parse_fl_ethics_orders_html(
        html,
        person_name,
    ))
}

#[wasm_bindgen]
pub fn fl_jqc_posts_url_js(name: &str) -> Option<String> {
    electionizer_core::scrutiny::fl_jqc_posts_url(name)
}

#[wasm_bindgen]
pub fn fl_jqc_news_url_js() -> String {
    electionizer_core::scrutiny::fl_jqc_news_url().to_string()
}

#[wasm_bindgen]
pub fn parse_fl_jqc_posts_json_js(json: &str, person_name: &str) -> Result<JsValue, JsError> {
    to_js(&electionizer_core::scrutiny::parse_fl_jqc_posts_json(
        json,
        person_name,
    ))
}

#[wasm_bindgen]
pub fn parse_fl_jqc_news_html_js(html: &str, person_name: &str) -> Result<JsValue, JsError> {
    to_js(&electionizer_core::scrutiny::parse_fl_jqc_news_html(
        html,
        person_name,
    ))
}

#[wasm_bindgen]
pub fn merge_record_hits_js(base_json: &str, extra_json: &str) -> Result<JsValue, JsError> {
    let base: Vec<electionizer_core::scrutiny::RecordHit> =
        serde_json::from_str(base_json).map_err(|e| JsError::new(&e.to_string()))?;
    let extra: Vec<electionizer_core::scrutiny::RecordHit> =
        serde_json::from_str(extra_json).map_err(|e| JsError::new(&e.to_string()))?;
    to_js(&electionizer_core::scrutiny::merge_record_hits(
        &base, &extra,
    ))
}

#[wasm_bindgen]
pub fn ballotpedia_state_measures_url_js(state: &str, year: i32) -> Option<String> {
    electionizer_core::scrutiny::ballotpedia_state_measures_url(state, year)
}

#[wasm_bindgen]
pub fn ballotpedia_measure_links_from_index_js(html: &str) -> Result<JsValue, JsError> {
    to_js(&electionizer_core::scrutiny::ballotpedia_measure_links_from_index(html))
}

#[wasm_bindgen]
pub fn match_ballotpedia_measure_js(
    links_json: &str,
    title: &str,
    code: Option<String>,
) -> Result<JsValue, JsError> {
    let links: Vec<electionizer_core::scrutiny::BallotpediaMeasureLink> =
        if links_json.trim().is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(links_json).map_err(|e| JsError::new(&e.to_string()))?
        };
    to_js(&electionizer_core::scrutiny::match_ballotpedia_measure(
        &links,
        title,
        code.as_deref(),
    ))
}

#[wasm_bindgen]
pub fn ballotpedia_html_matches_measure_js(
    html: &str,
    title: &str,
    code: Option<String>,
) -> bool {
    electionizer_core::scrutiny::ballotpedia_html_matches_measure(html, title, code.as_deref())
}

#[wasm_bindgen]
pub fn endorsements_from_ballotpedia_measure_html_js(
    html: &str,
    page_url: &str,
) -> Result<JsValue, JsError> {
    to_js(
        &electionizer_core::scrutiny::endorsements_from_ballotpedia_measure_html(html, page_url),
    )
}

#[wasm_bindgen]
pub fn merge_endorsement_lists_js(
    base_json: &str,
    extra_json: &str,
) -> Result<JsValue, JsError> {
    let base: Vec<Endorsement> = if base_json.trim().is_empty() {
        Vec::new()
    } else {
        serde_json::from_str(base_json).map_err(|e| JsError::new(&e.to_string()))?
    };
    let extra: Vec<Endorsement> = if extra_json.trim().is_empty() {
        Vec::new()
    } else {
        serde_json::from_str(extra_json).map_err(|e| JsError::new(&e.to_string()))?
    };
    to_js(&electionizer_core::scrutiny::merge_endorsement_lists(
        &base, &extra,
    ))
}

#[wasm_bindgen]
pub fn verdict_responses_url_js(provider: &str) -> Option<String> {
    electionizer_core::verdict::verdict_responses_url(provider).map(str::to_string)
}

#[wasm_bindgen]
pub fn verdict_default_model_js(provider: &str) -> Option<String> {
    electionizer_core::verdict::verdict_default_model(provider).map(str::to_string)
}

#[wasm_bindgen]
pub fn pack_verdict_context_js(
    subject_json: &str,
    enrich_json: &str,
) -> Result<JsValue, JsError> {
    match electionizer_core::verdict::pack_verdict_context(subject_json, enrich_json) {
        Some(ctx) => to_js(&ctx),
        None => Ok(JsValue::NULL),
    }
}

#[wasm_bindgen]
pub fn pack_verdict_context_json_js(subject_json: &str, enrich_json: &str) -> Option<String> {
    electionizer_core::verdict::pack_verdict_context(subject_json, enrich_json)
        .and_then(|ctx| serde_json::to_string(&ctx).ok())
}

#[wasm_bindgen]
pub fn packed_fingerprint_js(packed_json: &str) -> String {
    electionizer_core::verdict::packed_fingerprint(packed_json)
}

#[wasm_bindgen]
pub fn verdict_request_body_js(
    provider: &str,
    model: &str,
    packed_json: &str,
    with_search: bool,
) -> Result<Option<String>, JsError> {
    Ok(electionizer_core::verdict::verdict_request_body(
        provider,
        model,
        packed_json,
        with_search,
    ))
}

#[wasm_bindgen]
pub fn verdict_chat_request_body_js(
    provider: &str,
    model: &str,
    packed_json: &str,
    with_search: bool,
) -> Result<Option<String>, JsError> {
    Ok(electionizer_core::verdict::verdict_chat_request_body(
        provider,
        model,
        packed_json,
        with_search,
    ))
}

#[wasm_bindgen]
pub fn parse_verdict_response_js(
    response_json: &str,
    packed_json: &str,
    model: &str,
    provider: &str,
) -> Option<String> {
    let card = electionizer_core::verdict::parse_verdict_card(
        response_json,
        packed_json,
        model,
        provider,
    )?;
    serde_json::to_string(&card).ok()
}

#[wasm_bindgen]
pub fn found_endorsements_from_verdict_js(card_json: &str) -> Result<JsValue, JsError> {
    let card: electionizer_core::verdict::VerdictCard =
        serde_json::from_str(card_json).map_err(|e| JsError::new(&e.to_string()))?;
    to_js(&electionizer_core::verdict::found_endorsements_from_verdict(
        &card,
    ))
}

#[wasm_bindgen]
pub fn rubric_for_js(kind: &str, party: &str, is_judge: bool) -> Result<JsValue, JsError> {
    to_js(&electionizer_core::verdict::rubric_for(
        kind, party, is_judge,
    ))
}

#[wasm_bindgen]
pub fn voter_profile_axes_js() -> Result<JsValue, JsError> {
    to_js(&electionizer_core::verdict::voter_profile_axes())
}

#[wasm_bindgen]
pub fn apply_voter_profile_js(card_json: &str, profile_json: &str) -> Option<String> {
    let card: electionizer_core::verdict::VerdictCard = serde_json::from_str(card_json).ok()?;
    let fitted = electionizer_core::verdict::apply_voter_profile(&card, profile_json);
    serde_json::to_string(&fitted).ok()
}
