rm -rf target
rm web/*.wasm

echo "Compiling binaries....";
cargo +nightly build --target wasm32-unknown-unknown --release
wasm-gc target/wasm32-unknown-unknown/release/distil_wasm.wasm -o web/distil_wasm.gc.wasm

cp index.html web/

