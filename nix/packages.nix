{ pkgs, bun2nixOverlay, rustOverlay, repoRoot }:

let
  pkgs' = pkgs.extend (pkgs.lib.composeExtensions rustOverlay bun2nixOverlay);

  rustWasm = pkgs'.rust-bin.stable.latest.default.override {
    targets = [ "wasm32-unknown-unknown" ];
  };

  wasmPlatform = pkgs'.makeRustPlatform {
    cargo = rustWasm;
    rustc = rustWasm;
  };

  cargoSrc = pkgs.lib.fileset.toSource {
    root = repoRoot;
    fileset = pkgs.lib.fileset.unions [
      (repoRoot + "/Cargo.toml")
      (repoRoot + "/Cargo.lock")
      (repoRoot + "/courses-api")
      (repoRoot + "/courses-index")
      (repoRoot + "/courses-index-wasm")
      (repoRoot + "/courses-scraper")
      (repoRoot + "/courses-web-api")
    ];
  };

  courses-index-wasm-raw = pkgs'.stdenv.mkDerivation {
    pname = "courses-index-wasm";
    version = "0.1.0";
    src = cargoSrc;

    cargoDeps = pkgs'.rustPlatform.importCargoLock {
      lockFile = "${repoRoot}/Cargo.lock";
    };

    nativeBuildInputs = [
      rustWasm
      pkgs'.rustPlatform.cargoSetupHook
    ];

    buildPhase = ''
      runHook preBuild
      cargo build \
        --release \
        --target wasm32-unknown-unknown \
        -p courses-index-wasm \
        --offline
      runHook postBuild
    '';

    installPhase = ''
      runHook preInstall
      mkdir -p $out
      cp target/wasm32-unknown-unknown/release/*.wasm $out/
      runHook postInstall
    '';
  };

  wasm-bindgen-cli = pkgs'.rustPlatform.buildRustPackage rec {
    pname = "wasm-bindgen-cli";
    version = "0.2.120";
    src = pkgs'.fetchCrate {
      inherit pname version;
      hash = "sha256-Dkkx8Bhfk+y/jEz9Fzwytmv2N3Gj/7ST+5MlPRzzetU=";
    };
    cargoHash = "sha256-5Zu/Sh9aBMxB+KGC1MHWJAQ8PuE40M6lsenkpFEwJ6A=";
    nativeBuildInputs = [ pkgs'.pkg-config ];
    buildInputs = [ pkgs'.openssl ];
    doCheck = false;
  };

  courses-index-wasm-bindings = pkgs.runCommand "courses-index-wasm-bindings"
    { nativeBuildInputs = [ wasm-bindgen-cli pkgs.binaryen ]; }
    ''
      mkdir -p $out
      wasm-bindgen \
        --target web \
        --out-dir $out \
        --out-name courses_index_wasm \
        ${courses-index-wasm-raw}/courses_index_wasm.wasm
      for f in $out/*_bg.wasm; do
        wasm-opt -O4 --enable-bulk-memory --enable-nontrapping-float-to-int "$f" -o "$f.opt"
        mv "$f.opt" "$f"
      done
    '';

  cargoNix = pkgs.callPackage "${repoRoot}/Cargo.nix" { };
  courses-api = cargoNix.workspaceMembers.courses-api.build;
  courses-web-api = cargoNix.workspaceMembers.courses-web-api.build;

  courses-web-src = pkgs.runCommand "courses-web-src" { } ''
    cp -r ${repoRoot}/courses-web $out
    chmod -R u+w $out
    mkdir -p $out/src/lib/courses-index
    cp -r ${courses-index-wasm-bindings}/. $out/src/lib/courses-index/
  '';

  courses-web = pkgs.stdenv.mkDerivation {
    pname = "courses-web";
    version = "0.1.0";
    src = courses-web-src;

    nativeBuildInputs = [ pkgs'.bun pkgs'.bun2nix.hook ];
    bunDeps = pkgs'.bun2nix.fetchBunDeps {
      bunNix = "${repoRoot}/courses-web/bun.nix";
    };
    bunInstallFlags = pkgs.lib.optionals pkgs.stdenv.isDarwin [
      "--linker=hoisted"
      "--backend=copyfile"
    ];

    buildPhase = ''
      runHook preBuild
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
  docsSrc = pkgs.lib.fileset.toSource {
    root = repoRoot;
    fileset = repoRoot + "/docs";
  };

  docs = pkgs.runCommand "courses-docs"
    { nativeBuildInputs = [ pkgs.mdbook ]; }
    ''
      mkdir -p $out
      mdbook build -d $out ${docsSrc}/docs
    '';
in
{
  inherit
    courses-api
    courses-web-api
    courses-web
    courses-index-wasm-bindings
    docs
    ;
}
