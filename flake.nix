{
  description = "Repo configuration";

  # References
  # - https://ryantm.github.io/nixpkgs/languages-frameworks/rust/

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.05";
    nixpkgs-unstable.url = "github:nixos/nixpkgs/nixos-unstable";
    pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";
    flake-parts.url = "github:hercules-ci/flake-parts";
    # For a pure binary installation of the Rust toolchain
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs@{ self, nixpkgs, nixpkgs-unstable, flake-parts, rust-overlay, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      flake = { };

      systems = [
        "aarch64-darwin"
        "x86_64-darwin"
        "x86_64-linux"
      ];

      # This is needed for pkgs-unstable - https://github.com/hercules-ci/flake-parts/discussions/105
      imports = [ inputs.flake-parts.flakeModules.easyOverlay ];

      perSystem = { system, ... }:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import inputs.nixpkgs {
            inherit system overlays;
            config.allowUnfree = true;
          };
          pkgs-unstable = import inputs.nixpkgs-unstable {
            inherit system overlays;
            config.allowUnfree = true;
          };
          # v = "1.77.2";
          v = "latest";
          rustChannel = "stable";
          # rustChannel = nightly
          # rustChannel = beta
          pinnedRust = pkgs.rust-bin.${rustChannel}.${v}.default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
            ];
          };

          # Used for 'nix build'
          rustPlatform = pkgs.makeRustPlatform {
            cargo = pinnedRust;
            rustc = pinnedRust;
          };
        in
        {
          # This is needed for pkgs-unstable - https://github.com/hercules-ci/flake-parts/discussions/105
          overlayAttrs = { inherit pkgs-unstable overlays; };

          formatter = pkgs.nixpkgs-fmt;

          # https://github.com/cachix/git-hooks.nix
          # 'nix flake check'
          checks = {
            # 'pre-commit run' to test directly
            pre-commit-check = inputs.pre-commit-hooks.lib.${system}.run {
              src = ./.;
              hooks = {
                # Project
                just-test = {
                  enable = true;
                  name = "just-test";
                  entry = "just test";
                  stages = [ "pre-commit" ];
                  pass_filenames = false;
                };

                just-lint = {
                  enable = true;
                  name = "just-lint";
                  entry = "just lint";
                  stages = [ "pre-commit" ];
                  pass_filenames = false;
                };

                # Nix
                deadnix.enable = true;
                nixpkgs-fmt.enable = true;
                statix.enable = true;

                # Rust
                rustfmt.enable = true;
                cargo-check.enable = true;

                # Shell
                shellcheck.enable = true;
                shfmt.enable = true;
              };
            };
          };

          packages = {
            # nix build '.#echo'
            # nix run '.#echo'
            echo = rustPlatform.buildRustPackage {
              pname = "echo";
              version = "0.1.0";
              src = ./.; # the folder with the cargo.toml
              cargoLock.lockFile = ./Cargo.lock;
            };

            # nix build '.#unique'
            # nix run '.#unique'
            unique = rustPlatform.buildRustPackage {
              pname = "unique";
              version = "0.1.0";
              src = ./.; # the folder with the cargo.toml
              cargoLock.lockFile = ./Cargo.lock;
            };
          };

          devShells.default = pkgs.mkShell {
            inherit (self.checks.${system}.pre-commit-check) shellHook;
            nativeBuildInputs = with pkgs; [
              gcc
            ];
            buildInputs = with pkgs; [
              pinnedRust
            ] ++ self.checks.${system}.pre-commit-check.enabledPackages;

            # https://github.com/NixOS/nixpkgs/blob/736142a5ae59df3a7fc5137669271183d8d521fd/doc/build-helpers/special/mkshell.section.md?plain=1#L1
            packages = [
              pkgs.just
              pkgs.statix
              pkgs.nixpkgs-fmt
              pkgs-unstable.nil
              pkgs.hadolint
              pkgs.dockerfile-language-server-nodejs
              pkgs.dive

              # Rust
              # NOTES:
              # - Be careful defining rust tools (e.g., clippy) here because
              # you need to guarantee they use the same Tust version as defined
              # in rustVersion.
              pkgs.openssl
              pkgs.rust-analyzer

              # Maelstrom
              pkgs.jdk22_headless
              pkgs.gnuplot
            ];
          };
        };
    };
}
