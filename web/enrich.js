import { cacheGet, cachePut, cachedFetch, withFreshCache } from "./cache.js";
import {
  getCycle,
  getFecApiKey,
  getOpenStatesApiKey,
  hasOpenStatesKey,
  getFtmApiKey,
  hasFtmKey,
  getCourtListenerToken,
  getCorsProxy,
  getLlmApiKey,
  getLlmProvider,
  getLlmModel,
  hasLlmKey,
  getVoterProfile,
} from "./settings.js";
import {
  curlCloseSession,
  curlFetchBytes,
  curlFetchText,
  curlPostForm,
  curlPostJson,
  curlRequest,
  curlSession,
  ensureCurl,
  hasWispConfigured,
  sessionHasCookie,
} from "./curl-transport.js";
import { tryFetchText, tryPostForm, tryPostJson } from "./state.js";
import {
  looks_like_fec_id_js,
  parse_fec_totals_js,
  parse_fec_ie_js,
  parse_fec_sched_a_js,
  parse_fec_sched_a_lines_js,
  fec_occupation_facts_from_sched_a_js,
  parse_fec_size_js,
  parse_fec_principal_js,
  match_legislator_by_fec_js,
  empty_dossier_js,
  assess_career_from_cl_js,
  assess_career_js,
  apply_career_to_dossier_js,
  apply_member_bio_to_dossier_js,
  apply_photo_to_dossier_js,
  unitedstates_congress_photo_url_js,
  wikidata_entity_url_js,
  wikidata_label_ids_needed_js,
  parse_wikidata_entity_bio_js,
  wikipedia_summary_api_url_js,
  wikipedia_extract_api_url_js,
  parse_wikipedia_extract_json_js,
  parse_wikipedia_summary_photo_js,
  wikipedia_summary_match_person_js,
  apply_member_bio_fill_gaps_js,
  note_source_checked_js,
  polish_dossier_empty_notes_js,
  ballotpedia_page_url_js,
  parse_ballotpedia_member_html_js,
  ballotpedia_title_candidates_js,
  ballotpedia_html_matches_person_js,
  ballotpedia_campaign_website_js,
  is_campaign_site_url_js,
  campaign_about_urls_js,
  parse_campaign_about_html_js,
  official_about_urls_js,
  parse_official_member_about_html_js,
  dbpedia_ntriples_url_js,
  dbpedia_describe_ntriples_url_js,
  parse_dbpedia_ntriples_js,
  grokipedia_typeahead_url_js,
  match_grokipedia_typeahead_js,
  parse_grokipedia_page_html_js,
  parse_fl_senate_member_html_js,
  parse_fl_chamber_member_html_js,
  fl_courts_index_url_js,
  fl_bar_member_search_url_js,
  fl_judge_decision_portals_js,
  fl_judicial_opinion_portals_js,
  parse_fl_courts_next_index_js,
  parse_fl_circuit_directory_links_js,
  match_fl_courts_judge_link_js,
  parse_fl_courts_judge_html_js,
  parse_fl_circuit_wp_bio_html_js,
  merge_career_spans_js,
  endorsement_from_ie_js,
  federal_disclosure_portals_js,
  efd_split_person_name_js,
  efd_abs_url_js,
  parse_efd_search_data_json_js,
  pick_efd_annual_report_js,
  parse_senate_efd_annual_html_js,
  house_clerk_abs_url_js,
  parse_house_clerk_search_html_js,
  pick_house_clerk_fd_report_js,
  parse_house_clerk_fd_pdf_js,
  apply_holdings_to_dossier_js,
  merge_endorsements_js,
  ballot_affiliations_js,
  campaign_committee_affiliation_js,
  parse_vote_voter_list_js,
  assemble_govtrack_votes_js,
  vote_voter_total_count_js,
  vote_ids_needing_detail_js,
  pick_openstates_person_js,
  openstates_person_detail_js,
  merge_affiliation_spans_js,
  extract_openstates_votes_js,
  vote_sessions_js,
  district_from_office_js,
  state_code_from_jurisdiction_js,
  courtlistener_court_ids_js,
  courtlistener_people_search_url_js,
  courtlistener_positions_url_js,
  courtlistener_opinions_search_url_js,
  courtlistener_search_portal_url_js,
  pick_courtlistener_person_js,
  person_positions_match_courts_js,
  courtlistener_positions_bio_js,
  courtlistener_opinions_from_search_js,
  courtlistener_opinions_cap_js,
  parse_fl_acct_external_id_js,
  parse_dos_account_js,
  fl_trefin_contrib_url_js,
  parse_trefin_finance_js,
  fl_contrib_url_js,
  fl_can_list_url_js,
  fl_can_list_form_js,
  fl_contrib_candidate_totals_form_js,
  fl_gen_elec_id_js,
  split_candidate_first_last_js,
  office_code_from_ballot_js,
  district_from_ballot_office_js,
  parse_contrib_candidate_totals_js,
  parse_can_list_js,
  match_fl_contrib_candidate_js,
  match_fl_can_list_account_js,
  ftm_data_year_js,
  ftm_office_type_code_js,
  ftm_candidates_url_js,
  ftm_top_donors_url_js,
  ftm_error_message_js,
  parse_ftm_candidate_records_js,
  match_ftm_candidate_js,
  ftm_finance_from_hit_js,
  nc_cf_search_url_js,
  nc_cf_committee_search_form_js,
  nc_cf_search_name_fragment_js,
  nc_cf_documents_url_js,
  nc_cf_summary_csv_url_js,
  parse_nc_cf_committee_search_html_js,
  parse_nc_cf_documents_html_js,
  pick_latest_nc_disclosure_js,
  parse_nc_cf_summary_csv_js,
  match_nc_cf_committee_js,
  nc_cf_finance_from_parts_js,
  az_cf_candidates_url_js,
  az_cf_datatables_body_js,
  az_cf_search_name_fragment_js,
  az_cf_office_id_js,
  parse_az_cf_table_json_js,
  match_az_cf_candidate_js,
  az_cf_finance_from_hit_js,
  md_cf_committee_list_url_js,
  md_cf_financial_summary_url_js,
  md_cf_committee_list_body_js,
  md_cf_financial_summary_body_js,
  md_cf_search_name_fragment_js,
  parse_md_cf_committee_list_json_js,
  match_md_cf_candidate_js,
  md_cf_finance_from_hit_js,
  fl_soe_candidate_list_url_js,
  fl_soe_contact_url_js,
  voterfocus_county_param_js,
  parse_fl_soe_candidate_list_html_js,
  match_fl_soe_candidate_js,
  fl_soe_finance_from_hit_js,
  money_signals_from_json_js,
  endorsements_from_ballotpedia_html_js,
  campaign_endorsement_urls_js,
  endorsements_from_campaign_html_js,
  gdelt_artlist_url_js,
  google_news_rss_url_js,
  news_hits_from_gdelt_json_js,
  news_hits_from_google_rss_js,
  merge_news_hits_js,
  scrutiny_portals_js,
  claims_from_ballotpedia_html_js,
  campaign_claim_urls_js,
  claims_from_campaign_html_js,
  merge_claims_js,
  pair_claims_with_votes_js,
  llm_chat_url_js,
  llm_default_model_js,
  llm_contrast_request_body_js,
  apply_llm_chat_response_js,
  fl_bar_search_url_js,
  parse_fl_bar_search_html_js,
  fl_ethics_filings_url_js,
  fl_ethics_orders_url_js,
  parse_fl_ethics_filings_json_js,
  parse_fl_ethics_orders_html_js,
  fl_jqc_posts_url_js,
  fl_jqc_news_url_js,
  parse_fl_jqc_posts_json_js,
  parse_fl_jqc_news_html_js,
  merge_record_hits_js,
  pack_verdict_context_js,
  pack_verdict_context_json_js,
  packed_fingerprint_js,
  verdict_request_body_js,
  verdict_chat_request_body_js,
  verdict_responses_url_js,
  verdict_default_model_js,
  parse_verdict_response_js,
  found_endorsements_from_verdict_js,
} from "./pkg/electionizer_wasm.js";

const LEGISLATORS_URL =
  "https://unitedstates.github.io/congress-legislators/legislators-current.json";
const LEGISLATORS_HISTORICAL_URL =
  "https://unitedstates.github.io/congress-legislators/legislators-historical.json";

function fecUrl(path, params) {
  const u = new URL(`https://api.open.fec.gov${path}`);
  u.searchParams.set("api_key", getFecApiKey());
  for (const [k, v] of Object.entries(params || {})) {
    if (v != null && v !== "") u.searchParams.set(k, String(v));
  }
  return u.toString();
}

/** DEMO_KEY is ~40 req/h — keep Sched A shallow; personal keys page deeper. */
function fecSchedADepth() {
  const demo = getFecApiKey() === "DEMO_KEY";
  return {
    demo,
    perPage: 100,
    maxPages: demo ? 1 : 3,
    contribCap: demo ? 25 : 50,
  };
}

/**
 * Fetch OpenFEC Schedule A pages and merge `results` for aggregation.
 * @returns {{ body: string, lines: number, pagesFetched: number, totalCount: number|null, demo: boolean, contribCap: number }}
 */
async function fetchFecSchedAMerged(baseParams, keyBase) {
  const depth = fecSchedADepth();
  const results = [];
  let totalCount = null;
  let pagesFetched = 0;
  for (let page = 1; page <= depth.maxPages; page++) {
    const params = {
      ...baseParams,
      per_page: depth.perPage,
      page,
    };
    const url = fecUrl(`/v1/schedules/schedule_a/`, params);
    const { body } = await cachedFetch(url, {
      key: `${keyBase}:p${page}:n${depth.perPage}`,
    });
    pagesFetched += 1;
    let parsed;
    try {
      parsed = JSON.parse(body);
    } catch {
      break;
    }
    const pageResults = Array.isArray(parsed?.results) ? parsed.results : [];
    if (parsed?.pagination?.count != null) {
      totalCount = Number(parsed.pagination.count);
    }
    results.push(...pageResults);
    const pages = Number(parsed?.pagination?.pages) || 1;
    if (!pageResults.length || page >= pages) break;
  }
  return {
    body: JSON.stringify({ results }),
    lines: results.length,
    pagesFetched,
    totalCount,
    demo: depth.demo,
    contribCap: depth.contribCap,
  };
}

/** FL DOS TreFin account from external_id or CanDetail source_url. */
function flDosAccount(c) {
  const ext = (c.external_id || "").trim();
  if (ext && typeof parse_fl_acct_external_id_js === "function") {
    const a = parse_fl_acct_external_id_js(ext);
    if (a) return a;
  }
  if (ext.startsWith("fl:acct:")) {
    const a = ext.slice("fl:acct:".length).trim();
    if (/^\d+$/.test(a)) return a;
  }
  const url = c.source_url || "";
  if (
    /CanDetail\.asp/i.test(url) ||
    /dos\.elections\.myflorida\.com\/candidates/i.test(url)
  ) {
    try {
      const a =
        (typeof parse_dos_account_js === "function" && parse_dos_account_js(url)) ||
        null;
      if (a && /^\d+$/.test(a) && a !== "0" && a !== "10") return a;
    } catch {
      /* ignore */
    }
    const m = url.match(/account=(\d+)/i);
    if (m && m[1] !== "0" && m[1] !== "10") return m[1];
  }
  return null;
}

/** True when candidate is Florida state/local (not federal FEC). */
function candidateStateCode(c) {
  return (
    (c.state_code || "").toUpperCase() ||
    (typeof state_code_from_jurisdiction_js === "function"
      ? state_code_from_jurisdiction_js(
          c.jurisdiction || c.jurisdiction_ocd || "",
          c.office || ""
        )
      : "") ||
    ""
  );
}

function isFloridaCandidate(c) {
  const st = candidateStateCode(c);
  if (st === "FL") return true;
  const ocd = (c.jurisdiction_ocd || c.jurisdiction || "").toLowerCase();
  if (ocd.includes("state:fl")) return true;
  const url = (c.source_url || "").toLowerCase();
  if (
    url.includes("myflorida.com") ||
    url.includes("flsenate.gov") ||
    url.includes("flhouse.gov")
  ) {
    return true;
  }
  const pub = (c.source_publisher || "").toLowerCase();
  if (pub.includes("florida")) return true;
  return false;
}

function isNorthCarolinaCandidate(c) {
  const st = candidateStateCode(c);
  if (st === "NC") return true;
  const ocd = (c.jurisdiction_ocd || c.jurisdiction || "").toLowerCase();
  if (ocd.includes("state:nc")) return true;
  const url = (c.source_url || "").toLowerCase();
  if (url.includes("ncsbe.gov") || url.includes("dl.ncsbe.gov")) return true;
  const pub = (c.source_publisher || "").toLowerCase();
  if (pub.includes("north carolina") || pub.includes("n.c.") || pub.includes("ncsbe")) {
    return true;
  }
  return false;
}

function isArizonaCandidate(c) {
  const st = candidateStateCode(c);
  if (st === "AZ") return true;
  const ocd = (c.jurisdiction_ocd || c.jurisdiction || "").toLowerCase();
  if (ocd.includes("state:az")) return true;
  const url = (c.source_url || "").toLowerCase();
  if (
    url.includes("azleg.gov") ||
    url.includes("seethemoney.az.gov") ||
    url.includes("azsos.gov") ||
    url.includes("azcleanelections.gov")
  ) {
    return true;
  }
  const pub = (c.source_publisher || "").toLowerCase();
  if (pub.includes("arizona") || pub.includes("clean elections")) return true;
  return false;
}

function isMarylandCandidate(c) {
  const st = candidateStateCode(c);
  if (st === "MD") return true;
  const ocd = (c.jurisdiction_ocd || c.jurisdiction || "").toLowerCase();
  if (ocd.includes("state:md")) return true;
  const url = (c.source_url || "").toLowerCase();
  if (
    url.includes("elections.maryland.gov") ||
    url.includes("campaignfinance.maryland.gov") ||
    url.includes("mgaleg.maryland.gov")
  ) {
    return true;
  }
  const pub = (c.source_publisher || "").toLowerCase();
  if (pub.includes("maryland") || pub.includes("mdcris")) return true;
  return false;
}

function planStages(c) {
  const ext = (c.external_id || "").trim();
  if (ext && looks_like_fec_id_js(ext)) {
    const stages = [
      { id: "totals", label: "FEC cycle totals" },
      { id: "principal", label: "Principal campaign committee" },
      { id: "indiv", label: "Top individual contributors" },
      { id: "fec_occupation", label: "FEC occupation/employer (Schedule A)" },
      { id: "cmte", label: "Top committee contributors" },
      { id: "size", label: "Contribution size buckets" },
      { id: "outside", label: "Outside spending (Schedule E)" },
      { id: "member", label: "Match member of Congress" },
      { id: "official_about", label: "Official member About (House/Senate)" },
      { id: "ballotpedia_bio", label: "Ballotpedia bio (education, profession, family)" },
      { id: "campaign_about", label: "Campaign site About (if known)" },
      { id: "wiki_extract", label: "Wikipedia extract (fill bio gaps)" },
      { id: "dbpedia", label: "DBpedia infobox (fill gaps)" },
      { id: "grokipedia", label: "Grokipedia bio (fill gaps, optional)" },
      { id: "wikidata_bio", label: "Wikidata bio (family, education, career)" },
      { id: "wiki_photo", label: "Wikipedia photo (if no headshot)" },
      { id: "votes", label: "Roll-call votes (GovTrack)" },
    ];
    // Personal holdings (after bio hosts, before votes).
    const ch = (c.chamber || "").toLowerCase();
    const isSenate =
      ch.includes("senate") || /^S/i.test(ext) || /senate/i.test(c.office || "");
    const isHouse =
      ch.includes("house") ||
      /^H/i.test(ext) ||
      /\bhouse\b/i.test(c.office || "") ||
      /\brep(resentative)?\b/i.test(c.office || "");
    const vi = stages.findIndex((s) => s.id === "votes");
    const insertBeforeVotes = (stage) => {
      if (vi >= 0) stages.splice(vi, 0, stage);
      else stages.push(stage);
    };
    if (isSenate) {
      insertBeforeVotes({
        id: "senate_efd",
        label: "Senate eFD personal holdings (annual assets)",
      });
    } else if (isHouse) {
      insertBeforeVotes({
        id: "house_clerk_fd",
        label: "House Clerk FD personal holdings (Schedule A)",
      });
    }
    appendScrutinyStages(stages);
    return stages;
  }

  const chamber = c.chamber || "";
  const flAcct = flDosAccount(c);
  const stages = [];

  if (flAcct) {
    stages.push({
      id: "fl_trefin",
      label: "FL DOS campaign finance (TreFin)",
    });
  } else if (isFloridaCandidate(c)) {
    // A2: name-search fallback when ballot row has no fl:acct:*
    stages.push({
      id: "fl_name_search",
      label: "FL DOS name-search finance",
    });
    // A5: county SOE VoterFocus after DOS name-search (skip if finance already set)
    stages.push({
      id: "fl_soe_cf",
      label: "FL county SOE finance (VoterFocus)",
    });
  } else if (isNorthCarolinaCandidate(c)) {
    // A4: NCSBE campaign finance portal
    stages.push({
      id: "nc_cf",
      label: "NC SBE campaign finance",
    });
  } else if (isArizonaCandidate(c)) {
    // A4: AZ SeeTheMoney
    stages.push({
      id: "az_cf",
      label: "AZ SeeTheMoney finance",
    });
  } else if (isMarylandCandidate(c)) {
    // A4: MD MDCRIS — all MD chambers (leg/statewide/judicial/local)
    stages.push({
      id: "md_cf",
      label: "MD MDCRIS campaign finance",
    });
  } else if (hasFtmKey()) {
    // A3: FollowTheMoney for non-FL/NC/AZ/MD state/local/judicial
    stages.push({
      id: "ftm",
      label: "FollowTheMoney state finance",
    });
  }

  // F2/F3/F8: FL chamber member page (Senate or House profile URL)
  if (
    isFloridaCandidate(c) &&
    /flsenate\.gov\/Senators|(?:flhouse|myfloridahouse)\.gov/i.test(
      c.source_url || ""
    )
  ) {
    const house = /(?:flhouse|myfloridahouse)\.gov/i.test(c.source_url || "");
    stages.push({
      id: "fl_chamber_bio",
      label: house
        ? "FL House member bio (photo + career)"
        : "FL Senate member bio (photo + career)",
    });
  }

  // FL courts official bios (SC / DCA / known circuit directories) — before BP.
  const isJudge =
    !!(c.is_judge || chamber === "judicial" || /judge|justice/i.test(c.office || ""));
  if (isFloridaCandidate(c) && isJudge) {
    stages.push({
      id: "fl_courts_bio",
      label: "FL courts official bio (photo + education + career)",
    });
  }

  // Track K: CourtListener person + practice (before dense bio); opinions last.
  if (isJudge) {
    stages.push(
      { id: "cl_person", label: "Match judge (CourtListener)" },
      { id: "cl_positions", label: "Law practice & bench history (CourtListener)" }
    );
  }

  // OS resolve/votes — state legislature only (keyed).
  const isStateLeg =
    chamber === "state_senate" || chamber === "state_house";
  if (isStateLeg && hasOpenStatesKey()) {
    stages.push({
      id: "os_resolve",
      label: "Match state legislator (Open States)",
    });
  }

  // J1: dense bio hosts for every non-federal candidate (statewide / judicial /
  // local / leg / Civic). Honest empty when hosts miss — cite or omit.
  stages.push(
    { id: "ballotpedia_bio", label: "Ballotpedia bio (title match)" },
    { id: "campaign_about", label: "Campaign site About (if known)" },
    { id: "wiki_extract", label: "Wikipedia extract (fill bio gaps)" },
    { id: "dbpedia", label: "DBpedia infobox (fill gaps)" },
    { id: "grokipedia", label: "Grokipedia bio (fill gaps, optional)" },
    { id: "wikidata_bio", label: "Wikidata bio (if id known)" },
    { id: "wiki_photo", label: "Wikipedia photo (if no headshot)" }
  );

  if (isJudge) {
    stages.push({
      id: "cl_opinions",
      label: "Decisions & opinions (CourtListener)",
    });
  }

  if (isStateLeg && hasOpenStatesKey()) {
    stages.push({ id: "os_votes", label: "State roll-call votes" });
  }

  appendScrutinyStages(stages);
  return stages;
}

function appendScrutinyStages(stages) {
  stages.push(
    { id: "bp_endorsements", label: "Ballotpedia endorsements" },
    { id: "campaign_endorsements", label: "Campaign endorsements page" },
    { id: "gdelt_news", label: "News headlines (GDELT)" },
    { id: "news_rss", label: "News headlines (Google News)" },
    { id: "money_signals", label: "Money signals (from loaded finance)" },
    { id: "bp_claims", label: "Ballotpedia stated positions" },
    { id: "campaign_claims", label: "Campaign issues page" },
    { id: "claim_contrasts", label: "Keyword-pair claims vs votes" },
    { id: "llm_contrasts", label: "LLM contrast notes (optional key)" },
    { id: "fl_bar", label: "Florida Bar standing" },
    { id: "fl_ethics", label: "Florida Ethics filings / orders" },
    { id: "fl_jqc", label: "Florida JQC notices" },
    { id: "ai_verdict", label: "AI verdict (Grok / OpenAI)" }
  );
}

function subjectFromCandidate(c) {
  return {
    id: c.id,
    kind: c.is_judge ? "judge" : "candidate",
    name: c.name || "",
    party: c.party || "",
    office: c.office || "",
    jurisdiction: c.jurisdiction || "",
    state_code: c.state_code || "",
    is_judge: !!(c.is_judge || c.chamber === "judicial"),
    is_incumbent: !!c.is_incumbent,
    summary: c.summary || "",
    source_url: c.source_url || "",
  };
}

function packedJsonFrom(subject, enrich, opts = {}) {
  const merged = { ...(enrich || {}) };
  if (opts.profile !== false) merged.voter_profile = getVoterProfile();
  else delete merged.voter_profile;
  const subjectJson = JSON.stringify(subject || {});
  const enrichJson = JSON.stringify(merged);
  try {
    if (typeof pack_verdict_context_json_js === "function") {
      const s = pack_verdict_context_json_js(subjectJson, enrichJson);
      if (s && typeof s === "string" && s.length > 8) return s;
    }
  } catch (err) {
    console.warn("pack verdict json", err);
  }
  try {
    const packed = pack_verdict_context_js(subjectJson, enrichJson);
    if (!packed) return "";
    if (typeof packed === "string") return packed;
    if (packed instanceof Map) return JSON.stringify(Object.fromEntries(packed));
    return JSON.stringify(packed);
  } catch (err) {
    console.warn("pack verdict", err);
    return "";
  }
}

function unwrapCard(raw) {
  if (raw == null || raw === "") return null;
  let card = raw;
  if (typeof raw === "string") {
    try {
      card = JSON.parse(raw);
    } catch {
      return null;
    }
  }
  if (card instanceof Map) {
    try {
      card = Object.fromEntries(card);
    } catch {
      return null;
    }
  }
  if (!card || typeof card !== "object") return null;
  if (card.axes instanceof Map) card.axes = Array.from(card.axes.values());
  if (card.summary instanceof Map) card.summary = Array.from(card.summary.values());
  if (card.found instanceof Map) card.found = Array.from(card.found.values());
  return card;
}

function cardLooksUsable(card) {
  const c = unwrapCard(card);
  if (!c) return false;
  const headline = String(c.headline || "").trim();
  const axes = c.axes;
  const scored = Array.isArray(axes) && axes.some((a) => a && (a.score != null || a.verdict));
  return !!(headline || scored);
}

async function postVerdict(url, body) {
  return curlPostJson(url, body, {
    headers: { Authorization: `Bearer ${getLlmApiKey()}` },
  });
}

function isRateLimitErr(err) {
  const status = err && err.status;
  const msg = String((err && err.message) || err || "");
  return status === 429 || status === 503 || /429|rate limit|too many/i.test(msg);
}

async function postVerdictWithRetry(url, body) {
  let delay = 1500;
  let lastErr;
  for (let attempt = 0; attempt < 4; attempt++) {
    try {
      return await postVerdict(url, body);
    } catch (err) {
      lastErr = err;
      if (!isRateLimitErr(err) || attempt === 3) throw err;
      await new Promise((r) => setTimeout(r, delay + Math.random() * 400));
      delay = Math.min(delay * 2, 16000);
    }
  }
  throw lastErr;
}

function assistantTextFromResponse(resp) {
  const raw = typeof resp === "string" ? resp : JSON.stringify(resp || "");
  if (!raw) return "";
  let v = null;
  try {
    v = JSON.parse(raw);
  } catch {
    return raw;
  }
  const chunks = [];
  const take = (s) => {
    if (typeof s === "string" && s.trim()) chunks.push(s);
  };
  const msg = v && v.choices && v.choices[0] && v.choices[0].message;
  if (msg) {
    if (typeof msg.content === "string") take(msg.content);
    else if (Array.isArray(msg.content)) {
      for (const part of msg.content) take(part && (part.text || part.output_text || part));
    }
    take(msg.reasoning_content);
  }
  take(v && v.output_text);
  if (v && Array.isArray(v.output)) {
    for (const item of v.output) {
      const kind = item && item.type;
      if (kind && kind !== "message" && kind !== "output_text") continue;
      if (typeof item.content === "string") take(item.content);
      else if (Array.isArray(item.content)) {
        for (const part of item.content) {
          if (part && part.type === "refusal") continue;
          take(part && (part.text || part.output_text || part));
        }
      }
      take(item.text);
    }
  }
  return chunks.join("\n").trim() || raw;
}

function salvageCard(resp, model, provider) {
  const text = assistantTextFromResponse(resp);
  if (!text || text.length < 8) return null;
  let obj = null;
  const tryObj = (s) => {
    try {
      const v = JSON.parse(s);
      if (v && typeof v === "object" && (v.headline || v.axes || v.overall || v.scores || v.card)) {
        return v.card && typeof v.card === "object" ? v.card : v;
      }
    } catch {
      /* ignore */
    }
    return null;
  };
  obj = tryObj(text);
  if (!obj) {
    const start = text.search(/\{[\s\S]*"(headline|axes|overall|scores)"/);
    if (start >= 0) {
      const slice = text.slice(start);
      const end = slice.lastIndexOf("}");
      if (end > 0) obj = tryObj(slice.slice(0, end + 1));
    }
  }
  if (obj && (obj.headline || obj.axes || obj.overall || obj.scores)) {
    const axes = Array.isArray(obj.axes)
      ? obj.axes
      : obj.scores && typeof obj.scores === "object"
        ? Object.entries(obj.scores).map(([id, score]) => ({
            id,
            score: typeof score === "object" ? score.score : score,
            verdict: typeof score === "object" ? score.verdict : "",
          }))
        : [];
    const headline = String(obj.headline || obj.title || "").trim();
    const summary = Array.isArray(obj.summary)
      ? obj.summary.map((s) => (typeof s === "string" ? { text: s, cites: [] } : s)).filter((s) => s && s.text)
      : obj.summary
        ? [{ text: String(obj.summary), cites: [] }]
        : [];
    const card = {
      headline,
      overall: typeof obj.overall === "object" && obj.overall ? obj.overall : { score: obj.overall },
      summary,
      axes,
      found: Array.isArray(obj.found) ? obj.found : [],
      tab_cites: Array.isArray(obj.tab_cites) ? obj.tab_cites : [],
      model: model || "",
      provider: provider || "",
    };
    if (cardLooksUsable(card)) return card;
  }
  const clean = text.replace(/```json|```/g, "").trim();
  if (clean.length < 20) return null;
  if (clean.startsWith("{") && (clean.includes('"output"') || clean.includes('"choices"'))) {
    return null;
  }
  return {
    headline: clean.slice(0, 160),
    overall: {},
    summary: [{ text: clean.slice(0, 800), cites: [] }],
    axes: [],
    found: [],
    tab_cites: [],
    model: model || "",
    provider: provider || "",
  };
}

function parseVerdictCard(resp, packedJson, model, provider) {
  const r = typeof resp === "string" ? resp : JSON.stringify(resp || "");
  const p = typeof packedJson === "string" ? packedJson : JSON.stringify(packedJson || {});
  const m = String(model || "");
  const prov = String(provider || "xai");
  try {
    const raw = parse_verdict_response_js(r, p, m, prov);
    const card = unwrapCard(raw);
    if (cardLooksUsable(card)) return card;
  } catch (err) {
    console.warn("verdict parse", err);
  }
  try {
    const salvaged = salvageCard(r, m, prov);
    if (cardLooksUsable(salvaged)) return salvaged;
  } catch (err) {
    console.warn("verdict salvage", err);
  }
  return null;
}

export async function runVerdictPass(c, enrich, opts = {}) {
  if (!hasLlmKey()) return { skip: "no LLM key" };
  if (!hasWispConfigured()) return { skip: "Wisp required (LLM hosts block CORS)" };
  const provider = getLlmProvider();
  let url = "";
  let model = getLlmModel() || "";
  try {
    url = verdict_responses_url_js(provider) || "";
    if (!model) model = verdict_default_model_js(provider) || "";
  } catch (err) {
    console.warn("verdict url", err);
  }
  if (!url || !model) return { skip: "unknown provider" };
  const subject = opts.subject || subjectFromCandidate(c);
  const packedJson = packedJsonFrom(subject, enrich);
  if (!packedJson) return { skip: "could not pack subject" };
  const cachePacked = packedJsonFrom(subject, enrich, { profile: false });
  let hash = "";
  try {
    hash = packed_fingerprint_js(cachePacked || packedJson) || "";
  } catch {
    hash = "";
  }
  const prevHash = enrich && enrich.verdict && enrich.verdict.packed_hash;
  if (
    opts.pass === "refine" &&
    prevHash &&
    hash &&
    prevHash === hash &&
    enrich.verdict &&
    cardLooksUsable(enrich.verdict)
  ) {
    return { card: enrich.verdict, skip: "unchanged", hash };
  }
  const cacheKey = `verdict:v5:${provider}:${model}:${String(subject.id ?? "")}:${String(
    subject.office || ""
  )
    .toLowerCase()}:${String(subject.name || "").toLowerCase()}:${hash || "x"}`;
  let body = "";
  try {
    body = verdict_request_body_js(provider, model, packedJson, true) || "";
  } catch (err) {
    console.warn("verdict body", err);
  }
  if (!body) return { skip: "no prompt" };
  let resp = "";
  let lastErr = "";
  try {
    if (!opts.fresh) {
      const hitBody = await cacheGet(cacheKey);
      if (hitBody && hitBody.length >= 20) {
        resp = hitBody;
      }
    }
  } catch {
    resp = "";
  }
  if (!resp) {
    try {
      resp = await postVerdictWithRetry(url, body);
    } catch (err) {
      lastErr = err && err.message ? String(err.message) : "request failed";
      console.warn("verdict post", err);
      resp = "";
    }
  }
  let card = resp ? parseVerdictCard(resp, packedJson, model, provider) : null;
  if (!card) {
    try {
      const chatUrl = llm_chat_url_js(provider) || "https://api.x.ai/v1/chat/completions";
      const chatBody =
        (typeof verdict_chat_request_body_js === "function"
          ? verdict_chat_request_body_js(provider, model, packedJson, true)
          : "") || "";
      if (chatBody) {
        const chatResp = await postVerdictWithRetry(chatUrl, chatBody);
        lastErr = "";
        const chatCard = parseVerdictCard(chatResp, packedJson, model, provider);
        if (chatCard) {
          card = chatCard;
          resp = chatResp;
        } else if (!resp) {
          resp = chatResp;
        }
      }
    } catch (err) {
      lastErr = err && err.message ? String(err.message) : lastErr || "chat fallback failed";
      console.warn("verdict chat fallback", err);
    }
  }
  if (!resp) return { skip: lastErr || "empty model response" };
  if (!card) {
    const hint = assistantTextFromResponse(resp).replace(/\s+/g, " ").slice(0, 140);
    return {
      skip: lastErr || (hint ? `could not parse model card — ${hint}` : "could not parse model card"),
    };
  }
  try {
              await cachePut(cacheKey, resp, 0);
  } catch {
    /* ignore */
  }
  if (hash) card.packed_hash = hash;
  return { card, hash };
}

export async function runMeasureVerdict(m, opts = {}) {
  const subject = {
    kind: "measure",
    name: m.title || m.name || "",
    title: m.title || "",
    measure_code: m.measure_code || "",
    office: "Ballot measure",
    party: "",
    jurisdiction: m.jurisdiction || "",
    summary: m.summary || "",
    source_url: m.source_url || "",
    ballotpedia_url: m.ballotpedia_url || "",
    endorsements: m.endorsements || [],
    finance: m.finance || null,
  };
  return runVerdictPass(
    {
      name: subject.name,
      office: subject.office,
      party: "",
      is_judge: false,
    },
    {
      dossier: { endorsements: Array.isArray(m.endorsements) ? m.endorsements : [] },
      finance: m.finance || null,
    },
    { pass: "measure", subject, fresh: !!opts.fresh }
  );
}

function scrutinyLocale(c) {
  const j = `${c.jurisdiction || ""} ${c.office || ""}`.toLowerCase();
  if (j.includes("brevard")) return "Brevard";
  if ((c.state_code || "").toUpperCase() === "FL" || j.includes("florida")) {
    return "Florida";
  }
  return (c.state_code || "").toUpperCase();
}

function asEndorsementRows(v) {
  if (!v) return [];
  let arr = [];
  if (Array.isArray(v)) arr = v;
  else if (typeof v.length === "number") arr = Array.from(v);
  else if (typeof v === "object") {
    arr = Object.values(v).filter((x) => x && typeof x === "object" && x.org);
  }
  return arr
    .map((x) => ({
      org: String(x.org || "").trim(),
      stance: String(x.stance || "support").trim() || "support",
      source: String(x.source || "").trim(),
      source_url: x.source_url || null,
      kind: x.kind || null,
      trust: x.trust || null,
      date: x.date || null,
    }))
    .filter((x) => x.org);
}

function ensureScrutiny(enrich, c) {
  if (!enrich.scrutiny) {
    enrich.scrutiny = { money: null, news: [], portals: [], claims: [], contrasts: [], records: [], endorsements: [] };
  }
  if (!enrich.scrutiny.claims) enrich.scrutiny.claims = [];
  if (!enrich.scrutiny.contrasts) enrich.scrutiny.contrasts = [];
  if (!enrich.scrutiny.records) enrich.scrutiny.records = [];
  if (!enrich.scrutiny.endorsements) enrich.scrutiny.endorsements = [];
  if (!enrich.scrutiny.portals || !enrich.scrutiny.portals.length) {
    try {
      enrich.scrutiny.portals =
        scrutiny_portals_js(c.name || "", !!(c.is_judge || c.chamber === "judicial")) ||
        [];
    } catch {
      enrich.scrutiny.portals = [];
    }
  }
  return enrich.scrutiny;
}

/** Dated contributor rows → correlation timeline (stream tag for lane/color). */
function mergeTimelineReceipts(enrich, rows, stream) {
  if (!rows || !rows.length) return;
  const tagged = rows
    .filter((r) => r && (r.date || r.amount_display))
    .map((r) => ({ ...r, stream: stream || "receipt" }));
  if (!tagged.length) return;
  const prev = Array.isArray(enrich.timeline_receipts)
    ? enrich.timeline_receipts.filter((r) => r.stream !== stream)
    : [];
  enrich.timeline_receipts = [...prev, ...tagged];
}

function applyFlTrefinFinance(enrich, fin, account, cycle, c, url) {
  const lines = Number(fin.line_count) || 0;
  const profile =
    fin.committee_url ||
    c.source_url ||
    `https://dos.elections.myflorida.com/candidates/CanDetail.asp?account=${encodeURIComponent(account)}`;
  enrich.finance = {
    source: "fl_trefin",
    cycle: String(cycle),
    receipts_display: fin.contributions_sum_display || "—",
    disbursements_display: "—",
    cash_on_hand_display: "—",
    debts_display: null,
    individual_display: null,
    pac_display: null,
    party_display: null,
    coverage_end_date: null,
    source_label: "FL DOS TreFin (itemized contribution lines)",
    profile_url: profile,
    note: fin.note || "",
    line_count: lines,
    trefin_url: fin.trefin_url || url,
    account,
  };
  enrich.top_individuals = fin.top_contributors || [];
  mergeTimelineReceipts(enrich, fin.top_contributors, "individual");
  enrich.finance_unavailable = false;
  // B6: soft CF context — campaign account ≠ voter affiliation
  appendCommitteeAffiliation(enrich, {
    name: account ? `Account ${account}` : "",
    designation: "FL DOS campaign account",
    source: c.source_publisher || "Florida Division of Elections",
    source_url: profile,
  });
  return lines;
}

/** B6: one cited committee/account row; never mistaken for voter party. */
function appendCommitteeAffiliation(enrich, { name, designation, source, source_url }) {
  if (!name || !String(name).trim()) return;
  let span = null;
  try {
    span = campaign_committee_affiliation_js(
      String(name),
      designation || "Campaign committee",
      source || null,
      source_url || null
    );
  } catch {
    span = {
      party: "Committee",
      start: null,
      end: null,
      role: `${designation || "Campaign committee"} (≠ voter affiliation) · ${name}`,
      source: source || "Campaign finance",
      source_url: source_url || null,
    };
  }
  if (!span) return;
  const role = span.role || "";
  if ((enrich.affiliations || []).some((a) => a.role === role)) return;
  try {
    enrich.affiliations = merge_affiliation_spans_js(
      JSON.stringify(enrich.affiliations || []),
      JSON.stringify([span])
    );
  } catch {
    enrich.affiliations = [...(enrich.affiliations || []), span];
  }
}

/**
 * @param {object} c candidate shell fields
 * @param {(u: {id,label,status,detail?,completed,total}) => void} onStage
 */
export async function enrichCandidate(c, onStage = () => {}, opts = {}) {
  if (opts.fresh) {
    return withFreshCache(() =>
      enrichCandidate(c, onStage, { ...opts, fresh: false, _freshInner: true })
    );
  }
  const skipAi = !!opts.skipAi;
  const stages = planStages(c);
  const total = stages.length;
  let completed = 0;
  const cycle = getCycle();
  const enrich = {
    finance: null,
    finance_error: null,
    finance_unavailable: false,
    outside_spending: [],
    top_individuals: [],
    top_committees: [],
    /** Client aggregate cap for contributor tables (FEC/state). */
    contributors_fetch_cap: null,
    /** Itemized Sched A lines loaded before aggregation (FEC). */
    contributors_lines_loaded: null,
    /** Honest rate-limit / DEMO_KEY note for finance lists. */
    contributors_note: null,
    size_buckets: [],
    principal_committee: null,
    votes: [],
    votes_url: null,
    votes_source: null,
    votes_rate_limited: false,
    /** GovTrack/OS total available when API reports it (may exceed fetched cap). */
    votes_total_available: null,
    /** Client fetch cap for this stage (honest UI). */
    votes_fetch_cap: null,
    affiliations: ballot_affiliations_js(
      c.party || "",
      c.office || "",
      !!c.is_incumbent,
      !!(c.is_judge || c.chamber === "judicial"),
      c.source_publisher || null,
      c.source_url || null
    ),
    affiliations_source: null,
    openstates_configured: hasOpenStatesKey(),
    finance_cycle: cycle,
    dossier: empty_dossier_js(cycle || new Date().getFullYear()),
    scrutiny: { money: null, news: [], portals: [], claims: [], contrasts: [], records: [] },
    disclosure_portals: [],
  };

  const asOfYear = cycle || new Date().getFullYear();
  ensureScrutiny(enrich, c);
  // F6: official personal disclosure search portals (PDFs not auto-parsed).
  try {
    const ch = (c.chamber || "").toLowerCase();
    const fec = looks_like_fec_id_js((c.external_id || "").trim());
    if (fec || ch.includes("us_") || ch === "federal") {
      const portalCh =
        ch.includes("senate") || /^S/i.test((c.external_id || "").trim())
          ? "us_senate"
          : ch.includes("house") || /^H/i.test((c.external_id || "").trim())
            ? "us_house"
            : "";
      enrich.disclosure_portals = federal_disclosure_portals_js(portalCh || null) || [];
      if (enrich.dossier) {
        enrich.dossier.disclosure_portals = enrich.disclosure_portals;
      }
    }
  } catch {
    /* optional */
  }
  // Judges: bench time is political — note until dated service spans load.
  // Portals only here; fl_courts_bio + dense hosts fill facts when pages exist.
  if (c.is_judge || c.chamber === "judicial") {
    try {
      const d = enrich.dossier;
      if (d && d.career) {
        d.career.notes = [
          ...(d.career.notes || []),
          "Judicial / bench service counts as political time. Dated tenure not loaded yet — fraction incomplete.",
        ];
      }
      if (d && isFloridaCandidate(c)) {
        if (!d.disclosure_portals) d.disclosure_portals = [];
        const pushPortal = (label, url, note) => {
          if (!url) return;
          if (d.disclosure_portals.some((p) => p.url === url)) return;
          d.disclosure_portals.push({ label, url, note: note || "" });
        };
        try {
          const portals =
            fl_judge_decision_portals_js(c.office || "", c.name || "") || [];
          for (const p of portals) {
            if (p && p.url) pushPortal(p.label || "Portal", p.url, p.note || "");
          }
        } catch {
          let indexUrl = null;
          try {
            indexUrl = fl_courts_index_url_js(c.office || "") || null;
          } catch {
            indexUrl = null;
          }
          if (indexUrl) {
            pushPortal(
              "Florida courts judge directory",
              indexUrl,
              "Official court roster for this bench."
            );
          }
          try {
            const bar = fl_bar_member_search_url_js(c.name || "");
            if (bar) {
              pushPortal(
                "Florida Bar member search",
                bar,
                "Public lawyer directory (bot wall — open in browser)."
              );
            }
          } catch {
            /* optional */
          }
        }
        if (c.source_url) {
          pushPortal(
            "FL DOS candidate filing",
            c.source_url,
            "Division of Elections candidate detail (status, party designation, account)."
          );
        }
        enrich.judge_decision_portals = d.disclosure_portals.filter((p) =>
          /bar|opinion|court|flcourts|dca|directory|records/i.test(
            `${p.label || ""} ${p.url || ""}`
          )
        );
      }
      try {
        const clPortal = courtlistener_search_portal_url_js(c.name || "");
        if (clPortal && d) {
          if (!d.disclosure_portals) d.disclosure_portals = [];
          if (!d.disclosure_portals.some((p) => p.url === clPortal)) {
            d.disclosure_portals.push({
              label: "CourtListener search",
              url: clPortal,
              note: "Opinions and judicial biography database (Free Law Project).",
            });
          }
          enrich.courtlistener_search_url = clPortal;
        }
      } catch {
        /* optional */
      }
    } catch {
      /* ignore */
    }
  }
  const setDossierCareer = (career, photoUrl, photoSource, photoSourceUrl) => {
    if (!career) return;
    try {
      enrich.dossier = apply_career_to_dossier_js(
        JSON.stringify(enrich.dossier || empty_dossier_js(asOfYear)),
        JSON.stringify(career),
        photoUrl || null,
        photoSource || null,
        photoSourceUrl || null
      );
    } catch (e) {
      console.warn("dossier career", e);
    }
  };
  const addEndorsements = (rows) => {
    const extra = asEndorsementRows(rows);
    if (!extra.length) return 0;
    if (!enrich.dossier) {
      try {
        enrich.dossier = empty_dossier_js(asOfYear);
      } catch {
        enrich.dossier = { endorsements: [] };
      }
    }
    const list = enrich.dossier.endorsements || [];
    const seen = new Set(list.map((e) => `${e.stance}|${e.org}`.toLowerCase()));
    for (const r of extra) {
      const k = `${r.stance}|${r.org}`.toLowerCase();
      if (seen.has(k)) continue;
      seen.add(k);
      list.push(r);
    }
    enrich.dossier.endorsements = list;
    if (list.length) {
      enrich.dossier.empty_notes = (enrich.dossier.empty_notes || []).filter(
        (n) => !String(n).toLowerCase().startsWith("endorsements:")
      );
    }
    const sc = ensureScrutiny(enrich, c);
    sc.endorsements = list;
    return extra.length;
  };

  const emit = (id, label, status, detail) => {
    onStage({ id, label, status, detail, completed, total, enrich });
  };

  const done = (id, label, detail) => {
    completed += 1;
    emit(id, label, "done", detail);
  };
  const skip = (id, label, detail) => {
    completed += 1;
    if (id === "senate_efd" || id === "house_clerk_fd") {
      enrich.holdings_stage = id;
      enrich.holdings_skip = detail || "skipped";
    }
    emit(id, label, "skip", detail);
  };
  const doneHoldings = (id, label, detail) => {
    enrich.holdings_stage = id;
    enrich.holdings_skip = null;
    done(id, label, detail);
  };

  /** I4: record bio hosts we actually consulted (empty-state “Checked X — not found.”). */
  const markBioChecked = (label) => {
    if (!label || !enrich.dossier) return;
    try {
      enrich.dossier = note_source_checked_js(
        JSON.stringify(enrich.dossier),
        label
      );
    } catch (e) {
      const arr = enrich.dossier.sources_checked || [];
      if (!arr.some((s) => String(s).toLowerCase() === String(label).toLowerCase())) {
        arr.push(label);
      }
      enrich.dossier.sources_checked = arr;
    }
  };

  const run = async (stage, fn) => {
    emit(stage.id, stage.label, "running", null);
    try {
      await fn(stage);
    } catch (e) {
      console.warn(stage.id, e);
      completed += 1;
      emit(stage.id, stage.label, "error", e.message || String(e));
    }
  };

  const fecId = (c.external_id || "").trim();
  const isFec = fecId && looks_like_fec_id_js(fecId);

  const flAcct = flDosAccount(c);

  for (const stage of stages) {
    if (stage.id === "profile") {
      emit(stage.id, stage.label, "running", null);
      if (!enrich.finance && !flAcct) {
        enrich.finance_unavailable = true;
      }
      if (c.source_url) {
        enrich.profile_url = c.source_url;
      } else if (fecId && !isFec && /^https?:\/\//i.test(fecId)) {
        enrich.profile_url = fecId;
      }
      const note = c.source_url
        ? "source linked"
        : c.is_judge
          ? "judicial"
          : "no live API";
      done(stage.id, stage.label, note);
      continue;
    }

    if (stage.id === "fl_trefin") {
      await run(stage, async () => {
        const account = flAcct;
        if (!account) {
          enrich.finance_unavailable = true;
          skip(stage.id, stage.label, "no DOS account");
          return;
        }
        if (!hasWispConfigured() && !getCorsProxy()) {
          enrich.finance_error =
            "FL DOS TreFin needs Wisp (Settings) or a CORS proxy.";
          done(stage.id, stage.label, "needs Wisp");
          return;
        }
        if (c.source_url) enrich.profile_url = c.source_url;
        const url = fl_trefin_contrib_url_js(account);
        const key = `fl:trefin:contrib:${account}`;
        try {
          const html = await tryFetchText(url, key, 24 * 60 * 60 * 1000);
          if (!html) {
            enrich.finance_error =
              "TreFin fetch empty — enable Wisp or retry; public Wisp may throttle.";
            done(stage.id, stage.label, "empty fetch");
            return;
          }
          const STATE_CONTRIB_CAP = 50;
          const fin = parse_trefin_finance_js(html, account, STATE_CONTRIB_CAP);
          if (!fin) {
            enrich.finance_error =
              "TreFin response could not be parsed (HTML shape may have changed).";
            done(stage.id, stage.label, "parse fail");
            return;
          }
          const lines = applyFlTrefinFinance(enrich, fin, account, cycle, c, url);
          enrich.contributors_fetch_cap = STATE_CONTRIB_CAP;
          done(
            stage.id,
            stage.label,
            lines > 0
              ? `${lines} lines · ${(fin.top_contributors || []).length} donors · ${fin.contributions_sum_display || ""}`
              : "no itemized lines"
          );
        } catch (e) {
          enrich.finance_error = e.message || String(e);
          done(stage.id, stage.label, "error");
        }
      });
      continue;
    }

    if (stage.id === "fl_name_search") {
      await run(stage, async () => {
        // A2: contrib.exe name search → strict match → CanList account → TreFin
        if (flDosAccount(c)) {
          skip(stage.id, stage.label, "already has account");
          return;
        }
        if (!hasWispConfigured() && !getCorsProxy()) {
          enrich.finance_error =
            "FL DOS name-search needs Wisp (Settings) or a CORS proxy.";
          done(stage.id, stage.label, "needs Wisp");
          return;
        }
        if (c.source_url) enrich.profile_url = c.source_url;

        const elecId =
          (typeof fl_gen_elec_id_js === "function" && fl_gen_elec_id_js(cycle)) ||
          `${cycle}1103-GEN`;
        const parts =
          (typeof split_candidate_first_last_js === "function" &&
            split_candidate_first_last_js(c.name || "")) ||
          {};
        const last = (parts.last || "").trim();
        const first = (parts.first || "").trim();
        if (!last) {
          enrich.finance_unavailable = true;
          skip(stage.id, stage.label, "no last name");
          return;
        }
        const officeCode =
          (typeof office_code_from_ballot_js === "function" &&
            office_code_from_ballot_js(c.chamber || "", c.office || "")) ||
          "";
        const distNum =
          (typeof district_from_ballot_office_js === "function" &&
            district_from_ballot_office_js(c.office || "")) ||
          (typeof district_from_office_js === "function" &&
            district_from_office_js(c.office || "")) ||
          null;
        const distStr = distNum != null && distNum !== "" ? String(distNum) : "";

        const query = {
          name: c.name || "",
          office: c.office || "",
          chamber: c.chamber || "",
          party: c.party || "",
          district: distNum != null && distNum !== "" ? Number(distNum) : null,
          county: c.county || "",
        };

        const form =
          fl_contrib_candidate_totals_form_js(
            elecId,
            last,
            first.split(/\s+/)[0] || "",
            officeCode || "All",
            distStr
          ) || "";
        const contribUrl = fl_contrib_url_js();
        const cacheKey = `fl:contrib:totals:${elecId}:${last}:${first}:${officeCode || "All"}:${distStr || "-"}`.toLowerCase();

        let body = "";
        try {
          body = await tryPostForm(contribUrl, form, cacheKey, 24 * 60 * 60 * 1000, {
            minLength: 50,
          });
        } catch (e) {
          enrich.finance_error = e.message || String(e);
          done(stage.id, stage.label, "contrib fetch error");
          return;
        }
        if (!body) {
          enrich.finance_unavailable = true;
          done(stage.id, stage.label, "empty contrib response");
          return;
        }

        let hits = [];
        try {
          hits = parse_contrib_candidate_totals_js(body) || [];
        } catch (e) {
          enrich.finance_error = "contrib.exe parse failed";
          done(stage.id, stage.label, "parse fail");
          return;
        }
        if (!Array.isArray(hits)) hits = [];

        let matchOut;
        try {
          matchOut = match_fl_contrib_candidate_js(
            JSON.stringify(hits),
            JSON.stringify(query)
          );
        } catch (e) {
          enrich.finance_error = "name match failed";
          done(stage.id, stage.label, "match error");
          return;
        }

        const kind = (matchOut && matchOut.kind) || "";
        if (kind === "none") {
          enrich.finance_unavailable = true;
          done(stage.id, stage.label, "no DOS CF match");
          return;
        }
        if (kind === "ambiguous") {
          enrich.finance_unavailable = true;
          enrich.finance_error =
            `Ambiguous FL DOS name match (${matchOut.count || "?"} rows) — skipped to avoid wrong person.`;
          done(stage.id, stage.label, "ambiguous — skipped");
          return;
        }
        if (kind !== "unique" || !matchOut.hit) {
          enrich.finance_unavailable = true;
          done(stage.id, stage.label, "no unique match");
          return;
        }

        // Resolve CanDetail account via CanList (contrib totals have no account column).
        let account = null;
        try {
          const listUrl = fl_can_list_url_js();
          const listForm = fl_can_list_form_js(elecId);
          const listKey = `fl:canlist:${elecId}`;
          const listHtml = await tryPostForm(
            listUrl,
            listForm,
            listKey,
            24 * 60 * 60 * 1000
          );
          if (listHtml) {
            const listHits = parse_can_list_js(listHtml) || [];
            account = match_fl_can_list_account_js(
              JSON.stringify(listHits),
              JSON.stringify(query)
            );
          }
        } catch (e) {
          console.warn("[enrich] CanList account resolve", e);
        }
        if (account != null && typeof account !== "string") {
          account = account.account || account[0] || null;
        }
        if (typeof account !== "string" || !/^\d+$/.test(account)) {
          // Unique contrib hit but no account — surface totals only
          const hit = matchOut.hit;
          enrich.finance = {
            source: "fl_contrib_search",
            cycle: String(cycle),
            receipts_display: hit.total_display || "—",
            disbursements_display: "—",
            cash_on_hand_display: "—",
            debts_display: null,
            individual_display: null,
            pac_display: null,
            party_display: null,
            coverage_end_date: null,
            source_label: "FL DOS contributions search (name match; no TreFin account)",
            profile_url:
              c.source_url ||
              "https://dos.elections.myflorida.com/campaign-finance/contributions/",
            note:
              "Unique name+office+district match on contrib.exe totals, but CanDetail account was not resolved — itemized TreFin donors unavailable.",
            account: null,
            match_name: hit.name,
            match_office: hit.office_code,
            match_district: hit.district,
          };
          enrich.finance_unavailable = false;
          done(
            stage.id,
            stage.label,
            `unique · ${hit.total_display || "totals"} · no account`
          );
          return;
        }

        // TreFin path (same as A1)
        const url = fl_trefin_contrib_url_js(account);
        const key = `fl:trefin:contrib:${account}`;
        try {
          const html = await tryFetchText(url, key, 24 * 60 * 60 * 1000);
          if (!html) {
            enrich.finance_error =
              "TreFin fetch empty after name match — enable Wisp or retry.";
            done(stage.id, stage.label, "trefin empty");
            return;
          }
          const STATE_CONTRIB_CAP = 50;
          const fin = parse_trefin_finance_js(html, account, STATE_CONTRIB_CAP);
          if (!fin) {
            enrich.finance_error = "TreFin parse failed after name match.";
            done(stage.id, stage.label, "trefin parse fail");
            return;
          }
          const lines = applyFlTrefinFinance(enrich, fin, account, cycle, c, url);
          enrich.contributors_fetch_cap = STATE_CONTRIB_CAP;
          if (enrich.finance) {
            enrich.finance.source_label =
              "FL DOS TreFin via name-search (contrib.exe + CanList)";
            enrich.finance.resolved_via = "fl_name_search";
          }
          done(
            stage.id,
            stage.label,
            lines > 0
              ? `acct ${account} · ${lines} lines · ${(fin.top_contributors || []).length} donors · ${fin.contributions_sum_display || ""}`
              : `acct ${account} · no itemized lines`
          );
        } catch (e) {
          enrich.finance_error = e.message || String(e);
          done(stage.id, stage.label, "trefin error");
        }
      });
      continue;
    }

    if (stage.id === "fl_soe_cf") {
      await run(stage, async () => {
        // A5: FL county SOE via VoterFocus (after DOS name-search)
        if (enrich.finance && !enrich.finance_unavailable) {
          skip(stage.id, stage.label, "already have finance");
          return;
        }
        const county = (c.county || "").trim();
        const contactUrl =
          (typeof fl_soe_contact_url_js === "function" &&
            fl_soe_contact_url_js(county || "Florida")) ||
          "https://dos.myflorida.com/elections/contacts/supervisor-of-elections/";
        const vfParam =
          typeof voterfocus_county_param_js === "function"
            ? voterfocus_county_param_js(county)
            : null;
        if (!county || !vfParam) {
          if (!enrich.finance) {
            enrich.finance_unavailable = true;
            enrich.finance_note =
              (county
                ? `No VoterFocus SOE feed mapped for ${county}. `
                : "County unknown for SOE lookup. ") +
              `Contact the local Supervisor of Elections for campaign finance reports.`;
            enrich.profile_url = enrich.profile_url || contactUrl;
            enrich.soe_contact_url = contactUrl;
          }
          skip(
            stage.id,
            stage.label,
            county ? "no VoterFocus map — contact SOE" : "no county"
          );
          return;
        }
        if (!hasWispConfigured() && !getCorsProxy()) {
          enrich.finance_error =
            "FL county SOE (VoterFocus) needs Wisp (Settings) or a CORS proxy.";
          enrich.soe_contact_url = contactUrl;
          done(stage.id, stage.label, "needs Wisp");
          return;
        }

        const listUrl =
          (typeof fl_soe_candidate_list_url_js === "function" &&
            fl_soe_candidate_list_url_js(county)) ||
          "";
        if (!listUrl) {
          enrich.finance_unavailable = true;
          enrich.soe_contact_url = contactUrl;
          skip(stage.id, stage.label, "no list URL");
          return;
        }
        const cacheKey = `fl:soe:vf:${String(vfParam).toLowerCase()}:list`;
        let html = "";
        try {
          html = await tryFetchText(listUrl, cacheKey, 24 * 60 * 60 * 1000);
        } catch (e) {
          enrich.finance_error = e.message || String(e);
          enrich.soe_contact_url = contactUrl;
          done(stage.id, stage.label, "fetch error");
          return;
        }
        if (!html) {
          enrich.finance_unavailable = true;
          enrich.finance_note =
            "VoterFocus SOE list empty — try Wisp “This origin” or open the SOE site.";
          enrich.soe_contact_url = contactUrl;
          done(stage.id, stage.label, "empty list");
          return;
        }

        let hits = [];
        try {
          hits = parse_fl_soe_candidate_list_html_js(html) || [];
        } catch (e) {
          enrich.finance_error = e.message || "SOE list parse failed";
          enrich.soe_contact_url = contactUrl;
          done(stage.id, stage.label, "parse fail");
          return;
        }
        if (!Array.isArray(hits)) hits = [];

        const distNum =
          (typeof district_from_ballot_office_js === "function" &&
            district_from_ballot_office_js(c.office || "")) ||
          (typeof district_from_office_js === "function" &&
            district_from_office_js(c.office || "")) ||
          null;
        const dist =
          distNum != null && distNum !== "" ? Number(distNum) : null;

        let matchOut;
        try {
          matchOut = match_fl_soe_candidate_js(
            JSON.stringify(hits),
            JSON.stringify({
              name: c.name || "",
              office: c.office || "",
              chamber: c.chamber || "",
              party: c.party || "",
              district: dist,
            })
          );
        } catch (e) {
          enrich.finance_error = "SOE name match failed";
          done(stage.id, stage.label, "match error");
          return;
        }

        const kind = (matchOut && matchOut.kind) || "";
        if (kind === "none") {
          if (!enrich.finance) {
            enrich.finance_unavailable = true;
            enrich.finance_note = `No ${county} SOE VoterFocus row matched ${
              c.name || "candidate"
            }. Reports may still be on file at the SOE.`;
            enrich.soe_contact_url = contactUrl;
            enrich.profile_url = enrich.profile_url || contactUrl;
          }
          done(stage.id, stage.label, "no row");
          return;
        }
        if (kind === "ambiguous") {
          if (!enrich.finance) {
            enrich.finance_unavailable = true;
            enrich.finance_error = `Ambiguous SOE match (${
              matchOut.count || "?"
            }) — skipped to avoid wrong person.`;
            enrich.soe_contact_url = contactUrl;
          }
          done(stage.id, stage.label, "ambiguous — skipped");
          return;
        }
        if (kind !== "unique" || !matchOut.hit) {
          if (!enrich.finance) {
            enrich.finance_unavailable = true;
            enrich.soe_contact_url = contactUrl;
          }
          done(stage.id, stage.label, "no unique match");
          return;
        }

        let fin;
        try {
          fin = fl_soe_finance_from_hit_js(
            JSON.stringify(matchOut.hit),
            cycle,
            county
          );
        } catch (e) {
          enrich.finance_error = e.message || "SOE finance build failed";
          done(stage.id, stage.label, "build fail");
          return;
        }
        if (!fin) {
          enrich.finance_unavailable = true;
          done(stage.id, stage.label, "empty finance");
          return;
        }

        enrich.finance = {
          source: "fl_soe",
          cycle: String(fin.cycle || cycle),
          receipts_display: fin.receipts_display || "—",
          disbursements_display: fin.disbursements_display || "—",
          cash_on_hand_display: fin.cash_on_hand_display || "—",
          in_kind_display: fin.in_kind_display || null,
          debts_display: null,
          individual_display: null,
          pac_display: null,
          party_display: null,
          coverage_end_date: null,
          source_label: fin.source_label || "County SOE (VoterFocus)",
          profile_url: fin.profile_url || contactUrl,
          note: fin.note || "",
          account: fin.account || String(matchOut.hit.candidate_id || ""),
          match_name: fin.match_name || matchOut.hit.name || "",
          match_office: fin.match_office || matchOut.hit.office || "",
        };
        enrich.finance_unavailable = false;
        enrich.finance_error = null;
        enrich.soe_contact_url = contactUrl;
        if (fin.profile_url) enrich.profile_url = fin.profile_url;
        done(
          stage.id,
          stage.label,
          `unique · ${fin.receipts_display || "totals"}`
        );
      });
      continue;
    }

    if (stage.id === "md_cf") {
      await run(stage, async () => {
        // A4: Maryland MDCRIS (api-campaignfinance.maryland.gov)
        if (!hasWispConfigured() && !getCorsProxy()) {
          enrich.finance_error =
            "MD MDCRIS needs Wisp (Settings) or a CORS proxy.";
          done(stage.id, stage.label, "needs Wisp");
          return;
        }
        if (c.source_url) enrich.profile_url = c.source_url;

        const frag =
          (typeof md_cf_search_name_fragment_js === "function" &&
            md_cf_search_name_fragment_js(c.name || "")) ||
          (c.name || "").trim().split(/\s+/).pop() ||
          "";
        if (!frag) {
          enrich.finance_unavailable = true;
          skip(stage.id, stage.label, "no name");
          return;
        }

        const distNum =
          (typeof district_from_ballot_office_js === "function" &&
            district_from_ballot_office_js(c.office || "")) ||
          (typeof district_from_office_js === "function" &&
            district_from_office_js(c.office || "")) ||
          null;
        const dist =
          distNum != null && distNum !== "" ? Number(distNum) : null;

        const listUrl =
          (typeof md_cf_committee_list_url_js === "function" &&
            md_cf_committee_list_url_js()) ||
          "https://api-campaignfinance.maryland.gov/api/PublicGrid/GetCommitteeList";
        const listBody =
          (typeof md_cf_committee_list_body_js === "function" &&
            md_cf_committee_list_body_js(frag, 50)) ||
          JSON.stringify({ pageNumber: 1, pageSize: 50, filerName: frag });
        const cacheKey = `md:cf:committee:${frag.toLowerCase()}`;

        let raw = "";
        try {
          raw = await tryPostJson(listUrl, listBody, cacheKey, 24 * 60 * 60 * 1000, {
            minLength: 20,
            headers: {
              Origin: "https://campaignfinance.maryland.gov",
              Referer: "https://campaignfinance.maryland.gov/",
            },
          });
        } catch (e) {
          enrich.finance_error = e.message || String(e);
          done(stage.id, stage.label, "fetch error");
          return;
        }
        if (!raw) {
          enrich.finance_unavailable = true;
          done(stage.id, stage.label, "empty committee list");
          return;
        }

        let hits = [];
        try {
          hits = parse_md_cf_committee_list_json_js(raw) || [];
        } catch (e) {
          enrich.finance_error = e.message || "MD CF parse failed";
          done(stage.id, stage.label, "parse fail");
          return;
        }
        if (!Array.isArray(hits)) hits = [];

        let matchOut;
        try {
          matchOut = match_md_cf_candidate_js(
            JSON.stringify(hits),
            JSON.stringify({
              name: c.name || "",
              office: c.office || "",
              chamber: c.chamber || "",
              party: c.party || "",
              district: dist,
              county: c.county || "",
            })
          );
        } catch (e) {
          enrich.finance_error = "MD CF name match failed";
          done(stage.id, stage.label, "match error");
          return;
        }

        const kind = (matchOut && matchOut.kind) || "";
        if (kind === "none") {
          enrich.finance_unavailable = true;
          enrich.finance = null;
          enrich.finance_note = `No MDCRIS committee matched ${c.name || "candidate"}.`;
          done(stage.id, stage.label, "no row");
          return;
        }
        if (kind === "ambiguous") {
          enrich.finance_unavailable = true;
          enrich.finance_error = `Ambiguous MDCRIS match (${
            matchOut.count || "?"
          }) — skipped to avoid wrong person.`;
          done(stage.id, stage.label, "ambiguous — skipped");
          return;
        }
        if (kind !== "unique" || !matchOut.hit) {
          enrich.finance_unavailable = true;
          done(stage.id, stage.label, "no unique match");
          return;
        }

        const hit = matchOut.hit;
        let summaryRaw = "";
        const guid = (hit.filer_registration_guid || "").trim();
        if (guid) {
          try {
            const sumUrl =
              (typeof md_cf_financial_summary_url_js === "function" &&
                md_cf_financial_summary_url_js()) ||
              "https://api-campaignfinance.maryland.gov/api/PublicFilerDetails/GetFinancialSummaryDetails";
            const sumBody =
              (typeof md_cf_financial_summary_body_js === "function" &&
                md_cf_financial_summary_body_js(guid)) ||
              JSON.stringify({ filerRegistrationGuid: guid });
            summaryRaw = await tryPostJson(
              sumUrl,
              sumBody,
              `md:cf:summary:${guid}`,
              24 * 60 * 60 * 1000,
              {
                minLength: 20,
                headers: {
                  Origin: "https://campaignfinance.maryland.gov",
                  Referer: "https://campaignfinance.maryland.gov/",
                },
              }
            );
          } catch {
            summaryRaw = "";
          }
        }

        let fin;
        try {
          fin = md_cf_finance_from_hit_js(
            JSON.stringify(hit),
            cycle,
            summaryRaw || undefined
          );
        } catch (e) {
          enrich.finance_error = e.message || "MD CF finance build failed";
          done(stage.id, stage.label, "build fail");
          return;
        }
        if (!fin) {
          enrich.finance_unavailable = true;
          done(stage.id, stage.label, "empty finance");
          return;
        }

        enrich.finance = {
          source: "md_cf",
          cycle: String(fin.cycle || cycle),
          receipts_display: fin.receipts_display || "—",
          disbursements_display: fin.disbursements_display || "—",
          cash_on_hand_display: fin.cash_on_hand_display || "—",
          debts_display: null,
          individual_display: null,
          pac_display: null,
          party_display: null,
          coverage_end_date: null,
          source_label: fin.source_label || "Maryland MDCRIS",
          profile_url: fin.profile_url || c.source_url || "",
          note: fin.note || "",
          account: fin.account || guid || String(hit.filing_entity_id || ""),
          match_name: fin.match_name || hit.candidate_name || "",
          match_office: fin.match_office || hit.office_sought || "",
        };
        enrich.finance_unavailable = false;
        if (fin.profile_url) enrich.profile_url = fin.profile_url;
        // C5/B6: soft CF committee context (≠ voter affiliation)
        appendCommitteeAffiliation(enrich, {
          name:
            hit.committee_name ||
            hit.committee ||
            fin.match_name ||
            fin.account ||
            "",
          designation: "MDCRIS campaign committee",
          source: fin.source_label || "Maryland MDCRIS",
          source_url: fin.profile_url || "",
        });
        done(
          stage.id,
          stage.label,
          `unique · ${fin.receipts_display || "totals"}`
        );
      });
      continue;
    }

    if (stage.id === "az_cf") {
      await run(stage, async () => {
        // A4: Arizona SeeTheMoney (seethemoney.az.gov)
        if (!hasWispConfigured() && !getCorsProxy()) {
          enrich.finance_error =
            "AZ SeeTheMoney needs Wisp (Settings) or a CORS proxy.";
          done(stage.id, stage.label, "needs Wisp");
          return;
        }
        if (c.source_url) enrich.profile_url = c.source_url;

        const frag =
          (typeof az_cf_search_name_fragment_js === "function" &&
            az_cf_search_name_fragment_js(c.name || "")) ||
          (c.name || "").trim().split(/\s+/).pop() ||
          "";
        const distNum =
          (typeof district_from_ballot_office_js === "function" &&
            district_from_ballot_office_js(c.office || "")) ||
          (typeof district_from_office_js === "function" &&
            district_from_office_js(c.office || "")) ||
          null;
        const dist =
          distNum != null && distNum !== "" ? Number(distNum) : null;
        const officeId =
          typeof az_cf_office_id_js === "function"
            ? az_cf_office_id_js(c.chamber || "", dist != null ? dist : undefined)
            : null;

        const endYear = cycle;
        const startYear = Math.max(2000, cycle - 2);
        const url =
          (typeof az_cf_candidates_url_js === "function" &&
            az_cf_candidates_url_js(
              startYear,
              endYear,
              frag,
              officeId != null ? officeId : undefined
            )) ||
          "";
        const body =
          (typeof az_cf_datatables_body_js === "function" &&
            az_cf_datatables_body_js()) ||
          "draw=1&start=0&length=25";
        const cacheKey = `az:cf:table:${startYear}-${endYear}:${frag}:${officeId || "all"}`;

        let raw = "";
        try {
          raw = await tryPostForm(url, body, cacheKey, 24 * 60 * 60 * 1000, {
            minLength: 2,
            headers: {
              "X-Requested-With": "XMLHttpRequest",
              Accept: "application/json, text/javascript, */*; q=0.01",
            },
          });
        } catch (e) {
          enrich.finance_error = e.message || String(e);
          done(stage.id, stage.label, "fetch error");
          return;
        }
        if (!raw || raw.trim() === '""') {
          enrich.finance_unavailable = true;
          enrich.finance_note =
            "SeeTheMoney returned no candidate rows (portal may block non-browser clients — try Wisp “This origin”).";
          done(stage.id, stage.label, "empty table");
          return;
        }

        let hits = [];
        try {
          hits = parse_az_cf_table_json_js(raw) || [];
        } catch (e) {
          enrich.finance_error = e.message || "AZ CF parse failed";
          done(stage.id, stage.label, "parse fail");
          return;
        }
        if (!Array.isArray(hits)) hits = [];

        let matchOut;
        try {
          matchOut = match_az_cf_candidate_js(
            JSON.stringify(hits),
            JSON.stringify({
              name: c.name || "",
              office: c.office || "",
              chamber: c.chamber || "",
              party: c.party || "",
              district: dist,
            })
          );
        } catch (e) {
          enrich.finance_error = "AZ CF name match failed";
          done(stage.id, stage.label, "match error");
          return;
        }

        const kind = (matchOut && matchOut.kind) || "";
        if (kind === "none") {
          enrich.finance_unavailable = true;
          enrich.finance = null;
          enrich.finance_note = `No SeeTheMoney row matched ${c.name || "candidate"}.`;
          done(stage.id, stage.label, "no row");
          return;
        }
        if (kind === "ambiguous") {
          enrich.finance_unavailable = true;
          enrich.finance_error = `Ambiguous SeeTheMoney match (${
            matchOut.count || "?"
          }) — skipped to avoid wrong person.`;
          done(stage.id, stage.label, "ambiguous — skipped");
          return;
        }
        if (kind !== "unique" || !matchOut.hit) {
          enrich.finance_unavailable = true;
          done(stage.id, stage.label, "no unique match");
          return;
        }

        let fin;
        try {
          fin = az_cf_finance_from_hit_js(JSON.stringify(matchOut.hit), cycle);
        } catch (e) {
          enrich.finance_error = e.message || "AZ CF finance build failed";
          done(stage.id, stage.label, "build fail");
          return;
        }
        if (!fin) {
          enrich.finance_unavailable = true;
          done(stage.id, stage.label, "empty finance");
          return;
        }

        enrich.finance = {
          source: "az_cf",
          cycle: String(fin.cycle || cycle),
          receipts_display: fin.receipts_display || "—",
          disbursements_display: fin.disbursements_display || "—",
          cash_on_hand_display: fin.cash_on_hand_display || "—",
          debts_display: null,
          individual_display: null,
          pac_display: null,
          party_display: null,
          coverage_end_date: null,
          source_label: fin.source_label || "Arizona SeeTheMoney",
          profile_url: fin.profile_url || c.source_url || "",
          note: fin.note || "",
          account: fin.account || String(matchOut.hit.entity_id || ""),
          match_name: fin.match_name || matchOut.hit.name || "",
          match_office: fin.match_office || matchOut.hit.office_name || "",
        };
        enrich.finance_unavailable = false;
        if (fin.profile_url) enrich.profile_url = fin.profile_url;
        // C5/B6: soft CF committee context (≠ voter affiliation)
        appendCommitteeAffiliation(enrich, {
          name:
            matchOut.hit.committee_name ||
            matchOut.hit.name ||
            fin.match_name ||
            fin.account ||
            "",
          designation: "SeeTheMoney campaign committee",
          source: fin.source_label || "Arizona SeeTheMoney",
          source_url: fin.profile_url || "",
        });
        done(
          stage.id,
          stage.label,
          `unique · ${fin.receipts_display || "totals"}`
        );
      });
      continue;
    }

    if (stage.id === "nc_cf") {
      await run(stage, async () => {
        // A4: NCSBE campaign finance (cf.ncsbe.gov)
        if (!hasWispConfigured() && !getCorsProxy()) {
          enrich.finance_error =
            "NC campaign finance needs Wisp (Settings) or a CORS proxy.";
          done(stage.id, stage.label, "needs Wisp");
          return;
        }
        if (c.source_url) enrich.profile_url = c.source_url;

        const frag =
          (typeof nc_cf_search_name_fragment_js === "function" &&
            nc_cf_search_name_fragment_js(c.name || "")) ||
          (c.name || "").trim().split(/\s+/).pop() ||
          "";
        if (!frag) {
          enrich.finance_unavailable = true;
          done(stage.id, stage.label, "no name");
          return;
        }

        const searchUrl =
          (typeof nc_cf_search_url_js === "function" && nc_cf_search_url_js()) ||
          "https://cf.ncsbe.gov/CFOrgLkup/";
        const form =
          (typeof nc_cf_committee_search_form_js === "function" &&
            nc_cf_committee_search_form_js(frag)) ||
          "";
        const searchKey = `nc:cf:search:${frag.toLowerCase()}`;
        let searchHtml = "";
        try {
          searchHtml = await tryPostForm(searchUrl, form, searchKey, 24 * 60 * 60 * 1000, {
            minLength: 200,
          });
        } catch (e) {
          enrich.finance_error = e.message || String(e);
          done(stage.id, stage.label, "search fetch error");
          return;
        }
        if (!searchHtml) {
          enrich.finance_error = "NC CF committee search empty — enable Wisp or retry.";
          done(stage.id, stage.label, "empty search");
          return;
        }

        let hits = [];
        try {
          hits = parse_nc_cf_committee_search_html_js(searchHtml) || [];
        } catch (e) {
          enrich.finance_error = e.message || "NC CF search parse failed";
          done(stage.id, stage.label, "parse fail");
          return;
        }
        if (!Array.isArray(hits)) hits = [];

        let matchOut;
        try {
          matchOut = match_nc_cf_committee_js(
            JSON.stringify(hits),
            JSON.stringify({
              name: c.name || "",
              office: c.office || "",
              chamber: c.chamber || "",
              party: c.party || "",
            })
          );
        } catch (e) {
          enrich.finance_error = "NC CF name match failed";
          done(stage.id, stage.label, "match error");
          return;
        }

        const kind = (matchOut && matchOut.kind) || "";
        if (kind === "none") {
          enrich.finance_unavailable = true;
          enrich.finance = null;
          enrich.finance_note = `No NCSBE campaign-finance committee matched ${c.name || "candidate"}.`;
          done(stage.id, stage.label, "no committee");
          return;
        }
        if (kind === "ambiguous") {
          enrich.finance_unavailable = true;
          enrich.finance_error = `Ambiguous NCSBE committee match (${
            matchOut.count || "?"
          }) — skipped to avoid wrong person.`;
          done(stage.id, stage.label, "ambiguous — skipped");
          return;
        }
        if (kind !== "unique" || !matchOut.hit) {
          enrich.finance_unavailable = true;
          done(stage.id, stage.label, "no unique match");
          return;
        }

        const hit = matchOut.hit;
        const sboe = hit.sboe_id || hit.sboeId || "";
        const ogid = Number(hit.org_group_id ?? hit.orgGroupId ?? 0) || 0;
        if (!sboe || !ogid) {
          enrich.finance_unavailable = true;
          enrich.finance_error = "NCSBE match missing committee id.";
          done(stage.id, stage.label, "no id");
          return;
        }

        const docsUrl =
          (typeof nc_cf_documents_url_js === "function" &&
            nc_cf_documents_url_js(sboe, ogid)) ||
          "";
        const docsKey = `nc:cf:docs:${sboe}:${ogid}`;
        let docsHtml = "";
        try {
          docsHtml = await tryFetchText(docsUrl, docsKey, 24 * 60 * 60 * 1000);
        } catch (e) {
          enrich.finance_error = e.message || String(e);
          done(stage.id, stage.label, "docs fetch error");
          return;
        }
        if (!docsHtml) {
          enrich.finance_error = "NCSBE documents page empty.";
          done(stage.id, stage.label, "empty docs");
          return;
        }

        let docs = [];
        try {
          docs = parse_nc_cf_documents_html_js(docsHtml) || [];
        } catch (e) {
          enrich.finance_error = e.message || "NC CF docs parse failed";
          done(stage.id, stage.label, "docs parse fail");
          return;
        }
        if (!Array.isArray(docs)) docs = [];

        let report = null;
        try {
          report = pick_latest_nc_disclosure_js(JSON.stringify(docs), cycle);
        } catch (e) {
          enrich.finance_error = "NC CF report pick failed";
          done(stage.id, stage.label, "pick fail");
          return;
        }
        if (!report || !(report.data_link || report.dataLink)) {
          enrich.finance_unavailable = true;
          enrich.finance_note = "NCSBE committee found but no disclosure report with data.";
          enrich.profile_url = docsUrl;
          done(stage.id, stage.label, "no report");
          return;
        }

        const rid = String(report.data_link || report.dataLink);
        const sumUrl =
          (typeof nc_cf_summary_csv_url_js === "function" &&
            nc_cf_summary_csv_url_js(rid)) ||
          "";
        const sumKey = `nc:cf:sum:${rid}`;
        let sumCsv = "";
        try {
          sumCsv = await tryFetchText(sumUrl, sumKey, 24 * 60 * 60 * 1000);
        } catch (e) {
          enrich.finance_error = e.message || String(e);
          done(stage.id, stage.label, "summary fetch error");
          return;
        }
        if (!sumCsv) {
          enrich.finance_error = "NCSBE summary CSV empty.";
          done(stage.id, stage.label, "empty summary");
          return;
        }

        let sum = null;
        try {
          sum = parse_nc_cf_summary_csv_js(sumCsv);
        } catch (e) {
          enrich.finance_error = e.message || "NC CF summary parse failed";
          done(stage.id, stage.label, "summary parse fail");
          return;
        }
        if (!sum) {
          enrich.finance_unavailable = true;
          done(stage.id, stage.label, "no summary");
          return;
        }

        let fin;
        try {
          fin = nc_cf_finance_from_parts_js(
            JSON.stringify(hit),
            JSON.stringify(sum),
            JSON.stringify(report),
            cycle
          );
        } catch (e) {
          enrich.finance_error = e.message || "NC CF finance build failed";
          done(stage.id, stage.label, "build fail");
          return;
        }
        if (!fin) {
          enrich.finance_unavailable = true;
          done(stage.id, stage.label, "empty finance");
          return;
        }

        enrich.finance = {
          source: "nc_cf",
          cycle: String(fin.cycle || cycle),
          receipts_display: fin.receipts_display || "—",
          disbursements_display: fin.disbursements_display || "—",
          cash_on_hand_display: fin.cash_on_hand_display || "—",
          debts_display: null,
          individual_display: null,
          pac_display: null,
          party_display: null,
          coverage_end_date: null,
          source_label: fin.source_label || "N.C. State Board of Elections campaign finance",
          profile_url: fin.profile_url || docsUrl,
          report_url: fin.report_url || "",
          note: fin.note || "",
          account: fin.account || sboe,
          match_name: fin.match_name || hit.cand_name || "",
          match_office: fin.match_office || hit.org_name || "",
          report_id: fin.report_id || rid,
          report_name: fin.report_name || "",
        };
        enrich.finance_unavailable = false;
        if (fin.profile_url) enrich.profile_url = fin.profile_url;
        // C5/B6: soft CF committee context (≠ voter affiliation)
        appendCommitteeAffiliation(enrich, {
          name:
            hit.org_name ||
            hit.cand_name ||
            fin.match_name ||
            sboe ||
            "",
          designation: "NCSBE campaign committee",
          source:
            fin.source_label || "N.C. State Board of Elections campaign finance",
          source_url: fin.profile_url || docsUrl || "",
        });
        done(
          stage.id,
          stage.label,
          `unique · ${fin.receipts_display || "totals"} · ${fin.report_name || rid}`
        );
      });
      continue;
    }

    if (stage.id === "ftm") {
      await run(stage, async () => {
        // A3: FollowTheMoney state CF for non-FL candidates
        const apiKey = getFtmApiKey();
        if (!apiKey) {
          enrich.finance_unavailable = true;
          skip(stage.id, stage.label, "no FTM key");
          return;
        }
        if (!hasWispConfigured() && !getCorsProxy()) {
          enrich.finance_error =
            "FollowTheMoney API needs Wisp (Settings) or a CORS proxy.";
          done(stage.id, stage.label, "needs Wisp");
          return;
        }
        if (c.source_url) enrich.profile_url = c.source_url;

        const st =
          (c.state_code || "").toUpperCase() ||
          (typeof state_code_from_jurisdiction_js === "function"
            ? state_code_from_jurisdiction_js(
                c.jurisdiction || c.jurisdiction_ocd || "",
                c.office || ""
              )
            : "") ||
          "";
        if (!st || st.length !== 2) {
          enrich.finance_unavailable = true;
          done(stage.id, stage.label, "no state code");
          return;
        }

        const year =
          (typeof ftm_data_year_js === "function" && ftm_data_year_js(cycle)) ||
          Math.min(cycle, 2024);
        const ot =
          (typeof ftm_office_type_code_js === "function" &&
            ftm_office_type_code_js(c.chamber || "", c.office || "")) ||
          null;
        const distNum =
          (typeof district_from_ballot_office_js === "function" &&
            district_from_ballot_office_js(c.office || "")) ||
          (typeof district_from_office_js === "function" &&
            district_from_office_js(c.office || "")) ||
          null;

        const url = ftm_candidates_url_js(apiKey, st, year, ot || undefined);
        const cacheKey = `ftm:cands:${st}:${year}:${ot || "all"}`;
        let body = "";
        try {
          body = await tryFetchText(url, cacheKey, 24 * 60 * 60 * 1000);
        } catch (e) {
          enrich.finance_error = e.message || String(e);
          done(stage.id, stage.label, "fetch error");
          return;
        }
        if (!body) {
          enrich.finance_error =
            "FollowTheMoney fetch empty — check API key, Wisp, or retry.";
          done(stage.id, stage.label, "empty fetch");
          return;
        }
        const apiErr =
          typeof ftm_error_message_js === "function"
            ? ftm_error_message_js(body)
            : null;
        if (apiErr) {
          enrich.finance_error = `FollowTheMoney: ${apiErr}`;
          done(stage.id, stage.label, "api error");
          return;
        }

        let hits = [];
        try {
          hits = parse_ftm_candidate_records_js(body) || [];
        } catch (e) {
          enrich.finance_error = e.message || "FTM parse failed";
          done(stage.id, stage.label, "parse fail");
          return;
        }
        if (!Array.isArray(hits)) hits = [];

        const query = {
          name: c.name || "",
          office: c.office || "",
          chamber: c.chamber || "",
          party: c.party || "",
          district: distNum != null && distNum !== "" ? Number(distNum) : null,
          state: st,
        };
        let matchOut;
        try {
          matchOut = match_ftm_candidate_js(
            JSON.stringify(hits),
            JSON.stringify(query)
          );
        } catch (e) {
          enrich.finance_error = "FTM name match failed";
          done(stage.id, stage.label, "match error");
          return;
        }

        const kind = (matchOut && matchOut.kind) || "";
        if (kind === "none") {
          enrich.finance_unavailable = true;
          enrich.finance = null;
          enrich.finance_note =
            `No FollowTheMoney row for ${c.name || "candidate"} (${st} ${year}${
              ot ? ` · ${ot}` : ""
            }).`;
          done(stage.id, stage.label, "no row");
          return;
        }
        if (kind === "ambiguous") {
          enrich.finance_unavailable = true;
          enrich.finance_error = `Ambiguous FollowTheMoney name match (${
            matchOut.count || "?"
          } rows) — skipped to avoid wrong person.`;
          done(stage.id, stage.label, "ambiguous — skipped");
          return;
        }
        if (kind !== "unique" || !matchOut.hit) {
          enrich.finance_unavailable = true;
          done(stage.id, stage.label, "no unique match");
          return;
        }

        const hit = matchOut.hit;
        let donorsBody = "";
        try {
          const dUrl = ftm_top_donors_url_js(apiKey, hit.id, st, year);
          const dKey = `ftm:donors:${hit.id}:${st}:${year}`;
          donorsBody = await tryFetchText(dUrl, dKey, 24 * 60 * 60 * 1000);
        } catch (e) {
          console.warn("[enrich] FTM donors", e);
        }

        let fin;
        try {
          fin = ftm_finance_from_hit_js(
            JSON.stringify(hit),
            st,
            year,
            ot || undefined,
            donorsBody || ""
          );
        } catch (e) {
          enrich.finance_error = e.message || "FTM finance build failed";
          done(stage.id, stage.label, "build fail");
          return;
        }
        if (!fin) {
          enrich.finance_unavailable = true;
          done(stage.id, stage.label, "empty finance");
          return;
        }

        enrich.finance = {
          source: "ftm",
          cycle: String(fin.cycle || year),
          receipts_display: fin.receipts_display || "—",
          disbursements_display: "—",
          cash_on_hand_display: "—",
          debts_display: null,
          individual_display: null,
          pac_display: null,
          party_display: null,
          coverage_end_date: null,
          source_label: fin.source_label || "FollowTheMoney / OpenSecrets",
          profile_url: fin.profile_url || c.source_url || "",
          note: fin.note || "",
          show_me_url: fin.show_me_url || "",
          candidate_id: fin.candidate_id || hit.id,
          match_name: fin.candidate_name || hit.name,
          match_office: fin.office_sought || fin.office || "",
          account: null,
        };
        enrich.top_individuals = fin.top_contributors || [];
        mergeTimelineReceipts(enrich, fin.top_contributors, "individual");
        enrich.finance_unavailable = false;
        if (fin.profile_url) enrich.profile_url = fin.profile_url;
        appendCommitteeAffiliation(enrich, {
          name: fin.candidate_name || hit.name || "",
          designation: "FollowTheMoney candidate record",
          source: fin.source_label || "FollowTheMoney / OpenSecrets",
          source_url: fin.profile_url || fin.show_me_url || "",
        });
        const nDonors = (enrich.top_individuals && enrich.top_individuals.length) || 0;
        done(
          stage.id,
          stage.label,
          `${fin.receipts_display || "totals"}${
            nDonors ? ` · ${nDonors} donors` : ""
          }`
        );
      });
      continue;
    }

    if (stage.id === "totals" && isFec) {
      await run(stage, async () => {
        const url = fecUrl(`/v1/candidate/${fecId}/totals/`, {
          cycle,
          per_page: 20,
          sort: "-cycle",
        });
        try {
          const { body } = await cachedFetch(url, {
            key: `fec:totals:${fecId}:${cycle}`,
          });
          enrich.finance = parse_fec_totals_js(body, fecId, cycle);
          done(
            stage.id,
            stage.label,
            enrich.finance ? "loaded" : "no totals row"
          );
        } catch (e) {
          enrich.finance_error = e.message || String(e);
          done(stage.id, stage.label, "error");
        }
      });
      continue;
    }

    if (stage.id === "outside" && isFec) {
      await run(stage, async () => {
        const url = fecUrl(`/v1/schedules/schedule_e/by_candidate/`, {
          candidate_id: fecId,
          cycle,
          per_page: 20,
          sort: "-total",
        });
        const { body } = await cachedFetch(url, {
          key: `fec:ie:${fecId}:${cycle}:20`,
        });
        enrich.outside_spending = parse_fec_ie_js(body) || [];
        // F5: IE support/oppose orgs → dossier endorsements (cited FEC).
        try {
          const ends = [];
          for (const row of enrich.outside_spending) {
            const e = endorsement_from_ie_js(
              row.committee || "",
              row.support_oppose || "",
              row.url || ""
            );
            if (e) ends.push(e);
          }
          addEndorsements(ends);
          if (ends.length && enrich.dossier) {
            try {
              enrich.dossier = merge_endorsements_js(
                JSON.stringify(enrich.dossier),
                JSON.stringify(ends)
              );
            } catch {
              /* addEndorsements already updated in place */
            }
          }
        } catch {
          /* keep finance rows even if endorsement map fails */
        }
        done(stage.id, stage.label, `${enrich.outside_spending.length} rows`);
      });
      continue;
    }

    if (stage.id === "indiv" && isFec) {
      await run(stage, async () => {
        const cmteId = enrich.principal_committee?.committee_id;
        const params = {
          two_year_transaction_period: cycle,
          sort: "-contribution_receipt_amount",
          is_individual: true,
          hide_null: true,
        };
        if (cmteId) params.committee_id = cmteId;
        else params.candidate_id = fecId;
        const keyBase = `fec:sched_a_ind:${cmteId || fecId}:${cycle}`;
        const merged = await fetchFecSchedAMerged(params, keyBase);
        enrich._sched_a_indiv_body = merged.body;
        enrich.top_individuals =
          parse_fec_sched_a_js(merged.body, merged.contribCap) || [];
        try {
          const lines = parse_fec_sched_a_lines_js(merged.body, 400) || [];
          const prev = Array.isArray(enrich.timeline_receipts)
            ? enrich.timeline_receipts.filter((r) => r.stream !== "individual")
            : [];
          enrich.timeline_receipts = [
            ...prev,
            ...lines.map((r) => ({ ...r, stream: "individual" })),
          ];
        } catch {
          /* tops still usable */
        }
        enrich.contributors_fetch_cap = merged.contribCap;
        enrich.contributors_lines_loaded =
          (enrich.contributors_lines_loaded || 0) + merged.lines;
        if (merged.demo) {
          enrich.contributors_note =
            "OpenFEC DEMO_KEY: one Sched A page (~100 lines) · ~40 requests/hour. Set a personal key in Settings for up to 3 pages and 50 unique donors.";
        } else {
          const avail =
            merged.totalCount != null && merged.totalCount > merged.lines
              ? ` of ${merged.totalCount.toLocaleString()} available`
              : "";
          enrich.contributors_note = `Aggregated from ${merged.lines.toLocaleString()} itemized individual lines (${merged.pagesFetched} page${merged.pagesFetched === 1 ? "" : "s"}${avail}) → top ${merged.contribCap} unique donors.`;
        }
        done(
          stage.id,
          stage.label,
          `${enrich.top_individuals.length} donors · ${merged.lines} lines${
            merged.demo ? " (DEMO_KEY)" : ""
          }`
        );
      });
      continue;
    }

    if (stage.id === "fec_occupation" && isFec) {
      await run(stage, async () => {
        let body = enrich._sched_a_indiv_body;
        if (!body) {
          const cmteId = enrich.principal_committee?.committee_id;
          const params = {
            two_year_transaction_period: cycle,
            sort: "-contribution_receipt_amount",
            is_individual: true,
            hide_null: true,
          };
          if (cmteId) params.committee_id = cmteId;
          else params.candidate_id = fecId;
          const keyBase = `fec:sched_a_ind:${cmteId || fecId}:${cycle}`;
          const merged = await fetchFecSchedAMerged(params, keyBase);
          body = merged.body;
          enrich._sched_a_indiv_body = body;
        }
        const facts =
          fec_occupation_facts_from_sched_a_js(body, c.name || "") || [];
        if (!facts.length) {
          skip(
            stage.id,
            stage.label,
            "no self-itemized occupation (Form 2 has none)"
          );
          return;
        }
        const bio = { facts, spans: [] };
        markBioChecked("OpenFEC");
        enrich.dossier = apply_member_bio_to_dossier_js(
          JSON.stringify(enrich.dossier || empty_dossier_js(asOfYear)),
          JSON.stringify(bio),
          asOfYear
        );
        done(stage.id, stage.label, `${facts.length} facts`);
      });
      continue;
    }

    if (stage.id === "cmte" && isFec) {
      await run(stage, async () => {
        const cmteId = enrich.principal_committee?.committee_id;
        const params = {
          two_year_transaction_period: cycle,
          sort: "-contribution_receipt_amount",
          is_individual: false,
          hide_null: true,
        };
        if (cmteId) params.committee_id = cmteId;
        else params.candidate_id = fecId;
        const keyBase = `fec:sched_a_cmte:${cmteId || fecId}:${cycle}`;
        const merged = await fetchFecSchedAMerged(params, keyBase);
        enrich.top_committees =
          parse_fec_sched_a_js(merged.body, merged.contribCap) || [];
        try {
          const lines = parse_fec_sched_a_lines_js(merged.body, 400) || [];
          const prev = Array.isArray(enrich.timeline_receipts)
            ? enrich.timeline_receipts.filter((r) => r.stream !== "committee")
            : [];
          enrich.timeline_receipts = [
            ...prev,
            ...lines.map((r) => ({ ...r, stream: "committee" })),
          ];
        } catch {
          /* tops still usable */
        }
        enrich.contributors_fetch_cap = Math.max(
          enrich.contributors_fetch_cap || 0,
          merged.contribCap
        );
        enrich.contributors_lines_loaded =
          (enrich.contributors_lines_loaded || 0) + merged.lines;
        if (!enrich.contributors_note) {
          enrich.contributors_note = merged.demo
            ? "OpenFEC DEMO_KEY: one Sched A page · ~40 requests/hour. Personal key → deeper pages."
            : `Committee gifts aggregated from ${merged.lines} itemized lines → top ${merged.contribCap}.`;
        }
        done(
          stage.id,
          stage.label,
          `${enrich.top_committees.length} committees · ${merged.lines} lines${
            merged.demo ? " (DEMO_KEY)" : ""
          }`
        );
      });
      continue;
    }

    if (stage.id === "size" && isFec) {
      await run(stage, async () => {
        const url = fecUrl(`/v1/schedules/schedule_a/by_size/by_candidate/`, {
          candidate_id: fecId,
          cycle,
          per_page: 20,
          sort: "size",
        });
        const { body } = await cachedFetch(url, {
          key: `fec:size:${fecId}:${cycle}`,
        });
        enrich.size_buckets = parse_fec_size_js(body) || [];
        done(stage.id, stage.label, `${enrich.size_buckets.length} buckets`);
      });
      continue;
    }

    if (stage.id === "principal" && isFec) {
      await run(stage, async () => {
        const url = fecUrl(`/v1/candidate/${fecId}/committees/`, {
          cycle,
          per_page: 20,
        });
        const { body } = await cachedFetch(url, {
          key: `fec:cmte:${fecId}:${cycle}`,
        });
        enrich.principal_committee = parse_fec_principal_js(body);
        // B6: soft CF context — principal committee ≠ voter affiliation
        if (enrich.principal_committee) {
          const pc = enrich.principal_committee;
          appendCommitteeAffiliation(enrich, {
            name: pc.name || pc.committee_id || "",
            designation: pc.designation || "Principal campaign committee",
            source: "OpenFEC",
            source_url: pc.url || null,
          });
        }
        done(
          stage.id,
          stage.label,
          enrich.principal_committee ? "found" : "none"
        );
      });
      continue;
    }

    if (stage.id === "member" && isFec) {
      await run(stage, async () => {
        const ttl = 7 * 24 * 60 * 60 * 1000;
        const { body } = await cachedFetch(LEGISLATORS_URL, {
          key: "congress:legislators-current",
          ttlMs: ttl,
        });
        let m = match_legislator_by_fec_js(body, fecId);
        let fromHistorical = false;
        let clBody = body;
        if (!m) {
          // Former members (same FEC id) live in historical — not challengers.
          try {
            const hist = await cachedFetch(LEGISLATORS_HISTORICAL_URL, {
              key: "congress:legislators-historical",
              ttlMs: ttl,
            });
            m = match_legislator_by_fec_js(hist.body, fecId);
            fromHistorical = !!m;
            if (m) clBody = hist.body;
          } catch {
            /* keep ballot-only affiliations */
          }
        }
        if (m) {
          enrich._member = m;
          // B4: never drop ballot filing rows; merge CL timeline (cited).
          if (m.affiliations && m.affiliations.length) {
            try {
              enrich.affiliations = merge_affiliation_spans_js(
                JSON.stringify(m.affiliations),
                JSON.stringify(enrich.affiliations || [])
              );
            } catch {
              enrich.affiliations = [
                ...m.affiliations,
                ...(enrich.affiliations || []),
              ];
            }
            enrich.affiliations_source = fromHistorical
              ? "congress-legislators (historical)"
              : "congress-legislators";
          }
          // Keep ballot spans when CL has ids but empty terms (edge case).
          enrich.votes_url = m.profile_url;
          if (m.govtrack_id) enrich.govtrack_id = m.govtrack_id;
          if (m.bioguide) {
            enrich.bioguide = m.bioguide;
            enrich.bioguide_url = `https://bioguide.congress.gov/search/bio/${m.bioguide}`;
            enrich.congress_gov_url = `https://www.congress.gov/member/${m.bioguide}`;
          }
          if (m.wikidata) {
            enrich.wikidata = m.wikidata;
            try {
              enrich.wikidata_url = wikidata_entity_url_js(m.wikidata) || null;
            } catch {
              enrich.wikidata_url = `https://www.wikidata.org/wiki/${m.wikidata}`;
            }
          }
          if (m.wikipedia) enrich.wikipedia = m.wikipedia;
          if (m.ballotpedia) {
            enrich.ballotpedia = m.ballotpedia;
            try {
              enrich.ballotpedia_url =
                ballotpedia_page_url_js(m.ballotpedia) || null;
            } catch {
              enrich.ballotpedia_url = `https://ballotpedia.org/${String(
                m.ballotpedia
              ).replace(/ /g, "_")}`;
            }
          }
          if (m.official_url) {
            enrich.official_url = m.official_url;
          }
          // F1: career politician assessment from CL terms + birthday.
          try {
            const career = assess_career_from_cl_js(clBody, fecId, asOfYear);
            if (career) setDossierCareer(career, null, null, null);
          } catch (e) {
            console.warn("cl career", e);
          }
          // F2: Bioguide → unitedstates/images headshot (no extra fetch).
          if (m.bioguide && enrich.dossier && !enrich.dossier.photo_url) {
            try {
              const photo = unitedstates_congress_photo_url_js(m.bioguide);
              if (photo) {
                enrich.dossier = apply_photo_to_dossier_js(
                  JSON.stringify(enrich.dossier),
                  photo,
                  "unitedstates/images (Bioguide)",
                  enrich.bioguide_url ||
                    `https://bioguide.congress.gov/search/bio/${m.bioguide}`
                );
              }
            } catch (e) {
              console.warn("bioguide photo", e);
            }
          }
          done(
            stage.id,
            stage.label,
            fromHistorical ? `${m.name} (former)` : m.name
          );
        } else {
          // Challenger / non-member: keep ballot_affiliations only — no invented spans.
          skip(stage.id, stage.label, "no match (challenger)");
        }
      });
      continue;
    }

    if (stage.id === "official_about") {
      await run(stage, async () => {
        const site = (enrich.official_url || "").trim();
        if (!site) {
          skip(stage.id, stage.label, "no official member site (CL term url)");
          return;
        }
        if (!hasWispConfigured() && !getCorsProxy()) {
          skip(stage.id, stage.label, "needs Wisp (house.gov/senate.gov CORS)");
          return;
        }
        let urls = [];
        try {
          urls = official_about_urls_js(site) || [];
        } catch {
          urls = [];
        }
        if (!urls.length) {
          skip(stage.id, stage.label, "no about URL candidates");
          return;
        }
        let html = null;
        let pageUrl = null;
        for (const u of urls) {
          try {
            const body = await tryFetchText(
              u,
              `official:about:${u}`,
              7 * 24 * 60 * 60 * 1000
            );
            if (body && body.length > 400) {
              // Soft-skip obvious 404 shells still returning 200.
              const low = body.toLowerCase();
              if (
                low.includes("page not found") ||
                low.includes("404 not found") ||
                (low.includes("not found") && body.length < 8000)
              ) {
                continue;
              }
              html = body;
              pageUrl = u;
              break;
            }
          } catch (e) {
            console.warn("official_about fetch", u, e);
          }
        }
        if (!html || !pageUrl) {
          skip(stage.id, stage.label, "about page not loaded");
          return;
        }
        enrich.official_about_url = pageUrl;
        try {
          const bio = parse_official_member_about_html_js(html, pageUrl);
          if (!bio || (!(bio.facts && bio.facts.length) && !bio.photo_url)) {
            skip(stage.id, stage.label, "parse empty");
            return;
          }
          // Official is highest-priority prose source — full merge (cited).
          markBioChecked("official site");
          enrich.dossier = apply_member_bio_to_dossier_js(
            JSON.stringify(enrich.dossier || empty_dossier_js(asOfYear)),
            JSON.stringify(bio),
            asOfYear
          );
          const bits = [];
          if (bio.photo_url) bits.push("photo");
          if (bio.facts?.length) bits.push(`${bio.facts.length} facts`);
          if (bio.birth_year) bits.push(`b. ${bio.birth_year}`);
          done(
            stage.id,
            stage.label,
            bits.length ? bits.join(", ") : "parsed"
          );
        } catch (e) {
          console.warn("official_about", e);
          skip(stage.id, stage.label, e.message || "parse failed");
        }
      });
      continue;
    }

    if (stage.id === "dbpedia") {
      await run(stage, async () => {
        const title = (enrich.wikipedia || "").trim();
        if (!title) {
          skip(stage.id, stage.label, "no Wikipedia title");
          return;
        }
        const tryUrls = [];
        try {
          const u = dbpedia_ntriples_url_js(title);
          if (u) tryUrls.push(u);
        } catch {
          /* ignore */
        }
        try {
          const u = dbpedia_describe_ntriples_url_js(title);
          if (u) tryUrls.push(u);
        } catch {
          /* ignore */
        }
        if (!tryUrls.length) {
          skip(stage.id, stage.label, "bad title");
          return;
        }
        let body = null;
        let used = null;
        for (const apiUrl of tryUrls) {
          try {
            const res = await cachedFetch(apiUrl, {
              key: `dbpedia:nt:${title}:${apiUrl.includes("sparql") ? "sparql" : "data"}`,
              ttlMs: 7 * 24 * 60 * 60 * 1000,
            });
            const text = res.body || "";
            const low = text.slice(0, 80).toLowerCase();
            if (
              text.length > 80 &&
              !low.includes("<html") &&
              !low.includes("service unavailable")
            ) {
              body = text;
              used = apiUrl;
              break;
            }
          } catch (e) {
            console.warn("dbpedia fetch", apiUrl, e);
          }
        }
        if (!body) {
          skip(stage.id, stage.label, "DBpedia unavailable");
          return;
        }
        try {
          const bio = parse_dbpedia_ntriples_js(body, title);
          if (!bio || (!(bio.facts && bio.facts.length) && !bio.birth_year)) {
            skip(stage.id, stage.label, "no DBpedia facts");
            return;
          }
          markBioChecked("DBpedia");
          enrich.dossier = apply_member_bio_fill_gaps_js(
            JSON.stringify(enrich.dossier || empty_dossier_js(asOfYear)),
            JSON.stringify(bio),
            asOfYear
          );
          if (used) enrich.dbpedia_url = used.includes("sparql")
            ? `https://dbpedia.org/page/${String(title).replace(/ /g, "_")}`
            : used.replace(/\.ntriples$/, "").replace("/data/", "/page/");
          const bits = [];
          if (bio.facts?.length) bits.push(`${bio.facts.length} facts`);
          if (bio.birth_year) bits.push(`b. ${bio.birth_year}`);
          done(stage.id, stage.label, bits.length ? bits.join(", ") : "parsed");
        } catch (e) {
          console.warn("dbpedia", e);
          skip(stage.id, stage.label, e.message || "parse failed");
        }
      });
      continue;
    }

    if (stage.id === "grokipedia") {
      await run(stage, async () => {
        if (!hasWispConfigured() && !getCorsProxy()) {
          skip(stage.id, stage.label, "needs Wisp (grokipedia.com no CORS)");
          return;
        }
        const queries = [];
        const nm = (c.name || "").trim();
        if (nm) queries.push(nm);
        const wiki = (enrich.wikipedia || "").trim().replace(/_/g, " ");
        if (wiki && wiki.toLowerCase() !== nm.toLowerCase()) queries.push(wiki);
        if (!queries.length) {
          skip(stage.id, stage.label, "no candidate name");
          return;
        }
        let hit = null;
        for (const q of queries) {
          let apiUrl = null;
          try {
            apiUrl = grokipedia_typeahead_url_js(q);
          } catch {
            apiUrl = null;
          }
          if (!apiUrl) continue;
          try {
            const body = await tryFetchText(
              apiUrl,
              `grokipedia:ta:${q}`,
              7 * 24 * 60 * 60 * 1000
            );
            if (!body || body.length < 10) continue;
            const m = match_grokipedia_typeahead_js(body, nm || q);
            if (m && m.slug && m.page_url) {
              hit = m;
              break;
            }
          } catch (e) {
            console.warn("grokipedia typeahead", q, e);
          }
        }
        if (!hit) {
          skip(stage.id, stage.label, "no unique Grokipedia match");
          return;
        }
        let html = null;
        try {
          html = await tryFetchText(
            hit.page_url,
            `grokipedia:page:${hit.slug}`,
            7 * 24 * 60 * 60 * 1000
          );
        } catch (e) {
          console.warn("grokipedia page", hit.page_url, e);
        }
        if (!html || html.length < 200) {
          skip(stage.id, stage.label, "page not loaded");
          return;
        }
        const low = html.toLowerCase();
        if (low.includes("page not found") && html.length < 20000) {
          skip(stage.id, stage.label, "page not found");
          return;
        }
        try {
          const bio = parse_grokipedia_page_html_js(html, hit.page_url);
          if (!bio || (!(bio.facts && bio.facts.length) && !bio.birth_year)) {
            skip(stage.id, stage.label, "parse empty");
            return;
          }
          // Gap-only; family/citizenship never emitted by core parser.
          markBioChecked("Grokipedia");
          enrich.dossier = apply_member_bio_fill_gaps_js(
            JSON.stringify(enrich.dossier || empty_dossier_js(asOfYear)),
            JSON.stringify(bio),
            asOfYear
          );
          enrich.grokipedia_url = hit.page_url;
          enrich.grokipedia_slug = hit.slug;
          const bits = [];
          if (bio.facts?.length) bits.push(`${bio.facts.length} facts`);
          if (bio.birth_year) bits.push(`b. ${bio.birth_year}`);
          done(stage.id, stage.label, bits.length ? bits.join(", ") : "parsed");
        } catch (e) {
          console.warn("grokipedia", e);
          skip(stage.id, stage.label, e.message || "parse failed");
        }
      });
      continue;
    }

    if (stage.id === "wiki_extract") {
      await run(stage, async () => {
        let title = (enrich.wikipedia || "").trim();
        // J3: no CL/OS title — try name (+ office) title candidates via summary match.
        if (!title) {
          const st =
            (c.state_code || "").toUpperCase() ||
            (typeof state_code_from_jurisdiction_js === "function"
              ? state_code_from_jurisdiction_js(
                  c.jurisdiction || c.jurisdiction_ocd || "",
                  c.office || ""
                ) || ""
              : "") ||
            "";
          let guesses = [];
          try {
            guesses =
              ballotpedia_title_candidates_js(
                c.name || "",
                st,
                c.chamber || "",
                c.office || ""
              ) || [];
          } catch {
            guesses = [];
          }
          for (const g of (Array.isArray(guesses) ? guesses : []).slice(0, 6)) {
            let sumUrl = null;
            try {
              sumUrl = wikipedia_summary_api_url_js(g);
            } catch {
              sumUrl = null;
            }
            if (!sumUrl) continue;
            try {
              const { body: sumBody } = await cachedFetch(sumUrl, {
                key: `wikipedia:summary:${g}`,
                ttlMs: 7 * 24 * 60 * 60 * 1000,
              });
              if (!sumBody || sumBody.length < 20) continue;
              let hit = null;
              try {
                hit = wikipedia_summary_match_person_js(
                  sumBody,
                  c.name || "",
                  st,
                  c.office || ""
                );
              } catch {
                hit = null;
              }
              if (hit && String(hit).trim()) {
                title = String(hit).trim();
                enrich.wikipedia = title;
                break;
              }
            } catch (e) {
              console.warn("wiki title guess", g, e);
            }
          }
        }
        if (!title) {
          skip(stage.id, stage.label, "no Wikipedia title");
          return;
        }
        let apiUrl = null;
        try {
          apiUrl = wikipedia_extract_api_url_js(title);
        } catch {
          apiUrl = null;
        }
        if (!apiUrl) {
          skip(stage.id, stage.label, "bad title");
          return;
        }
        try {
          const { body } = await cachedFetch(apiUrl, {
            key: `wikipedia:extract:${title}`,
            ttlMs: 7 * 24 * 60 * 60 * 1000,
          });
          const bio = parse_wikipedia_extract_json_js(body);
          if (!bio || (!(bio.facts && bio.facts.length) && !bio.birth_year)) {
            skip(stage.id, stage.label, "no extract facts");
            return;
          }
          markBioChecked("Wikipedia");
          enrich.dossier = apply_member_bio_fill_gaps_js(
            JSON.stringify(enrich.dossier || empty_dossier_js(asOfYear)),
            JSON.stringify(bio),
            asOfYear
          );
          const bits = [];
          if (bio.facts?.length) bits.push(`${bio.facts.length} facts`);
          if (bio.birth_year) bits.push(`b. ${bio.birth_year}`);
          done(stage.id, stage.label, bits.length ? bits.join(", ") : "parsed");
        } catch (e) {
          console.warn("wiki_extract", e);
          skip(stage.id, stage.label, e.message || "fetch failed");
        }
      });
      continue;
    }

    if (stage.id === "ballotpedia_bio") {
      await run(stage, async () => {
        const st =
          (c.state_code || "").toUpperCase() ||
          (typeof state_code_from_jurisdiction_js === "function"
            ? state_code_from_jurisdiction_js(
                c.jurisdiction || c.jurisdiction_ocd || "",
                c.office || ""
              ) || ""
            : "") ||
          "";
        let title = (enrich.ballotpedia || "").trim();
        let pageUrl = (enrich.ballotpedia_url || "").trim();
        let html = null;
        let fromGuess = false;

        // H6: challengers — try name+state+office title candidates when no CL id.
        if (!title) {
          let guesses = [];
          try {
            guesses =
              ballotpedia_title_candidates_js(
                c.name || "",
                st,
                c.chamber || "",
                c.office || ""
              ) || [];
          } catch {
            guesses = [];
          }
          if (!Array.isArray(guesses) || !guesses.length) {
            skip(stage.id, stage.label, "no Ballotpedia id or title guess");
            return;
          }
          for (const g of guesses) {
            let u = "";
            try {
              u = ballotpedia_page_url_js(g) || "";
            } catch {
              u = "";
            }
            if (!u) continue;
            try {
              const res = await cachedFetch(u, {
                key: `ballotpedia:person:${g}`,
                ttlMs: 7 * 24 * 60 * 60 * 1000,
              });
              const body = res.body || "";
              if (body.length < 200) continue;
              let ok = false;
              try {
                ok = !!ballotpedia_html_matches_person_js(body, c.name || "", st);
              } catch {
                ok = false;
              }
              if (!ok) continue;
              title = g;
              pageUrl = u;
              html = body;
              fromGuess = true;
              break;
            } catch (e) {
              console.warn("ballotpedia guess", g, e);
            }
          }
          if (!title || !html) {
            skip(stage.id, stage.label, "no unique Ballotpedia page");
            return;
          }
        }

        if (!pageUrl) {
          try {
            pageUrl = ballotpedia_page_url_js(title) || "";
          } catch {
            pageUrl = "";
          }
        }
        if (!pageUrl) {
          skip(stage.id, stage.label, "bad title");
          return;
        }
        enrich.ballotpedia = title;
        enrich.ballotpedia_url = pageUrl;
        try {
          if (!html) {
            const res = await cachedFetch(pageUrl, {
              key: `ballotpedia:person:${title}`,
              ttlMs: 7 * 24 * 60 * 60 * 1000,
            });
            html = res.body || "";
          }
          if (!html || html.length < 200) {
            skip(stage.id, stage.label, "empty page");
            return;
          }
          // Soft-validate CL titles too when state known (wrong redirects rare).
          if (!fromGuess && st) {
            try {
              if (!ballotpedia_html_matches_person_js(html, c.name || "", st)) {
                // CL title can include middle initials ballot name lacks — retry name-only match.
                if (!ballotpedia_html_matches_person_js(html, c.name || "", "")) {
                  skip(stage.id, stage.label, "page name mismatch");
                  return;
                }
              }
            } catch {
              /* keep */
            }
          }
          const bio = parse_ballotpedia_member_html_js(html, pageUrl);
          if (!bio) {
            skip(stage.id, stage.label, "parse empty");
            return;
          }
          // Prefer full merge for BP (structured); challenger guess same source quality.
          markBioChecked("Ballotpedia");
          enrich.dossier = apply_member_bio_to_dossier_js(
            JSON.stringify(enrich.dossier || empty_dossier_js(asOfYear)),
            JSON.stringify(bio),
            asOfYear
          );
          try {
            const camp = ballotpedia_campaign_website_js(html);
            if (camp && is_campaign_site_url_js(camp)) {
              enrich.campaign_url = camp;
            }
          } catch {
            /* ignore */
          }
          try {
            addEndorsements(endorsements_from_ballotpedia_html_js(html, pageUrl));
          } catch (err) {
            console.warn("bp endorsements from bio", err);
          }
          const bits = [];
          if (fromGuess) bits.push("title guess");
          if (bio.photo_url) bits.push("photo");
          if (bio.facts?.length) bits.push(`${bio.facts.length} facts`);
          if (bio.spans?.length) bits.push(`${bio.spans.length} spans`);
          if (bio.birth_year) bits.push(`b. ${bio.birth_year}`);
          if (enrich.campaign_url) bits.push("campaign url");
          done(
            stage.id,
            stage.label,
            bits.length ? bits.join(", ") : "no fields"
          );
        } catch (e) {
          console.warn("ballotpedia_bio", e);
          skip(stage.id, stage.label, e.message || "fetch failed");
        }
      });
      continue;
    }

    if (stage.id === "campaign_about") {
      await run(stage, async () => {
        const candidates = [];
        const pushSite = (u) => {
          const s = (u || "").trim();
          if (!s) return;
          try {
            if (!is_campaign_site_url_js(s)) return;
          } catch {
            return;
          }
          if (!candidates.includes(s)) candidates.push(s);
        };
        pushSite(enrich.campaign_url);
        pushSite(c.source_url);
        pushSite(enrich.profile_url);
        if (!candidates.length) {
          skip(stage.id, stage.label, "no campaign site url");
          return;
        }
        if (!hasWispConfigured() && !getCorsProxy()) {
          // Some campaign hosts may allow CORS; still try direct via tryFetchText.
        }
        let html = null;
        let pageUrl = null;
        for (const site of candidates) {
          let urls = [];
          try {
            urls = campaign_about_urls_js(site) || [];
          } catch {
            urls = [];
          }
          if (!urls.length) urls = [site];
          for (const u of urls) {
            try {
              const body = await tryFetchText(
                u,
                `campaign:about:${u}`,
                7 * 24 * 60 * 60 * 1000
              );
              if (!body || body.length < 400) continue;
              const low = body.toLowerCase();
              if (
                low.includes("page not found") ||
                low.includes("404 not found") ||
                (low.includes("not found") && body.length < 8000)
              ) {
                continue;
              }
              html = body;
              pageUrl = u;
              break;
            } catch (e) {
              console.warn("campaign_about fetch", u, e);
            }
          }
          if (html) break;
        }
        if (!html || !pageUrl) {
          skip(stage.id, stage.label, "about page not loaded");
          return;
        }
        enrich.campaign_about_url = pageUrl;
        try {
          const bio = parse_campaign_about_html_js(html, pageUrl);
          if (!bio || (!(bio.facts && bio.facts.length) && !bio.photo_url && !bio.birth_year)) {
            skip(stage.id, stage.label, "parse empty");
            return;
          }
          // Lower trust than official/BP — gap-only.
          markBioChecked("campaign site");
          enrich.dossier = apply_member_bio_fill_gaps_js(
            JSON.stringify(enrich.dossier || empty_dossier_js(asOfYear)),
            JSON.stringify(bio),
            asOfYear
          );
          const bits = [];
          if (bio.photo_url) bits.push("photo");
          if (bio.facts?.length) bits.push(`${bio.facts.length} facts`);
          if (bio.birth_year) bits.push(`b. ${bio.birth_year}`);
          done(stage.id, stage.label, bits.length ? bits.join(", ") : "parsed");
        } catch (e) {
          console.warn("campaign_about", e);
          skip(stage.id, stage.label, e.message || "parse failed");
        }
      });
      continue;
    }

    if (stage.id === "wiki_photo") {
      await run(stage, async () => {
        if (enrich.dossier && enrich.dossier.photo_url) {
          skip(stage.id, stage.label, "already have photo");
          return;
        }
        const title = (enrich.wikipedia || "").trim();
        if (!title) {
          skip(stage.id, stage.label, "no Wikipedia title");
          return;
        }
        let apiUrl = null;
        try {
          apiUrl = wikipedia_summary_api_url_js(title);
        } catch {
          apiUrl = null;
        }
        if (!apiUrl) {
          skip(stage.id, stage.label, "bad title");
          return;
        }
        const { body } = await cachedFetch(apiUrl, {
          key: `wikipedia:summary:${title}`,
          ttlMs: 7 * 24 * 60 * 60 * 1000,
        });
        let photo = null;
        try {
          photo = parse_wikipedia_summary_photo_js(body);
        } catch (e) {
          console.warn("wiki photo parse", e);
        }
        if (!photo || !photo.photo_url) {
          skip(stage.id, stage.label, "no image on summary");
          return;
        }
        markBioChecked("Wikipedia");
        enrich.dossier = apply_photo_to_dossier_js(
          JSON.stringify(enrich.dossier || empty_dossier_js(asOfYear)),
          photo.photo_url,
          "Wikipedia",
          photo.page_url || null
        );
        done(stage.id, stage.label, "photo");
      });
      continue;
    }

    if (stage.id === "senate_efd") {
      await run(stage, async () => {
        const EFD_HOME = "https://efdsearch.senate.gov/search/home/";
        const EFD_SEARCH = "https://efdsearch.senate.gov/search/";
        const EFD_DATA = "https://efdsearch.senate.gov/search/report/data/";
        const UA =
          "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";
        const browserHeaders = {
          "User-Agent": UA,
          Accept: "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
          "Accept-Language": "en-US,en;q=0.9",
        };

        if (!hasWispConfigured() && !getCorsProxy()) {
          skip(stage.id, stage.label, "needs Wisp (or CORS proxy) for efdsearch.senate.gov");
          return;
        }

        let nameParts = { first: "", last: "" };
        try {
          nameParts = efd_split_person_name_js(c.name || "") || nameParts;
        } catch {
          /* ignore */
        }
        const first = (nameParts.first || "").trim();
        const last = (nameParts.last || "").trim();
        if (!last) {
          skip(stage.id, stage.label, "no last name");
          return;
        }

        const st =
          (c.state_code || "").toUpperCase() ||
          (typeof state_code_from_jurisdiction_js === "function"
            ? state_code_from_jurisdiction_js(
                c.jurisdiction || c.jurisdiction_ocd || "",
                c.office || ""
              ) || ""
            : "") ||
          "";

        let session = null;
        try {
          if (hasWispConfigured()) {
            try {
              await ensureCurl();
              session = await curlSession();
            } catch (e) {
              console.warn("senate_efd session", e);
              session = null;
            }
          }
          if (!session) {
            skip(stage.id, stage.label, "libcurl session unavailable");
            return;
          }

          // 1) Home → CSRF
          let homeHtml = "";
          try {
            homeHtml = await curlFetchText(EFD_HOME, {
              session,
              headers: browserHeaders,
            });
          } catch (e) {
            console.warn("senate_efd home", e);
          }
          const csrf1 = (homeHtml.match(
            /name=["']csrfmiddlewaretoken["']\s+value=["']([^"']+)["']/i
          ) ||
            homeHtml.match(
              /value=["']([^"']+)["']\s+name=["']csrfmiddlewaretoken["']/i
            ) ||
            [])[1];
          if (!csrf1) {
            skip(stage.id, stage.label, "eFD home CSRF missing");
            return;
          }

          // 2) Agree to EIGA terms (POST → 302 /search/ with empty body; cookies set)
          // sessionid must land in the jar before report view (else 302 → home).
          const agreeBody = new URLSearchParams();
          agreeBody.set("csrfmiddlewaretoken", csrf1);
          agreeBody.set("prohibition_agreement", "1");
          let searchHtml = "";
          try {
            searchHtml = await curlPostForm(EFD_HOME, agreeBody.toString(), {
              session,
              headers: {
                ...browserHeaders,
                Origin: "https://efdsearch.senate.gov",
                Referer: EFD_HOME,
              },
            });
          } catch (e) {
            console.warn("senate_efd agree", e);
          }
          // Belt-and-suspenders if redirect follow failed: GET search with session cookie
          if (!searchHtml || searchHtml.length < 500) {
            try {
              searchHtml = await curlFetchText(EFD_SEARCH, {
                session,
                headers: {
                  ...browserHeaders,
                  Referer: EFD_HOME,
                },
              });
            } catch (e) {
              console.warn("senate_efd search after agree", e);
            }
          }
          if (!searchHtml || searchHtml.length < 500) {
            skip(stage.id, stage.label, "eFD agreement failed");
            return;
          }
          if (!sessionHasCookie(session, "sessionid")) {
            skip(stage.id, stage.label, "eFD session cookie missing after agree");
            return;
          }
          const csrf2 = (searchHtml.match(
            /name=["']csrfmiddlewaretoken["']\s+value=["']([^"']+)["']/i
          ) ||
            searchHtml.match(
              /value=["']([^"']+)["']\s+name=["']csrfmiddlewaretoken["']/i
            ) ||
            [])[1] || csrf1;

          // 3) Search form (injects filters into results page DataTables config)
          const searchBody = new URLSearchParams();
          searchBody.set("csrfmiddlewaretoken", csrf2);
          searchBody.set("first_name", first);
          searchBody.set("last_name", last);
          // Senator + Candidate annuals
          searchBody.append("filer_type", "1");
          searchBody.append("filer_type", "4");
          if (st) {
            searchBody.set("senator_state", st);
            searchBody.set("candidate_state", st);
          }
          searchBody.append("report_type", "7"); // Annual
          searchBody.set("submitted_start_date", "");
          searchBody.set("submitted_end_date", "");
          let resultsHtml = "";
          try {
            resultsHtml = await curlPostForm(EFD_SEARCH, searchBody.toString(), {
              session,
              headers: {
                ...browserHeaders,
                Origin: "https://efdsearch.senate.gov",
                Referer: EFD_SEARCH,
              },
            });
          } catch (e) {
            console.warn("senate_efd search form", e);
          }
          // Pull injected DataTables filter values when present
          const inject = (key) => {
            const re = new RegExp(
              String.raw`d\.${key}\s*=\s*["']([^"']*)["']`,
              "i"
            );
            const m = (resultsHtml || "").match(re);
            return m ? m[1] : "";
          };
          // Live site injects JSON-ish lists without quoted ints: "[7]", "[1, 4]"
          const reportTypes = inject("report_types") || "[7]";
          const filerTypes = inject("filer_types") || "[1, 4]";
          const startDate = inject("submitted_start_date") || "01/01/2012 00:00:00";
          const endDate = inject("submitted_end_date") || "";
          const senState = inject("senator_state") || st;
          const canState = inject("candidate_state") || st;
          const qFirst = inject("first_name") || first;
          const qLast = inject("last_name") || last;

          // 4) DataTables JSON
          const csrfCookie = csrf2;
          const dataBody = new URLSearchParams();
          dataBody.set("draw", "1");
          dataBody.set("start", "0");
          dataBody.set("length", "25");
          dataBody.set("search[value]", "");
          dataBody.set("search[regex]", "false");
          dataBody.set("order[0][column]", "4");
          dataBody.set("order[0][dir]", "desc");
          dataBody.set("report_types", reportTypes);
          dataBody.set("filer_types", filerTypes);
          dataBody.set("submitted_start_date", startDate);
          dataBody.set("submitted_end_date", endDate);
          dataBody.set("candidate_state", canState);
          dataBody.set("senator_state", senState);
          dataBody.set("office_id", "");
          dataBody.set("first_name", qFirst);
          dataBody.set("last_name", qLast);

          let dataJson = "";
          try {
            const res = await curlRequest(
              EFD_DATA,
              {
                method: "POST",
                headers: {
                  ...browserHeaders,
                  "Content-Type": "application/x-www-form-urlencoded; charset=UTF-8",
                  "X-CSRFToken": csrfCookie,
                  "X-Requested-With": "XMLHttpRequest",
                  Origin: "https://efdsearch.senate.gov",
                  Referer: EFD_SEARCH,
                  Accept: "application/json, text/javascript, */*; q=0.01",
                },
                body: dataBody.toString(),
              },
              session
            );
            if (!res.ok) {
              skip(
                stage.id,
                stage.label,
                res.status === 503
                  ? "eFD search API unavailable (503)"
                  : `eFD search HTTP ${res.status}`
              );
              return;
            }
            dataJson = res.text || "";
          } catch (e) {
            console.warn("senate_efd data", e);
            skip(stage.id, stage.label, "eFD search request failed");
            return;
          }
          if (!dataJson || dataJson.trim().startsWith("<")) {
            skip(stage.id, stage.label, "eFD search returned non-JSON");
            return;
          }

          let hits = [];
          try {
            hits = parse_efd_search_data_json_js(dataJson) || [];
          } catch (e) {
            console.warn("senate_efd parse hits", e);
            skip(stage.id, stage.label, "eFD search parse failed");
            return;
          }
          if (!Array.isArray(hits) || !hits.length) {
            skip(stage.id, stage.label, "no eFD annual reports for name");
            return;
          }

          let pick = null;
          try {
            pick = pick_efd_annual_report_js(
              JSON.stringify(hits),
              c.name || "",
              st || null
            );
          } catch (e) {
            console.warn("senate_efd pick", e);
          }
          if (!pick || !pick.report_path) {
            skip(stage.id, stage.label, "no unique Annual report match");
            return;
          }

          let reportUrl = "";
          try {
            reportUrl = efd_abs_url_js(pick.report_path);
          } catch {
            reportUrl = pick.report_path.startsWith("http")
              ? pick.report_path
              : `https://efdsearch.senate.gov${pick.report_path}`;
          }

          const fetchReportHtml = async () => {
            // Manual redirect: unauthenticated report view is 302 → /search/home/
            const r = await curlRequest(
              reportUrl,
              {
                method: "GET",
                headers: {
                  ...browserHeaders,
                  Referer: EFD_SEARCH,
                },
                redirect: "manual",
              },
              session
            );
            if (r.status >= 300 && r.status < 400) {
              const loc = r.location || "";
              if (/\/search\/home\/?/i.test(loc)) {
                return { html: "", lost: true };
              }
              const followed = await curlRequest(
                loc,
                {
                  method: "GET",
                  headers: {
                    ...browserHeaders,
                    Referer: EFD_SEARCH,
                  },
                },
                session
              );
              return {
                html: followed.text || "",
                lost: false,
                status: followed.status,
              };
            }
            return { html: r.text || "", lost: false, status: r.status };
          };

          let reportHtml = "";
          try {
            let got = await fetchReportHtml();
            // One re-agree retry if jar raced or session dropped mid-flow
            if (
              got.lost ||
              (/efd:\s*home/i.test(got.html || "") && !/part\s*3/i.test(got.html || ""))
            ) {
              try {
                const home2 = await curlFetchText(EFD_HOME, {
                  session,
                  headers: browserHeaders,
                });
                const csrfR = (home2.match(
                  /name=["']csrfmiddlewaretoken["']\s+value=["']([^"']+)["']/i
                ) ||
                  home2.match(
                    /value=["']([^"']+)["']\s+name=["']csrfmiddlewaretoken["']/i
                  ) ||
                  [])[1];
                if (csrfR) {
                  const ab = new URLSearchParams();
                  ab.set("csrfmiddlewaretoken", csrfR);
                  ab.set("prohibition_agreement", "1");
                  await curlPostForm(EFD_HOME, ab.toString(), {
                    session,
                    headers: {
                      ...browserHeaders,
                      Origin: "https://efdsearch.senate.gov",
                      Referer: EFD_HOME,
                    },
                  });
                }
              } catch (e) {
                console.warn("senate_efd re-agree", e);
              }
              got = await fetchReportHtml();
            }
            reportHtml = got.html || "";
            if (
              got.lost ||
              (/efd:\s*home/i.test(reportHtml) && !/part\s*3/i.test(reportHtml))
            ) {
              skip(stage.id, stage.label, "session lost on report view");
              return;
            }
          } catch (e) {
            console.warn("senate_efd report", e);
            skip(stage.id, stage.label, "annual report fetch failed");
            return;
          }
          if (!reportHtml || reportHtml.length < 500) {
            skip(stage.id, stage.label, "empty annual report");
            return;
          }

          let holdings = [];
          try {
            holdings = parse_senate_efd_annual_html_js(reportHtml, reportUrl) || [];
          } catch (e) {
            console.warn("senate_efd parse annual", e);
            skip(stage.id, stage.label, "annual Part 3 parse failed");
            return;
          }
          if (!Array.isArray(holdings) || !holdings.length) {
            skip(stage.id, stage.label, "no Part 3 assets with values");
            return;
          }

          try {
            enrich.dossier = apply_holdings_to_dossier_js(
              JSON.stringify(enrich.dossier || empty_dossier_js(asOfYear)),
              JSON.stringify(holdings)
            );
          } catch (e) {
            console.warn("senate_efd apply", e);
            skip(stage.id, stage.label, "apply holdings failed");
            return;
          }
          enrich.efd_report_url = reportUrl;
          enrich.efd_report_date = pick.date_filed || null;
          doneHoldings(
            stage.id,
            stage.label,
            `${holdings.length} holding${holdings.length === 1 ? "" : "s"} · ${pick.date_filed || "annual"}`
          );
        } finally {
          curlCloseSession(session);
        }
      });
      continue;
    }

    if (stage.id === "house_clerk_fd") {
      await run(stage, async () => {
        const HC_SEARCH = "https://disclosures-clerk.house.gov/FinancialDisclosure/ViewSearch";
        const HC_MEMBER =
          "https://disclosures-clerk.house.gov/FinancialDisclosure/ViewMemberSearchResult";
        const HC_CAND =
          "https://disclosures-clerk.house.gov/FinancialDisclosure/ViewCandidateSearchResult";
        const UA =
          "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";
        const browserHeaders = {
          "User-Agent": UA,
          Accept: "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
          "Accept-Language": "en-US,en;q=0.9",
        };

        if (!hasWispConfigured() && !getCorsProxy()) {
          skip(stage.id, stage.label, "needs Wisp (or CORS proxy) for disclosures-clerk.house.gov");
          return;
        }

        let nameParts = { first: "", last: "" };
        try {
          nameParts = efd_split_person_name_js(c.name || "") || nameParts;
        } catch {
          /* ignore */
        }
        const last = (nameParts.last || "").trim();
        if (!last) {
          skip(stage.id, stage.label, "no last name");
          return;
        }

        const st =
          (c.state_code || "").toUpperCase() ||
          (typeof state_code_from_jurisdiction_js === "function"
            ? state_code_from_jurisdiction_js(
                c.jurisdiction || c.jurisdiction_ocd || "",
                c.office || ""
              ) || ""
            : "") ||
          "";
        let district = null;
        const distRaw = c.district != null ? String(c.district) : "";
        const dm = distRaw.match(/(\d{1,2})/);
        if (dm) district = parseInt(dm[1], 10);
        else {
          const om = String(c.office || "").match(/\b(\d{1,2})\b/);
          if (om) district = parseInt(om[1], 10);
        }

        let session = null;
        try {
          if (hasWispConfigured()) {
            try {
              await ensureCurl();
              session = await curlSession();
            } catch (e) {
              console.warn("house_clerk_fd session", e);
              session = null;
            }
          }
          if (!session) {
            skip(stage.id, stage.label, "libcurl session unavailable");
            return;
          }

          let searchHtml = "";
          try {
            searchHtml = await curlFetchText(HC_SEARCH, {
              session,
              headers: {
                ...browserHeaders,
                "X-Requested-With": "XMLHttpRequest",
                Referer: "https://disclosures-clerk.house.gov/FinancialDisclosure",
              },
            });
          } catch (e) {
            console.warn("house_clerk_fd ViewSearch", e);
          }
          const token = (searchHtml.match(
            /name=["']__RequestVerificationToken["']\s+value=["']([^"']+)["']/i
          ) ||
            searchHtml.match(
              /value=["']([^"']+)["']\s+name=["']__RequestVerificationToken["']/i
            ) ||
            [])[1];
          if (!token) {
            skip(stage.id, stage.label, "House Clerk antiforgery token missing");
            return;
          }

          const postSearch = async (url, extra) => {
            const body = new URLSearchParams();
            body.set("__RequestVerificationToken", token);
            body.set("LastName", last);
            if (st) body.set("State", st);
            else body.set("State", "");
            for (const [k, v] of Object.entries(extra || {})) {
              body.set(k, v == null ? "" : String(v));
            }
            return curlPostForm(url, body.toString(), {
              session,
              headers: {
                ...browserHeaders,
                Origin: "https://disclosures-clerk.house.gov",
                Referer: "https://disclosures-clerk.house.gov/FinancialDisclosure",
                "X-Requested-With": "XMLHttpRequest",
              },
            });
          };

          // Empty FilingYear returns all years for the last name (probed live).
          let resultsHtml = "";
          try {
            resultsHtml = await postSearch(HC_MEMBER, {
              FilingYear: "",
              District: district != null ? String(district) : "",
            });
          } catch (e) {
            console.warn("house_clerk_fd member search", e);
          }

          let hits = [];
          try {
            hits = parse_house_clerk_search_html_js(resultsHtml || "") || [];
          } catch (e) {
            console.warn("house_clerk_fd parse member", e);
          }

          // Candidate filings when member search empty (challengers).
          if (!hits.length) {
            try {
              const cycle = getCycle() || new Date().getFullYear();
              const candHtml = await postSearch(HC_CAND, {
                ElectionYear: String(cycle),
                District: district != null ? String(district) : "",
              });
              hits = parse_house_clerk_search_html_js(candHtml || "") || [];
            } catch (e) {
              console.warn("house_clerk_fd candidate search", e);
            }
          }

          if (!Array.isArray(hits) || !hits.length) {
            skip(stage.id, stage.label, "no House Clerk filings for name");
            return;
          }

          let pick = null;
          try {
            pick = pick_house_clerk_fd_report_js(
              JSON.stringify(hits),
              c.name || "",
              st || null,
              district != null ? district : undefined
            );
          } catch (e) {
            console.warn("house_clerk_fd pick", e);
          }
          if (!pick || !pick.pdf_path) {
            skip(stage.id, stage.label, "no unique FD Original match");
            return;
          }

          let pdfUrl = "";
          try {
            pdfUrl = house_clerk_abs_url_js(pick.pdf_path);
          } catch {
            pdfUrl = `https://disclosures-clerk.house.gov/${pick.pdf_path.replace(/^\//, "")}`;
          }

          let pdfBytes = null;
          try {
            pdfBytes = await curlFetchBytes(pdfUrl, {
              session,
              headers: {
                ...browserHeaders,
                Accept: "application/pdf,*/*",
                Referer: "https://disclosures-clerk.house.gov/FinancialDisclosure",
              },
            });
          } catch (e) {
            console.warn("house_clerk_fd pdf", e);
            skip(stage.id, stage.label, "House Clerk PDF fetch failed");
            return;
          }
          if (!pdfBytes || !pdfBytes.length) {
            skip(stage.id, stage.label, "House Clerk PDF empty");
            return;
          }

          let holdings = [];
          try {
            holdings = parse_house_clerk_fd_pdf_js(pdfBytes, pdfUrl) || [];
          } catch (e) {
            console.warn("house_clerk_fd parse pdf", e);
            skip(stage.id, stage.label, "House Clerk PDF parse failed");
            return;
          }
          if (!Array.isArray(holdings) || !holdings.length) {
            // Image-only PDFs are common — keep portal, skip honestly.
            skip(
              stage.id,
              stage.label,
              "no extractable Schedule A text (scanned PDF or empty assets)"
            );
            return;
          }

          try {
            enrich.dossier = apply_holdings_to_dossier_js(
              JSON.stringify(enrich.dossier || empty_dossier_js(asOfYear)),
              JSON.stringify(holdings)
            );
          } catch (e) {
            console.warn("house_clerk_fd apply", e);
            skip(stage.id, stage.label, "apply holdings failed");
            return;
          }
          enrich.house_clerk_pdf_url = pdfUrl;
          enrich.house_clerk_filing_year = pick.filing_year || null;
          doneHoldings(
            stage.id,
            stage.label,
            `${holdings.length} holding${holdings.length === 1 ? "" : "s"} · ${pick.filing_year || "FD"}`
          );
        } finally {
          curlCloseSession(session);
        }
      });
      continue;
    }

    if (stage.id === "wikidata_bio") {
      await run(stage, async () => {
        const qid = (enrich.wikidata || "").trim();
        if (!qid) {
          skip(stage.id, stage.label, "no Wikidata id (challenger or unmatched)");
          return;
        }
        const api =
          `https://www.wikidata.org/w/api.php?action=wbgetentities&ids=${encodeURIComponent(qid)}` +
          `&props=claims|labels&languages=en&format=json&origin=*`;
        const { body } = await cachedFetch(api, {
          key: `wikidata:entity:${qid}`,
          ttlMs: 7 * 24 * 60 * 60 * 1000,
        });
        let entityObj = null;
        try {
          const parsed = JSON.parse(body);
          entityObj = parsed?.entities?.[qid] || null;
        } catch {
          entityObj = null;
        }
        if (!entityObj) {
          skip(stage.id, stage.label, "entity missing");
          return;
        }
        const entityJson = JSON.stringify(entityObj);
        let needIds = [];
        try {
          needIds = wikidata_label_ids_needed_js(entityJson) || [];
        } catch (e) {
          console.warn("wikidata ids", e);
        }
        const labels = {};
        // Prefer labels already on the entity response.
        if (entityObj.labels?.en?.value) {
          /* person label unused for facts */
        }
        if (needIds.length) {
          const chunk = 40;
          for (let i = 0; i < needIds.length; i += chunk) {
            const part = needIds.slice(i, i + chunk);
            const labUrl =
              `https://www.wikidata.org/w/api.php?action=wbgetentities&ids=${part.map(encodeURIComponent).join("|")}` +
              `&props=labels&languages=en&format=json&origin=*`;
            try {
              const labRes = await cachedFetch(labUrl, {
                key: `wikidata:labels:${part.join(",")}`,
                ttlMs: 7 * 24 * 60 * 60 * 1000,
              });
              const labParsed = JSON.parse(labRes.body);
              for (const [id, ent] of Object.entries(labParsed.entities || {})) {
                const lab = ent?.labels?.en?.value;
                if (lab) labels[id] = lab;
              }
            } catch (e) {
              console.warn("wikidata labels", e);
            }
          }
        }
        const bio = parse_wikidata_entity_bio_js(entityJson, JSON.stringify(labels));
        if (!bio) {
          skip(stage.id, stage.label, "parse empty");
          return;
        }
        markBioChecked("Wikidata");
        enrich.dossier = apply_member_bio_to_dossier_js(
          JSON.stringify(enrich.dossier || empty_dossier_js(asOfYear)),
          JSON.stringify(bio),
          asOfYear
        );
        const bits = [];
        if (bio.facts?.length) bits.push(`${bio.facts.length} facts`);
        if (bio.spans?.length) bits.push(`${bio.spans.length} spans`);
        if (bio.birth_year) bits.push(`b. ${bio.birth_year}`);
        if (bio.citizenship?.disclosed) bits.push("citizenship");
        done(stage.id, stage.label, bits.length ? bits.join(", ") : "no claims");
      });
      continue;
    }

    if (stage.id === "votes" && isFec) {
      await run(stage, async () => {
        const m = enrich._member;
        if (!m || !m.govtrack_id) {
          skip(stage.id, stage.label, "no member match");
          return;
        }
        // GovTrack list embeds question/result/link — page with offset.
        // Higher cap for correlation timeline (was 100; members often have 1k–6k+).
        const GT_PAGE = 100;
        const GT_CAP = 500;
        enrich.votes_fetch_cap = GT_CAP;
        const objects = [];
        let totalAvailable = null;
        let offset = 0;
        while (objects.length < GT_CAP) {
          const pageLimit = Math.min(GT_PAGE, GT_CAP - objects.length);
          const listUrl = `https://www.govtrack.us/api/v2/vote_voter?person=${m.govtrack_id}&limit=${pageLimit}&offset=${offset}&order_by=-created`;
          const { body: listBody } = await cachedFetch(listUrl, {
            key: `gt:vote_voter:${m.govtrack_id}:p${pageLimit}:o${offset}:v2`,
            ttlMs: 6 * 60 * 60 * 1000,
          });
          const tc = vote_voter_total_count_js(listBody);
          if (tc != null) totalAvailable = Number(tc);
          let pageObjs = [];
          try {
            const parsed = JSON.parse(listBody);
            pageObjs = Array.isArray(parsed?.objects) ? parsed.objects : [];
          } catch {
            pageObjs = [];
          }
          if (!pageObjs.length) break;
          objects.push(...pageObjs);
          offset += pageObjs.length;
          if (pageObjs.length < pageLimit) break;
          if (totalAvailable != null && objects.length >= totalAvailable) break;
        }
        const listBody = JSON.stringify({ objects });
        const needIds = vote_ids_needing_detail_js(listBody) || [];
        const details = {};
        // Rare path: id-only rows (legacy shape). Cap detail fetches.
        const detailCap = Math.min(needIds.length, 24);
        for (let i = 0; i < detailCap; i++) {
          const vid = needIds[i];
          if (!vid) continue;
          try {
            const dUrl = `https://www.govtrack.us/api/v2/vote/${vid}`;
            const { body } = await cachedFetch(dUrl, {
              key: `gt:vote:${vid}`,
              ttlMs: 7 * 24 * 60 * 60 * 1000,
            });
            details[String(vid)] = body;
          } catch {
            /* stub fallback in wasm */
          }
        }
        enrich.votes = assemble_govtrack_votes_js(
          listBody,
          JSON.stringify(details)
        );
        if (totalAvailable != null) enrich.votes_total_available = totalAvailable;
        enrich.votes_source = "GovTrack";
        enrich.votes_url = m.profile_url;
        const n = enrich.votes.length;
        const extra =
          totalAvailable != null && totalAvailable > n
            ? ` of ${totalAvailable} available (cap ${GT_CAP})`
            : "";
        done(stage.id, stage.label, `${n} votes${extra}`);
      });
      continue;
    }

    if (stage.id === "fl_chamber_bio") {
      await run(stage, async () => {
        const pageUrl = (c.source_url || "").trim();
        if (
          !/flsenate\.gov\/Senators|(?:flhouse|myfloridahouse)\.gov/i.test(
            pageUrl
          )
        ) {
          skip(stage.id, stage.label, "no chamber profile URL");
          return;
        }
        if (!hasWispConfigured() && !getCorsProxy()) {
          skip(stage.id, stage.label, "needs Wisp");
          return;
        }
        try {
          const html = await tryFetchText(
            pageUrl,
            `fl:chamber:member:${pageUrl}`,
            24 * 60 * 60 * 1000
          );
          if (!html || html.length < 200) {
            skip(stage.id, stage.label, "empty page");
            return;
          }
          const parseFn =
            typeof parse_fl_chamber_member_html_js === "function"
              ? parse_fl_chamber_member_html_js
              : parse_fl_senate_member_html_js;
          const bio = parseFn(html, pageUrl);
          if (!bio) {
            skip(stage.id, stage.label, "parse empty");
            return;
          }
          markBioChecked("FL chamber");
          enrich.dossier = apply_member_bio_to_dossier_js(
            JSON.stringify(enrich.dossier || empty_dossier_js(asOfYear)),
            JSON.stringify(bio),
            asOfYear
          );
          const bits = [];
          if (bio.photo_url) bits.push("photo");
          if (bio.facts && bio.facts.length) bits.push(`${bio.facts.length} facts`);
          if (bio.spans && bio.spans.length) bits.push(`${bio.spans.length} spans`);
          done(
            stage.id,
            stage.label,
            bits.length ? bits.join(", ") : "no fields"
          );
        } catch (e) {
          console.warn("fl_chamber_bio", e);
          skip(stage.id, stage.label, e.message || "fetch failed");
        }
      });
      continue;
    }

    if (stage.id === "fl_courts_bio") {
      await run(stage, async () => {
        let indexUrl = null;
        try {
          indexUrl = fl_courts_index_url_js(c.office || "") || null;
        } catch {
          indexUrl = null;
        }
        if (!indexUrl) {
          skip(
            stage.id,
            stage.label,
            "no FL courts directory mapped for this office (circuit site unknown)"
          );
          return;
        }
        if (!hasWispConfigured() && !getCorsProxy()) {
          // flcourts often allows CORS-less browser fetch; still try tryFetchText.
        }
        try {
          const isNextHost = /flcourts\.gov/i.test(indexUrl);
          const indexHtml = await tryFetchText(
            indexUrl,
            `fl:courts:index:${indexUrl}`,
            24 * 60 * 60 * 1000
          );
          if (!indexHtml || indexHtml.length < 200) {
            skip(stage.id, stage.label, "courts index empty");
            return;
          }
          const normLinks = (arr) =>
            (Array.isArray(arr) ? arr : [])
              .map((l) =>
                l && typeof l === "object"
                  ? {
                      name: l.name || "",
                      url: l.url || "",
                      kind: l.kind || "",
                    }
                  : null
              )
              .filter((l) => l && l.url && l.name);
          let links = [];
          try {
            // Next.js SC/DCA indexes and circuit HTML directories may both apply;
            // merge + dedupe by URL (circuit hosts rarely have __NEXT_DATA__).
            const nextLinks = isNextHost
              ? normLinks(parse_fl_courts_next_index_js(indexHtml, indexUrl))
              : [];
            const circuitLinks = normLinks(
              parse_fl_circuit_directory_links_js(indexHtml, indexUrl)
            );
            const seen = new Set();
            for (const l of [...nextLinks, ...circuitLinks]) {
              if (seen.has(l.url)) continue;
              seen.add(l.url);
              links.push(l);
            }
          } catch (e) {
            console.warn("fl_courts index parse", e);
            links = [];
          }
          if (!links.length) {
            // Roster-only host (no per-judge pages) — keep as portal, honest skip.
            enrich.fl_courts_url = indexUrl;
            markBioChecked("FL courts");
            skip(
              stage.id,
              stage.label,
              "directory mapped but no per-judge bio links (portal only)"
            );
            return;
          }
          let hit = null;
          try {
            hit = match_fl_courts_judge_link_js(
              JSON.stringify(links),
              c.name || ""
            );
          } catch {
            hit = null;
          }
          if (!hit || !hit.url) {
            markBioChecked("FL courts");
            skip(
              stage.id,
              stage.label,
              "name not on official roster (challenger or different spelling)"
            );
            return;
          }
          const pageUrl = hit.url;
          enrich.fl_courts_url = pageUrl;
          const pageHtml = await tryFetchText(
            pageUrl,
            `fl:courts:judge:${pageUrl}`,
            24 * 60 * 60 * 1000
          );
          if (!pageHtml || pageHtml.length < 200) {
            skip(stage.id, stage.label, "judge page empty");
            return;
          }
          let bio = null;
          try {
            const preferCircuit =
              hit.kind === "wp_bio" ||
              /biography|\/judges\/judge-|Judges-Magistrates|\/gallery\//i.test(
                pageUrl
              );
            bio = preferCircuit
              ? parse_fl_circuit_wp_bio_html_js(pageHtml, pageUrl)
              : parse_fl_courts_judge_html_js(pageHtml, pageUrl);
            // If Next-style page returned empty, try circuit prose parser (and vice versa).
            const empty =
              !bio ||
              (!(bio.facts && bio.facts.length) &&
                !bio.photo_url &&
                !(bio.spans && bio.spans.length));
            if (empty) {
              bio = preferCircuit
                ? parse_fl_courts_judge_html_js(pageHtml, pageUrl)
                : parse_fl_circuit_wp_bio_html_js(pageHtml, pageUrl);
            }
          } catch (e) {
            console.warn("fl_courts parse", e);
            bio = null;
          }
          markBioChecked("FL courts");
          if (
            !bio ||
            (!(bio.facts && bio.facts.length) &&
              !bio.photo_url &&
              !(bio.spans && bio.spans.length))
          ) {
            skip(stage.id, stage.label, "official page has no parseable bio fields");
            return;
          }
          enrich.dossier = apply_member_bio_to_dossier_js(
            JSON.stringify(enrich.dossier || empty_dossier_js(asOfYear)),
            JSON.stringify(bio),
            asOfYear
          );
          const bits = [];
          if (bio.photo_url) bits.push("photo");
          if (bio.facts?.length) bits.push(`${bio.facts.length} facts`);
          if (bio.spans?.length) bits.push(`${bio.spans.length} spans`);
          done(stage.id, stage.label, bits.length ? bits.join(", ") : "parsed");
        } catch (e) {
          console.warn("fl_courts_bio", e);
          skip(stage.id, stage.label, e.message || "fetch failed");
        }
      });
      continue;
    }

    if (stage.id === "os_resolve") {
      await run(stage, async () => {
        const key = getOpenStatesApiKey();
        const chamber = c.chamber || "";
        const wantOrg = chamber === "state_senate" ? "upper" : "lower";
        const st =
          c.state_code ||
          state_code_from_jurisdiction_js(c.jurisdiction || "", c.office || "") ||
          "";
        const district = district_from_office_js(c.office || "");
        const nameQ = (c.name || "").split(/\s+/).slice(-1)[0] || c.name;
        const u = new URL("https://v3.openstates.org/people");
        u.searchParams.set("jurisdiction", st);
        u.searchParams.set("name", nameQ);
        u.searchParams.set("per_page", "20");
        u.searchParams.set("apikey", key);
        const { body } = await cachedFetch(u.toString(), {
          key: `os:people:${st}:${nameQ}:${wantOrg}`,
          init: { headers: { "X-API-KEY": key } },
        });
        let person = pick_openstates_person_js(
          body,
          c.name || "",
          wantOrg,
          district ?? undefined,
          st
        );
        // G3: detail fetch densifies extras / identifiers when list payload is thin.
        if (person && person.person_id) {
          try {
            const du = new URL(
              `https://v3.openstates.org/people/${encodeURIComponent(person.person_id)}`
            );
            du.searchParams.set("apikey", key);
            const det = await cachedFetch(du.toString(), {
              key: `os:person:${person.person_id}`,
              ttlMs: 7 * 24 * 60 * 60 * 1000,
              init: { headers: { "X-API-KEY": key } },
            });
            const richer = openstates_person_detail_js(det.body, st);
            if (richer && richer.person_id) {
              person = { ...person, ...richer };
            }
          } catch (e) {
            console.warn("os person detail", e);
          }
        }
        if (person) {
          enrich._os_person = person;
          enrich.votes_url = person.profile_url;
          if (person.wikidata) {
            enrich.wikidata = person.wikidata;
            try {
              enrich.wikidata_url = wikidata_entity_url_js(person.wikidata) || null;
            } catch {
              enrich.wikidata_url = `https://www.wikidata.org/wiki/${person.wikidata}`;
            }
          }
          if (person.wikipedia) enrich.wikipedia = person.wikipedia;
          const osAff = person.affiliations || [];
          if (osAff.length) {
            try {
              enrich.affiliations = merge_affiliation_spans_js(
                JSON.stringify(enrich.affiliations || []),
                JSON.stringify(osAff)
              );
            } catch {
              enrich.affiliations = [
                ...(enrich.affiliations || []),
                ...osAff,
              ];
            }
            if (!enrich.affiliations_source) {
              enrich.affiliations_source = "Open States";
            }
          }
          // F1/F2/G3: OS roles → career; image; extras facts (do not clobber chamber photo).
          try {
            const spans = person.career_spans || [];
            const hasPhoto = !!(enrich.dossier && enrich.dossier.photo_url);
            const osPhoto = !hasPhoto && person.image_url ? person.image_url : null;
            if (spans.length || person.birth_year) {
              // Merge OS spans with any chamber spans already on dossier.
              let career;
              if (enrich.dossier && enrich.dossier.career && enrich.dossier.career.spans) {
                const merged = merge_career_spans_js(
                  JSON.stringify(enrich.dossier.career.spans),
                  JSON.stringify(spans)
                );
                const birth =
                  person.birth_year ?? enrich.dossier.career.birth_year ?? null;
                career = assess_career_js(
                  JSON.stringify(merged),
                  birth,
                  asOfYear
                );
              } else {
                career = assess_career_js(
                  JSON.stringify(spans),
                  person.birth_year ?? null,
                  asOfYear
                );
              }
              setDossierCareer(
                career,
                osPhoto,
                osPhoto ? "Open States" : null,
                osPhoto ? person.profile_url || null : null
              );
            } else if (osPhoto && enrich.dossier) {
              enrich.dossier = apply_photo_to_dossier_js(
                JSON.stringify(enrich.dossier),
                osPhoto,
                "Open States",
                person.profile_url || null
              );
            }
            if (person.bio_facts && person.bio_facts.length) {
              const bio = {
                facts: person.bio_facts,
                birth_year: person.birth_year || null,
                spans: [],
              };
              markBioChecked("Open States");
              enrich.dossier = apply_member_bio_to_dossier_js(
                JSON.stringify(enrich.dossier || empty_dossier_js(asOfYear)),
                JSON.stringify(bio),
                asOfYear
              );
            }
          } catch (e) {
            console.warn("os career", e);
          }
          const bits = [person.name];
          if (person.bio_facts?.length) bits.push(`${person.bio_facts.length} facts`);
          if (person.wikidata) bits.push("WD");
          done(stage.id, stage.label, bits.join(", "));
        } else {
          skip(stage.id, stage.label, "no match");
        }
      });
      continue;
    }

    if (stage.id === "os_votes") {
      await run(stage, async () => {
        const person = enrich._os_person;
        if (!person) {
          skip(stage.id, stage.label, "no match");
          return;
        }
        const key = getOpenStatesApiKey();
        const st =
          c.state_code ||
          state_code_from_jurisdiction_js(c.jurisdiction || "", c.office || "") ||
          "";
        // Free tier ~10 req/min — prefer fewer fat pages over deep pagination.
        const OS_CAP = 48;
        enrich.votes_fetch_cap = OS_CAP;
        const sessions = vote_sessions_js(cycle) || [];
        let votes = [];
        for (const y of sessions) {
          if (votes.length >= OS_CAP) break;
          try {
            const u = new URL("https://v3.openstates.org/bills");
            u.searchParams.set("jurisdiction", st);
            u.searchParams.set("session", String(y));
            u.searchParams.set("sort", "updated_desc");
            u.searchParams.set("per_page", "50");
            u.searchParams.set("page", "1");
            u.searchParams.set("include", "votes");
            u.searchParams.set("apikey", key);
            const { body } = await cachedFetch(u.toString(), {
              key: `os:bills:${st}:${y}:p50`,
              init: { headers: { "X-API-KEY": key } },
            });
            const more = extract_openstates_votes_js(
              body,
              person.person_id,
              OS_CAP - votes.length
            );
            votes = votes.concat(more || []);
          } catch (e) {
            if (/429|rate/i.test(e.message || "")) {
              enrich.votes_rate_limited = true;
              break;
            }
          }
        }
        enrich.votes = votes;
        enrich.votes_source = "Open States";
        enrich.votes_url = person.profile_url;
        const extra =
          votes.length >= OS_CAP
            ? ` (cap ${OS_CAP}; OS free tier ~10 req/min)`
            : "";
        done(stage.id, stage.label, `${votes.length} votes${extra}`);
      });
      continue;
    }

    if (stage.id === "cl_person") {
      await run(stage, async () => {
        const st =
          c.state_code ||
          state_code_from_jurisdiction_js(c.jurisdiction || "", c.office || "") ||
          "";
        let prefer = [];
        try {
          prefer = courtlistener_court_ids_js(st, c.office || "") || [];
        } catch {
          prefer = [];
        }
        let searchUrl = null;
        try {
          searchUrl = courtlistener_people_search_url_js(c.name || "") || null;
        } catch {
          searchUrl = null;
        }
        if (!searchUrl) {
          enrich.courtlistener_skip = "no searchable name";
          skip(stage.id, stage.label, "no searchable name");
          return;
        }
        try {
          const portal = courtlistener_search_portal_url_js(c.name || "");
          if (portal) enrich.courtlistener_search_url = portal;
        } catch {
          /* optional */
        }
        const token = getCourtListenerToken();
        const headers = { Accept: "application/json" };
        if (token) headers.Authorization = `Token ${token}`;
        let body = null;
        try {
          const r = await cachedFetch(searchUrl, {
            key: `cl:people:${(c.name || "").toLowerCase()}`,
            ttlMs: 24 * 60 * 60 * 1000,
            init: { headers },
          });
          body = r.body;
        } catch (e) {
          // CORS / network — try Wisp text fetch
          try {
            if (hasWispConfigured()) {
              body = await tryFetchText(
                searchUrl,
                `cl:people:wisp:${(c.name || "").toLowerCase()}`,
                24 * 60 * 60 * 1000
              );
            }
          } catch (e2) {
            console.warn("cl_person", e, e2);
          }
          if (!body) {
            enrich.courtlistener_skip = e.message || "fetch failed";
            skip(stage.id, stage.label, e.message || "fetch failed");
            return;
          }
        }
        let person = null;
        try {
          person = pick_courtlistener_person_js(
            body,
            c.name || "",
            JSON.stringify(prefer || [])
          );
        } catch (e) {
          console.warn("cl pick", e);
          person = null;
        }
        if (!person || !person.person_id) {
          enrich.courtlistener_checked = true;
          enrich.courtlistener_skip = "no unique person match";
          skip(stage.id, stage.label, "no unique person match");
          return;
        }
        enrich._cl_person = person;
        enrich._cl_prefer_courts = prefer || [];
        enrich.courtlistener_id = person.person_id;
        enrich.courtlistener_url = person.profile_url;
        enrich.votes_url = person.profile_url;
        done(
          stage.id,
          stage.label,
          `${person.name || "matched"} (#${person.person_id})`
        );
      });
      continue;
    }

    if (stage.id === "cl_positions") {
      await run(stage, async () => {
        const person = enrich._cl_person;
        if (!person || !person.person_id) {
          skip(stage.id, stage.label, "no person");
          return;
        }
        let posUrl = null;
        try {
          posUrl = courtlistener_positions_url_js(person.person_id) || null;
        } catch {
          posUrl = null;
        }
        if (!posUrl) {
          skip(stage.id, stage.label, "no positions url");
          return;
        }
        const token = getCourtListenerToken();
        const headers = { Accept: "application/json" };
        if (token) headers.Authorization = `Token ${token}`;
        let body = null;
        try {
          const r = await cachedFetch(posUrl, {
            key: `cl:positions:${person.person_id}`,
            ttlMs: 24 * 60 * 60 * 1000,
            init: { headers },
          });
          body = r.body;
        } catch (e) {
          try {
            if (hasWispConfigured()) {
              body = await tryFetchText(
                posUrl,
                `cl:positions:wisp:${person.person_id}`,
                24 * 60 * 60 * 1000
              );
            }
          } catch (e2) {
            console.warn("cl_positions", e, e2);
          }
          if (!body) {
            skip(stage.id, stage.label, e.message || "fetch failed");
            return;
          }
        }
        const prefer = enrich._cl_prefer_courts || [];
        if (prefer.length) {
          try {
            const ok = person_positions_match_courts_js(
              body,
              JSON.stringify(prefer)
            );
            if (ok === false) {
              // Soft: still use positions for practice; do not drop person.
            }
          } catch {
            /* ignore */
          }
        }
        let bio = null;
        try {
          bio = courtlistener_positions_bio_js(
            body,
            person.profile_url || ""
          );
        } catch (e) {
          console.warn("cl positions bio", e);
          bio = null;
        }
        if (!bio || (!(bio.spans && bio.spans.length) && !(bio.facts && bio.facts.length))) {
          skip(stage.id, stage.label, "no parseable positions");
          return;
        }
        try {
          const spans = bio.spans || [];
          if (spans.length || bio.birth_year) {
            let career;
            if (enrich.dossier && enrich.dossier.career && enrich.dossier.career.spans) {
              const merged = merge_career_spans_js(
                JSON.stringify(enrich.dossier.career.spans),
                JSON.stringify(spans)
              );
              const birth =
                bio.birth_year ??
                person.birth_year ??
                enrich.dossier.career.birth_year ??
                null;
              career = assess_career_js(JSON.stringify(merged), birth, asOfYear);
            } else {
              career = assess_career_js(
                JSON.stringify(spans),
                bio.birth_year ?? person.birth_year ?? null,
                asOfYear
              );
            }
            setDossierCareer(career, null, null, null);
          }
          if (bio.facts && bio.facts.length) {
            markBioChecked("CourtListener");
            enrich.dossier = apply_member_bio_to_dossier_js(
              JSON.stringify(enrich.dossier || empty_dossier_js(asOfYear)),
              JSON.stringify({
                facts: bio.facts,
                spans: [],
                birth_year: bio.birth_year || person.birth_year || null,
              }),
              asOfYear
            );
          }
        } catch (e) {
          console.warn("cl positions apply", e);
        }
        const bits = [];
        if (bio.spans?.length) bits.push(`${bio.spans.length} spans`);
        if (bio.facts?.length) bits.push(`${bio.facts.length} facts`);
        done(stage.id, stage.label, bits.join(", ") || "parsed");
      });
      continue;
    }

    if (stage.id === "cl_opinions") {
      await run(stage, async () => {
        const person = enrich._cl_person;
        if (!person || !person.person_id) {
          enrich.courtlistener_checked = true;
          if (!enrich.courtlistener_skip) {
            enrich.courtlistener_skip = "no person match";
          }
          skip(stage.id, stage.label, "no person");
          return;
        }
        let cap = 100;
        try {
          cap = courtlistener_opinions_cap_js() || 100;
        } catch {
          cap = 100;
        }
        let searchUrl = null;
        try {
          // CL search page size is fixed (~20); page_size hint only.
          searchUrl =
            courtlistener_opinions_search_url_js(person.person_id, 20) || null;
        } catch {
          searchUrl = null;
        }
        if (!searchUrl) {
          enrich.courtlistener_checked = true;
          skip(stage.id, stage.label, "no opinions url");
          return;
        }
        const token = getCourtListenerToken();
        const headers = { Accept: "application/json" };
        if (token) headers.Authorization = `Token ${token}`;

        const fetchClJson = async (url, pageKey) => {
          try {
            const r = await cachedFetch(url, {
              key: pageKey,
              ttlMs: 24 * 60 * 60 * 1000,
              init: { headers },
            });
            return r.body;
          } catch (e) {
            if (hasWispConfigured()) {
              try {
                return await tryFetchText(
                  url,
                  `${pageKey}:wisp`,
                  24 * 60 * 60 * 1000
                );
              } catch (e2) {
                console.warn("cl_opinions page", e, e2);
              }
            }
            throw e;
          }
        };

        let votes = [];
        let totalAvailable = null;
        let source = "CourtListener";
        let nextUrl = searchUrl;
        let page = 0;
        const seen = new Set();
        try {
          while (nextUrl && votes.length < cap && page < 8) {
            page += 1;
            const body = await fetchClJson(
              nextUrl,
              `cl:opinions:${person.person_id}:p${page}`
            );
            if (!body) break;
            let parsed = null;
            try {
              parsed = courtlistener_opinions_from_search_js(
                body,
                person.person_id
              );
            } catch (e) {
              console.warn("cl opinions parse", e);
              break;
            }
            if (parsed && parsed.source) source = parsed.source;
            if (parsed && parsed.total_available != null && totalAvailable == null) {
              totalAvailable = parsed.total_available;
            }
            for (const v of parsed?.votes || []) {
              const k = `${v.date}|${v.url}|${v.position}`;
              if (seen.has(k)) continue;
              seen.add(k);
              votes.push(v);
              if (votes.length >= cap) break;
            }
            let nxt = null;
            try {
              const root = JSON.parse(body);
              nxt = root && root.next ? String(root.next) : null;
            } catch {
              nxt = null;
            }
            nextUrl = nxt;
          }
        } catch (e) {
          enrich.courtlistener_checked = true;
          enrich.courtlistener_skip = e.message || "fetch failed";
          skip(stage.id, stage.label, e.message || "fetch failed");
          return;
        }
        enrich.courtlistener_checked = true;
        enrich.votes = votes;
        enrich.votes_source = source;
        enrich.votes_url = person.profile_url;
        enrich.votes_fetch_cap = cap;
        if (totalAvailable != null) {
          enrich.votes_total_available = totalAvailable;
        }
        if (!votes.length) {
          skip(stage.id, stage.label, "no authored opinions found");
          return;
        }
        const extra =
          enrich.votes_total_available != null &&
          Number(enrich.votes_total_available) > votes.length
            ? ` of ${enrich.votes_total_available}`
            : "";
        done(stage.id, stage.label, `${votes.length}${extra} opinions`);
      });
      continue;
    }

    if (stage.id === "bp_endorsements") {
      await run(stage, async () => {
        let pageUrl = (enrich.ballotpedia_url || "").trim();
        let title = (enrich.ballotpedia || "").trim();
        if (!pageUrl) {
          title = title || (c.name || "").trim();
          if (title) {
            try {
              pageUrl = ballotpedia_page_url_js(title) || "";
            } catch {
              pageUrl = "";
            }
            if (!pageUrl) {
              pageUrl = `https://ballotpedia.org/${title.replace(/\s+/g, "_")}`;
            }
            enrich.ballotpedia_url = pageUrl;
          }
        }
        if (!pageUrl) {
          skip(stage.id, stage.label, "no Ballotpedia page");
          return;
        }
        title = title || pageUrl;
        let html = "";
        try {
          const res = await cachedFetch(pageUrl, {
            key: `ballotpedia:person:${title}`,
            ttlMs: 7 * 24 * 60 * 60 * 1000,
          });
          html = res.body || "";
        } catch {
          html = "";
        }
        if (html.length < 200) {
          html =
            (await tryFetchText(
              pageUrl,
              `ballotpedia:person:${title}`,
              7 * 24 * 60 * 60 * 1000
            )) || "";
        }
        if (html.length < 200) {
          skip(stage.id, stage.label, "empty page");
          return;
        }
        const extra = asEndorsementRows(
          endorsements_from_ballotpedia_html_js(html, pageUrl)
        );
        if (!extra.length) {
          skip(stage.id, stage.label, "no endorsement list");
          return;
        }
        addEndorsements(extra);
        done(stage.id, stage.label, `${extra.length} endorsements`);
      });
      continue;
    }

    if (stage.id === "campaign_endorsements") {
      await run(stage, async () => {
        const site =
          (enrich.campaign_url || "").trim() ||
          ((c.source_url || "") &&
          typeof is_campaign_site_url_js === "function" &&
          is_campaign_site_url_js(c.source_url)
            ? c.source_url
            : "");
        if (!site) {
          skip(stage.id, stage.label, "no campaign site");
          return;
        }
        let urls = [];
        try {
          urls = campaign_endorsement_urls_js(site) || [];
        } catch {
          urls = [];
        }
        if (!urls.length) {
          skip(stage.id, stage.label, "no endorsement URLs");
          return;
        }
        let merged = 0;
        for (const u of urls.slice(0, 4)) {
          try {
            const html = await tryFetchText(
              u,
              `campaign:endorsements:${u}`,
              7 * 24 * 60 * 60 * 1000
            );
            if (!html || html.length < 200) continue;
            const extra = asEndorsementRows(
              endorsements_from_campaign_html_js(html, u)
            );
            if (!extra.length) continue;
            merged += addEndorsements(extra);
            break;
          } catch (err) {
            console.warn("campaign_endorsements", u, err);
          }
        }
        if (!merged) {
          skip(stage.id, stage.label, "no endorsements page");
          return;
        }
        done(stage.id, stage.label, `${merged} self-reported`);
      });
      continue;
    }

    if (stage.id === "gdelt_news") {
      await run(stage, async () => {
        const name = (c.name || "").trim();
        if (name.length < 5) {
          skip(stage.id, stage.label, "name too short");
          return;
        }
        const loc = scrutinyLocale(c);
        let url = "";
        try {
          url = gdelt_artlist_url_js(name, loc) || "";
        } catch {
          url = "";
        }
        if (!url) {
          skip(stage.id, stage.label, "no query");
          return;
        }
        const res = await cachedFetch(url, {
          key: `gdelt:artlist:${name}:${loc}`,
          ttlMs: 24 * 60 * 60 * 1000,
        });
        const hits = news_hits_from_gdelt_json_js(res.body || "", name) || [];
        const sc = ensureScrutiny(enrich, c);
        sc.news = merge_news_hits_js(JSON.stringify(sc.news || []), JSON.stringify(hits)) || hits;
        if (!hits.length) {
          skip(stage.id, stage.label, "no name-matched headlines");
          return;
        }
        done(stage.id, stage.label, `${hits.length} headlines`);
      });
      continue;
    }

    if (stage.id === "news_rss") {
      await run(stage, async () => {
        const name = (c.name || "").trim();
        if (name.length < 5) {
          skip(stage.id, stage.label, "name too short");
          return;
        }
        const loc = scrutinyLocale(c);
        let url = "";
        try {
          url = google_news_rss_url_js(name, loc) || "";
        } catch {
          url = "";
        }
        if (!url) {
          skip(stage.id, stage.label, "no query");
          return;
        }
        const xml = await tryFetchText(
          url,
          `news:rss:${name}:${loc}`,
          24 * 60 * 60 * 1000
        );
        const hits = news_hits_from_google_rss_js(xml || "", name) || [];
        const sc = ensureScrutiny(enrich, c);
        sc.news = merge_news_hits_js(JSON.stringify(sc.news || []), JSON.stringify(hits)) || [
          ...(sc.news || []),
          ...hits,
        ];
        if (!hits.length) {
          skip(stage.id, stage.label, "no name-matched RSS items");
          return;
        }
        done(stage.id, stage.label, `${hits.length} RSS`);
      });
      continue;
    }

    if (stage.id === "money_signals") {
      await run(stage, async () => {
        const f = enrich.finance || {};
        const loc = `${c.jurisdiction || ""}`;
        let county = "";
        const cm = loc.match(/brevard/i);
        if (cm) county = "Brevard";
        const st =
          (c.state_code || "").toUpperCase() ||
          (loc.toLowerCase().includes("florida") ? "FL" : "");
        const money = money_signals_from_json_js(
          JSON.stringify(enrich.size_buckets || []),
          JSON.stringify(enrich.top_individuals || []),
          JSON.stringify(enrich.top_committees || []),
          JSON.stringify(enrich.outside_spending || []),
          f.receipts_display || "",
          f.pac_display || "",
          f.individual_display || "",
          c.name || "",
          st,
          county
        );
        const sc = ensureScrutiny(enrich, c);
        sc.money = money;
        const n = (money && money.signals && money.signals.length) || 0;
        if (!n) {
          skip(stage.id, stage.label, (money && money.empty_note) || "no finance rows");
          return;
        }
        done(stage.id, stage.label, `${n} signals`);
      });
      continue;
    }

    if (stage.id === "bp_claims") {
      await run(stage, async () => {
        const pageUrl = (enrich.ballotpedia_url || "").trim();
        if (!pageUrl) {
          skip(stage.id, stage.label, "no Ballotpedia page");
          return;
        }
        const title = (enrich.ballotpedia || "").trim() || pageUrl;
        const res = await cachedFetch(pageUrl, {
          key: `ballotpedia:person:${title}`,
          ttlMs: 7 * 24 * 60 * 60 * 1000,
        });
        const html = res.body || "";
        if (html.length < 200) {
          skip(stage.id, stage.label, "empty page");
          return;
        }
        const extra = claims_from_ballotpedia_html_js(html, pageUrl) || [];
        const sc = ensureScrutiny(enrich, c);
        sc.claims = merge_claims_js(JSON.stringify(sc.claims || []), JSON.stringify(extra)) || extra;
        if (!extra.length) {
          skip(stage.id, stage.label, "no Campaign themes / positions section");
          return;
        }
        done(stage.id, stage.label, `${extra.length} stated`);
      });
      continue;
    }

    if (stage.id === "campaign_claims") {
      await run(stage, async () => {
        const site =
          (enrich.campaign_url || "").trim() ||
          ((c.source_url || "") &&
          typeof is_campaign_site_url_js === "function" &&
          is_campaign_site_url_js(c.source_url)
            ? c.source_url
            : "");
        if (!site) {
          skip(stage.id, stage.label, "no campaign site");
          return;
        }
        let urls = [];
        try {
          urls = campaign_claim_urls_js(site) || [];
        } catch {
          urls = [];
        }
        if (!urls.length) {
          skip(stage.id, stage.label, "no issues URLs");
          return;
        }
        let merged = 0;
        const sc = ensureScrutiny(enrich, c);
        for (const u of urls.slice(0, 5)) {
          try {
            const html = await tryFetchText(
              u,
              `campaign:issues:${u}`,
              7 * 24 * 60 * 60 * 1000
            );
            if (!html || html.length < 200) continue;
            const low = html.toLowerCase();
            if (
              low.includes("page not found") ||
              low.includes("404 not found") ||
              (low.includes("not found") && html.length < 8000)
            ) {
              continue;
            }
            const extra = claims_from_campaign_html_js(html, u) || [];
            if (!extra.length) continue;
            sc.claims =
              merge_claims_js(JSON.stringify(sc.claims || []), JSON.stringify(extra)) || extra;
            merged += extra.length;
            break;
          } catch (err) {
            console.warn("campaign_claims", u, err);
          }
        }
        if (!merged) {
          skip(stage.id, stage.label, "no issues page");
          return;
        }
        done(stage.id, stage.label, `${merged} self-stated`);
      });
      continue;
    }

    if (stage.id === "claim_contrasts") {
      await run(stage, async () => {
        const sc = ensureScrutiny(enrich, c);
        const claims = sc.claims || [];
        if (!claims.length) {
          skip(stage.id, stage.label, "no stated positions to pair");
          return;
        }
        const votes = enrich.votes || [];
        let cards = [];
        try {
          cards =
            pair_claims_with_votes_js(JSON.stringify(claims), JSON.stringify(votes)) || [];
        } catch (err) {
          console.warn("claim_contrasts", err);
          cards = [];
        }
        sc.contrasts = cards;
        const hit = cards.filter((x) => x.matches && x.matches.length).length;
        if (!cards.length) {
          skip(stage.id, stage.label, "no cards");
          return;
        }
        done(
          stage.id,
          stage.label,
          hit
            ? `${hit}/${cards.length} with keyword overlap`
            : `${cards.length} stated · no vote overlap`
        );
      });
      continue;
    }

    if (stage.id === "llm_contrasts") {
      if (skipAi) {
        skip(stage.id, stage.label, "skipped (scrape reload)");
        continue;
      }
      await run(stage, async () => {
        if (!hasLlmKey()) {
          skip(stage.id, stage.label, "no LLM key");
          return;
        }
        const sc = ensureScrutiny(enrich, c);
        const cards = sc.contrasts || [];
        const hit = cards.filter((x) => x.matches && x.matches.length).length;
        if (!hit) {
          skip(stage.id, stage.label, "no keyword overlap to contrast");
          return;
        }
        const provider = getLlmProvider();
        let url = "";
        let model = "";
        try {
          url = llm_chat_url_js(provider) || "";
          model = llm_default_model_js(provider) || "";
        } catch (err) {
          console.warn("llm_contrasts url", err);
        }
        if (!url || !model) {
          skip(stage.id, stage.label, "unknown provider");
          return;
        }
        let body = "";
        try {
          body =
            llm_contrast_request_body_js(
              model,
              JSON.stringify(cards),
              c.name || "",
              c.office || ""
            ) || "";
        } catch (err) {
          console.warn("llm_contrasts body", err);
        }
        if (!body) {
          skip(stage.id, stage.label, "no prompt");
          return;
        }
        if (!hasWispConfigured()) {
          skip(stage.id, stage.label, "Wisp required (LLM hosts block CORS)");
          return;
        }
        const key = getLlmApiKey();
        const sig = cards
          .map((x) => String(x.claim_text || "").slice(0, 32))
          .join("|")
          .slice(0, 160);
        const cacheKey = `llm-contrast:${provider}:${model}:${(c.name || "").toLowerCase()}:${sig}`;
        let resp = "";
        try {
          const hitBody = await cacheGet(cacheKey);
          if (hitBody && hitBody.length >= 20) {
            resp = hitBody;
          } else {
            resp = await curlPostJson(url, body, {
              headers: { Authorization: `Bearer ${key}` },
            });
            if (resp && resp.length >= 20) {
    await cachePut(cacheKey, resp, 0);
            }
          }
        } catch (err) {
          console.warn("llm_contrasts post", err);
          skip(stage.id, stage.label, err.message || "request failed");
          return;
        }
        if (!resp) {
          skip(stage.id, stage.label, "empty model response");
          return;
        }
        let updated = [];
        try {
          updated = apply_llm_chat_response_js(JSON.stringify(cards), resp, model) || [];
        } catch (err) {
          console.warn("llm_contrasts apply", err);
          skip(stage.id, stage.label, "could not parse model notes");
          return;
        }
        sc.contrasts = updated;
        const noted = updated.filter((x) => x.llm_note).length;
        if (!noted) {
          skip(stage.id, stage.label, "no usable notes (or verdict language dropped)");
          return;
        }
        done(stage.id, stage.label, `${noted} comparison note${noted === 1 ? "" : "s"}`);
      });
      continue;
    }

    if (stage.id === "fl_bar") {
      await run(stage, async () => {
        if (!isFloridaCandidate(c)) {
          skip(stage.id, stage.label, "not a Florida candidate");
          return;
        }
        const name = (c.name || "").trim();
        let url = "";
        try {
          url = fl_bar_search_url_js(name) || "";
        } catch {
          url = "";
        }
        if (!url) {
          skip(stage.id, stage.label, "name too short");
          return;
        }
        const html = await tryFetchText(url, `flbar:search:${name}`, 24 * 60 * 60 * 1000);
        if (!html || html.length < 80) {
          skip(stage.id, stage.label, "empty Bar page");
          return;
        }
        const hits = parse_fl_bar_search_html_js(html, name) || [];
        const sc = ensureScrutiny(enrich, c);
        sc.records = merge_record_hits_js(JSON.stringify(sc.records || []), JSON.stringify(hits)) || hits;
        if (!hits.length) {
          skip(stage.id, stage.label, "no unique Bar match");
          return;
        }
        done(stage.id, stage.label, hits[0].status || "directory hit");
      });
      continue;
    }

    if (stage.id === "fl_ethics") {
      await run(stage, async () => {
        if (!isFloridaCandidate(c)) {
          skip(stage.id, stage.label, "not a Florida candidate");
          return;
        }
        const name = (c.name || "").trim();
        let filingsUrl = "";
        let ordersUrl = "";
        try {
          filingsUrl = fl_ethics_filings_url_js(name) || "";
          ordersUrl = fl_ethics_orders_url_js() || "";
        } catch {
          filingsUrl = "";
          ordersUrl = "";
        }
        if (!filingsUrl && !ordersUrl) {
          skip(stage.id, stage.label, "name too short");
          return;
        }
        let hits = [];
        if (filingsUrl) {
          try {
            const body = await tryFetchText(
              filingsUrl,
              `flethics:filings:${name}`,
              24 * 60 * 60 * 1000
            );
            const extra = parse_fl_ethics_filings_json_js(body || "", name) || [];
            hits = merge_record_hits_js(JSON.stringify(hits), JSON.stringify(extra)) || extra;
          } catch (err) {
            console.warn("fl_ethics filings", err);
          }
        }
        if (ordersUrl) {
          try {
            const html = await tryFetchText(
              ordersUrl,
              "flethics:orders",
              7 * 24 * 60 * 60 * 1000
            );
            const extra = parse_fl_ethics_orders_html_js(html || "", name) || [];
            hits = merge_record_hits_js(JSON.stringify(hits), JSON.stringify(extra)) || extra;
          } catch (err) {
            console.warn("fl_ethics orders", err);
          }
        }
        const sc = ensureScrutiny(enrich, c);
        sc.records = merge_record_hits_js(JSON.stringify(sc.records || []), JSON.stringify(hits)) || hits;
        if (!hits.length) {
          skip(stage.id, stage.label, "no unique Ethics match");
          return;
        }
        const nFile = hits.filter((h) => h.kind === "ethics_filing").length;
        const nOrd = hits.filter((h) => h.kind === "ethics_order").length;
        done(
          stage.id,
          stage.label,
          [nFile ? `${nFile} filing${nFile === 1 ? "" : "s"}` : "", nOrd ? `${nOrd} order${nOrd === 1 ? "" : "s"}` : ""]
            .filter(Boolean)
            .join(" · ") || `${hits.length} records`
        );
      });
      continue;
    }

    if (stage.id === "fl_jqc") {
      await run(stage, async () => {
        const isJudge = !!(c.is_judge || c.chamber === "judicial");
        if (!isFloridaCandidate(c) || !isJudge) {
          skip(stage.id, stage.label, isJudge ? "not a Florida candidate" : "not a judge");
          return;
        }
        const name = (c.name || "").trim();
        let postsUrl = "";
        let newsUrl = "";
        try {
          postsUrl = fl_jqc_posts_url_js(name) || "";
          newsUrl = fl_jqc_news_url_js() || "";
        } catch {
          postsUrl = "";
          newsUrl = "";
        }
        let hits = [];
        if (postsUrl) {
          try {
            const body = await tryFetchText(
              postsUrl,
              `fljqc:posts:${name}`,
              24 * 60 * 60 * 1000
            );
            const extra = parse_fl_jqc_posts_json_js(body || "", name) || [];
            hits = merge_record_hits_js(JSON.stringify(hits), JSON.stringify(extra)) || extra;
          } catch (err) {
            console.warn("fl_jqc posts", err);
          }
        }
        if (!hits.length && newsUrl) {
          try {
            const html = await tryFetchText(newsUrl, "fljqc:news", 24 * 60 * 60 * 1000);
            const extra = parse_fl_jqc_news_html_js(html || "", name) || [];
            hits = merge_record_hits_js(JSON.stringify(hits), JSON.stringify(extra)) || extra;
          } catch (err) {
            console.warn("fl_jqc news", err);
          }
        }
        const sc = ensureScrutiny(enrich, c);
        sc.records = merge_record_hits_js(JSON.stringify(sc.records || []), JSON.stringify(hits)) || hits;
        if (!hits.length) {
          skip(stage.id, stage.label, "no name-matched JQC notice");
          return;
        }
        done(stage.id, stage.label, `${hits.length} notice${hits.length === 1 ? "" : "s"}`);
      });
      continue;
    }

    if (stage.id === "ai_verdict") {
      if (skipAi) {
        skip(stage.id, stage.label, "skipped (scrape reload)");
        continue;
      }
      await run(stage, async () => {
        const result = await runVerdictPass(c, enrich, { pass: "refine" });
        if (result && result.card) {
          enrich.verdict = result.card;
          try {
            const extra = found_endorsements_from_verdict_js(
              JSON.stringify(result.card)
            );
            if (extra) addEndorsements(extra);
          } catch (err) {
            console.warn("verdict found endorsements", err);
          }
          if (result.skip === "unchanged") {
            done(stage.id, stage.label, "cached");
            return;
          }
          done(stage.id, stage.label, result.card.headline || "verdict");
          return;
        }
        skip(stage.id, stage.label, (result && result.skip) || "no card");
      });
      continue;
    }

    // unknown stage
    skip(stage.id, stage.label, "skipped");
  }

  delete enrich._member;
  delete enrich._os_person;
  delete enrich._cl_person;
  delete enrich._cl_prefer_courts;
  if (enrich.dossier) {
    try {
      enrich.dossier = polish_dossier_empty_notes_js(
        JSON.stringify(enrich.dossier)
      );
    } catch (e) {
      console.warn("polish empty notes", e);
    }
  }
  return { stages, enrich };
}

export { planStages };
