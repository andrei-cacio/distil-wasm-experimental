rm -rf target
rm web/distil.js
rm web/*.wasm

echo "Compiling binaries....";
rustc --target=wasm32-unknown-emscripten src/lib.rs -o web/index.html

