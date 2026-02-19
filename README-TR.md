# LocalGhost CLI 👻

> **Linux için Yerel Yapay Zeka Terminal Asistanı**
> *Sıfır bağımlılık. %100 Gizlilik. Hibrit & Dağıtım Bağımsız.*

**LocalGhost**, doğal dilde yazdığınız istekleri Linux terminal komutlarına dönüştüren, [Ollama](https://ollama.com) tabanlı hafif bir CLI aracıdır.

**Güvenli, şeffaf ve dağıtım bağımsız** olacak şekilde tasarlanmıştır. NixOS, Arch, Debian, Fedora ve diğerlerinde sorunsuz çalışır.

## Özellikler

- **🔒 Önce Gizlilik:** Tamamen yerel çalışır (offline). Verileriniz asla cihazınızdan çıkmaz.
- **🛡️ Güvenli:** `rm -rf` gibi yıkıcı komutları engelleyen sıkı bir güvenlik filtresi vardır.
- **🐧 Hibrit:** Dağıtımınızı (NixOS, Arch vb.) ve donanımınızı (AMD, NVIDIA) otomatik algılar.
- **⚡ Hızlı:** Saf Python ile yazılmıştır. `pip install` gerektirmez.

## Kurulum

**Gereksinimler:** Linux, Python 3, [Ollama](https://ollama.com)

```bash
# 1. İndir
git clone https://github.com/xmrah/localghost.git
cd localghost

# 2. Kur (interaktif sihirbaz)
./install.sh
```

Kurulum sihirbazı iki mod sunar:
- **Hızlı Kurulum:** Ollama kontrolü, model indirme, PATH ayarlama — tam rehberli
- **Uzman Kurulum:** Sadece symlink, kontrol yok

```bash
./install.sh --dry-run    # Ne yapacağını önceden gör
./install.sh --uninstall  # Temiz kaldırma
```

### NixOS Kullanıcıları (Kurulumsuz)

Hiçbir şey indirmeden veya kurmadan direkt çalıştırabilirsiniz:

```bash
# GitHub üzerinden anında çalıştır
nix run github:xmrah/localghost -- "sistemi güncelle"
```

Veya geliştirme ortamına girmek için:

```bash
git clone https://github.com/xmrah/localghost.git
cd localghost
nix develop
```

## Kullanım Örnekleri

```bash
# Sistem güncelleme
localghost "sistemi güncelle"

# Dosya bulma
localghost "500MB'dan büyük dosyaları bul"

# Video sıkıştırma
localghost "video.mp4 dosyasını 720p olarak sıkıştır"

# Donanım bilgisi
localghost "ekran kartı sıcaklığını göster"
```

## Güvenlik

LocalGhost **sadece komutu ekrana yazar**, asla otomatik çalıştırmaz. Yine de modelin ürettiği komutları çalıştırmadan önce gözden geçirmeniz önerilir.

- Tüm işlem `localhost` (127.0.0.1) üzerinde yapılır
- İnternet bağlantısı gerekmez
- Hiçbir veri kaydedilmez veya dışarı gönderilmez
- Kaynak kodu tek bir Python dosyasından ibarettir, kendiniz inceleyebilirsiniz

## CLI Komutları

```bash
localghost --models          # Yüklü modelleri gösterir
localghost --help            # Yardım menüsü
localghost --version         # Versiyon bilgisi
localghost --env             # Ortam profilini göster
localghost --history         # Komut geçmişini göster
localghost --clear-history   # Geçmişi temizle
```

---

MIT Lisansı © [xmrah](https://github.com/xmrah)

📖 [English README](README.md)
