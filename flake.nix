{
  description = "workspace flake for NovelSaga project";

  inputs = {
    devenv-root = {
      url = "file+file:///dev/null";
      flake = false;
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    devenv.url = "github:cachix/devenv";
    nix2container = {
      url = "github:nlewo/nix2container";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    mk-shell-bin.url = "github:rrbutani/nix-mk-shell-bin";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rustowl-flake = {
      url = "github:nix-community/rustowl-flake";
      inputs = {
        flake-parts.follows = "flake-parts";
        nixpkgs.follows = "nixpkgs";
        rust-overlay.follows = "rust-overlay";
      };
    };
  };

  nixConfig = {
    extra-trusted-public-keys = "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=";
    extra-substituters = "https://devenv.cachix.org";
  };

  outputs = inputs @ {
    flake-parts,
    devenv-root,
    treefmt-nix,
    rust-overlay,
    ...
  }: let
    inherit (inputs.nixpkgs) lib;
    devenv-root-path = builtins.readFile devenv-root;
  in
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        inputs.devenv.flakeModule
      ];
      systems = with lib;
        intersectLists (platforms.linux ++ platforms.darwin) (intersectLists (platforms.aarch64 ++ platforms.x86_64) systems.flakeExposed);
      perSystem = {
        config,
        self',
        inputs',
        system,
        pkgs,
        ...
      }: {
        _module.args.pkgs = import inputs.nixpkgs {
          inherit system;
          config.allowUnfreePredicate = pkg: true;
          overlays = [(import rust-overlay)];
        };
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
        };
        devenv.shells.default = {
          name = "novelsaga";
          env = {
            RUSTC_BOOTSTRAP = "1";
            CC_x86_64_unknown_linux_gnu = "gcc";
            CXX_x86_64_unknown_linux_gnu = "g++";
            AR_x86_64_unknown_linux_gnu = "ar";
            CC_aarch64_unknown_linux_gnu = "aarch64-unknown-linux-gnu-gcc";
            CXX_aarch64_unknown_linux_gnu = "aarch64-unknown-linux-gnu-g++";
            AR_aarch64_unknown_linux_gnu = "aarch64-unknown-linux-gnu-ar";
            CC_x86_64_pc_windows_gnu = "x86_64-w64-mingw32-gcc";
            CXX_x86_64_pc_windows_gnu = "x86_64-w64-mingw32-g++";
            AR_x86_64_pc_windows_gnu = "x86_64-w64-mingw32-ar";
            CC_aarch64_pc_windows_gnullvm = "aarch64-w64-mingw32-clang";
            CXX_aarch64_pc_windows_gnullvm = "aarch64-w64-mingw32-clang++";
            AR_aarch64_pc_windows_gnullvm = "aarch64-w64-mingw32-ar";
            CC_aarch64_linux_android = "aarch64-unknown-linux-android-clang";
            CXX_aarch64_linux_android = "aarch64-unknown-linux-android-clang++";
            AR_aarch64_linux_android = "aarch64-unknown-linux-android-ar";
            CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER = "aarch64-unknown-linux-android-clang";
            CARGO_TARGET_AARCH64_LINUX_ANDROID_AR = "aarch64-unknown-linux-android-ar";
          };
          packages =
            (with pkgs; [
              shfmt
              cargo-zigbuild
              cargo-make
              inputs.rustowl-flake.packages.${system}.rustowl
            ])
            ++ (with pkgs.pkgsCross; [
              mingwW64.stdenv.cc
              ucrtAarch64.stdenv.cc
              aarch64-multiplatform.stdenv.cc
              aarch64-android-prebuilt.stdenv.cc
            ]);
          languages = {
            nix = {
              enable = true;
              lsp.package = pkgs.nil;
            };
            rust = {
              enable = true;
              channel = "nightly";
              components = [
                "rustc"
                "cargo"
                "clippy"
                "rustfmt"
                "rust-analyzer"
                "miri"
                "rust-std"
              ];
              targets = [
                "aarch64-apple-darwin"
                "x86_64-apple-darwin"
                "aarch64-pc-windows-gnullvm"
                "x86_64-pc-windows-gnu"
                "aarch64-unknown-linux-gnu"
                "x86_64-unknown-linux-gnu"
                "wasm32-unknown-unknown"
                "aarch64-linux-android"
              ];
            };
          };
          git-hooks.hooks = {
            commitizen.enable = true;
            clippy = {
              enable = true;
              entry = "direnv exec . cargo clippy --offline --all-features --all-targets --";
              pass_filenames = false;
              language = "system";
            };
            treefmt = {
              enable = true;
              packageOverrides = {
                treefmt = config.devenv.shells.default.treefmt.config.build.wrapper;
              };
            };
          };
          treefmt = {
            enable = true;
            config = {
              projectRootFile = "flake.nix";
              programs = {
                dos2unix = {
                  enable = true;
                  priority = -1;
                };
                formatjson5 = {
                  enable = true;
                  includes = [
                    "**/*.json5"
                    "**/*.jsonc"
                    "**/*.json"
                  ];
                  excludes = [
                    ".vscode/**"
                  ];
                  indent = 2;
                  noTrailingCommas = true;
                  oneElementLines = true;
                };
                alejandra.enable = true;
                shellcheck.enable = true;
                taplo = {
                  enable = true;
                  settings = {
                    formatting = {
                      reorder_keys = true;
                      indent_entries = true;
                      indent_tables = true;
                      align_comments = true;
                      align_entries = true;
                      allowed_blank_lines = 1;
                      array_auto_collapse = true;
                      array_auto_expand = true;
                    };
                  };
                };
                rustfmt = {
                  enable = true;
                  package = config.devenv.shells.default.languages.rust.toolchain.rustfmt;
                };
              };
            };
          };
        };
      };
    };
}
