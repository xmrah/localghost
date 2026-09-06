# 👻 LocalGhost v1.2

> **Local AI Terminal Assistant for Linux**
> Doğal dili terminal komutlarına çeviren, tamamen çevrimdışı, gizlilik odaklı CLI ve TUI aracı.

LocalGhost, makinenizde yerel olarak çalışan Ollama modellerini kullanarak doğal dildeki isteklerinizi (örneğin *"sistemi güncelle"*, *"büyük log dosyalarını bul"*) güvenli ve doğru kabuk komutlarına dönüştürür. Eski Python sürümünün (v0.8) tamamen **Rust** ile yeniden yazılmış halidir.

## ✨ Özellikler

- **Tamamen Çevrimdışı:** Verileriniz asla cihazınızdan çıkmaz. Tüm işlemler yerel ağınızdaki veya makinenizdeki Ollama üzerinden yürütülür.
- **Sıfır Bağımlılık (Zero-dep):** Tek, statik derlenmiş bir Rust binary dosyasıdır. Python ortamlarına veya harici betiklere ihtiyaç duymaz.
- **Yapılandırılmış Çıktı (Structured Output):** Komut üretiminde Markdown sızıntısı veya halüsinasyon riskini yok eder. LLM sadece komut döndürmeye zorlanır.
- **TUI (İnteraktif Mod):** `localghost -i` komutuyla, komut geçmişini ve çıktıları tek ekranda görebileceğiniz Ratatui tabanlı tam ekran oturum moduna geçebilirsiniz.
- **Zero-Trust Execution Katmanı:** Üretilen komutu `-x` bayrağı ile anında güvenle çalıştırabilirsiniz. Sistem "Default-Deny" prensibiyle çalışır; Shell injection ve gizli dosyalara sızmayı engeller.
- **Shell Entegrasyonu (`??` kısayolu):** `localghost install fish` komutuyla terminalinize `??` kısayolunu ekleyin. Artık doğrudan `?? "disk kullanımını göster"` yazarak komut üretin.
- **Dosya Bağlamı (`-f`):** `localghost -f script.sh "bu scripti optimize et"` komutuyla dosya içeriğini yapay zekaya bağlam olarak verebilirsiniz. (Maks. 64KB)
- **Rol/Profil Sistemi (`--role`):** `~/.config/localghost/roles/` içine özel profiller oluşturup `--role pro` gibi kullanabilirsiniz.
- **İnteraktif Model Seçimi:** `localghost select-model` ile Ollama'daki modelleriniz arasında kolayca geçiş yapın.
- **Dağıtım ve Donanım Farkındalığı:** Hangi Linux dağıtımında olduğunuzu, paket yöneticinizi ve donanımınızı algılar; LLM'e bu bağlamı sağlar.
- **Akıllı Çevre Profili:** Sisteminizde kurulu modern CLI araçlarını (`fd`, `eza`, `bat`, `rg`) otomatik algılar.
- **Açıklama Modu (`--explain`):** Üretilen komutu parça parça Türkçe açıklar.

## 🚀 Kurulum

### NixOS / Nix Flake

```bash
nix shell github:xmrah/localghost
# veya projeyi klonladıktan sonra:
nix run .#
```

### Kaynak Koddan (Source)

```bash
cargo build --release
sudo cp target/release/localghost /usr/local/bin/
```

### Ön Gereksinimler

- [Ollama](https://ollama.ai) kurulu ve çalışıyor olmalı (`ollama serve`)
- En az bir model yüklü olmalı (örn: `ollama pull qwen2.5-coder:7b`)

## 💻 Kullanım

### Temel Kullanım
```bash
localghost "disk kullanımını göster"
localghost -m qwen2.5-coder:7b "sistemi güncelle"
```

### Komut Çalıştırma (`-x`)
Üretilen komutu güvenlik filtresinden geçirip otomatik çalıştır:
```bash
localghost -x "şu anki dizini listele"
```

### Shell Kısayolu (`??`)
Fish, Bash veya Zsh'ye `??` kısayolunu kurun:
```bash
localghost install fish    # veya: bash, zsh
```
Artık doğrudan terminalde:
```bash
?? "büyük dosyaları bul"
```

### Dosya Bağlamı (`-f`)
Bir dosyanın içeriğini yapay zekaya bağlam olarak verin:
```bash
localghost -x "bu scriptteki hataları düzelt" -f script.sh
```

### Rol/Profil Sistemi (`--role`)
Özel roller oluşturup kullanın:
```bash
# Profil oluştur:
mkdir -p ~/.config/localghost/roles
echo "Sen uzman bir NixOS yöneticisisin." > ~/.config/localghost/roles/nix.txt

# Kullan:
localghost -x "sistemi güncelle" --role nix
```

### Açıklama Modu
Komutun ne yaptığını Türkçe öğrenin:
```bash
localghost --explain "son 5 gündeki büyük dosyaları bul"
```

### İnteraktif Mod (TUI)
```bash
localghost -i
```

### Diğer Komutlar
```bash
localghost models          # Ollama'daki yüklü modelleri listeler
localghost select-model    # Varsayılan modeli interaktif seçin
localghost history         # Komut geçmişini listeler
localghost clear-history   # Geçmişi temizler
localghost env             # Algılanan ortam araçlarını gösterir
```

## ⚙️ Yapılandırma

İlk çalıştırmanın ardından `~/.config/localghost/config.toml` dosyası oluşur:

```toml
[model]
default = "qwen2.5-coder:7b"
fallback = "qwen2.5vl:7b"

[history]
ttl_days = 7
max_entries = 100

[safety]
block_dangerous = true
```

## 📜 Lisans

MIT

---
**Not:** Önceki Python versiyonuna ulaşmak için git geçmişindeki `python-v0.8.1` etiketine bakabilirsiniz.
