rm -rf target
rm distil.gc.wasm

echo "Compiling binaries....";
cargo +nightly build --target wasm32-unknown-unknown --release
wasm-gc target/wasm32-unknown-unknown/release/distil.wasm -o distil.gc.wasm

echo "Copying wasm to web"
cp distil.gc.wasm web/distil.wasm

echo "Powering up a HTTP server"
cd web
python -m SimpleHTTPServer 8000
