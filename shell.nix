{pkgs ? import <nixpkgs> {}}:
with pkgs;
  mkShell {
    # Will be turned into a flake whenever it's ready for that.
    name = "brainzbot";
    description = "A discord bot for listenbrainz";
    buildInputs = [
      # reserved for future deps
    ];

    packages = [
      just
      valkey

      taplo # yaml formatter
    ];
  }
