{ config, pkgs, ... }:
{
  inherit (config.flake-root) projectRootFile;
  package = pkgs.treefmt;
  settings = {
    global.excludes = [
      ".direnv"
      ".envrc"
      "Cargo.lock"
      "treefmt.toml"
    ];
    formatter = {
      "justfile" = {
        command = "${pkgs.just}/bin/just";
        options = [
          "--unstable"
          "--fmt"
          "--justfile"
        ];
        includes = [ "justfile" ];
      };
    };
  };
  programs = {
    nixfmt.enable = true;
    mdformat.enable = true;
    yamlfmt.enable = true;
    taplo.enable = true;
    rustfmt.enable = true;
  };
}
