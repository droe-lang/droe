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
      print_no_newline: (offset, length) => {
        // Print without adding a newline
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer, offset, length);
        const text = new TextDecoder("utf-8").decode(bytes);
        process.stdout.write(text);
      },
      print_newline: () => {
        // Print just a newline
        console.log();
      },
      print_i32_no_newline: (value) => {
        // Print an integer value without newline
        process.stdout.write(value.toString());
      },
      print_string_from_offset_no_newline: (offset) => {
        // Print a null-terminated string without newline
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        let length = 0;
        
        // Find the null terminator
        while (bytes[offset + length] !== 0 && offset + length < bytes.length) {
          length++;
        }
        
        // Decode and print the string without newline
        const stringBytes = new Uint8Array(memory.buffer, offset, length);
        const text = new TextDecoder("utf-8").decode(stringBytes);
        process.stdout.write(text);
      },
      print_decimal: (scaledValue) => {
        // Print a scaled decimal value (scaled by 100) with proper formatting
        const integerPart = Math.floor(scaledValue / 100);
        const fractionalPart = Math.abs(scaledValue % 100);
        const formattedDecimal = `${integerPart}.${fractionalPart.toString().padStart(2, '0')}`;
        console.log(formattedDecimal);
      },
      print_decimal_no_newline: (scaledValue) => {
        // Print a scaled decimal value without newline
        const integerPart = Math.floor(scaledValue / 100);
        const fractionalPart = Math.abs(scaledValue % 100);
        const formattedDecimal = `${integerPart}.${fractionalPart.toString().padStart(2, '0')}`;
        process.stdout.write(formattedDecimal);
      },
      print_date: (offset) => {
        // Print a date from a string stored in memory
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        let length = 0;
        
        // Find the null terminator
        while (bytes[offset + length] !== 0 && offset + length < bytes.length) {
          length++;
        }
        
        // Decode the date string and format it
        const stringBytes = new Uint8Array(memory.buffer, offset, length);
        const dateString = new TextDecoder("utf-8").decode(stringBytes);
        
        // For now, just display the date string as-is
        // In a full implementation, this could parse and reformat the date
        console.log(dateString);
      },
      print_date_no_newline: (offset) => {
        // Print a date from a string stored in memory without newline
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        let length = 0;
        
        // Find the null terminator
        while (bytes[offset + length] !== 0 && offset + length < bytes.length) {
          length++;
        }
        
        // Decode the date string and format it
        const stringBytes = new Uint8Array(memory.buffer, offset, length);
        const dateString = new TextDecoder("utf-8").decode(stringBytes);
        
        // Display the date string without newline
        process.stdout.write(dateString);
      },
      format_date: (dateOffset, patternOffset) => {
        // Format a date string according to a pattern
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        
        // Get the date string
        let dateLength = 0;
        while (bytes[dateOffset + dateLength] !== 0 && dateOffset + dateLength < bytes.length) {
          dateLength++;
        }
        const dateBytes = new Uint8Array(memory.buffer, dateOffset, dateLength);
        const dateString = new TextDecoder("utf-8").decode(dateBytes);
        
        // Get the pattern string
        let patternLength = 0;
        while (bytes[patternOffset + patternLength] !== 0 && patternOffset + patternLength < bytes.length) {
          patternLength++;
        }
        const patternBytes = new Uint8Array(memory.buffer, patternOffset, patternLength);
        const pattern = new TextDecoder("utf-8").decode(patternBytes);
        
        // Format the date based on pattern
        let formatted = dateString;
        try {
          const date = new Date(dateString);
          if (pattern === "MM/dd/yyyy") {
            const month = (date.getMonth() + 1).toString().padStart(2, '0');
            const day = date.getDate().toString().padStart(2, '0');
            const year = date.getFullYear();
            formatted = `${month}/${day}/${year}`;
          } else if (pattern === "dd/MM/yyyy") {
            const month = (date.getMonth() + 1).toString().padStart(2, '0');
            const day = date.getDate().toString().padStart(2, '0');
            const year = date.getFullYear();
            formatted = `${day}/${month}/${year}`;
          } else if (pattern === "MMM dd, yyyy") {
            const months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun",
                          "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
            const month = months[date.getMonth()];
            const day = date.getDate();
            const year = date.getFullYear();
            formatted = `${month} ${day}, ${year}`;
          } else if (pattern === "long") {
            formatted = date.toLocaleDateString('en-US', { 
              weekday: 'long', 
              year: 'numeric', 
              month: 'long', 
              day: 'numeric' 
            });
          }
          // Default: return original format if pattern not recognized
        } catch (e) {
          // If date parsing fails, return original string
        }
        
        // Store the formatted result in memory and return its offset
        const formattedBytes = new TextEncoder().encode(formatted + '\0');
        const resultOffset = memory.buffer.byteLength - 1024; // Use end of memory
        const memoryBytes = new Uint8Array(memory.buffer);
        memoryBytes.set(formattedBytes, resultOffset);
        return resultOffset;
      },
      format_decimal: (scaledValue, patternOffset) => {
        // Format a decimal value according to a pattern
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        
        // Get the pattern string
        let patternLength = 0;
        while (bytes[patternOffset + patternLength] !== 0 && patternOffset + patternLength < bytes.length) {
          patternLength++;
        }
        const patternBytes = new Uint8Array(memory.buffer, patternOffset, patternLength);
        const pattern = new TextDecoder("utf-8").decode(patternBytes);
        
        // Convert scaled value to decimal
        const integerPart = Math.floor(scaledValue / 100);
        const fractionalPart = Math.abs(scaledValue % 100);
        
        let formatted;
        if (pattern === "0.00") {
          formatted = `${integerPart}.${fractionalPart.toString().padStart(2, '0')}`;
        } else if (pattern === "#,##0.00") {
          formatted = `${integerPart.toLocaleString()}.${fractionalPart.toString().padStart(2, '0')}`;
        } else if (pattern === "$0.00") {
          formatted = `$${integerPart}.${fractionalPart.toString().padStart(2, '0')}`;
        } else {
          // Default formatting
          formatted = `${integerPart}.${fractionalPart.toString().padStart(2, '0')}`;
        }
        
        // Store the formatted result in memory and return its offset
        const formattedBytes = new TextEncoder().encode(formatted + '\0');
        const resultOffset = memory.buffer.byteLength - 2048; // Use end of memory
        const memoryBytes = new Uint8Array(memory.buffer);
        memoryBytes.set(formattedBytes, resultOffset);
        return resultOffset;
      },
      format_number: (value, patternOffset) => {
        // Format an integer value according to a pattern
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        
        // Get the pattern string
        let patternLength = 0;
        while (bytes[patternOffset + patternLength] !== 0 && patternOffset + patternLength < bytes.length) {
          patternLength++;
        }
        const patternBytes = new Uint8Array(memory.buffer, patternOffset, patternLength);
        const pattern = new TextDecoder("utf-8").decode(patternBytes);
        
        let formatted;
        if (pattern === "#,##0") {
          formatted = value.toLocaleString();
        } else if (pattern === "0000") {
          formatted = value.toString().padStart(4, '0');
        } else if (pattern === "hex") {
          formatted = "0x" + value.toString(16).toUpperCase();
        } else {
          // Default formatting
          formatted = value.toString();
        }
        
        // Store the formatted result in memory and return its offset
        const formattedBytes = new TextEncoder().encode(formatted + '\0');
        const resultOffset = memory.buffer.byteLength - 3072; // Use end of memory
        const memoryBytes = new Uint8Array(memory.buffer);
        memoryBytes.set(formattedBytes, resultOffset);
        return resultOffset;
      },
      // Core library functions
      string_concat: (str1Offset, str2Offset) => {
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        
        // Get first string
        let str1Length = 0;
        while (bytes[str1Offset + str1Length] !== 0) str1Length++;
        const str1 = new TextDecoder().decode(new Uint8Array(memory.buffer, str1Offset, str1Length));
        
        // Get second string
        let str2Length = 0;
        while (bytes[str2Offset + str2Length] !== 0) str2Length++;
        const str2 = new TextDecoder().decode(new Uint8Array(memory.buffer, str2Offset, str2Length));
        
        // Concatenate and store result
        const result = str1 + str2;
        const resultBytes = new TextEncoder().encode(result + '\0');
        const resultOffset = memory.buffer.byteLength - 4096;
        const memoryBytes = new Uint8Array(memory.buffer);
        memoryBytes.set(resultBytes, resultOffset);
        return resultOffset;
      },
      string_length: (strOffset) => {
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        let length = 0;
        while (bytes[strOffset + length] !== 0) length++;
        return length;
      },
      string_substring: (strOffset, start, length) => {
        const memory = instance.exports.memory;
        const bytes = new Uint8Array(memory.buffer);
        
        // Get original string
        let strLength = 0;
        while (bytes[strOffset + strLength] !== 0) strLength++;
        const str = new TextDecoder().decode(new Uint8Array(memory.buffer, strOffset, strLength));
        
        // Extract substring
        const result = str.substring(start, start + length);
        const resultBytes = new TextEncoder().encode(result + '\0');
        const resultOffset = memory.buffer.byteLength - 5120;
        const memoryBytes = new Uint8Array(memory.buffer);
        memoryBytes.set(resultBytes, resultOffset);
        return resultOffset;
      },
      // Math functions
      math_abs_i32: (value) => Math.abs(value),
      math_abs_decimal: (scaledValue) => Math.abs(scaledValue),
      math_max_i32: (a, b) => Math.max(a, b),
      math_min_i32: (a, b) => Math.min(a, b),
      math_power_i32: (base, exp) => Math.pow(base, exp),
      math_decimal_multiply: (a, b) => Math.round((a * b) / 100),
      math_decimal_divide: (a, b) => b !== 0 ? Math.round((a * 100) / b) : 0,
      math_sqrt_decimal: (scaledValue) => Math.round(Math.sqrt(scaledValue / 100) * 100),
    },
  };

  const { instance } = await WebAssembly.instantiate(wasmBuffer, importObject);

  instance.exports.main();
})();
