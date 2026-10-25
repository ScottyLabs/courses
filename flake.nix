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
      url = "git+https://git.cmu.dev/ScottyLabs/kennel";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      scottylabs,
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
          helpers = scottylabs.mkLib pkgs;

          web = helpers.buildDenoTask {
            src = ./sites/web;
            pname = "courses-web";
            version = "0.1.0";
            task = "build";
            output = "build";
          };

          docs = helpers.buildZensical {
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
              # The service embeds the pinned catalog release (deployments/catalog.json), which the helper's
              # Cargo-only source filter leaves out; the dependency build keeps the filtered source.
              src = pkgs.lib.fileset.toSource {
                root = ./.;
                fileset = pkgs.lib.fileset.unions [
                  ./Cargo.toml
                  ./Cargo.lock
                  ./crates
                  ./secretspec.toml
                  ./deployments/catalog.json
                ];
              };
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
