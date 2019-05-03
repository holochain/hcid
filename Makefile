SHELL		= /bin/bash

.PHONY: all test tools build clean

all: build build_web

test: build build_web
	RUST_BACKTRACE=1 cargo test -p hcid -- --nocapture
	cd hcid-js && (which node_modules/.bin/standard || npm ci) && npm test

tools:
	rustup override set nightly-2019-01-24
	rustup target add wasm32-unknown-unknown
	if ! (which wasm-bindgen) || [ "$(shell wasm-bindgen --version)" != "wasm-bindgen 0.2.40" ]; then cargo install --force wasm-bindgen-cli --version "=0.2.40"; fi

build: tools
	cargo build -p hcid --release
	cargo build -p hcid_js --target wasm32-unknown-unknown --release
	wasm-bindgen target/wasm32-unknown-unknown/release/hcid_js.wasm --out-dir hcid-js/lib --out-name bindgen --nodejs --no-typescript

build_web: build
	wasm-bindgen target/wasm32-unknown-unknown/release/hcid_js.wasm --out-dir hcid-js/lib/browser --out-name bindgen --browser --no-typescript
	wasm2es6js --base64 -o hcid-js/lib/browser/bindgen_bg.js hcid-js/lib/browser/bindgen_bg.wasm
	rm hcid-js/lib/browser/bindgen_bg.wasm

clean:
	rm -rf target hcid-js/rust/target hcid-js/lib/bindgen_bg.wasm
