{
  devenv-root-path,
  pkgs,
  lib,
  ...
}: let
  devenvRootPathBin = "${devenv-root-path}/.devenv/profile/bin";
in {
  "rust-analyzer.server.path" = "${devenvRootPathBin}/rust-analyzer";
  "rust-analyzer.check.command" = "clippy";
  "rust-analyzer.check.extraArgs" = ["--all-features"];
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
}
