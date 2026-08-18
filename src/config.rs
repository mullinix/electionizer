use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Serve,
    Daemon,
    Both,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    Fixture,
    Live,
}

/// On-disk app config (`electionizer.toml`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConfig {
    #[serde(default = "default_provider")]
    pub provider: ProviderKind,
    #[serde(default = "default_db")]
    pub db: PathBuf,
    #[serde(default = "default_bind")]
    pub bind: String,
    #[serde(default = "default_mode")]
    pub mode: Mode,
    #[serde(default = "default_fixture_dir")]
    pub fixture_dir: PathBuf,
    #[serde(default = "default_daemon_interval")]
    pub daemon_interval_secs: u64,
    #[serde(default = "default_refresh_hours")]
    pub refresh_hours: i64,
    #[serde(default = "default_stage_delay")]
    pub stage_delay_ms: u64,
    #[serde(default = "default_cycle")]
    pub cycle: i32,
    #[serde(default)]
    pub fec: FecSection,
    #[serde(default)]
    pub openstates: OpenStatesSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FecSection {
    #[serde(default = "default_fec_key")]
    pub api_key: String,
    /// How long to reuse FEC candidate list responses (hours)
    #[serde(default = "default_fec_cache_ttl")]
    pub cache_ttl_hours: i64,
}

impl Default for FecSection {
    fn default() -> Self {
        Self {
            api_key: default_fec_key(),
            cache_ttl_hours: default_fec_cache_ttl(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenStatesSection {
    /// Free key from https://open.pluralpolicy.com/accounts/profile/
    #[serde(default)]
    pub api_key: String,
}

fn default_provider() -> ProviderKind {
    ProviderKind::Live
}
fn default_db() -> PathBuf {
    PathBuf::from("electionizer.db")
}
fn default_bind() -> String {
    "127.0.0.1:3000".into()
}
fn default_mode() -> Mode {
    Mode::Both
}
fn default_fixture_dir() -> PathBuf {
    PathBuf::from("testdata")
}
fn default_daemon_interval() -> u64 {
    3600
}
fn default_refresh_hours() -> i64 {
    24
}
fn default_stage_delay() -> u64 {
    400
}
fn default_cycle() -> i32 {
    2026
}
fn default_fec_key() -> String {
    "DEMO_KEY".into()
}
fn default_fec_cache_ttl() -> i64 {
    24
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            db: default_db(),
            bind: default_bind(),
            mode: default_mode(),
            fixture_dir: default_fixture_dir(),
            daemon_interval_secs: default_daemon_interval(),
            refresh_hours: default_refresh_hours(),
            stage_delay_ms: default_stage_delay(),
            cycle: default_cycle(),
            fec: FecSection::default(),
            openstates: OpenStatesSection::default(),
        }
    }
}

impl FileConfig {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("read config {}", path.display()))?;
        let cfg: Self = toml::from_str(&text)
            .with_context(|| format!("parse config {}", path.display()))?;
        Ok(cfg)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("create config dir {}", parent.display()))?;
            }
        }
        let text = toml::to_string_pretty(self).context("serialize config toml")?;
        let header = "# electionizer local config — do not commit secrets\n\
                      # Example: electionizer.example.toml\n\
                      # FEC key: https://api.open.fec.gov/developers/\n\
                      # OpenStates key: https://open.pluralpolicy.com/accounts/profile/\n\n";
        std::fs::write(path, format!("{header}{text}"))
            .with_context(|| format!("write config {}", path.display()))?;
        Ok(())
    }

}

/// CLI overrides (optional flags win over file).
#[derive(Debug, Clone, Parser)]
#[command(name = "electionizer", about = "ZIP-based election voter reports")]
pub struct Cli {
    /// Path to TOML config file
    #[arg(long, env = "ELECTIONIZER_CONFIG", default_value = "electionizer.toml")]
    pub config: PathBuf,

    /// SQLite database path (overrides config file)
    #[arg(long, env = "ELECTIONIZER_DB")]
    pub db: Option<PathBuf>,

    /// Bind address for HTTP server (overrides config file)
    #[arg(long, env = "ELECTIONIZER_BIND")]
    pub bind: Option<String>,

    /// Run mode (overrides config file)
    #[arg(long, env = "ELECTIONIZER_MODE", value_enum)]
    pub mode: Option<Mode>,

    /// Data provider (overrides config file)
    #[arg(long, env = "ELECTIONIZER_PROVIDER", value_enum)]
    pub provider: Option<ProviderKind>,

    /// Delete the SQLite DB file before starting
    #[arg(long, env = "ELECTIONIZER_FRESH", default_value_t = false)]
    pub fresh: bool,

    /// Directory containing fixture JSON files (overrides config file)
    #[arg(long, env = "ELECTIONIZER_FIXTURE_DIR")]
    pub fixture_dir: Option<PathBuf>,

    #[arg(long, env = "ELECTIONIZER_DAEMON_INTERVAL_SECS")]
    pub daemon_interval_secs: Option<u64>,

    #[arg(long, env = "ELECTIONIZER_REFRESH_HOURS")]
    pub refresh_hours: Option<i64>,

    #[arg(long, env = "ELECTIONIZER_STAGE_DELAY_MS")]
    pub stage_delay_ms: Option<u64>,

    /// OpenFEC API key (overrides config file)
    #[arg(long, env = "ELECTIONIZER_FEC_API_KEY")]
    pub fec_api_key: Option<String>,

    /// OpenStates API key for state legislature roll-calls (overrides config file)
    #[arg(long, env = "ELECTIONIZER_OPENSTATES_API_KEY")]
    pub openstates_api_key: Option<String>,

    /// Federal election cycle year (overrides config file)
    #[arg(long, env = "ELECTIONIZER_CYCLE")]
    pub cycle: Option<i32>,

    /// FEC response cache TTL in hours (overrides config file)
    #[arg(long, env = "ELECTIONIZER_FEC_CACHE_TTL_HOURS")]
    pub fec_cache_ttl_hours: Option<i64>,
}

/// Merged runtime configuration.
#[derive(Debug, Clone)]
pub struct Config {
    pub config_path: PathBuf,
    pub db: PathBuf,
    pub bind: String,
    pub mode: Mode,
    pub provider: ProviderKind,
    pub fresh: bool,
    pub fixture_dir: PathBuf,
    pub daemon_interval_secs: u64,
    pub refresh_hours: i64,
    pub stage_delay_ms: u64,
    pub cycle: i32,
    /// Shared so Settings can update without restart.
    pub fec_api_key: Arc<RwLock<String>>,
    pub fec_cache_ttl_hours: i64,
    /// Shared OpenStates key for state roll-calls (empty = disabled).
    pub openstates_api_key: Arc<RwLock<String>>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let cli = Cli::parse();
        let file = FileConfig::load(&cli.config)?;
        Ok(Self::merge(cli, file))
    }

    fn merge(cli: Cli, file: FileConfig) -> Self {
        let fec_key = cli
            .fec_api_key
            .unwrap_or(file.fec.api_key)
            .trim()
            .to_string();
        let fec_key = if fec_key.is_empty() {
            default_fec_key()
        } else {
            fec_key
        };
        let os_key = cli
            .openstates_api_key
            .unwrap_or(file.openstates.api_key)
            .trim()
            .to_string();

        Self {
            config_path: cli.config,
            db: cli.db.unwrap_or(file.db),
            bind: cli.bind.unwrap_or(file.bind),
            mode: cli.mode.unwrap_or(file.mode),
            provider: cli.provider.unwrap_or(file.provider),
            fresh: cli.fresh,
            fixture_dir: cli.fixture_dir.unwrap_or(file.fixture_dir),
            daemon_interval_secs: cli
                .daemon_interval_secs
                .unwrap_or(file.daemon_interval_secs),
            refresh_hours: cli.refresh_hours.unwrap_or(file.refresh_hours),
            stage_delay_ms: cli.stage_delay_ms.unwrap_or(file.stage_delay_ms),
            cycle: cli.cycle.unwrap_or(file.cycle),
            fec_api_key: Arc::new(RwLock::new(fec_key)),
            fec_cache_ttl_hours: cli
                .fec_cache_ttl_hours
                .unwrap_or(file.fec.cache_ttl_hours)
                .max(1),
            openstates_api_key: Arc::new(RwLock::new(os_key)),
        }
    }

    pub fn fec_api_key_snapshot(&self) -> String {
        self.fec_api_key
            .read()
            .map(|g| g.clone())
            .unwrap_or_else(|_| default_fec_key())
    }

    pub fn set_fec_api_key(&self, key: String) {
        if let Ok(mut g) = self.fec_api_key.write() {
            *g = key;
        }
    }

    pub fn openstates_api_key_snapshot(&self) -> String {
        self.openstates_api_key
            .read()
            .map(|g| g.clone())
            .unwrap_or_default()
    }

    pub fn set_openstates_api_key(&self, key: String) {
        if let Ok(mut g) = self.openstates_api_key.write() {
            *g = key;
        }
    }

    /// Update API keys on disk without clobbering other file values with CLI overrides.
    pub fn save_api_keys_to_file(&self) -> Result<()> {
        let mut file = FileConfig::load(&self.config_path)?;
        file.fec.api_key = self.fec_api_key_snapshot();
        file.openstates.api_key = self.openstates_api_key_snapshot();
        file.save(&self.config_path)
    }

}

pub fn mask_api_key(key: &str) -> String {
    let key = key.trim();
    if key.is_empty() {
        return "(not set)".into();
    }
    if key == "DEMO_KEY" {
        return "DEMO_KEY (rate-limited)".into();
    }
    if key.len() <= 4 {
        return "••••".into();
    }
    format!("••••{}", &key[key.len() - 4..])
}

pub fn is_demo_key(key: &str) -> bool {
    key.trim().is_empty() || key.trim() == "DEMO_KEY"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_default_toml() {
        let cfg = FileConfig::default();
        let s = toml::to_string_pretty(&cfg).unwrap();
        let back: FileConfig = toml::from_str(&s).unwrap();
        assert_eq!(back.cycle, 2026);
        assert_eq!(back.fec.api_key, "DEMO_KEY");
    }

    #[test]
    fn mask_key() {
        assert!(mask_api_key("DEMO_KEY").contains("DEMO_KEY"));
        assert_eq!(mask_api_key("abcdefghij"), "••••ghij");
    }
}
