//! Pure electionizer domain: models, secret redaction, parsers, ballot grouping.
//! No Axum, SQLx, Tokio runtime, or filesystem I/O.

pub mod bio;
pub mod civic;
pub mod courtlistener;
pub mod scrutiny;
pub mod verdict;
pub mod federal;
pub mod fec;
pub mod ftm;
pub mod govtrack;
pub mod models;
pub mod openstates;
pub mod redact;
pub mod state_ballot;
pub mod states;

pub use federal::{
    ballot_report_from_federal_live, ballot_report_from_federal_live_ex,
    ballot_report_from_live_with_state, ballot_report_from_snapshot, federal_ballot_snapshot,
    filter_fec_candidates, format_person_name, geo_from_zippo_and_census, geo_summary_from_jsons,
    map_fec_candidates, parse_cd_number, parse_census_coordinates_json, parse_district_geo_json,
    parse_fcc_area_json, parse_fec_candidates_json, parse_geo_from_jsons, parse_geo_from_jsons_ex,
    parse_tigerweb_identify_json, parse_zippo_json, CensusGeo, FecCandidateRow, GeoSummaryJs,
    ZippoPlace,
};
pub use state_ballot::{
    apply_state_extras, az_extras_from_rosters, extras_from_state_bodies, fl_extras_from_bodies,
    openstates_extras_from_people_geo, parse_state_bodies_json, StateBallotExtras, BODY_AZ_HOUSE,
    BODY_AZ_MEASURES, BODY_AZ_OFFICIALS, BODY_AZ_SENATE, BODY_FL_DOS, BODY_FL_HOUSE,
    BODY_FL_MEASURES, BODY_FL_SAMPLE_BALLOT, BODY_FL_SENATE, BODY_FL_SOE,
    BODY_CIVIC_VOTERINFO, BODY_MD_LOCAL, BODY_MD_MEASURES, BODY_MD_PHASE, BODY_MD_STATEWIDE,
    BODY_NC_CANDIDATES, BODY_NC_MEASURES, BODY_NC_MEASURES_URL, BODY_OS_PEOPLE_GEO,
    md_extras_from_csv, nc_extras_from_csv,
};
pub use civic::{civic_extras_from_voterinfo, merge_civic_into, CIVIC_API_ROOT, CIVIC_PUBLISHER};
pub use models::*;
pub use redact::{fec_source_url_public, redact_secrets};
