{ pkgs, inputs, lib, config, ... }:

let
  system = pkgs.stdenv.hostPlatform.system;
  built = import ./nix/packages.nix {
    inherit pkgs;
    bun2nixOverlay = inputs.bun2nix.overlays.default;
    rustOverlay = import inputs.rust-overlay;
    repoRoot = ./.;
  };
  inherit (built) courses-api courses-web-api courses-web docs;

  catalogPath = "${config.devenv.root}/exported/catalog/binary";
in
{
  imports = [ inputs.scottylabs.devenvModules.default ];

  scottylabs = {
    enable = true;
    project.name = "courses";
    rust.enable = true;
    bun.enable = true;
    kennel = {
      services.courses-api = { };
      services.courses-web-api = { };
      sites.docs = { };
    };
  };

  packages = with pkgs; [
    wasm-pack
    wasm-bindgen-cli
    courses-api
    courses-web-api
  ];

  env = {
    CATALOG_PATH = catalogPath;
  };

  processes = {
    courses-api.exec = "${courses-api}/bin/courses-api --bind 127.0.0.1:3001 --catalog-path ${catalogPath}";
    courses-web-api.exec = "${courses-web-api}/bin/courses-web-api --bind 127.0.0.1:3002 --catalog-path ${catalogPath} --static-dir ${courses-web}";
  };

  outputs = { inherit courses-api courses-web-api courses-web docs; };
}
