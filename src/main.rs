mod cli;
mod config;
mod distro;
mod env_profile;
mod hardware;
mod history;
mod ollama;
mod output;
mod safety;
mod tui;

use anyhow::Result;
use cli::{Cli, Commands};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Models) => {
            output::print_header();
            let models = ollama::list_models(&cli.ollama_url).await?;
            if models.is_empty() {
                output::warn("Hiç model bulunamadı. Ollama çalışıyor mu?");
                output::info(&format!("Denenen URL: {}/api/tags", cli.ollama_url));
            } else {
                let default = config::load()?.model.default;
                println!("Mevcut modeller ({}):", models.len());
                for m in &models {
                    if *m == default {
                        println!("  {} {} ← varsayılan", output::bullet(), output::highlight(m));
                    } else {
                        println!("  {} {}", output::bullet(), m);
                    }
                }
            }
        }

        Some(Commands::History) => {
            history::print_history()?;
        }

        Some(Commands::ClearHistory) => {
            history::clear()?;
            output::success("Geçmiş temizlendi.");
        }

        Some(Commands::Env) => {
            env_profile::print_profile()?;
        }

        Some(Commands::RefreshEnv) => {
            env_profile::refresh()?;
            output::success("Ortam profili yeniden tarandı.");
            env_profile::print_profile()?;
        }

        Some(Commands::GenerateCompletion { shell }) => {
            cli::generate_completion(shell);
        }

        Some(Commands::Interactive) => {
            tui::run_interactive(&cli).await?;
        }

        None => {
            // Ana akış: sorgu al, komut üret
            let query = match &cli.query {
                Some(q) if !q.trim().is_empty() => q.clone(),
                _ => {
                    // Stdin'den oku (pipe desteği)
                    let stdin = read_stdin();
                    if stdin.trim().is_empty() {
                        eprintln!("{}", output::error_msg("Sorgu boş. Kullanım: localghost \"komut açıklaman\""));
                        std::process::exit(1);
                    }
                    stdin
                }
            };

            run_query(&cli, &query).await?;
        }
    }

    Ok(())
}

async fn run_query(cli: &Cli, query: &str) -> Result<()> {
    let config = config::load()?;

    // Sorguyu temizle ve kısalt
    let query = sanitize_query(query, 500);

    // Sistem bilgisi topla
    let distro = distro::detect();
    let hw = hardware::detect();
    let env = env_profile::load_or_detect()?;
    let hist = history::load()?;

    // Model seç
    let model = cli.model
        .clone()
        .unwrap_or_else(|| config.model.default.clone());

    // Mevcut modelleri kontrol et
    let available = ollama::list_models(&cli.ollama_url).await
        .unwrap_or_default();

    let selected_model = if available.is_empty() {
        model.clone()
    } else if !available.contains(&model) {
        let fallback = available[0].clone();
        output::warn(&format!("'{}' bulunamadı. '{}' kullanılıyor.", model, fallback));
        fallback
    } else {
        model
    };

    // Prompt oluştur
    let system_prompt = build_system_prompt(&distro, &hw, &env, &hist, cli.explain);

    // Ollama'ya sor
    let result = ollama::generate(
        &cli.ollama_url,
        &selected_model,
        &query,
        &system_prompt,
        cli.explain,
    ).await?;

    // Güvenlik kontrolü
    let cmd = &result.command;
    let (is_safe, pattern) = safety::check(cmd);

    if !is_safe {
        output::danger_box(cmd, pattern.as_deref().unwrap_or(""));
    } else {
        // Binary var mı kontrol et
        if let Some(missing) = safety::validate_binary(cmd) {
            output::warn(&format!("'{}' bu sistemde bulunamadı.", missing));
        }

        if cli.explain {
            output::explain_box(cmd, &result.explanation.unwrap_or_default());
        } else {
            println!("{}", cmd);
        }

        // Geçmişe kaydet
        history::append(&query, &cmd)?;
    }

    Ok(())
}

fn build_system_prompt(
    distro: &distro::DistroInfo,
    hw: &hardware::HardwareInfo,
    env: &env_profile::EnvProfile,
    hist: &[history::HistoryEntry],
    explain: bool,
) -> String {
    let hw_hints = hw.to_context_string();
    let alias_hints = env.to_context_string();
    let history_ctx = if hist.is_empty() {
        "None".to_string()
    } else {
        hist.iter()
            .rev()
            .take(5)
            .map(|e| format!("Q: {} → A: {}", e.query, e.command))
            .collect::<Vec<_>>()
            .join("; ")
    };

    if explain {
        format!(
            "You are an expert Linux command explainer running on {name} ({id}). \
            Output JSON with this schema: {{\"command\": \"<shell command>\", \"explanation\": \"<detailed explanation of each part>\"}}. \
            Explanation must be in Turkish. Be specific about each flag and argument. \
            RULES: 1. Output ONLY valid JSON. 2. Use {pkg} for package management. \
            CONTEXT: Hardware: {hw}. Aliases: {alias}. Recent: {hist}.",
            name = distro.name,
            id = distro.id,
            pkg = distro.pkg_manager,
            hw = hw_hints,
            alias = alias_hints,
            hist = history_ctx,
        )
    } else {
        format!(
            "You are an expert Linux terminal command generator running on {name} ({id}). \
            Output JSON with this schema: {{\"command\": \"<exact shell command>\", \"risk_level\": \"safe|caution|dangerous\"}}. \
            RULES: \
            1. Output ONLY valid JSON. \
            2. The command field must contain ONE valid shell command, no explanations, no markdown. \
            3. Use {pkg} for package management. For NixOS use nixos-rebuild or nix profile, NEVER apt/dpkg. \
            4. EXAMPLES: \
            Q: update system → {{\"command\": \"sudo nixos-rebuild switch --upgrade\", \"risk_level\": \"safe\"}} \
            Q: find large files → {{\"command\": \"fd --size +500M\", \"risk_level\": \"safe\"}} \
            CONTEXT: Hardware: {hw}. Aliases: {alias}. Recent: {hist}.",
            name = distro.name,
            id = distro.id,
            pkg = distro.pkg_manager,
            hw = hw_hints,
            alias = alias_hints,
            hist = history_ctx,
        )
    }
}

fn sanitize_query(input: &str, max_len: usize) -> String {
    let cleaned: String = input
        .chars()
        .filter(|c| !matches!(*c as u32, 0x00..=0x08 | 0x0b | 0x0c | 0x0e..=0x1f | 0x7f))
        .collect();
    if cleaned.len() > max_len {
        eprintln!("{}", output::warn_msg(&format!("Sorgu {} karaktere kısaltıldı.", max_len)));
        cleaned[..max_len].to_string()
    } else {
        cleaned
    }
}

fn read_stdin() -> String {
    use std::io::Read;
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf).unwrap_or(0);
    buf.trim().to_string()
}
