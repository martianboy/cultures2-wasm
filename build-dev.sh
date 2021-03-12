WBG=$PWD/../../Git/wasm-bindgen/target/release/wasm-bindgen

cargo +nightly build --lib --target "wasm32-unknown-unknown" -Z build-std=panic_abort,std
$WBG --debug target/wasm32-unknown-unknown/debug/cultures2_wasm.wasm --typescript --out-dir ./pkg --target bundler
