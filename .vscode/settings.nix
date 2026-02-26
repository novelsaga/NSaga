{
  devenv-root-path,
  pkgs,
  lib,
  ...
}: let
  devenvRootPathBin = "${devenv-root-path}/.devenv/profile/bin";
  nodePath = lib.getExe pkgs.nodejs-slim_24;
in {
  "rust-analyzer.server.path" = "${devenvRootPathBin}/rust-analyzer";
  "rust-analyzer.check.command" = "clippy";
  "rust-analyzer.rustc.source" = "discover";
  "rust-analyzer.check.extraArgs" = [
    # "--all-features"
  ];
  "nix.enableLanguageServer" = true;
  "nix.serverPath" = lib.getExe pkgs.nixd;
  "nix.formatterPath" = "alejandra";
  "nix.serverSettings" = {
    "nixd" = {
      "nixpkgs" = {
        "expr" = ''(builtins.getFlake "git+file://${devenv-root-path}").debug.config.allSystems.${pkgs.system}.allModuleArgs.pkgs'';
      };
      "formatting" = {
        "command" = [
          (lib.getExe pkgs.alejandra)
        ];
      };
      "options" = {
        "flake-parts" = {
          "expr" = ''(builtins.getFlake "git+file://${devenv-root-path}").debug.options'';
        };
      };
    };
  };
  "dependi.rust.lockFileEnabled" = true;
  "files.associations" = {
    "*.json5" = "json5";
  };
  "eslint.format.enable" = true;
  "eslint.enable" = true;
  "eslint.ignoreUntitled" = true;
  "eslint.lintTask.enable" = true;
  "eslint.useFlatConfig" = true;
  "eslint.runtime" = nodePath;
  "eslint.nodePath" = "${devenv-root-path}/node_modules";
  "typescript.experimental.useTsgo" = true;
  "typescript.native-preview.tsdk" = "${devenv-root-path}/node_modules/.bin";
  "[javascript]" = {
    "editor.defaultFormatter" = "dbaeumer.vscode-eslint";
  };
  "[javascriptreact]" = {
    "editor.defaultFormatter" = "dbaeumer.vscode-eslint";
  };
  "[typescript]" = {
    "editor.defaultFormatter" = "dbaeumer.vscode-eslint";
  };
  "[typescriptreact]" = {
    "editor.defaultFormatter" = "dbaeumer.vscode-eslint";
  };
}
