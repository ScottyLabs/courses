{ pkgs, inputs, lib, config, ... }:

let
  cargoNix = pkgs.callPackage ./Cargo.nix { };
  courses-api = cargoNix.workspaceMembers.courses-api.build;
  courses-web-api = cargoNix.workspaceMembers.courses-web-api.build;

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
    courses-web-api.exec = "${courses-web-api}/bin/courses-web-api --bind 127.0.0.1:3002 --catalog-path ${catalogPath}";
  };

  outputs = { inherit courses-api courses-web-api; };
}
