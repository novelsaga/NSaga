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
  "nix.serverPath" = lib.getExe pkgs.nil;
  "nix.formatterPath" = "alejandra";
  "nix.serverSettings" = {
    "nil" = {
      "formatting" = {
        "command" = [
          (lib.getExe pkgs.alejandra)
        ];
      };
    };
  };
  "dependi.rust.lockFileEnabled" = true;
  "files.associations" = {
    "*.json5" = "json5";
  };
  "typescript.tsdk" = "${devenv-root-path}/node_modules/typescript/lib";
  "eslint.format.enable" = true;
  "eslint.enable" = true;
  "eslint.ignoreUntitled" = true;
  "eslint.lintTask.enable" = true;
  "eslint.useFlatConfig" = true;
  "eslint.runtime" = nodePath;
  "eslint.nodePath" = "${devenv-root-path}/node_modules";
  "typescript.tsserver.nodePath" = nodePath;
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
