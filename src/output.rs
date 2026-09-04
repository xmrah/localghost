use colored::*;

pub fn bullet() -> String {
    "•".cyan().to_string()
}

pub fn highlight(s: &str) -> String {
    s.yellow().bold().to_string()
}

pub fn print_header() {
    println!("{}", "👻 LocalGhost".bold());
}

pub fn success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg);
}

pub fn warn(msg: &str) {
    eprintln!("{} {}", "⚠".yellow().bold(), msg);
}

pub fn warn_msg(msg: &str) -> String {
    format!("{} {}", "⚠".yellow().bold(), msg)
}

pub fn info(msg: &str) {
    println!("{} {}", "ℹ".blue(), msg);
}

pub fn error_msg(msg: &str) -> String {
    format!("{} {}", "✗".red().bold(), msg)
}

pub fn danger_box(command: &str, pattern: &str) {
    eprintln!("\n{}", "╔══ ⚠  TEHLİKELİ KOMUT TESPİT EDİLDİ ══╗".red().bold());
    eprintln!("{} {}", "║ Komut:".red(), command.yellow());
    eprintln!("{} {}", "║ Eşleşen:".red(), pattern.dimmed());
    eprintln!("{}", "╚══════════════════════════════════════╝".red().bold());
    eprintln!("{}", "  Çalıştırmadan önce dikkatlice inceleyin!".red());
}

pub fn explain_box(command: &str, explanation: &str) {
    println!("{}", command.green().bold());
    if !explanation.is_empty() {
        println!();
        println!("{}", "📖 Açıklama:".cyan().bold());
        for line in explanation.lines() {
            println!("   {}", line.dimmed());
        }
    }
}

/// Spinner — async ile birlikte kullan
pub async fn with_spinner<F, T>(msg: &str, fut: F) -> T
where
    F: std::future::Future<Output = T>,
{
    use std::io::Write;
    let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let msg = msg.to_string();

    let (tx, mut rx) = tokio::sync::oneshot::channel::<()>();

    // Spinner task
    tokio::spawn(async move {
        let mut i = 0usize;
        loop {
            tokio::select! {
                _ = &mut rx => break,
                _ = tokio::time::sleep(std::time::Duration::from_millis(80)) => {
                    eprint!("\r{} {}  ", frames[i % frames.len()].cyan(), msg);
                    let _ = std::io::stderr().flush();
                    i += 1;
                }
            }
        }
        eprint!("\r{}\r", " ".repeat(msg.len() + 10));
        let _ = std::io::stderr().flush();
    });

    let result = fut.await;
    let _ = tx.send(());
    result
}
