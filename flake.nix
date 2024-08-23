{
  description = "Repo configuration";

  # References
  # - https://ryantm.github.io/nixpkgs/languages-frameworks/rust/

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.05";
    nixpkgs-unstable.url = "github:nixos/nixpkgs/nixos-unstable";
    pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";
    flake-parts.url = "github:hercules-ci/flake-parts";
    # Why do you need this?
    # - My understanding is for a pure binary installation of the Rust toolchain
    # as opposed to ... (IDK).
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
          v = "1.79.0";
          # v = "latest";
          rustChannel = "stable";
          # rustChannel = nightly
          # rustChannel = beta
          rustVersion = pkgs.rust-bin.${rustChannel}.${v}.default.override {
            extensions = [ "rust-src" "rust-analyzer" ];
          };

          rustPlatform = pkgs.makeRustPlatform {
            cargo = rustVersion;
            rustc = rustVersion;
          };
        in
        {
          # This is needed for pkgs-unstable - https://github.com/hercules-ci/flake-parts/discussions/105
          overlayAttrs = { inherit pkgs-unstable; };

          formatter = pkgs.nixpkgs-fmt;

          # https://github.com/cachix/git-hooks.nix
          # 'nix flake check'
          checks = {
            pre-commit-check = inputs.pre-commit-hooks.lib.${system}.run {
              src = ./.;
              hooks = {
                # Nix
                deadnix.enable = true;
                nixpkgs-fmt.enable = true;
                statix.enable = true;

                # Rust
                rustfmt.enable = true;
                cargo-check.enable = true;
                cargo-test.enable = true;
                # clippy.enable = true;

                # Shell
                shellcheck.enable = true;
                shfmt.enable = true;
              };
            };
          };

          # nix build
          packages.default = rustPlatform.buildRustPackage {
            pname = "app";
            version = "0.1.0";
            src = ./.; # the folder with the cargo.toml

            cargoLock.lockFile = ./Cargo.lock;
          };

          devShells.default = pkgs.mkShell {
            inherit (self.checks.${system}.pre-commit-check) shellHook;
            nativeBuildInputs = with pkgs; [ rustc cargo gcc rustfmt clippy ];
            buildInputs = self.checks.${system}.pre-commit-check.enabledPackages;

            RUST_SRC_PATH = "${pkgs.rust.packages.${rustChannel}.rustPlatform.rustLibSrc}";

            # https://github.com/NixOS/nixpkgs/blob/736142a5ae59df3a7fc5137669271183d8d521fd/doc/build-helpers/special/mkshell.section.md?plain=1#L1
            packages = [
              pkgs.just
              pkgs.statix
              pkgs.nixpkgs-fmt
              pkgs-unstable.nil

              # Rust
              pkgs.openssl
              pkgs.rust-analyzer
              pkgs.rustPackages.clippy

              # Maelstrom
              pkgs.jdk22_headless
              pkgs.gnuplot
            ];
          };
        };
    };
}
