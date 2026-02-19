{
  description = "Ghost CLI — Local AI Terminal Assistant for Linux";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        ghost = pkgs.writeShellApplication {
          name = "ghost";
          runtimeInputs = [ pkgs.python3 pkgs.pciutils ];
          text = ''
            exec python3 ${./ghost.py} "$@"
          '';
        };
      in
      {
        # nix run github:xmrah/ghost -- "update my system"
        packages.default = ghost;
        apps.default = flake-utils.lib.mkApp { drv = ghost; };

        # nix develop — dev shell with Python
        devShells.default = pkgs.mkShell {
          buildInputs = [ pkgs.python3 pkgs.pciutils ];
          shellHook = ''
            export LC_ALL=en_US.UTF-8
            echo "👻 Ghost Dev Shell"
            echo "Python: $(python3 --version)"
          '';
        };
      }
    );
}
