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
      dirs = [ "sites/web" ];
    };

    secrets.enable = true;
    postgres.enable = true;

    ricochet = {
      enable = true;
      appUrl = "http://localhost:5173";
    };

    kennel = {
      services = {
        api.customDomain = "api.cmu.courses";
        web-api.customDomain = "web-api.cmu.courses";
      };
      sites.docs.customDomain = "docs.cmu.courses";
    };
  };

  treefmt.config.settings.global.excludes = [
    ".devenv/**"
    ".direnv/**"
    "exported/**"
    "data/**"
  ];

  # aws s3 against the CDN bucket, shadowing the sccache AWS_* keys
  scripts.cdn-s3 = {
    packages = [ pkgs.awscli2 ];
    exec = ''
      AWS_ACCESS_KEY_ID="$CDN_ACCESS_KEY_ID" AWS_SECRET_ACCESS_KEY="$CDN_SECRET_ACCESS_KEY" AWS_DEFAULT_REGION=us-east-1 \
        exec aws s3 "$@" --endpoint-url https://s3.scottylabs.org
    '';
  };
}
