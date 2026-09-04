# LocalGhost - Proje Durum Özeti

**LocalGhost**, tamamen yerel çalışan (Ollama tabanlı) ve gizlilik odaklı bir Linux terminal asistanıdır. Doğal dildeki istekleri işletim sistemi ve donanım farkındalığına sahip shell komutlarına çevirir.

## 🛠️ Son Yapılan Değişiklikler (01 Mayıs 2026)

*   **Prompt (Sistem Yönergesi) Optimizasyonu:** Modelin markdown ("```"), numara ve gevezelik (açıklayıcı metin) üretmesi `temperature=0.0` ayarı ve sıkı prompt kuralları ile tamamen engellendi.
*   **Modern NixOS Uyumluluğu:** Eski `nix-env` alışkanlıkları koddan temizlendi. Dağıtım algılama motoru geliştirildi. Artık NixOS üzerindeyken `apt` önermez; doğrudan `nixos-rebuild switch --flake .#` veya `nix profile` komutlarına yönelir.
*   **Güvenlik:** `rm -rf /` gibi zararlı komut engelleme (Safety Filter) mekanizması korunmaya devam ediyor.

## 🚀 Bekleyen/Olası Görevler

*   **Varsayılan Model Kontrolü:** Şu an `gemma3:latest` varsayılan görünüyor ancak bulunamazsa listedeki ilk modele (`qwen2.5-coder` vb.) düşüyor. Güçlü donanım (RX 7700 XT) ile uyumlu standart bir model (ör. `deepseek-r1:8b` veya `qwen2.5-coder:7b`) varsayılan yapılabilir.
*   **Kurulum Betiği İyileştirmesi:** `install.sh` ve `flake.nix` tarafında güncel Nix Flake standartlarına uygun iyileştirmeler yapılabilir.
