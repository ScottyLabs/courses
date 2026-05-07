{ pkgs, inputs, lib, config, ... }:

let
  system = pkgs.stdenv.hostPlatform.system;
  bun2nixCli = inputs.bun2nix.packages.${system}.default;
  built = import ./nix/packages.nix {
    inherit pkgs;
    bun2nixOverlay = inputs.bun2nix.overlays.default;
    rustOverlay = import inputs.rust-overlay;
    repoRoot = ./.;
  };
  inherit (built) courses-api courses-web-api courses-web docs;

  credentialsEnv = "${config.env.DEVENV_STATE}/garage/credentials.env";

  apiExec = bin: extraArgs: ''
    set -e
    while [ ! -f "${credentialsEnv}" ]; do sleep 0.5; done
    set -a
    source "${credentialsEnv}"
    set +a
    exec ${bin} \
      --s3-bucket courses-catalog \
      --s3-endpoint "$GARAGE_S3_ENDPOINT" \
      ${extraArgs}
  '';
in
{
  imports = [
    inputs.scottylabs.devenvModules.default
  ];

  scottylabs = {
    enable = true;
    project.name = "courses";
    rust.enable = true;
    bun.enable = true;
    secrets.enable = true;
    kennel = {
      services.courses-api.oidc.redirectPaths = [ "/oauth2/callback" ];
      services.courses-web-api = { };
      sites.docs = { };
    };
  };

  treefmt.config.settings.global.excludes = [
    "exported/**"
    "exported_old/**"
    "data/**"
  ];

  services.garage = {
    enable = true;
    buckets = [ "courses-catalog" ];
    afterStart = ''
      mkdir -p "${config.env.DEVENV_STATE}/garage"
      if [ ! -f "${credentialsEnv}" ]; then
        if $GARAGE key info dev-key >/dev/null 2>&1; then
          $GARAGE key delete dev-key --yes >/dev/null 2>&1 || true
        fi
        OUTPUT=$($GARAGE key create dev-key)
        ACCESS=$(echo "$OUTPUT" | awk '/Key ID:/ {print $NF}')
        SECRET=$(echo "$OUTPUT" | awk '/Secret key:/ {print $NF}')
        {
          echo "AWS_ACCESS_KEY_ID=$ACCESS"
          echo "AWS_SECRET_ACCESS_KEY=$SECRET"
        } > "${credentialsEnv}"
      fi
      $GARAGE bucket allow --read --write --key dev-key courses-catalog
    '';
  };

  packages = (with pkgs; [
    wasm-pack
    wasm-bindgen-cli
    courses-api
    courses-web-api
  ]) ++ [ bun2nixCli ];

  processes = {
    courses-api.exec =
      apiExec "${courses-api}/bin/courses-api" "--bind 127.0.0.1:3001";
    courses-web-api.exec =
      apiExec "${courses-web-api}/bin/courses-web-api" ''--bind 127.0.0.1:3002 --static-dir ${courses-web}'';
  };

  outputs = { inherit courses-api courses-web-api courses-web docs; };
}
