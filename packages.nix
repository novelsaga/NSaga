{pkgs, ...}: let
  inherit (pkgs) lib;
  cargoLock = lib.importTOML ./Cargo.lock;
  wasmBindgenPkg = lib.findFirst (p: p.name == "wasm-bindgen") {version = "0.0";} cargoLock.package;
  wasmBindgenVersion = wasmBindgenPkg.version;
  wasmBindgenHashes = {
    # Add entries if the lock bumps wasm-bindgen.
    "0.2.106" = {
      crateSha256 = "13pv613cxx57pyfw9z1q7rdi3vkjdqdahnvnlilf7bn4bqdax99k";
      cargoHash = "sha256-ElDatyOwdKwHg3bNH/1pcxKI7LXkhsotlDPQjiLHBwA=";
    };
  };
  wasmBindgenSelected = wasmBindgenHashes.${wasmBindgenVersion} or (throw "wasm-bindgen version ${wasmBindgenVersion} not mapped; please add sha256/cargoHash");
  wasmBindgenCliDrv = pkgs.rustPlatform.buildRustPackage rec {
    pname = "wasm-bindgen-cli";
    version = wasmBindgenVersion;
    src = pkgs.fetchCrate {
      inherit pname version;
      sha256 = wasmBindgenSelected.crateSha256;
    };
    cargoHash = wasmBindgenSelected.cargoHash;
  };
  rustToolchain = pkgs.rust-bin.nightly.latest.default.override {
    targets = [
      "x86_64-unknown-linux-gnu"
      "aarch64-unknown-linux-gnu"
      "aarch64-linux-android"
      "wasm32-unknown-unknown"
      "x86_64-pc-windows-gnu"
      "aarch64-pc-windows-gnullvm"
      "x86_64-apple-darwin"
      "aarch64-apple-darwin"
    ];
    extensions = ["rust-src"];
  };
in {
  packages = rec {
    default = pkgs.rustPlatform.buildRustPackage {
      pname = "novelsaga";
      version = (lib.importTOML ./Cargo.toml).workspace.package.version;
      src = ./.;
      cargoLock = {
        lockFile = ./Cargo.lock;
      };
      # 构建 CLI 子项目
      buildAndTestSubdir = "projects/cli";
      meta = with lib; {
        license = licenses.lgpl3Only;
        mainProgram = "novelsaga";
      };
    };
    cli = default;

    makers-cli-bundle = pkgs.rustPlatform.buildRustPackage {
      pname = "novelsaga-makers";
      version = (lib.importTOML ./Cargo.toml).workspace.package.version;
      src = ./.;
      cargoLock = {
        lockFile = ./Cargo.lock;
      };

      cargo = rustToolchain;
      rustc = rustToolchain;

      # Disable auditable linking to avoid unsupported --undefined flag with zig cc wrappers.
      auditable = false;

      nativeBuildInputs = [
        rustToolchain
        pkgs.cargo-make
        pkgs.cargo-zigbuild
        pkgs.pkgsCross.ucrt64.stdenv.cc
        pkgs.pkgsCross.ucrtAarch64.stdenv.cc
        pkgs.pkgsCross.aarch64-multiplatform.stdenv.cc
        pkgs.pkgsCross.aarch64-android-prebuilt.stdenv.cc
        pkgs.wasm-pack
        pkgs.binaryen
        wasmBindgenCliDrv
        pkgs.zig
      ];

      buildPhase = ''
        runHook preBuild

        export HOME="$TMPDIR"
        export CARGO_HOME="$TMPDIR/cargo-home"
        export CARGO_TARGET_DIR="$TMPDIR/target"
        export CARGO_ZIGBUILD_CACHE_DIR="$TMPDIR/cargo-zigbuild-cache"
        export ZIG_GLOBAL_CACHE_DIR="$TMPDIR/zig-cache"
        export CARGO_NET_OFFLINE=true
        export CLI_OUT_DIR="$TMPDIR/cli-out"
        export WASM_OUT_DIR="$TMPDIR/wasm-out"
        export ANDROID_OUT_DIR="$TMPDIR/android-out"
        export OUT_DIR="$TMPDIR/out"
        export WASM_PACK_CACHE="$TMPDIR/wasm-pack-cache"
        export WASM_BINDGEN="${wasmBindgenCliDrv}/bin/wasm-bindgen"

        # Nix sandboxes lack /usr/bin/env; rewrite task shebangs to an absolute bash path.
        substituteInPlace Makefile.toml --replace-warn '#!/usr/bin/env bash' '#!${pkgs.bash}/bin/bash'

        # Build all targets defined in Makefile (uses cargo-zigbuild per target).
        cargo make --loglevel info \
          --env CLI_OUT_DIR="$CLI_OUT_DIR" \
          --env WASM_OUT_DIR="$WASM_OUT_DIR" \
          --env ANDROID_OUT_DIR="$ANDROID_OUT_DIR" \
          --env OUT_DIR="$OUT_DIR" \
          --env CARGO_TARGET_DIR="$CARGO_TARGET_DIR" \
          build-all

        runHook postBuild
      '';

      installPhase = ''
        runHook preInstall

        export CLI_OUT_DIR="''${CLI_OUT_DIR:-$TMPDIR/cli-out}"
        export WASM_OUT_DIR="''${WASM_OUT_DIR:-$TMPDIR/wasm-out}"
        export ANDROID_OUT_DIR="''${ANDROID_OUT_DIR:-$TMPDIR/android-out}"

        install_root="$out/share/novelsaga"
        mkdir -p "$install_root/cli" "$install_root/wasm" "$install_root/android"

        if [ -d "$CLI_OUT_DIR" ]; then
          cp -r "$CLI_OUT_DIR"/. "$install_root/cli/"
        fi

        if [ -d "$WASM_OUT_DIR" ]; then
          cp -r "$WASM_OUT_DIR"/. "$install_root/wasm/"
        fi

        if [ -d "$ANDROID_OUT_DIR" ]; then
          cp -r "$ANDROID_OUT_DIR"/. "$install_root/android/"
        fi

        runHook postInstall
      '';

      doCheck = false;

      meta = with lib; {
        license = licenses.lgpl3Only;
        description = "NovelSaga multi-target CLI bundle built via cargo-make";
      };
    };
  };
}
