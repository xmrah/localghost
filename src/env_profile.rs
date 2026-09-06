use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, process::Command, time::SystemTime};
use dirs::data_local_dir;

const CACHE_FILE: &str = "localghost/env.json";
const CACHE_MAX_AGE_SECS: u64 = 86400; // 24 saat

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnvProfile {
    pub shell: String,
    pub aliases: HashMap<String, String>,
    pub detected_at: String,
}

impl Default for EnvProfile {
    fn default() -> Self {
        Self {
            shell: std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string()),
            aliases: HashMap::new(),
            detected_at: chrono::Local::now().to_rfc3339(),
        }
    }
}

impl EnvProfile {
    pub fn to_context_string(&self) -> String {
        if self.aliases.is_empty() {
            return "Alias yok".to_string();
        }
        self.aliases
            .iter()
            .map(|(orig, real)| format!("{} yerine {} kullan", orig, real))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn cache_path() -> std::path::PathBuf {
    data_local_dir()
        .expect("XDG_DATA_HOME veya HOME ortam değişkeni bulunamadı")
        .join(CACHE_FILE)
}

pub fn load_or_detect() -> Result<EnvProfile> {
    let path = cache_path();

    // Cache taze mi?
    if path.exists() {
        if let Ok(meta) = fs::metadata(&path) {
            if let Ok(modified) = meta.modified() {
                if let Ok(age) = SystemTime::now().duration_since(modified) {
                    if age.as_secs() < CACHE_MAX_AGE_SECS {
                        let contents = fs::read_to_string(&path)?;
                        if let Ok(profile) = serde_json::from_str::<EnvProfile>(&contents) {
                            return Ok(profile);
                        }
                    }
                }
            }
        }
    }

    detect_and_save()
}

pub fn refresh() -> Result<EnvProfile> {
    let path = cache_path();
    if path.exists() {
        let _ = fs::remove_file(&path);
    }
    detect_and_save()
}

fn detect_and_save() -> Result<EnvProfile> {
    let mut profile = EnvProfile::default();

    // Modern araç kontrolleri
    let checks: &[(&str, &str, &str, &str)] = &[
        ("find", "find", "fd", "fd"),
        ("ls",   "ls",   "eza", "eza"),
        ("cat",  "cat",  "bat", "bat"),
        ("grep", "grep", "ripgrep", "rg"),
        ("du",   "du",   "dust", "dust"),
        ("ps",   "ps",   "procs", "procs"),
    ];

    for (orig, test_cmd, marker, real) in checks {
        let result = Command::new(test_cmd)
            .arg("--version")
            .output();

        if let Ok(out) = result {
            let text = String::from_utf8_lossy(&out.stdout).to_lowercase()
                + &String::from_utf8_lossy(&out.stderr).to_lowercase();
            if text.contains(marker) {
                profile.aliases.insert(orig.to_string(), real.to_string());
            }
        }
    }

    // Cache'e kaydet
    let path = cache_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(&profile)?;
    fs::write(&path, json)?;

    Ok(profile)
}

pub fn print_profile() -> Result<()> {
    let profile = load_or_detect()?;
    println!("\x1b[1mOrtam Profili\x1b[0m");
    println!("  Shell: {}", profile.shell);
    println!("  Tarandı: {}", profile.detected_at);
    if profile.aliases.is_empty() {
        println!("  Alias: yok");
    } else {
        println!("  Modern araçlar:");
        for (orig, real) in &profile.aliases {
            println!("    {} → {}", orig, real);
        }
    }
    println!("  Cache: {}", cache_path().display());
    Ok(())
}
