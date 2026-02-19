# Ghost CLI - Türkçe Kullanım Kılavuzu 🇹🇷

Ghost, Linux terminalinizi yapay zeka ile güçlendiren **yerel** bir asistandır. İnternete ihtiyaç duymaz, verilerinizi dışarı göndermez ve kullandığınız dağıtıma uyum sağlar.

## Ne Yapar?

Doğal dilde yazdığınız isteği terminal komutuna çevirir:

```bash
ghost "sistemi güncelle"
# NixOS  → sudo nixos-rebuild switch --upgrade
# Arch   → sudo pacman -Syu
# Debian → sudo apt update && sudo apt upgrade
```

## Kurulum

**Gereksinimler:** Linux, Python 3, [Ollama](https://ollama.com)

```bash
# 1. İndir
git clone https://github.com/xmrah/ghost.git
cd ghost

# 2. Kur (interaktif sihirbaz)
./install.sh
```

Kurulum sihirbazı iki mod sunar:
- **Hızlı Kurulum:** Ollama kontrolü, model indirme, PATH ayarlama — tam rehberli
- **Uzman Kurulum:** Sadece symlink, kontrol yok

```bash
./install.sh --uninstall  # Temiz kaldırma
```

### NixOS Kullanıcıları (Kurulumsuz)

Hiçbir şey indirmeden veya kurmadan direkt çalıştırabilirsiniz:

```bash
# GitHub üzerinden anında çalıştır
nix run github:xmrah/ghost -- "sistemi güncelle"
```

Veya geliştirme ortamına girmek için:

```bash
git clone https://github.com/xmrah/ghost.git
cd ghost
nix develop
```

## Kullanım Örnekleri

```bash
ghost "10 MB'dan büyük mp4 dosyalarını bul"
ghost "açık portları listele"
ghost "disk kullanımını göster"
ghost "ekran kartı bilgisi"
```

## Güvenlik

Ghost, tehlikeli komutları otomatik algılar ve uyarır:

```
$ ghost "her şeyi sil"
⚠  DANGEROUS COMMAND DETECTED
   Command: rm -rf /
   Review carefully before executing.
```

**Ghost asla komut çalıştırmaz.** Sadece ekrana yazar. Çalıştırıp çalıştırmamak size kalmış.

## Gizlilik

- Tüm işlem `localhost` (127.0.0.1) üzerinde yapılır
- İnternet bağlantısı gerekmez
- Hiçbir veri kaydedilmez veya dışarı gönderilmez
- Kaynak kodu tek bir Python dosyasından ibarettir, kendiniz inceleyebilirsiniz

## Mevcut Modeller

```bash
ghost --models     # Yüklü modelleri gösterir
ghost --help       # Yardım menüsü
ghost --version    # Versiyon bilgisi
```

---

📖 [English README](README.md)
