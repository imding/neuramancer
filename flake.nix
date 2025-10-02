{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
          "x86_64-linux"
          # "aarch64-linux"
          # "x86_64-darwin"
          # "aarch64-darwin"
      ];
      perSystem = { config, self', pkgs, lib, system, ... }:
        let
          runtimeDeps = [
              # alsa-lib
              # speechd
          ];
          buildDeps = with pkgs; [
              pkg-config
              # Uncomment if binding to C libraries
              # rustPlatform.bindgenHook
          ];
          devDeps = with pkgs; [
              # Uncomment if need for debugging
              # gdb
              cargo-binstall
              deno
              dioxus-cli
              flyctl
              git
              helix
              jujutsu
          ];

          cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
          msrv = cargoToml.package.rust-version;

          rustPackage = features:
            (pkgs.makeRustPlatform {
              cargo = pkgs.rust-bin.stable.latest.minimal;
              rustc = pkgs.rust-bin.stable.latest.minimal;
            }).buildRustPackage {
              inherit (cargoToml.package) name version;
              src = ./.;
              cargoLock.lockFile = ./Cargo.lock;
              buildFeatures = features;
              buildInputs = runtimeDeps;
              nativeBuildInputs = buildDeps;
              # Uncomment if your cargo tests require networking or otherwise
              # don't play nicely with the Nix build sandbox:
              # doCheck = false;
            };

          mkDevShell = rustc:
            pkgs.mkShell {
              shellHook = ''
                export RUST_SRC_PATH=${pkgs.rustPlatform.rustLibSrc}

                # WSL display setup
                export DISPLAY=:0
                
                # Ensure machine cargo bin is in nix shell PATH at the beginning
                # This is necessary as long as we're installing dependencies outside of nix
                export PATH="$HOME/.cargo/bin:$PATH"
                cargo binstall wasm-bindgen-cli@0.2.104 -y

                if [ -f .config/helix ]; then
                    cp -r .config/helix ~/.config/helix
                fi

                if [ -f .config/jujutsu.toml ]; then
                    mkdir -p ~/.config/jj
                    cp .config/jujutsu.toml ~/.config/jj/config.toml
                fi
              '';
              buildInputs = runtimeDeps;
              nativeBuildInputs = buildDeps ++ devDeps ++ [ rustc ];
            };
        in {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ (import inputs.rust-overlay) ];
          };

          packages.default = self'.packages.example;
          devShells.default = self'.devShells.nightly;

          packages.example = (rustPackage "foobar");
          packages.example-base = (rustPackage "");

          devShells.nightly = (mkDevShell (pkgs.rust-bin.selectLatestNightlyWith
            (toolchain: toolchain.default.override {
                targets = [ "wasm32-unknown-unknown" ];
            })));
          devShells.stable = (mkDevShell pkgs.rust-bin.stable.latest.default.override {
                targets = [ "wasm32-unknown-unknown" ];
            });
          devShells.msrv = (mkDevShell pkgs.rust-bin.stable.${msrv}.default.override {
                targets = [ "wasm32-unknown-unknown" ];
            });
        };
    };
}
