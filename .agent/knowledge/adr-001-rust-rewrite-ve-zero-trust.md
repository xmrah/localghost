# ADR 001: Rust Rewrite ve Zero-Trust Execution Mimarisinin Kurulması

* **Tarih:** 2026-09-04
* **Durum:** Kabul Edildi

## Karar
LocalGhost projesi tamamen Python'dan Rust'a yeniden yazılmış (Rewrite) ve komut yürütme (Execution) katmanı için **GTFOBins Filtrelemeli Zero-Trust Path Sensitivity** mimarisi uygulanmıştır.

## Bağlam/Neden
1. **Bağımlılık (Dependency) Sorunu:** Python versiyonu, Nix dışındaki sistemlerde interpreter ve venv sorunları yaratıyordu. Tek bir native (Rust) binary dağıtımı (zero-dep) ile doğrudan her sistemde çalışma (offline-first) garantilenmeliydi.
2. **Güvenlik Açıkları (Blacklist Yetersizliği):**
   - Eski mimaride komutlar salt regex (kara liste) ile filtreleniyordu. Bu durum `sh -c` kullanımından doğan shell injection (`|`, `;`, `&&`, `$()`) risklerine tamamen açıktı.
   - Salt-okunur (read-only) kabul edilen araçlar (ör. `find -exec sh`, `awk system()`) GTFOBins zafiyetleri ile sisteme root erişimi sağlayabiliyordu.
   - Sadece komut ismine (binary allowlist) bakılarak `.ssh` veya `id_rsa` gibi dosyaların `cat` ile okunması engellenemiyor, veri sızıntısına (exfiltration) yol açıyordu.

## Sonuç
* **Rust Geçişi:** `clap`, `tokio`, `reqwest` ve `ratatui` (TUI için) crate'leri ile v1.0 Rust kod tabanı oluşturuldu. Proje derleme hızı ve dağıtım kolaylığı bakımından tam bağımsız oldu.
* **Tier-0 (Oto-Exec) Katmanı:**
  - `shell-words` ile komutlar metne değil `argv` dizilerine çevrilerek shell bypass (injection) tamamen durduruldu. `sh -c` ortadan kalktı (`execvp` tarzı syscall seviyesi çalışma sağlandı).
  - GTFOBins listesinde shell açabilen tüm komutlar onaylanmış (allowlist) listeden atıldı.
* **Tier-1 (Zero-Trust Path):** 
  - Geriye kalan masum komutlar için `Default-Deny` stratejisi uygulandı: İstenen yol (path) `$PWD` dışında bir mutlak yol (absolute path) ise veya üst dizine (`..`) geçiş yapıyorsa ya da gizli dosyaysa (`.`) komut anında sisteme takılır ve interaktif kullanıcı onayına (`[Y/n]`) sunulur.

## İlgili Kaynaklar
- [LocalGhost Repository](https://codeberg.org/xmrah/localghost)
