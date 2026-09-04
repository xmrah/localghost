use regex::Regex;
use std::process::Command;

/// Tehlikeli komut kalıpları — Python versiyonundan port edildi ve genişletildi
const DANGEROUS_PATTERNS: &[&str] = &[
    // Recursive/forced deletion
    r"rm\s+.*(-[a-z]*r|-[a-z]*f|--recursive|--force).*(/|~|\$HOME)",
    r"rm\s+(-rf|-fr)\b",
    // Disk formatting / raw writes
    r"mkfs\b",
    r"dd\s+.*(if|of)=.*/dev/",
    r"wipefs\b",
    r"fdisk\s+/dev/",
    r"parted\s+/dev/",
    r"sgdisk\s+/dev/",
    // Fork bombs
    r":\(\)\{.*:\|:",
    r"\.\s*\(\)\s*\{.*\|",
    // Block device overwrite
    r">\s*/dev/(sd|nvme|vd|hd|loop)",
    // Dangerous permissions
    r"chmod\s+(-[a-z]*\s+)?777\s+/",
    r"chmod\s+(-[a-z]*\s+)?(777|666)\s+/etc",
    r"chown\s+.*\s+/\s",
    r"chown\s+.*\s+/$",
    // Moving/overwriting root
    r"mv\s+/\s",
    r"mv\s+/$",
    // Remote code execution via pipe
    r"(curl|wget)\s+.*\|\s*(sudo\s+)?(bash|sh|zsh|python|perl|ruby)",
    r"(curl|wget)\s+.*-o\s*-\s*\|",
    // eval/exec with variables
    r"eval\s+\$",
    r#"eval\s+['"].*\$"#,
    r"exec\s+\$",
    // Python/Perl attacks
    r"python[23]?\s+-c\s+.*os\.(system|remove|unlink|rmdir)",
    r"perl\s+-e\s+.*unlink",
    // History manipulation
    r"history\s+-c",
    r"shred.*\.(bash_history|zsh_history|fish_history)",
    // Critical system files
    r"rm\s+.*/boot/",
    r"rm\s+.*/etc/(passwd|shadow|fstab|sudoers)",
    // NixOS specific
    r"nix-store\s+--delete\s+/nix/store",
    r"rm\s+-rf\s+/nix",
];

pub fn check(command: &str) -> (bool, Option<String>) {
    for pattern in DANGEROUS_PATTERNS {
        let Ok(re) = Regex::new(pattern) else { continue };
        if re.is_match_at(command, 0) || re.is_match(&command.to_lowercase()) {
            return (false, Some(pattern.to_string()));
        }
    }
    (true, None)
}

/// Komutun ilk binary'sinin sistemde var olup olmadığını kontrol et
pub fn validate_binary(command: &str) -> Option<String> {
    let skip = ["sudo", "env", "nix-shell", "doas", "pkexec", "time", "nice"];

    let parts: Vec<&str> = command.split_whitespace().collect();
    let cmd_name = parts.iter().find(|p| {
        !p.starts_with('-') && !skip.contains(p) && !p.contains('=')
    })?;

    // Pipeline'daki ilk komutu al
    let cmd_name = cmd_name.split('|').next()?.trim();
    let cmd_name = cmd_name.split(';').next()?.trim();
    let cmd_name = cmd_name.split("&&").next()?.trim();

    // which ile kontrol
    let output = Command::new("sh")
        .args(["-c", &format!("command -v {}", cmd_name)])
        .output()
        .ok()?;

    if output.status.success() {
        None
    } else {
        Some(cmd_name.to_string())
    }
}
