use anyhow::Result;
use chrono::{DateTime, Duration, Local, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use dirs::data_local_dir;

const HISTORY_FILE: &str = "localghost/history.json";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryEntry {
    pub ts: DateTime<Utc>,
    pub query: String,
    pub command: String,
}

fn history_path() -> std::path::PathBuf {
    data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("~/.local/share"))
        .join(HISTORY_FILE)
}

pub fn load() -> Result<Vec<HistoryEntry>> {
    let path = history_path();
    if !path.exists() {
        return Ok(vec![]);
    }

    let contents = fs::read_to_string(&path)?;
    let mut entries: Vec<HistoryEntry> = serde_json::from_str(&contents).unwrap_or_default();

    // TTL prune (7 gün)
    let ttl_days = std::env::var("LOCALGHOST_HISTORY_TTL")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(7);
    let cutoff = Utc::now() - Duration::days(ttl_days);
    let before = entries.len();
    entries.retain(|e| e.ts > cutoff);

    // Max 100 giriş
    if entries.len() > 100 {
        let drain_to = entries.len() - 100;
        entries.drain(0..drain_to);
    }

    // Prune olduysa kaydet
    if entries.len() != before {
        save_all(&entries)?;
    }

    Ok(entries)
}

pub fn append(query: &str, command: &str) -> Result<()> {
    let mut entries = load()?;
    entries.push(HistoryEntry {
        ts: Utc::now(),
        query: query.to_string(),
        command: command.to_string(),
    });
    if entries.len() > 100 {
        entries.drain(0..entries.len() - 100);
    }
    save_all(&entries)
}

pub fn clear() -> Result<()> {
    let path = history_path();
    if path.exists() {
        fs::remove_file(&path)?;
    }
    Ok(())
}

fn save_all(entries: &[HistoryEntry]) -> Result<()> {
    let path = history_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(entries)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn print_history() -> Result<()> {
    let entries = load()?;
    if entries.is_empty() {
        println!("Henüz geçmiş yok.");
        return Ok(());
    }
    println!(
        "\x1b[1mKomut Geçmişi ({} giriş)\x1b[0m",
        entries.len()
    );
    for e in entries.iter().rev().take(20) {
        let local: DateTime<Local> = e.ts.into();
        let ts = local.format("%Y-%m-%d %H:%M").to_string();
        println!("  \x1b[90m[{}]\x1b[0m {}", ts, e.query);
        println!("       \x1b[32m→\x1b[0m {}", e.command);
    }
    Ok(())
}
