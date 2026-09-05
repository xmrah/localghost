use anyhow::{anyhow, Result};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

pub fn install(shell: Option<&str>) -> Result<()> {
    let shell = shell.unwrap_or("fish");
    match shell {
        "fish" => install_fish()?,
        "bash" => install_bash()?,
        "zsh" => install_zsh()?,
        _ => return Err(anyhow!("Desteklenmeyen shell: {}", shell)),
    }
    crate::output::success(&format!("{} entegrasyonu başarıyla kuruldu! Terminalinizi yeniden başlatın veya kaynak dosyayı (source) yeniden yükleyin.", shell));
    Ok(())
}

fn install_fish() -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("Ev dizini bulunamadı"))?;
    let config_dir = home.join(".config/fish/functions");
    std::fs::create_dir_all(&config_dir)?;
    let fish_config = config_dir.join("??.fish");
    
    let fish_script = r#"
# Localghost Integration
function ??
    localghost -x $argv
end
"#;

    write_to_file(&fish_config, fish_script)?;
    Ok(())
}

fn install_bash() -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("Ev dizini bulunamadı"))?;
    let bash_config = home.join(".bashrc");
    
    let bash_script = r#"
# Localghost Integration
function ??() {
    localghost -x "$*"
}
"#;

    append_to_file(&bash_config, bash_script)?;
    Ok(())
}

fn install_zsh() -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("Ev dizini bulunamadı"))?;
    let zsh_config = home.join(".zshrc");
    
    let zsh_script = r#"
# Localghost Integration
function ??() {
    localghost -x "$*"
}
"#;

    append_to_file(&zsh_config, zsh_script)?;
    Ok(())
}

fn append_to_file(path: &PathBuf, content: &str) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file, "{}", content)?;
    Ok(())
}

fn write_to_file(path: &PathBuf, content: &str) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;
    writeln!(file, "{}", content)?;
    Ok(())
}
