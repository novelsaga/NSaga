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
  # Fetch npm dependencies using pnpm-lock.yaml (v9 = fetcher v3)
  # 更新 hash 的步骤：
  # 1. 将 hash 改为 "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="
  # 2. 运行 nix build .#bundle 2>&1 | grep "got:"
  # 3. 将输出的 sha256 值更新到下面的 hash 字段
  pnpmDeps = pname:
    pkgs.fetchPnpmDeps {
      inherit pname;
      src = ./.;
      hash = "sha256-BHxMv/OXZ0Lpv1C2urz0B5Gyonek5okobQS2go3+ndE=";
      fetcherVersion = 3;
    };
  license = with lib.licenses; [lgpl3 mit];

  commonArgs = {
    version = (lib.importTOML ./Cargo.toml).workspace.package.version;
    src = ./.;
    cargoLock = {
      lockFile = ./Cargo.lock;
    };
    cargo = rustToolchain;
    rustc = rustToolchain;
    RUSTC_BOOTSTRAP = "1";
  };
  commonBuildInputs = with pkgs; [
    nodejs
    pnpm
    pnpmConfigHook
    deno
  ];
in {
  packages = rec {
    default = pkgs.rustPlatform.buildRustPackage (commonArgs
      // {
        pname = "novelsaga";
        pnpmDeps = pnpmDeps default.pname;

        nativeBuildInputs = commonBuildInputs;

        # 构建 CLI 子项目
        buildAndTestSubdir = "projects/cli";

        preBuild = ''
          # Build JavaScript bridges
          echo "🔨 Building JavaScript bridges..."
          cd projects/cli-js-bridges/bridge-core
          node build.mts
          cd ../bridge-nodejs
          node build.mts
          cd ../bridge-bun
          node build.mts
          cd ../bridge-deno
          node build.mts
          cd ../config-bridge
          node build.mts
          cd ../../..
        '';

        postInstall = ''
          # Install JS assets
          mkdir -p $out/share/novelsaga/assets/js/dist
          cp -r projects/cli/assets/js/dist/* $out/share/novelsaga/assets/js/dist/
        '';

        meta = {
          inherit license;
          mainProgram = "novelsaga";
        };
      });
    cli = default;

    bundle = pkgs.rustPlatform.buildRustPackage (commonArgs
      // {
        pname = "novelsaga-bundle";

        # Disable auditable linking to avoid unsupported --undefined flag with zig cc wrappers.
        auditable = false;
        pnpmDeps = pnpmDeps bundle.pname;

        nativeBuildInputs =
          commonBuildInputs
          ++ (with pkgs; [
            rustToolchain
            cargo-zigbuild
            pkgsCross.ucrt64.stdenv.cc
            pkgsCross.ucrtAarch64.stdenv.cc
            pkgsCross.aarch64-multiplatform.stdenv.cc
            pkgsCross.aarch64-android-prebuilt.stdenv.cc
            zig
          ]);

        buildPhase = ''
          runHook preBuild

          export HOME="$TMPDIR"
          export CARGO_HOME="$TMPDIR/cargo-home"
          export CARGO_TARGET_DIR="$TMPDIR/target"
          export CARGO_ZIGBUILD_CACHE_DIR="$TMPDIR/cargo-zigbuild-cache"
          export ZIG_GLOBAL_CACHE_DIR="$TMPDIR/zig-cache"
          export CLI_OUT_DIR="$TMPDIR/cli-out"
          export WASM_OUT_DIR="$TMPDIR/wasm-out"
          export SO_OUT_DIR="$TMPDIR/so-out"
          export OUT_DIR="$TMPDIR/out"

          # Build JavaScript bridges using pre-fetched dependencies
          # 直接使用 node 运行构建脚本，避免 pnpm 的自我管理机制触发网络请求
          echo "🔨 Building JavaScript bridges..."

          # Build bridge-core first (dependency of others)
          cd projects/cli-js-bridges/bridge-core
          node build.mts

          # Build runtime adapters
          cd ../bridge-nodejs
          node build.mts
          cd ../bridge-bun
          node build.mts
          cd ../bridge-deno
          node build.mts

          # Build config-bridge
          cd ../config-bridge
          node build.mts

          cd ../../..

          # Set offline mode for Cargo
          export CARGO_NET_OFFLINE=true

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

          # Build all targets using xtask (skip JS since we built it manually)
          cargo run -p xtask --release -- build-all --skip-js

          # Copy JS assets to CLI output directory
          cargo run -p xtask --release -- build-js --copy-only

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

        meta = {
          inherit license;
          description = "NovelSaga multi-target bundle built via xtask";
        };
      });
  };
}
