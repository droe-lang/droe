def compile_roe_to_wat(input_path, output_path):
    input_path = str(input_path)
    output_path = str(output_path)

    with open(input_path, "r") as f:
        line = f.read().strip()

    if not line.startswith("Display "):
        raise ValueError("Only 'Display <text>' is supported.")

    message = line[len("Display "):]
    escaped = message.replace('"', '\\"')

    wat = f"""(module
  (import "env" "print" (func $print (param i32 i32)))
  (memory (export "memory") 1)
  (data (i32.const 0) "{escaped}")
  (func (export "main")
    i32.const 0
    i32.const {len(message)}
    call $print
  )
)"""

    with open(output_path, "w") as f:
        f.write(wat)

    print(f"✅ Compiled {input_path} to {output_path}")
