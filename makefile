.PHONY: build clean bundle-web bundle-linux bundle-windows

.DEFAULT_GOAL := build

build:
	cargo build --release

clean:
	cargo clean
	rm -rf .vite .cargo target bin __pycache__

bundle-web: build
	w4 bundle target/wasm32-unknown-unknown/release/cart.wasm --html bin/index.html --title "WASM4 Adventure"

bundle-linux: build
	w4 bundle target/wasm32-unknown-unknown/release/cart.wasm --linux bin/adventure

bundle-windows: build
	w4 bundle target/wasm32-unknown-unknown/release/cart.wasm --windows bin/adventure.exe

watch:
	w4 watch