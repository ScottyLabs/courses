{ pkgs, inputs, ... }:

{
  imports = [
    inputs.scottylabs.devenvModules.default
  ];

  scottylabs = {
    enable = true;
    project.name = "courses";

    rust.enable = true;
    deno = {
      enable = true;
      svelte.enable = true;
      svelte.dir = "sites/web";
    };
    secrets.enable = true;
    ricochet = {
      enable = true;
      appUrl = "http://localhost:5173";
    };

    kennel = {
      services.api = { };
      services.web-api = { };
      sites.docs = { };
    };
  };

  languages.rust.targets = [ "wasm32-unknown-unknown" ];

  treefmt.config.settings.global.excludes = [
    "exported/**"
    "data/**"
  ];

  packages = with pkgs; [
    wasm-pack
    wasm-bindgen-cli_0_2_126
  ];

  scripts.build-wasm.exec = ''
    env -u RUSTFLAGS cargo build --release --target wasm32-unknown-unknown -p courses-index-wasm
    wasm-bindgen --target web --out-dir sites/web/src/lib/courses-index \
      "''${CARGO_TARGET_DIR:-target}/wasm32-unknown-unknown/release/courses_index_wasm.wasm"
  '';

  enterShell = ''
    [ -f sites/web/src/lib/courses-index/courses_index_wasm.js ] || build-wasm
  '';
}
