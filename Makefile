SHELL		= /bin/bash

.PHONY: all test tools build clean

all: build

test: build
	RUST_BACKTRACE=1 cargo test -p hcid -- --nocapture
	cd hcid-js && (which node_modules/.bin/standard || npm ci) && npm test

tools:
	rustup override set nightly-2019-01-24
	rustup target add wasm32-unknown-unknown
	which wasm-bindgen || cargo install --force wasm-bindgen-cli --version "=0.2.33"

build: tools
	cargo build -p hcid --release
	cargo build -p hcid_js --target wasm32-unknown-unknown --release
	wasm-bindgen target/wasm32-unknown-unknown/release/hcid_js.wasm --out-dir hcid-js/lib --out-name bindgen --nodejs --no-typescript

clean:
	rm -rf hcid-js/rust/target hcid-js/lib/bindgen_bg.wasm
