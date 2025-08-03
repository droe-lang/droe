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
      print_i32: (value) => {
        // Print an integer value directly
        console.log(value);
      },
      print_string_from_offset: (offset) => {
        // Print a null-terminated string from memory starting at offset
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        let length = 0;
        
        // Find the null terminator
        while (bytes[offset + length] !== 0 && offset + length < bytes.length) {
          length++;
        }
        
        // Decode and print the string
        const stringBytes = new Uint8Array(memory.buffer, offset, length);
        const text = new TextDecoder("utf-8").decode(stringBytes);
        console.log(text);
      },
    },
  };

  const { instance } = await WebAssembly.instantiate(wasmBuffer, importObject);

  instance.exports.main();
})();
