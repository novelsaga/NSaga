{pkgs, ...}: let
  inherit (pkgs) lib;
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

    bundle = pkgs.rustPlatform.buildRustPackage {
      pname = "novelsaga-bundle";
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
        pkgs.cargo-zigbuild
        pkgs.pkgsCross.ucrt64.stdenv.cc
        pkgs.pkgsCross.ucrtAarch64.stdenv.cc
        pkgs.pkgsCross.aarch64-multiplatform.stdenv.cc
        pkgs.pkgsCross.aarch64-android-prebuilt.stdenv.cc
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
        export SO_OUT_DIR="$TMPDIR/so-out"
        export OUT_DIR="$TMPDIR/out"

        # Cross-compilation environment variables
        export CC_x86_64_unknown_linux_gnu="gcc"
        export CXX_x86_64_unknown_linux_gnu="g++"
        export AR_x86_64_unknown_linux_gnu="ar"
        export CC_aarch64_unknown_linux_gnu="aarch64-unknown-linux-gnu-gcc"
        export CXX_aarch64_unknown_linux_gnu="aarch64-unknown-linux-gnu-g++"
        export AR_aarch64_unknown_linux_gnu="aarch64-unknown-linux-gnu-ar"
        export CC_x86_64_pc_windows_gnu="x86_64-w64-mingw32-gcc"
        export CXX_x86_64_pc_windows_gnu="x86_64-w64-mingw32-g++"
        export AR_x86_64_pc_windows_gnu="x86_64-w64-mingw32-ar"
        export CC_aarch64_pc_windows_gnullvm="aarch64-w64-mingw32-clang"
        export CXX_aarch64_pc_windows_gnullvm="aarch64-w64-mingw32-clang++"
        export AR_aarch64_pc_windows_gnullvm="aarch64-w64-mingw32-ar"
        export CC_aarch64_linux_android="aarch64-unknown-linux-android-clang"
        export CXX_aarch64_linux_android="aarch64-unknown-linux-android-clang++"
        export AR_aarch64_linux_android="aarch64-unknown-linux-android-ar"
        export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="aarch64-unknown-linux-android-clang"
        export CARGO_TARGET_AARCH64_LINUX_ANDROID_AR="aarch64-unknown-linux-android-ar"

        # Android NDK sysroot configuration
        ANDROID_CC="${pkgs.pkgsCross.aarch64-android-prebuilt.stdenv.cc}"
        if [ -d "$ANDROID_CC/ndk-bundle" ]; then
          export CARGO_TARGET_AARCH64_LINUX_ANDROID_RUSTFLAGS="-C link-arg=--sysroot=$ANDROID_CC/ndk-bundle/sysroot"
        fi

        # Build all targets using xtask
        cargo run -p xtask --release -- build-all

        runHook postBuild
      '';

      installPhase = ''
        runHook preInstall

        export CLI_OUT_DIR="''${CLI_OUT_DIR:-$TMPDIR/cli-out}"
        export WASM_OUT_DIR="''${WASM_OUT_DIR:-$TMPDIR/wasm-out}"
        export SO_OUT_DIR="''${SO_OUT_DIR:-$TMPDIR/so-out}"

        install_root="$out/share/novelsaga"
        mkdir -p "$install_root/cli" "$install_root/wasm" "$install_root/so"

        if [ -d "$CLI_OUT_DIR" ]; then
          cp -r "$CLI_OUT_DIR"/. "$install_root/cli/"
        fi

        if [ -d "$WASM_OUT_DIR" ]; then
          cp -r "$WASM_OUT_DIR"/. "$install_root/wasm/"
        fi

        if [ -d "$SO_OUT_DIR" ]; then
          cp -r "$SO_OUT_DIR"/. "$install_root/so/"
        fi

        runHook postInstall
      '';

      doCheck = false;

      meta = with lib; {
        license = licenses.lgpl3Only;
        description = "NovelSaga multi-target bundle built via xtask";
      };
    };
  };
}
