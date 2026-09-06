use crate::safety::{analyze_command, ExecutionTier};
use anyhow::Result;
use std::io::{self, Write};
use std::process::{Command, Stdio};

/// Üretilen LLM komutunu güvenlik Tier'ına göre güvenli bir şekilde çalıştırır.
pub fn execute_command(cmd: &str) -> Result<()> {
    let tier = analyze_command(cmd);

    match tier {
        ExecutionTier::Tier0AutoExec => {
            // GÜVENLİ: Shell'siz doğrudan argv tabanlı çalıştırma. Shell injection riski sıfır.
            let argv = match shell_words::split(cmd) {
                Ok(args) if !args.is_empty() => args,
                Ok(_) => {
                    eprintln!("❌ Boş komut çalıştırılamaz.");
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("❌ Komut ayrıştırılamadı: {}", e);
                    return Ok(());
                }
            };
            
            println!("🚀 Çalıştırılıyor (Oto-Onaylı): {}\n", cmd);
            
            let mut child = Command::new(&argv[0])
                .args(&argv[1..])
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()?;
            
            let status = child.wait()?;
            if !status.success() {
                eprintln!("\n❌ Komut başarısız oldu: {}", status);
            }
        }
        ExecutionTier::Tier1ConfirmRequired(reason) => {
            // ONAY GEREKİYOR: Pipe, metakarakter veya allowlist dışı komut
            println!("\n🚨 [ONAY GEREKİYOR] {}", reason);
            println!("Komut: \x1b[1;31m{}\x1b[0m", cmd); // Kırmızı renkli gösterim
            print!("Çalıştırmak istiyor musunuz? [E/h]: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim().to_lowercase();

            if input == "e" || input == "evet" || input == "y" || input == "yes" {
                println!(); // Boş satır
                // sh -c kullanarak çalıştır (çünkü pipe veya metakarakter içerebilir)
                let mut child = Command::new("sh")
                    .arg("-c")
                    .arg(cmd)
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .spawn()?;
                
                let status = child.wait()?;
                if !status.success() {
                    eprintln!("\n❌ Komut başarısız oldu: {}", status);
                }
            } else {
                println!("İşlem iptal edildi.");
            }
        }
    }

    Ok(())
}
