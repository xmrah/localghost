use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
pub enum ExecutionTier {
    Tier0AutoExec,
    Tier1ConfirmRequired(String), // İhtiyaç duyulan onayın nedeni
}

pub fn analyze_command(cmd: &str) -> ExecutionTier {
    let cmd = cmd.trim();

    // 1. Metacharacter Kontrolü (Shell Injection Risk)
    let metachars = [";", "|", "&", "<", ">", "$", "`", "\n"];
    for ch in metachars.iter() {
        if cmd.contains(ch) {
            return ExecutionTier::Tier1ConfirmRequired(format!("Shell metakarakteri tespit edildi: '{}'", ch));
        }
    }

    // 2. Tokenizer (Argv ayrıştırma)
    let argv = match shell_words::split(cmd) {
        Ok(args) if !args.is_empty() => args,
        _ => return ExecutionTier::Tier1ConfirmRequired("Komut parse edilemedi veya boş".into()),
    };
    
    let binary = &argv[0];

    // 3. Allowlist (GTFOBins Korumalı, SADECE Saf Okuyucular)
    let mut allowlist = HashSet::new();
    let safe_bins = vec![
        "ls", "cat", "echo", "pwd", "date", "whoami", "uname", "uptime", "free", "df",
        "ps", "grep", "head", "tail", "ip", "ping", "netstat", "ss",
        "systemctl", "journalctl", "dmesg", "lsblk", "lscpu", "lspci", "lsusb",
        "stat", "file", "which", "whereis", "type", "history", "clear",
        "bat", "eza", "exa", "fd", "rg", "neofetch", "fastfetch"
        // GTFOBins çıkırılanlar: find, awk, sed, less, more, nmap, curl, wget, top, htop, btm
    ];
    
    for bin in safe_bins {
        allowlist.insert(bin);
    }
    
    let is_safe_systemctl = binary == "systemctl" && argv.len() >= 2 && argv[1] == "status";
    
    if !allowlist.contains(binary.as_str()) && !is_safe_systemctl && binary != "journalctl" {
        return ExecutionTier::Tier1ConfirmRequired(format!("Binary allowlist'te yok veya GTFOBins riski taşıyor: '{}'", binary));
    }

    // 4. Path-Sensitivity: Zero-Trust (Default-Deny) Modeli
    for arg in &argv[1..] {
        // Parametreler (-) path olarak değerlendirilmez (bayraklar)
        if arg.starts_with('-') {
            continue;
        }

        // a. Gizli dosya veya dizin kontrolü (.ssh, .config, vs)
        if arg.starts_with('.') && !arg.starts_with("./") {
            // Doğrudan gizli dosyaysa (.bashrc gibi)
            return ExecutionTier::Tier1ConfirmRequired(format!("Gizli dosya/dizin erişimi reddedildi: '{}'", arg));
        }
        
        // Yolun içinde '/.' varsa (ör: /home/user/.ssh, ./src/.env)
        if arg.contains("/.") && !arg.contains("/./") {
             return ExecutionTier::Tier1ConfirmRequired(format!("Yol içinde gizli dosya/dizin tespit edildi: '{}'", arg));
        }
        
        let path = Path::new(arg);
        
        // b. Parent dir çıkışı engeli (.. ile dışarı sızma)
        if path.components().any(|comp| comp == std::path::Component::ParentDir) {
            return ExecutionTier::Tier1ConfirmRequired(format!("Üst dizine (..) geçiş reddedildi: '{}'", arg));
        }

        // c. Absolute Path kontrolü (Sadece /tmp/ altında olanlara izin var)
        if path.is_absolute() {
            if !arg.starts_with("/tmp/") {
                return ExecutionTier::Tier1ConfirmRequired(format!("Güvenli bölge ($PWD) dışı mutlak yol erişimi: '{}'", arg));
            }
        }
    }

    ExecutionTier::Tier0AutoExec
}

/// Geriye dönük uyumluluk (TUI için)
pub fn is_dangerous(cmd: &str) -> bool {
    match analyze_command(cmd) {
        ExecutionTier::Tier1ConfirmRequired(_) => true,
        ExecutionTier::Tier0AutoExec => false,
    }
}
