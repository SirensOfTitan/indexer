{
  description = "A service for publishing json and pdf files from lab processors";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    nixpkgs-master.url = "github:nixos/nixpkgs/master";
    flake-parts.url = "github:hercules-ci/flake-parts";
    devshell.url = "github:numtide/devshell";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      nixpkgs-master,
      flake-parts,
      devshell,
      fenix,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = nixpkgs.lib.systems.flakeExposed;
      perSystem =
        {
          self',
          pkgs,
          system,
          config,
          ...
        }:
        {
          # Per: https://flake.parts/overlays.html#consuming-an-overlay
          # An overlay must be consumed this way, since there isn't an endorsed
          # way to initialize the pkgs argument.
          _module.args.pkgs = import nixpkgs {
            inherit system;
            overlays = [
              fenix.overlays.default
              devshell.overlays.default
              (final: prev: { master = import nixpkgs-master { system = prev.system; }; })
            ];
          };

          devShells.default = pkgs.devshell.mkShell (
            { config, ... }:
            {

              packages = with pkgs; [
                (fenix.packages.${system}.complete.withComponents [
                  "cargo"
                  "clippy"
                  "rust-src"
                  "rustc"
                  "rustfmt"
                ])
                rust-analyzer-nightly
              ];
            }
          );
        };
    };
}
