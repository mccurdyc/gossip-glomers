{
  description = "Repo configuration";

  # References
  # - https://ryantm.github.io/nixpkgs/languages-frameworks/rust/
  # - https://github.com/tfc/rust_async_http_code_experiment/blob/master/flake.nix

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
          # v = "1.80.1";
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

          maelstromDeps = with pkgs; [
            # breakpointHook # debugging
            jdk22_headless
            gnuplot
            git # not sure why maelstrom needs this
          ];
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
              version = "1.0.0";
              src = pkgs.lib.cleanSource ./.; # the folder with the cargo.toml
              cargoLock.lockFile = ./Cargo.lock;
              cargoBuildFlags = [ "--bin" "echo" ];
              doCheck = false; # disable so that these can be built independently
              # https://nixos.org/manual/nixpkgs/stable/#ssec-installCheck-phase
              doInstallCheck = true; # disable so that these can be built independently
              nativeInstallCheckInputs = maelstromDeps;
              installCheckPhase = ''
                echo "===> running 'maelstrom echo' tests"
                java -Djava.awt.headless=true -jar "./maelstrom.jar" test -w echo --bin $out/bin/echo --node-count 1 --time-limit 10
              '';
            };

            # nix build '.#unique'
            # nix run '.#unique'
            unique = rustPlatform.buildRustPackage {
              pname = "unique";
              version = "1.0.0";
              src = pkgs.lib.cleanSource ./.; # the folder with the cargo.toml
              cargoLock.lockFile = ./Cargo.lock;
              cargoBuildFlags = [ "--bin" "unique" ];
              doCheck = false; # disable so that these can be built independently
              # https://nixos.org/manual/nixpkgs/stable/#ssec-installCheck-phase
              doInstallCheck = true; # disable so that these can be built independently
              nativeInstallCheckInputs = maelstromDeps;
              installCheckPhase = ''
                echo "===> running 'maelstrom unique-id' tests"
                java -Djava.awt.headless=true -jar "./maelstrom.jar" test -w unique-ids --bin $out/bin/unique --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition
              '';
            };

            # nix build '.#broadcast'
            # nix run '.#broadcast'
            broadcast = rustPlatform.buildRustPackage {
              pname = "broadcast";
              version = "1.0.0";
              src = pkgs.lib.cleanSource ./.; # the folder with the cargo.toml
              cargoLock.lockFile = ./Cargo.lock;
              cargoBuildFlags = [ "--bin" "broadcast" ];
              doCheck = false; # disable so that these can be built independently
              # https://nixos.org/manual/nixpkgs/stable/#ssec-installCheck-phase
              doInstallCheck = true; # disable so that these can be built independently
              nativeInstallCheckInputs = maelstromDeps;
              installCheckPhase = ''
                echo "===> running 'maelstrom broadcast' tests"
                java -Djava.awt.headless=true -jar "./maelstrom.jar" test -w broadcast --bin $out/bin/broadcast --node-count 1 --time-limit 20 --rate 10
              '';
            };

            # nix build '.#counter'
            # nix run '.#counter'
            counter = rustPlatform.buildRustPackage {
              pname = "counter";
              version = "1.0.0";
              src = pkgs.lib.cleanSource ./.; # the folder with the cargo.toml
              cargoLock.lockFile = ./Cargo.lock;
              cargoBuildFlags = [ "--bin" "counter" ];
              doCheck = false; # disable so that these can be built independently
              # https://nixos.org/manual/nixpkgs/stable/#ssec-installCheck-phase
              doInstallCheck = true; # disable so that these can be built independently
              nativeInstallCheckInputs = maelstromDeps;
              # https://nixos.org/manual/nixpkgs/stable/#ssec-installCheck-phase
              installCheckPhase = ''
                echo "===> running 'maelstrom counter' tests"
                java -Djava.awt.headless=true -jar "./maelstrom.jar" test -w g-counter --bin $out/bin/counter --node-count 3 --time-limit 20 --rate 100 --nemesis partition
              '';
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
