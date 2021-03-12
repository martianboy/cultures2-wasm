WBG=$PWD/../../Git/wasm-bindgen/target/release/wasm-bindgen
WOPT=$PWD/../../Git/binaryen/bin/wasm-opt

cargo +nightly build --lib --release --target "wasm32-unknown-unknown" -Z build-std=panic_abort,std
$WBG target/wasm32-unknown-unknown/release/cultures2_wasm.wasm --reference-types --typescript --out-dir ./pkg --target bundler

$WOPT pkg/cultures2_wasm_bg.wasm \
  -o pkg/cultures2_wasm_bg.wasm \
  -O3 \
  --enable-mutable-globals \
  --enable-reference-types
#   --enable-bulk-memory \
#   --enable-threads