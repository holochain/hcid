SHELL		= /bin/bash

# 
# External targets; Uses a nix-shell environment to obtain Holochain development environemnt, run tests, etc.
# 
.PHONY: all
all: nix-test nix-build nix-build_web

# nix-test, nix-install, ...
nix-%:
	nix-shell --pure --run "make $*"

# 
# Internal targets; require a Nix Holochain-compatible development environment (or other defined
# development environment) in order to be deterministic.
# 
# - Uses the version of `cargo`, `node`, `npm` on the system PATH.
# - Normally called from within a Nix environment, eg. run `nix-shell` from within development
#
test: build build_web
	RUST_BACKTRACE=1 cargo test -p hcid -- --nocapture
	cd hcid-js && (which node_modules/.bin/standard || npm ci) && npm test


build:
	cargo build -p hcid --release
	cargo build -p hcid_js --target wasm32-unknown-unknown --release
	wasm-bindgen target/wasm32-unknown-unknown/release/hcid_js.wasm --out-dir hcid-js/lib --out-name bindgen --nodejs --no-typescript

build_web: build
	wasm-bindgen target/wasm32-unknown-unknown/release/hcid_js.wasm --out-dir hcid-js/lib/browser --out-name bindgen --browser --no-typescript
	wasm2es6js --base64 -o hcid-js/lib/browser/bindgen_bg.js hcid-js/lib/browser/bindgen_bg.wasm
	rm hcid-js/lib/browser/bindgen_bg.wasm

clean:
	rm -rf target hcid-js/rust/target hcid-js/lib/bindgen_bg.wasm

# 
# Manually configure tools, if desired; Not recommended. Use the Holochain-compatible Nix defined toolset
# 
tools:
	rustup override set nightly-2019-01-24
	rustup target add wasm32-unknown-unknown
	if ! (which wasm-bindgen) || [ "$(shell wasm-bindgen --version)" != "wasm-bindgen 0.2.40" ]; then cargo install --force wasm-bindgen-cli --version "=0.2.40"; fi
