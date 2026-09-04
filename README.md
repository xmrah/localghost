# 👻 LocalGhost v1.0

> **Local AI Terminal Assistant for Linux**
> Doğal dili terminal komutlarına çeviren, tamamen çevrimdışı, gizlilik odaklı CLI ve TUI aracı.

LocalGhost, makinenizde yerel olarak çalışan Ollama modellerini kullanarak doğal dildeki isteklerinizi (örneğin *"sistemi güncelle"*, *"büyük log dosyalarını bul"*) güvenli ve doğru kabuk komutlarına dönüştürür. Eski Python sürümünün (v0.8) tamamen **Rust** ile yeniden yazılmış halidir.

## ✨ Özellikler

- **Tamamen Çevrimdışı:** Verileriniz asla cihazınızdan çıkmaz. Tüm işlemler yerel ağınızdaki veya makinenizdeki Ollama üzerinden yürütülür.
- **Sıfır Bağımlılık (Zero-dep):** Tek, statik derlenmiş bir Rust binary dosyasıdır. Python ortamlarına veya harici betiklere ihtiyaç duymaz.
- **Yapılandırılmış Çıktı (Structured Output):** Komut üretiminde Markdown sızıntısı veya halüsinasyon riskini yok eder. LLM sadece komut döndürmeye zorlanır.
- **TUI (İnteraktif Mod):** `localghost -i` komutuyla, komut geçmişini ve çıktıları tek ekranda görebileceğiniz Ratatui tabanlı tam ekran oturum moduna geçebilirsiniz.
- **Güvenlik Filtresi:** Kötü niyetli veya yıkıcı komutları (`rm -rf /` vb.) çalıştırmadan önce yakalayan genişletilmiş bir regex güvenlik duvarı barındırır.
- **Dağıtım (Distro) ve Donanım Farkındalığı:** Hangi Linux dağıtımında olduğunuzu, paket yöneticinizi (`nix`, `pacman`, `apt` vb.) ve donanımınızı (AMD/Nvidia/Intel) algılar; LLM'e bu bağlamı sağlar.
- **Akıllı Çevre Profili:** Sisteminizde kurulu modern CLI araçlarını (`fd`, `eza`, `bat`, `rg`) otomatik algılar ve üretilen komutları bunlara göre iyileştirir.
- **Açıklama Modu (`--explain`):** Üretilen komutu parça parça Türkçe açıklar.

## 🚀 Kurulum

NixOS kullanıcıları `flake.nix` üzerinden projeyi anında çalıştırabilir:

```bash
nix shell github:xmrah/localghost
# veya projeyi klonladıktan sonra:
nix run .#
```

Eğer kaynak koddan (Source) derleyecekseniz:
```bash
cargo build --release
sudo cp target/release/localghost /usr/local/bin/
```

## 💻 Kullanım

Sıradan bir komut istemek için:
```bash
localghost "disk kullanımını göster"
```

Spesifik bir model (örneğin `qwen2.5-coder:7b`) kullanarak komut istemek için:
```bash
localghost -m qwen2.5-coder:7b "sistemi güncelle"
```

### `--explain` (Açıklama Modu)
Komutun ne işe yaradığını flag'leri ile birlikte detaylı görmek için:
```bash
localghost --explain "son 5 gündeki büyük dosyaları bul"
```

### İnteraktif Mod (TUI)
Terminalinizde tam ekran bir asistan oturumu açmak için:
```bash
localghost -i
# veya
localghost interactive
```

### Diğer Komutlar
```bash
localghost env            # Sisteminizde algılanan ortam araçlarını ve shell'i gösterir
localghost models         # Ollama'daki yüklü modelleri listeler
localghost history        # Komut geçmişini listeler
```

## ⚙️ Yapılandırma
İlk çalıştırmanın ardından `~/.config/localghost/config.toml` dosyası oluşur. Buradan varsayılan modelinizi ve komut geçmişi TTL (yaşam süresi) ayarlarınızı değiştirebilirsiniz.

```toml
[model]
default = "qwen2.5-coder:7b"
fallback = "qwen2.5vl:7b"
```

---
**Not:** Önceki Python versiyonuna ulaşmak için git geçmişindeki `python-v0.8.1` etiketine (tag) bakabilirsiniz.
