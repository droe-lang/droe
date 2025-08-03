const fs = require("fs");
const path = process.argv[2];

(async () => {
  const wasmBuffer = fs.readFileSync(path);

  const importObject = {
    env: {
      // No memory passed from JS — let WASM module define and export it
      print: (offset, length) => {
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer, offset, length);
        const text = new TextDecoder("utf-8").decode(bytes);
        console.log(text);
      },
    },
  };

  const { instance } = await WebAssembly.instantiate(wasmBuffer, importObject);

  instance.exports.main();
})();
