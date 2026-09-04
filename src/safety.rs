use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq)]
pub enum ExecutionTier {
    Tier0AutoExec,
    Tier1ConfirmRequired(String), // İhtiyaç duyulan onayın nedeni
}

pub fn analyze_command(cmd: &str) -> ExecutionTier {
    let cmd = cmd.trim();

    // 1. Metacharacter Kontrolü (Shell Injection Risk)
    // Bu karakterlerin varlığı komutun bir shell tarafından (sh -c) çalıştırılmasını zorunlu kılar.
    // Bu yüzden asla Tier-0 olarak doğrudan çalıştırılamazlar, onaya düşerler.
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

    // 3. Allowlist (Salt Okunur ve Zararsız Komutlar)
    let mut allowlist = HashSet::new();
    let safe_bins = vec![
        "ls", "cat", "echo", "pwd", "date", "whoami", "uname", "uptime", "free", "df",
        "top", "htop", "btm", "neofetch", "fastfetch", "ps", "grep", "awk", "sed",
        "head", "tail", "less", "more", "find", "fd", "rg", "bat", "eza", "exa",
        "ip", "ping", "netstat", "ss", "nmap", "curl", "wget", "dig", "host",
        "systemctl", "journalctl", "dmesg", "lsblk", "lscpu", "lspci", "lsusb",
        "stat", "file", "which", "whereis", "type", "history", "clear", "which"
    ];
    for bin in safe_bins {
        allowlist.insert(bin);
    }
    
    // systemctl için sadece 'status' argümanı güvenlidir
    let is_safe_systemctl = binary == "systemctl" && argv.len() >= 2 && argv[1] == "status";
    
    if !allowlist.contains(binary.as_str()) && !is_safe_systemctl && binary != "journalctl" {
        return ExecutionTier::Tier1ConfirmRequired(format!("Binary allowlist'te yok: '{}'", binary));
    }

    // 4. Path-Sensitivity (Veri Sızıntısı Kontrolü)
    // Sadece okuma bile yapsa, hassas verilere erişenler onaylanmalıdır.
    let sensitive_paths = [
        "/etc/shadow", "/etc/passwd", "/etc/sudoers",
        ".ssh", "id_rsa", "id_ed25519", "authorized_keys",
        ".gnupg", "gpg", "age", "sops",
        ".aws/credentials", ".kube/config", ".env"
    ];

    for arg in &argv[1..] {
        for path in sensitive_paths.iter() {
            if arg.contains(path) {
                return ExecutionTier::Tier1ConfirmRequired(format!("Hassas veri/yol erişimi tespit edildi: '{}'", path));
            }
        }
    }

    // Tüm testlerden geçerse, şüphesiz güvenlidir ve shell'siz (argv) çalıştırılabilir.
    ExecutionTier::Tier0AutoExec
}

/// Geriye dönük uyumluluk (TUI için)
pub fn is_dangerous(cmd: &str) -> bool {
    match analyze_command(cmd) {
        ExecutionTier::Tier1ConfirmRequired(_) => true,
        ExecutionTier::Tier0AutoExec => false,
    }
}
