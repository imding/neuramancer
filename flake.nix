{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        # "aarch64-linux"
        # "x86_64-darwin"
        # "aarch64-darwin"
      ];
      perSystem =
        {
          config,
          self',
          pkgs,
          lib,
          system,
          ...
        }:
        let
          wasm-bindgen-cli_0_2_104 = pkgs.stdenv.mkDerivation {
            pname = "wasm-bindgen-cli";
            version = "0.2.104";

            src = pkgs.fetchurl {
              url = "https://github.com/rustwasm/wasm-bindgen/releases/download/0.2.104/wasm-bindgen-0.2.104-x86_64-unknown-linux-musl.tar.gz";
              sha256 = "sha256-lVN0CQfCwQCPmkS7AO0fWzn/xV2dMxWBta68iRpLdy8=";
            };

            nativeBuildInputs = [ pkgs.autoPatchelfHook ];

            installPhase = ''
              mkdir -p $out/bin
              cp wasm-bindgen $out/bin/
              cp wasm-bindgen-test-runner $out/bin/
              cp wasm2es6js $out/bin/
              chmod +x $out/bin/*
            '';
          };

          rustTargets = [ "wasm32-unknown-unknown" ];
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
            # deno
            dioxus-cli
            flyctl
            git
            helix
            jujutsu
            wasm-bindgen-cli_0_2_104
          ];

          mkDevShell =
            rustc:
            pkgs.mkShell {
              shellHook = ''
                export RUST_SRC_PATH=${pkgs.rustPlatform.rustLibSrc}
                export DISPLAY=:0
              '';
              buildInputs = runtimeDeps;
              nativeBuildInputs = buildDeps ++ devDeps ++ [ rustc ];
            };
        in
        {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ (import inputs.rust-overlay) ];
          };

          devShells.default = self'.devShells.nightly;

          devShells.nightly = (
            mkDevShell (
              pkgs.rust-bin.selectLatestNightlyWith (
                toolchain:
                toolchain.default.override {
                  targets = rustTargets;
                }
              )
            )
          );
        };
    };
}
