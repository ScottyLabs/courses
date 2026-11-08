{
  description = "CMU Courses";

  nixConfig = {
    extra-substituters = [ "https://scottylabs.cachix.org" ];
    extra-trusted-public-keys = [
      "scottylabs.cachix.org-1:hajjEX5SLi/Y7yYloiXTt2IOr3towcTGRhMh1vu6Tjg="
    ];
  };

  inputs = {
    scottylabs.url = "git+https://git.cmu.dev/ScottyLabs/kennel";
    nixpkgs.follows = "scottylabs/nixpkgs";
    flake-parts.follows = "scottylabs/flake-parts";
  };

  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [ inputs.scottylabs.flakeModules.default ];
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];

      perSystem =
        { pkgs, helpers, ... }:
        let
          web = helpers.buildDenoTask {
            src = ./sites/web;
            pname = "courses-web";
            version = "0.1.0";
            task = "build";
            output = "build";
          };
        in
        {
          scottylabs = {
            project.name = "courses";

            rust.enable = true;
            deno = {
              enable = true;
              svelte.enable = true;
              dirs = [ "sites/web" ];
            };

            postgres.enable = true;
            sqlite.enable = true;

            ricochet = {
              enable = true;
              appUrl = "http://localhost:5173";
            };

            processes = {
              web-api.run = "secretspec run -P dev -- cargo run -p courses-web-api";
              web.run = "cd sites/web && deno task dev";
            };

            shell = {
              packages = [
                pkgs.zensical
                pkgs.nodejs
              ];

              # aws s3 against the CDN bucket, shadowing the sccache AWS_* keys
              scripts.cdn-s3 = ''
                AWS_ACCESS_KEY_ID="$CDN_ACCESS_KEY_ID" AWS_SECRET_ACCESS_KEY="$CDN_SECRET_ACCESS_KEY" AWS_DEFAULT_REGION=us-east-1 \
                  exec ${pkgs.awscli2}/bin/aws s3 "$@" --endpoint-url https://s3.scottylabs.org
              '';
            };
          };

          treefmt.settings.global.excludes = [
            "exported/**"
            "data/**"
          ];

          services = {
            api = {
              package = helpers.buildRustService {
                src = ./.;
                pname = "courses-api";
                version = "0.1.0";
                buildArgs.cargoExtraArgs = "--locked --bin courses-api";
              };
              customDomain = "api.cmu.courses";
            };

            web-api = {
              package = helpers.buildRustService {
                src = ./.;
                pname = "courses-web-api";
                version = "0.1.0";
                buildArgs.cargoExtraArgs = "--locked --bin courses-web-api";
              };
              customDomain = "cmu.courses";
              env.STATIC_DIR = "${web}";
            };
          };

          sites.docs = {
            package = helpers.buildZensical {
              src = ./sites/docs;
              name = "courses-docs";
            };
            customDomain = "docs.cmu.courses";
          };
        };
    };
}
