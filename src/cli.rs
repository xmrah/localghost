use clap::{Parser, Subcommand};
use clap_complete::Shell;

/// 👻 LocalGhost — Yerel AI Terminal Asistanı
///
/// Doğal dili Linux terminal komutlarına çevirir.
/// Tamamen çevrimdışı. Verileriniz asla cihazınızdan çıkmaz.
#[derive(Parser, Debug)]
#[command(
    name = "localghost",
    version = env!("CARGO_PKG_VERSION"),
    author,
    about = "👻 Yerel AI terminal asistanı — offline, privacy-first, Ollama tabanlı",
    long_about = None,
)]
pub struct Cli {
    /// Doğal dil sorgunuz
    #[arg(value_name = "QUERY", help = "Komuta dönüştürülecek açıklama")]
    pub query: Option<String>,

    /// Kullanılacak Ollama modeli
    #[arg(short = 'm', long, value_name = "MODEL", help = "Örn: qwen3:8b, gemma2:2b")]
    pub model: Option<String>,

    /// Komutu açıkla (Türkçe)
    #[arg(short = 'e', long, help = "Her flag ve argümanı açıklar")]
    pub explain: bool,

    /// Dosya bağlamı
    #[arg(short = 'f', long = "file", help = "Komut üretilirken bağlam (context) olarak okunacak dosya")]
    pub file: Option<String>,

    /// Ollama API URL
    #[arg(
        long,
        env = "LOCALGHOST_OLLAMA_URL",
        default_value = "http://127.0.0.1:11434",
        value_name = "URL"
    )]
    pub ollama_url: String,

    /// İnteraktif TUI modunu başlat
    #[arg(short = 'i', long, help = "İnteraktif TUI modunu başlat")]
    pub interactive: bool,

    /// Üretilen komutu güvenli bir şekilde çalıştırır (Execution Layer)
    #[arg(short = 'x', long, help = "Üretilen komutu çalıştır")]
    pub execute: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Mevcut Ollama modellerini listele
    Models,

    /// Varsayılan Ollama modelini interaktif olarak seç
    SelectModel,

    /// Shell entegrasyonu kur (kısayol kullanımı için)
    Install {
        #[arg(help = "Hangi shell'e kurulacak? (fish, bash, zsh)")]
        shell: Option<String>,
    },

    /// Komut geçmişini göster
    History,

    /// Tüm geçmişi sil
    ClearHistory,

    /// Algılanan ortam profilini göster (shell, alias'lar)
    Env,

    /// Ortam profilini yeniden tara
    RefreshEnv,

    /// Shell completion scripti oluştur
    GenerateCompletion {
        #[arg(value_enum)]
        shell: Shell,
    },
}

pub fn generate_completion(shell: Shell) {
    use clap::CommandFactory;
    use clap_complete::generate;
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    generate(shell, &mut cmd, name, &mut std::io::stdout());
}
