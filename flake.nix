{
  description = "LocalGhost CLI — Local AI Terminal Assistant for Linux";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        locallocalghost = pkgs.writeShellApplication {
          name = "locallocalghost";
          runtimeInputs = [ pkgs.python3 pkgs.pciutils ];
          text = ''
            exec python3 ${./locallocalghost.py} "$@"
          '';
        };
      in
      {
        # nix run github:xmrah/localghost -- "update my system"
        packages.default = locallocalghost;
        apps.default = flake-utils.lib.mkApp { drv = locallocalghost; };

        # nix develop — dev shell with Python
        devShells.default = pkgs.mkShell {
          buildInputs = [ pkgs.python3 pkgs.pciutils ];
          shellHook = ''
            export LC_ALL=en_US.UTF-8
            echo "👻 LocalLocalGhost Dev Shell"
            echo "Python: $(python3 --version)"
          '';
        };
      }
    );
}
