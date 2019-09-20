{ pkgs ? import ./pkgs.nix {} }:

with pkgs;

{
  hcid = rustPlatform.buildRustPackage {
    name = "hcid";
    src = gitignoreSource ./.;

    nativeBuildInputs = [
      holochain-cli
      holochain-conductor
      nodejs-12_x
    ];
			  
    cargoSha256 = "0a2sws2a19ykg4m86viylf5jzmvb0wyif3kri90sp6aqm84kqqkx";

    meta.platforms = lib.platforms.all;
  };

  hcid-js = stdenv.mkDerivation rec {
    name = "hcid-js";
    src = gitignoreSource hcid-js/.;
    
    nativeBuildInputs = [
      holochain-cli
      holochain-conductor
      nodejs-12_x
    ];

    preConfigure = ''
      cp -r ${npmToNix { inherit src; }} node_modules
      chmod -R +w node_modules
      patchShebangs node_modules
    '';
    
    buildPhase = ''
      #cargo build -p hcid --release
      cargo build -p hcid_js --target wasm32-unknown-unknown --release
      wasm-bindgen target/wasm32-unknown-unknown/release/hcid_js.wasm --out-dir hcid-js/lib --out-name bindgen --nodejs --no-typescript
    '';
    
    installPhase = ''
      mkdir $out
      mv * $out
    '';

    fixupPhase = ''
      patchShebangs $out
    '';
  };
}
