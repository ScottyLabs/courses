{
  description = "CMU Courses";

  nixConfig = {
    extra-substituters = [ "https://scottylabs.cachix.org" ];
    extra-trusted-public-keys = [
      "scottylabs.cachix.org-1:hajjEX5SLi/Y7yYloiXTt2IOr3towcTGRhMh1vu6Tjg="
    ];
  };

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    scottylabs = {
      url = "git+https://codeberg.org/ScottyLabs/devenv";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      scottylabs,
      rust-overlay,
      ...
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          inherit (nixpkgs) lib;
          helpers = scottylabs.mkLib pkgs;

          # courses-index compiled to wasm, then run through wasm-bindgen to
          # produce the browser bindings the frontend imports. wasm-bindgen-cli
          # must match the pinned wasm-bindgen crate (0.2.126)
          wasmToolchain = (pkgs.extend (import rust-overlay)).rust-bin.stable.latest.default.override {
            targets = [ "wasm32-unknown-unknown" ];
          };
          wasmRustPlatform = pkgs.makeRustPlatform {
            cargo = wasmToolchain;
            rustc = wasmToolchain;
          };
          courses-index-wasm = wasmRustPlatform.buildRustPackage {
            pname = "courses-index-wasm";
            version = "0.1.0";
            src = ./.;
            cargoLock = {
              lockFile = ./Cargo.lock;
              outputHashes = {
                "axum-oidc-1.0.0-dev-2" = "sha256-zCnb7XAbDORzblB3BcK+CCRXGXsJDTQk/BcfiiOkD/8=";
              };
            };
            nativeBuildInputs = [ pkgs.wasm-bindgen-cli_0_2_126 ];
            doCheck = false;
            buildPhase = ''
              runHook preBuild
              cargo build --release --offline --target wasm32-unknown-unknown -p courses-index-wasm
              runHook postBuild
            '';
            installPhase = ''
              runHook preInstall
              mkdir -p $out
              wasm-bindgen --target web --out-dir $out \
                target/wasm32-unknown-unknown/release/courses_index_wasm.wasm
              runHook postInstall
            '';
          };

          # sites/web with the generated wasm bindings injected
          webSrc = pkgs.runCommandLocal "courses-web-src" { } ''
            cp -r ${lib.cleanSource ./sites/web} $out
            chmod -R u+w $out
            mkdir -p $out/src/lib/courses-index
            cp ${courses-index-wasm}/* $out/src/lib/courses-index/
          '';

          web = helpers.buildDenoTask {
            src = webSrc;
            pname = "courses-web";
            version = "0.1.0";
            task = "build";
            output = "build";
          };

          docs = helpers.buildMdbook {
            src = ./sites/docs;
            name = "courses-docs";
          };

          api = helpers.buildRustService {
            src = ./.;
            pname = "courses-api";
            version = "0.1.0";
            buildArgs.cargoExtraArgs = "--bin courses-api";
          };

          web-api = helpers.buildRustService {
            src = ./.;
            pname = "courses-web-api";
            version = "0.1.0";
            nativeBuildInputs = [ pkgs.makeWrapper ];
            buildArgs = {
              cargoExtraArgs = "--bin courses-web-api";
              postInstall = ''
                wrapProgram $out/bin/courses-web-api --set STATIC_DIR ${web}
              '';
            };
          };
        in
        {
          inherit
            api
            web-api
            web
            docs
            ;
        }
      );
    };
}
