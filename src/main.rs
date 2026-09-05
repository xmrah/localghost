mod cli;
mod config;
mod distro;
mod env_profile;
mod exec;
mod hardware;
mod history;
mod ollama;
mod output;
mod safety;
mod integration;
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

        Some(Commands::SelectModel) => {
            let models = ollama::list_models(&cli.ollama_url).await?;
            if models.is_empty() {
                crate::output::warn("Ollama'da yüklü model bulunamadı!");
                return Ok(());
            }

            let selection = dialoguer::Select::new()
                .with_prompt("Varsayılan olarak kullanmak istediğiniz modeli seçin")
                .items(&models)
                .default(0)
                .interact()?;

            let selected = &models[selection];
            
            let mut config = config::load()?;
            config.model.default = selected.clone();
            config::save(&config)?;
            
            crate::output::success(&format!("Varsayılan model '{}' olarak ayarlandı!", selected));
        }

        Some(Commands::Install { shell }) => {
            crate::integration::install(shell.as_deref())?;
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

        None => {
            if cli.interactive {
                let (cmd_opt, _, _) = tui::run_interactive(&cli).await?;
                if let Some(cmd) = cmd_opt {
                    exec::execute_command(&cmd)?;
                }
            } else {
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
    }

    Ok(())
}

async fn run_query(cli: &Cli, query: &str) -> Result<()> {
    let config = config::load()?;

    // Sorguyu temizle ve kısalt
    let original_query = sanitize_query(query, 500);

    let final_query = match &cli.file {
        Some(filepath) => {
            match std::fs::read_to_string(filepath) {
                Ok(content) => format!("{}\n\n[AŞAĞIDAKİ DOSYA İÇERİĞİNİ BAZ AL]: {}\n```\n{}\n```", original_query, filepath, content),
                Err(e) => {
                    crate::output::warn(&format!("Dosya okunamadı ({}): {}", filepath, e));
                    return Ok(());
                }
            }
        },
        None => original_query.clone(),
    };

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
    let mut system_prompt = build_system_prompt(&distro, &hw, &env, &hist, cli.explain);

    if let Some(role_name) = &cli.role {
        if let Some(config_dir) = dirs::config_dir() {
            let role_path = config_dir.join("localghost").join("roles").join(format!("{}.txt", role_name));
            if role_path.exists() {
                if let Ok(role_content) = std::fs::read_to_string(&role_path) {
                    system_prompt.push_str("\n\n[KULLANICI ÖZEL ROL/PROFİL TALİMATI]:\n");
                    system_prompt.push_str(&role_content);
                }
            } else {
                crate::output::warn(&format!("Rol dosyası bulunamadı: {:?}. Lütfen dosyayı oluşturun.", role_path));
            }
        }
    }

    // Ollama'ya sor
    let result = ollama::generate(
        &cli.ollama_url,
        &selected_model,
        &final_query,
        &system_prompt,
        cli.explain,
    ).await?;

    // Güvenlik kontrolü
    let cmd = &result.command;
    let tier = safety::analyze_command(cmd);

    match tier {
        safety::ExecutionTier::Tier1ConfirmRequired(reason) => {
            output::danger_box(cmd, &reason);
        }
        safety::ExecutionTier::Tier0AutoExec => {
            // Komutu göster
            println!("\n> \x1b[1;32m{}\x1b[0m\n", cmd);
        }
    }

    if cli.explain {
        if let Some(expl) = result.explanation {
            println!("📖 Açıklama:\n   {}\n", expl.replace("\n", "\n   "));
        }
    }

    // Geçmişe kaydet
    history::append(&original_query, &cmd)?;
    
    // Execute modundaysak çalıştır (execute_command zaten Tier'a göre davranır)
    if cli.execute {
        exec::execute_command(&cmd)?;
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
            4. STRICT RULE: DO NOT generate commands with placeholders (like /path/to/dir, <file>, etc). If the user request is missing required specific information (like a filename or directory), your command MUST be an echo statement asking for the missing info. Example: echo 'Lütfen silinecek dosyanın adını belirtin.' \
            5. EXAMPLES: \
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
