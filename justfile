set allow-duplicate-recipes := true

# Varsayılan görev: Tüm görevleri listele
default:
    @just --list

# Projeyi derle (debug)
build:
    cargo build

# Projeyi derle (release)
release:
    cargo build --release

# Projeyi interaktif modda çalıştır (TUI)
run: build
    ./target/debug/localghost -i

# Projeyi komut satırı argümanı ile çalıştır (Örn: just run-cmd "sistemi güncelle")
run-cmd +args: build
    ./target/debug/localghost {{args}}

# Kodu formatla
fmt:
    cargo fmt

# Linter çalıştır (clippy)
lint:
    cargo clippy --all-targets --all-features

# Temizlik yap
clean:
    cargo clean

# Nix üzerinden build et
nix-build:
    nix build .#

# Hızlıca Git Push yap
push:
    git push
