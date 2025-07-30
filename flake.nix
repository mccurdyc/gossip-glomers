{
  # References
  # - https://ryantm.github.io/nixpkgs/languages-frameworks/rust/
  # - https://github.com/tfc/rust_async_http_code_experiment/blob/master/flake.nix

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
    nixpkgs-unstable.url = "github:nixos/nixpkgs/nixos-unstable";
    pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";
    flake-utils.url = "github:numtide/flake-utils";

    # For a pure binary installation of the Rust toolchain
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs@{ self, nixpkgs, nixpkgs-unstable, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (system:
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
          rustfmt = pinnedRust;
          clippy = pinnedRust;
        };

        maelstromDeps = with pkgs; [
          breakpointHook # debugging - https://discourse.nixos.org/t/debug-a-failed-derivation-with-breakpointhook-and-cntr/8669
          jdk23_headless
          gnuplot
          git # not sure why maelstrom needs this
        ];

        inherit (self.packages.${system}) echo unique broadcast counter replicated-log;

        ci_packages = {
          # Nix
          nix-fmt = pkgs.nixpkgs-fmt;

          inherit (pkgs) cargo rustc clippy rustfmt jdk23_headless gnuplot git; # is there a way to DRY this up? Could list flatten, etc.

          inherit (pkgs-unstable) just; # need just >1.33 for working-directory setting
        };
      in
      {
        # This is needed for pkgs-unstable - https://github.com/hercules-ci/flake-parts/discussions/105
        overlayAttrs = {
          inherit pkgs-unstable overlays;
        };

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

          # https://github.com/NixOS/nix/issues/8881
          # nix build '.#checks.x86_64-linux.echo' --print-build-logs --keep-failed
          # --keep-failed writes the sandbox directory at /tmp/nix-build-.../build/<hash>-source/
          # We use `nix build` instead of `nix run` because the check doesn't produce an executable to run.
          # We use mkDerivation instead of runCommand because we need to set `src`.
          echo = pkgs.stdenvNoCC.mkDerivation {
            name = "maelstrom-echo";
            src = ./.;
            nativeBuildInputs = maelstromDeps ++ [ echo ];
            buildPhase = ''
              echo "===> running 'maelstrom echo' tests"
              java -Djava.awt.headless=true -jar "./maelstrom.jar" test -w echo --bin ${echo}/bin/echo --node-count 1 --time-limit 10
              mkdir -p $out # required by derivations even though it's empty
            '';
          };

          # https://github.com/NixOS/nix/issues/8881
          # nix build '.#checks.x86_64-linux.unique' --print-build-logs --keep-failed
          # --keep-failed writes the sandbox directory at /tmp/nix-build-.../build/<hash>-source/
          # We use `nix build` instead of `nix run` because the check doesn't produce an executable to run.
          # We use mkDerivation instead of runCommand because we need to set `src`.
          unique = pkgs.stdenvNoCC.mkDerivation {
            name = "maelstrom-unique";
            src = ./.;
            nativeBuildInputs = maelstromDeps ++ [ unique ];
            buildPhase = ''
              echo "===> running 'maelstrom unique' tests"
              java -Djava.awt.headless=true -jar "./maelstrom.jar" test -w unique-ids --bin ${unique}/bin/unique --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition
              mkdir -p $out # required by derivations even though it's empty
            '';
          };

          # https://github.com/NixOS/nix/issues/8881
          # nix build '.#checks.x86_64-linux.broadcast' --print-build-logs --keep-failed
          # --keep-failed writes the sandbox directory at /tmp/nix-build-.../build/<hash>-source/
          # We use `nix build` instead of `nix run` because the check doesn't produce an executable to run.
          # We use mkDerivation instead of runCommand because we need to set `src`.
          broadcast = pkgs.stdenvNoCC.mkDerivation {
            name = "maelstrom-broadcast";
            src = ./.;
            nativeBuildInputs = maelstromDeps ++ [ broadcast ];
            buildPhase = ''
              echo "===> running 'maelstrom broadcast' tests"
              java -Djava.awt.headless=true -jar "./maelstrom.jar" test -w broadcast --bin ${broadcast}/bin/broadcast --node-count 1 --time-limit 20 --rate 10
              mkdir -p $out # required by derivations even though it's empty
            '';
          };

          # https://github.com/NixOS/nix/issues/8881
          # nix build '.#checks.x86_64-linux.counter' --print-build-logs --keep-failed
          # --keep-failed writes the sandbox directory at /tmp/nix-build-.../build/<hash>-source/
          # We use `nix build` instead of `nix run` because the check doesn't produce an executable to run.
          # We use mkDerivation instead of runCommand because we need to set `src`.
          counter = pkgs.stdenvNoCC.mkDerivation {
            name = "maelstrom-counter";
            src = ./.;
            nativeBuildInputs = maelstromDeps ++ [ counter ];
            buildPhase = ''
              echo "===> running 'maelstrom counter' tests"
              java -Djava.awt.headless=true -jar "./maelstrom.jar" test -w g-counter --bin ${counter}/bin/counter --node-count 3 --time-limit 20 --rate 100 --nemesis partition
              mkdir -p $out # required by derivations even though it's empty
            '';
          };

          # https://github.com/NixOS/nix/issues/8881
          # nix build '.#checks.x86_64-linux.replicated-log' --print-build-logs --keep-failed
          # --keep-failed writes the sandbox directory at /tmp/nix-build-.../build/<hash>-source/
          # We use `nix build` instead of `nix run` because the check doesn't produce an executable to run.
          # We use mkDerivation instead of runCommand because we need to set `src`.
          replicated-log = pkgs.stdenvNoCC.mkDerivation {
            name = "maelstrom-replicated-log";
            src = ./.;
            nativeBuildInputs = maelstromDeps ++ [ replicated-log ];
            buildPhase = ''
              echo "===> running 'maelstrom replicated-log' tests"
              java -Djava.awt.headless=true -jar "./maelstrom.jar" test -w kafka --bin ${replicated-log}/bin/replicated-log --node-count 1 --concurrency 2n --time-limit 20 --rate 1000
              mkdir -p $out # required by derivations even though it's empty
            '';
          };
        };

        packages = {
          # Proxy these to be able to be version in the docker image
          inherit (ci_packages) just cargo rustc clippy rustfmt nix-fmt jdk23_headless gnuplot git; # not sure why maelstrom needs git

          # nix build '.#echo'
          # nix run '.#echo'
          echo = rustPlatform.buildRustPackage {
            pname = "echo";
            version = "1.0.0";
            src = pkgs.lib.cleanSource ./.; # the folder with the cargo.toml
            cargoLock.lockFile = ./Cargo.lock;
            cargoBuildFlags = [ "--bin" "echo" ];
            doCheck = false; # disable so that these can be built independently
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
            doInstallCheck = false; # disable so that these can be built independently
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
            doInstallCheck = false; # disable so that these can be built independently
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
            doInstallCheck = false; # disable so that these can be built independently
          };

          # nix build '.#replicated-log'
          # nix run '.#replicated-log'
          replicated-log = rustPlatform.buildRustPackage {
            pname = "replicated-log";
            version = "1.0.0";
            src = pkgs.lib.cleanSource ./.; # the folder with the cargo.toml
            cargoLock.lockFile = ./Cargo.lock;
            cargoBuildFlags = [ "--bin" "replicated-log" ];
            doCheck = false; # disable so that these can be built independently
            # https://nixos.org/manual/nixpkgs/stable/#ssec-installCheck-phase
            doInstallCheck = false; # disable so that these can be built independently
          };
        };

        devShells.default = pkgs.mkShell {
          shellHook = (self.checks.${system}.pre-commit-check).shellHook +
            ''
            '';
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
            pkgs.cntr

            # Rust
            # NOTES:
            # - Be careful defining rust tools (e.g., clippy) here because
            # you need to guarantee they use the same Tust version as defined
            # in rustVersion.
            pkgs.openssl
            pkgs.rust-analyzer

            # Maelstrom
            pkgs.jdk23_headless
            pkgs.gnuplot
          ];
        };
      }
    );
}
