cargo build --lib --release --target "wasm32-unknown-unknown"
$HOME/.cargo/bin/wasm-bindgen target/wasm32-unknown-unknown/release/cultures2_wasm.wasm --typescript --out-dir ./pkg --target bundler
../../Git/binaryen/bin/wasm-opt pkg/cultures2_wasm_bg.wasm -o pkg/cultures2_wasm_bg.wasm -O3 --enable-mutable-globals 