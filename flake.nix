{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    crane.url = "github:ipetkov/crane";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };
  };

  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
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
          fenixPkgs = inputs.fenix.packages.${system};
          pinnedRust = fenixPkgs.toolchainOf {
            channel = "1.86.0";
            date = "2025-04-03";
            sha256 = "sha256-X/4ZBHO3iW0fOenQ3foEvscgAPJYl2abspaBThDOukI=";
          };
          wasm32Toolchain = fenixPkgs.targets.wasm32-unknown-unknown.toolchainOf {
            channel = "1.86.0";
            date = "2025-04-03";
            sha256 = "sha256-X/4ZBHO3iW0fOenQ3foEvscgAPJYl2abspaBThDOukI=";
          };
          toolchain = fenixPkgs.combine [
            (pinnedRust.withComponents [
              "cargo"
              "clippy"
              "rust-src"
              "rustc"
              "rustfmt"
            ])
            wasm32Toolchain.rust-std
          ];

          craneLib = (inputs.crane.mkLib pkgs).overrideToolchain toolchain;

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

          src = lib.cleanSourceWith {
            src = craneLib.path ./.;
            filter = path: type: (craneLib.filterCargoSources path type) || (lib.hasInfix "/assets" path);
          };

          web = craneLib.buildPackage {
            pname = "neuramancy-web";
            version = "0.1.0";

            inherit src;

            # Don't build dependencies separately since dx bundle does everything
            cargoArtifacts = null;
            doCheck = false;
            doNotPostBuildInstallCargoBinaries = true;

            nativeBuildInputs = with pkgs; [
              dioxus-cli
              pkg-config
              wasm-bindgen-cli_0_2_104
            ];

            buildInputs = with pkgs; [
              openssl
            ];

            buildPhase = ''
              runHook preBuild
              dx bundle -p web
              runHook postBuild
            '';

            installPhase = ''
              runHook preInstall
              mkdir -p $out
              cp -r target/dx/web/release/web/* $out/
              runHook postInstall
            '';
          };

          web-img = pkgs.dockerTools.streamLayeredImage {
            name = "neuramancy";
            tag = "latest";
            contents = [ web ];

            config = {
              Cmd = [ "/server" ];
              Env = [
                "PORT=8080"
                "IP=0.0.0.0"
              ];
              ExposedPorts = {
                "8080/tcp" = { };
              };
              WorkingDir = "/";
            };
          };
        in
        {
          packages = {
            inherit web;
          }
          // lib.optionalAttrs pkgs.stdenv.isLinux {
            # Docker images only available on Linux
            inherit web-img;
            default = web-img;
          };

          devShells.default = craneLib.devShell {
            packages = with pkgs; [
              # Nix tools
              nixpkgs-fmt
              nil

              # Deployment tools
              dive
              flyctl

              # Rust/Dioxus tools
              dioxus-cli
              wasm-bindgen-cli_0_2_104

              # Build dependencies (needed for dx serve/bundle)
              pkg-config
              openssl

              # Development tools
              git
              helix
              jujutsu
            ];

            shellHook = ''
              export RUST_SRC_PATH=${pkgs.rustPlatform.rustLibSrc}
              export DISPLAY=:0
            '';
          };
        };
    };
}
