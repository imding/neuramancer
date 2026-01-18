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
        # "aarch64-linux"
        # "x86_64-darwin"
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
          pkgs = import inputs.nixpkgs {
            inherit system;
            config.allowUnfreePredicate =
              pkg:
              builtins.elem (lib.getName pkg) [
                "surrealdb"
                # "surrealist"
              ];
          };
          hostSystem = pkgs.stdenv.hostPlatform.system;
          fenixPkgs = inputs.fenix.packages.${system};
          toolchain = fenixPkgs.combine [
            fenixPkgs.complete.toolchain
            fenixPkgs.targets.wasm32-unknown-unknown.latest.rust-std
          ];

          craneLib = (inputs.crane.mkLib pkgs).overrideToolchain toolchain;

          wasm-bindgen-cli_0_2_108 = pkgs.stdenv.mkDerivation {
            pname = "wasm-bindgen-cli";
            version = "0.2.108";

            src = pkgs.fetchurl (
              if hostSystem == "aarch64-darwin" then
                {
                  url = "https://github.com/wasm-bindgen/wasm-bindgen/releases/download/0.2.108/wasm-bindgen-0.2.108-aarch64-apple-darwin.tar.gz";
                  sha256 = "sha256-OQPIHciUNZLf2dmMjCHJoF+CZaprABT2QahFfOxJJd0=";
                }
              else
                {
                  url = "https://github.com/wasm-bindgen/wasm-bindgen/releases/download/0.2.108/wasm-bindgen-0.2.108-x86_64-unknown-linux-musl.tar.gz";
                  sha256 = "sha256-0V1+R2/ux40Oye3Ce493J57Ho+Yid1bnA25c0gZoAPQ=";
                }
            );

            nativeBuildInputs = lib.optionals pkgs.stdenv.isLinux [ pkgs.autoPatchelfHook ];

            installPhase = ''
              mkdir -p $out/bin
              cp wasm-bindgen $out/bin/
              cp wasm-bindgen-test-runner $out/bin/
              cp wasm2es6js $out/bin/
              chmod +x $out/bin/*
            '';
          };

          dioxus-cli_0_7_3 = pkgs.stdenv.mkDerivation {
            pname = "dioxus-cli";
            version = "0.7.3";

            src = pkgs.fetchurl (
              if hostSystem == "aarch64-darwin" then
                {
                  url = "https://github.com/DioxusLabs/dioxus/releases/download/v0.7.3/dx-aarch64-apple-darwin.tar.gz";
                  sha256 = "sha256-q2k0s/X3Hsk1D4vF7j/jTkObN4WZiZtI7UcX8o+pEuY=";
                }
              else
                {
                  url = "https://github.com/DioxusLabs/dioxus/releases/download/v0.7.3/dx-x86_64-unknown-linux-gnu.tar.gz";
                  sha256 = "sha256-8nTyLX7QOC1jh1nMALW3wBFFeCnoMBEFLKGRx4efeg4=";
                }
            );

            # The tarball contains just the binary directly, no directory structure
            sourceRoot = ".";

            nativeBuildInputs = lib.optionals pkgs.stdenv.isLinux [ pkgs.autoPatchelfHook ];
            buildInputs = lib.optionals pkgs.stdenv.isLinux [
              pkgs.openssl
              pkgs.stdenv.cc.cc.lib
              pkgs.zlib
            ];

            installPhase = ''
              mkdir -p $out/bin
              cp dx $out/bin/
              chmod +x $out/bin/*
            '';

            postFixup = lib.optionalString pkgs.stdenv.isDarwin ''
              ${pkgs.darwin.cctools}/bin/install_name_tool \
                -change /opt/homebrew/opt/openssl@3/lib/libssl.3.dylib ${pkgs.openssl.out}/lib/libssl.3.dylib \
                -change /opt/homebrew/opt/openssl@3/lib/libcrypto.3.dylib ${pkgs.openssl.out}/lib/libcrypto.3.dylib \
                $out/bin/dx
            '';
          };

          src = lib.cleanSourceWith {
            src = craneLib.path ./.;
            filter =
              path: type:
              (craneLib.filterCargoSources path type)
              || (lib.hasInfix "/assets" path)
              || (lib.hasInfix "/surreal/migrations" path)
              || (lib.hasInfix "/surreal/schemas" path)
              || (lib.hasInfix "/surreal/events" path)
              || (lib.hasSuffix ".surrealdb" path);
          };

          web = craneLib.buildPackage {
            pname = "neuramancy-web";
            version = "0.1.0";

            inherit src;

            # Don't build dependencies separately since `dx bundle` does everything
            cargoArtifacts = null;
            doCheck = false;
            doNotPostBuildInstallCargoBinaries = true;

            nativeBuildInputs = with pkgs; [
              binaryen
              dioxus-cli_0_7_3
              pkg-config
              wasm-bindgen-cli_0_2_108
            ];

            buildInputs = with pkgs; [
              openssl
              # onnxruntime
            ];

            # ORT_STRATEGY = "system";
            # ORT_LIB_LOCATION = "${pkgs.onnxruntime}/lib";

            buildPhase = ''
              runHook preBuild
              NO_DOWNLOADS=1 dx bundle -r -p web --debug-symbols=false
              runHook postBuild
            '';

            installPhase = ''
              runHook preInstall
              mkdir -p $out
              cp -r target/dx/web/release/web/* $out/
              runHook postInstall
            '';
          };

          migrationFiles = pkgs.runCommand "migration-files" { } ''
            mkdir -p $out/backend/src/surreal
            cp -r ${src}/backend/src/surreal/schemas $out/backend/src/surreal/ || true
            cp -r ${src}/backend/src/surreal/migrations $out/backend/src/surreal/ || true
            cp -r ${src}/backend/src/surreal/events $out/backend/src/surreal/ || true
            cp ${src}/.surrealdb $out/.surrealdb
          '';

          web-img = pkgs.dockerTools.streamLayeredImage {
            name = "neuramancy";
            tag = "latest";
            contents = [ web ];
            fakeRootCommands = ''
              mkdir -p ./backend/src/surreal
              cp -r ${migrationFiles}/backend/src/surreal/* ./backend/src/surreal/
              cp ${migrationFiles}/.surrealdb ./.surrealdb
            '';
            config = {
              Cmd = [ "/web" ];
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
              dioxus-cli_0_7_3
              wasm-bindgen-cli_0_2_108

              # Build dependencies (needed for dx serve/bundle)
              pkg-config
              openssl
              # onnxruntime

              # Development tools
              git
              helix
              jujutsu
              # surrealdb
              # surrealist
              surrealdb-migrations
            ];

            shellHook = ''
              export RUST_SRC_PATH=${pkgs.rustPlatform.rustLibSrc}
              export DISPLAY=:0
              # export ORT_STRATEGY=system
              # export ORT_LIB_LOCATION=${pkgs.onnxruntime}/lib
            '';
          };
        };
    };
}
