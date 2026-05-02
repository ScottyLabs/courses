{
  description = "CMU Courses";

  nixConfig = {
    extra-substituters = [ "https://scottylabs.cachix.org" ];
    extra-trusted-public-keys = [
      "scottylabs.cachix.org-1:hajjEX5SLi/Y7yYloiXTt2IOr3towcTGRhMh1vu6Tjg="
    ];
  };

  inputs = {
    nixpkgs.url = "github:cachix/devenv-nixpkgs/rolling";
    devenv.url = "github:cachix/devenv";
  };

  outputs = { self, nixpkgs, devenv, ... }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      pkgsFor = system: nixpkgs.legacyPackages.${system};
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = pkgsFor system;
          cargoNix = pkgs.callPackage ./Cargo.nix { };
          courses-api = cargoNix.workspaceMembers.courses-api.build;
          courses-web-api = cargoNix.workspaceMembers.courses-web-api.build;

          courses-web = pkgs.stdenv.mkDerivation {
            pname = "courses-web";
            version = "0.1.0";
            src = ./.;

            nativeBuildInputs = with pkgs; [
              bun
              cargo
              rustc
              wasm-pack
              wasm-bindgen-cli
              binaryen
            ];

            buildPhase = ''
              runHook preBuild
              cd courses-web
              export HOME=$(mktemp -d)
              bun install --frozen-lockfile --no-save
              bun run build:wasm
              bun run build
              runHook postBuild
            '';

            installPhase = ''
              runHook preInstall
              mkdir -p $out
              cp -r build/. $out/
              runHook postInstall
            '';
          };
        in
        {
          inherit courses-web courses-api courses-web-api;
          default = courses-web;
          devenv = devenv.packages.${system}.devenv;
        }
      );
    };
}
