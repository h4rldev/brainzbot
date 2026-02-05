{pkgs ? import <nixpkgs> {}}:
with pkgs;
  mkShell {
    name = "brainzbot";
    description = "A discord bot for listenbrainz";
    buildInputs = [
    ];
  }
