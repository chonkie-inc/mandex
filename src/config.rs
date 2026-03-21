use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;

use crate::storage::paths;

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct ConfigFile {
    pub search: SearchConfig,
    pub network: NetworkConfig,
    pub display: DisplayConfig,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct SearchConfig {
    pub results: usize,
    pub rerank: bool,
    pub rerank_model: String,
    pub rerank_candidates: usize,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            results: 10,
            rerank: true,
            rerank_model: "~/.mandex/models/reranker.onnx".to_string(),
            rerank_candidates: 20,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct NetworkConfig {
    pub cdn_url: String,
    pub api_url: String,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            cdn_url: "https://cdn.mandex.dev/v1".to_string(),
            api_url: "https://api.mandex.dev".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct DisplayConfig {
    pub color: String,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            color: "auto".to_string(),
        }
    }
}

const DEFAULT_CONFIG: &str = r#"[search]
results = 10             # number of results returned
rerank = true            # use ONNX reranker
rerank_model = "~/.mandex/models/reranker.onnx"
rerank_candidates = 20   # FTS5 candidates fetched before reranking

[network]
cdn_url = "https://cdn.mandex.dev/v1"
api_url = "https://api.mandex.dev"

[display]
color = "auto"           # "auto" | "always" | "never"
"#;

impl ConfigFile {
    /// Load config from ~/.mandex/config.toml, falling back to defaults.
    /// On first run, writes the default config and downloads the reranker model.
    /// Then applies environment variable overrides (MX_ prefix).
    pub fn load() -> Result<Self> {
        let mut config = Self::load_file()?;
        config.apply_env_overrides();
        Ok(config)
    }

    fn load_file() -> Result<Self> {
        let path = config_path()?;
        if !path.exists() {
            std::fs::write(&path, DEFAULT_CONFIG)?;
            return Ok(Self::default());
        }
        let contents = std::fs::read_to_string(&path)?;
        let config: ConfigFile = toml::from_str(&contents)?;
        Ok(config)
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(val) = std::env::var("MX_SEARCH_RESULTS") {
            if let Ok(n) = val.parse::<usize>() {
                self.search.results = n;
            }
        }
        if let Ok(val) = std::env::var("MX_SEARCH_RERANK") {
            self.search.rerank = val == "true" || val == "1";
        }
        if let Ok(val) = std::env::var("MX_SEARCH_RERANK_MODEL") {
            self.search.rerank_model = val;
        }
        if let Ok(val) = std::env::var("MX_SEARCH_RERANK_CANDIDATES") {
            if let Ok(n) = val.parse::<usize>() {
                self.search.rerank_candidates = n;
            }
        }
        if let Ok(val) = std::env::var("MX_NETWORK_CDN_URL") {
            self.network.cdn_url = val;
        }
        if let Ok(val) = std::env::var("MX_NETWORK_API_URL") {
            self.network.api_url = val;
        }
        if let Ok(val) = std::env::var("MX_DISPLAY_COLOR") {
            self.display.color = val;
        }
    }
}

/// Run first-time setup: download the reranker model if missing.
#[cfg(feature = "reranker")]
pub fn ensure_setup(config: &ConfigFile) -> Result<()> {
    let model_path = resolve_model_path(&config.search.rerank_model)?;
    if !model_path.exists() {
        crate::rerank::ensure_model(&model_path, &config.network.cdn_url)?;
    }
    Ok(())
}

#[cfg(not(feature = "reranker"))]
pub fn ensure_setup(_config: &ConfigFile) -> Result<()> {
    Ok(())
}

/// Resolve the model path, expanding ~ to home directory.
pub fn resolve_model_path(raw: &str) -> Result<PathBuf> {
    if raw.starts_with("~/") {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        Ok(home.join(&raw[2..]))
    } else {
        Ok(PathBuf::from(raw))
    }
}

fn config_path() -> Result<PathBuf> {
    Ok(paths::mandex_dir()?.join("config.toml"))
}
