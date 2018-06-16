# distil-wasm
A WebAssembly version of [Elliot Jackson](https://github.com/elliotekj)'s library called [distil](https://github.com/elliotekj/distil)

# Temporarily abandoned

I managed to render the results from the rust library. However, I cannot figure out why I am getting only black palettes, no matter what image I feed the algorithm.

Here is the unanswered Stackoverflow question: [color_quant::NeuQuant compiled to WebAssembly outputs zero values
](https://stackoverflow.com/questions/50553014/color-quantneuquant-compiled-to-webassembly-outputs-zero-values)

# Things I have learned threw the process

All the notes and references which resulted from this project can be found here: [learning-web-assembly](https://github.com/andrei-cacio/learning-web-assembly)

# Development

To setup a rust environment with WebAssembly superpowers I used this wonderful resource right [here](https://rust-lang-nursery.github.io/rust-wasm/setup.html).

# Run (Mac/Linux)

```bash
$ sh run.sh
```

This script will compile and power up an HTTP server via Python cli.

# Bibliography
1. https://hacks.mozilla.org/2018/03/making-webassembly-better-for-rust-for-all-languages/
2. https://rust-lang-nursery.github.io/rust-wasm/
3. https://hacks.mozilla.org/2018/04/javascript-to-rust-and-back-again-a-wasm-bindgen-tale/
4. https://kripken.github.io/emscripten-site/docs/api_reference/module.html#module
5. https://www.hellorust.com/
