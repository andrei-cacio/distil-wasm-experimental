echo "Compiling binaries....";
rustc --target=wasm32-unknown-emscripten src/lib.rs -o web/index.html

cp index.html web/

echo "Powering up a HTTP server"
cd web
python -m SimpleHTTPServer 8000
echo "Open http://localhost:8000";
