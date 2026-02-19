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

        localghost = pkgs.writeShellApplication {
          name = "localghost";
          runtimeInputs = [ pkgs.python3 pkgs.pciutils ];
          text = ''
            exec python3 ${./localghost.py} "$@"
          '';
        };
      in
      {
        # nix run github:xmrah/localghost -- "update my system"
        packages.default = localghost;
        apps.default = flake-utils.lib.mkApp { drv = localghost; };

        # nix develop — dev shell with Python
        devShells.default = pkgs.mkShell {
          buildInputs = [ pkgs.python3 pkgs.pciutils ];
          shellHook = ''
            export LC_ALL=en_US.UTF-8
            echo "👻 LocalGhost Dev Shell"
            echo "Python: $(python3 --version)"
          '';
        };
      }
    );
}
