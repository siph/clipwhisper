{
  description = "Quick and easy clip extractor.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs@{ flake-parts, fenix, crane, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "aarch64-linux"
        "x86_64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];

      perSystem = { pkgs, lib, system, self', ... }:
        let
          toolchain = fenix.packages.${system}.default.toolchain;

          craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

          src = lib.cleanSourceWith {
            src = craneLib.path ./.;
            filter = (path: type: craneLib.filterCargoSources path type);
          };

          commonArgs = with pkgs; {
            inherit src;
            nativeBuildInputs = [ pkg-config ];
            buildInputs = [ openssl ffmpeg ];
          };

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          clipwhisper = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
            doCheck = false;
          });

        in
        {

          packages = {

            clipwhisper = craneLib.buildPackage (commonArgs // {
              inherit cargoArtifacts;
              name = "clipwhisper";
              doCheck = false;
            });

          };

          checks = {
            inherit clipwhisper;

            clippy = craneLib.cargoClippy (commonArgs // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "-- -D warnings";
            });

            cargo-fmt = craneLib.cargoFmt { inherit src; };

            nextest = craneLib.cargoNextest (commonArgs // { inherit cargoArtifacts; });

            pre-commit-check = inputs.pre-commit-hooks.lib.${system}.run {
              src = ./.;
              hooks = {
                rustfmt.enable = true;
                nixpkgs-fmt.enable = true;
              };
            };
          };

          formatter = pkgs.nixpkgs-fmt;

          devShells = with pkgs; {
            default = mkShell {
              inherit (self'.checks.pre-commit-check) shellHook;
              inherit (commonArgs) nativeBuildInputs;
              buildInputs = (commonArgs.buildInputs ++ [ toolchain ]);
            };
          };
        };
    };
}
