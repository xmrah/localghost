# LocalGhost CLI — Development shell for NixOS users
# This file is optional. It provides a Python 3 environment for NixOS devs.
# Non-NixOS users can ignore this file entirely.
{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.python3
  ];

  shellHook = ''
    export LC_ALL=en_US.UTF-8
    echo "👻 LocalGhost Dev Shell (NixOS)"
    echo "Python: $(python3 --version)"
  '';
}
