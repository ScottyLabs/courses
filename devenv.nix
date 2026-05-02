{ pkgs, inputs, ... }:

{
  imports = [ inputs.scottylabs.devenvModules.default ];

  scottylabs = {
    enable = true;
    project.name = "courses";
    rust.enable = true;
    bun.enable = true;
    kennel.sites.courses-web = {
      spa = true;
    };
  };

  packages = with pkgs; [
    wasm-pack
    wasm-bindgen-cli
  ];
}
