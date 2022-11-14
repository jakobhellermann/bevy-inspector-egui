.PHONY: build

CRATE_NAME=bevy-inspector-egui-demo
FLAGS=--release
OUT_DIR=docs/demo

WASM_FILE=~/.cache/rust/wasm32-unknown-unknown/release/${CRATE_NAME}.wasm

serve: demo
	basic-http-server ${OUT_DIR}

demo: optimize

optimize: build
	wasm-bindgen ${WASM_FILE} --out-dir ${OUT_DIR} --target web --out-name wasm --no-typescript --remove-name-section --remove-producers-section
	@du -h ${OUT_DIR}/wasm_bg.wasm
	wasm-opt ${OUT_DIR}/wasm_bg.wasm -o ${OUT_DIR}/wasm_bg.wasm -O2
	@du -h ${OUT_DIR}/wasm_bg.wasm

build: src/main.rs Cargo.toml
	cargo build --target wasm32-unknown-unknown ${FLAGS}
	@du -h ${WASM_FILE}


clean:
	rm ${OUT_DIR}/wasm*
