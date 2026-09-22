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
    postgres.enable = true;

    ricochet = {
      enable = true;
      appUrl = "http://localhost:5173";
    };

    kennel = {
      services.api = {
        customDomain = "api.courses.scottylabs.org";
      };
      services.web-api = {
        customDomain = "web-api.courses.scottylabs.org";
      };
      sites.docs = {
        customDomain = "docs.cmu.courses";
      };
    };
  };

  languages.rust.targets = [ "wasm32-unknown-unknown" ];
  # SecretSpec's linkme provider registry needs linker sections Cranelift omits.
  languages.rust.cranelift.excludePackages = [ "secretspec" ];

  treefmt.config.settings.global.excludes = [
    ".devenv/**"
    ".direnv/**"
    "exported/**"
    "data/**"
  ];

  packages = with pkgs; [
    git
    wasm-pack
    wasm-bindgen-cli_0_2_126
  ];

  scripts.build-wasm.exec = ''
    set -euo pipefail
    env -u RUSTFLAGS cargo build --release --target wasm32-unknown-unknown -p courses-index-wasm
    wasm-bindgen --target web --out-dir sites/web/src/lib/courses-index \
      "''${CARGO_TARGET_DIR:-target}/wasm32-unknown-unknown/release/courses_index_wasm.wasm"
  '';

  enterShell = ''
    [ -f sites/web/src/lib/courses-index/courses_index_wasm.js ] || build-wasm
  '';
}
