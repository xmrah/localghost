use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use dirs::config_dir;

const CONFIG_FILENAME: &str = "localghost/config.toml";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub model: ModelConfig,
    pub history: HistoryConfig,
    pub safety: SafetyConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelConfig {
    pub default: String,
    pub fallback: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryConfig {
    pub ttl_days: u64,
    pub max_entries: usize,
    pub context_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SafetyConfig {
    pub block_dangerous: bool,
    pub validate_binary: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiConfig {
    pub color: bool,
    pub stream: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model: ModelConfig {
                default: "qwen2.5-coder:7b".to_string(),
                fallback: "qwen2.5vl:7b".to_string(),
            },
            history: HistoryConfig {
                ttl_days: 7,
                max_entries: 100,
                context_count: 5,
            },
            safety: SafetyConfig {
                block_dangerous: true,
                validate_binary: true,
            },
            ui: UiConfig {
                color: true,
                stream: true,
            },
        }
    }
}

pub fn load() -> Result<Config> {
    let path = config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("~/.config"))
        .join(CONFIG_FILENAME);

    if !path.exists() {
        return Ok(Config::default());
    }

    let contents = fs::read_to_string(&path)?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}

pub fn save(config: &Config) -> Result<()> {
    let path = config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("~/.config"))
        .join(CONFIG_FILENAME);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let contents = toml::to_string_pretty(config)?;
    fs::write(path, contents)?;
    Ok(())
}

pub fn config_path() -> std::path::PathBuf {
    config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("~/.config"))
        .join(CONFIG_FILENAME)
}
