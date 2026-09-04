{
  description = "LocalGhost — Local AI Terminal Assistant for Linux (Rust)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        localghost = pkgs.rustPlatform.buildRustPackage {
          pname = "localghost";
          version = "1.0.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs; [
            openssl
            pciutils
          ];

          meta = {
            description = "Local AI terminal assistant for Linux — privacy-first, Ollama-powered";
            homepage = "https://codeberg.org/xmrah/localghost";
            license = pkgs.lib.licenses.mit;
            mainProgram = "localghost";
          };
        };
      in
      {
        # nix run codeberg:xmrah/localghost -- "sistemi güncelle"
        packages.default = localghost;
        apps.default = flake-utils.lib.mkApp { drv = localghost; };

        # nix develop — Rust geliştirme ortamı
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            rust-analyzer
            clippy
            rustfmt
            pkg-config
            openssl
            pciutils
            gcc
          ];

          shellHook = ''
            export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig"
            echo "👻 LocalGhost Dev Shell (Rust)"
            echo "Rust: $(rustc --version)"
            echo "Cargo: $(cargo --version)"
            echo ""
            echo "Komutlar:"
            echo "  cargo build          — debug build"
            echo "  cargo build --release — release build"
            echo "  cargo run -- \"sorgu\" — çalıştır"
            echo "  cargo clippy         — lint"
          '';
        };
      }
    );
}
