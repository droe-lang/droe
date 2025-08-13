use std::fs;
use std::io::Write;
use std::path::Path;
use anyhow::Result;

const VM_BINARY_MARKER: &[u8] = b"__ROEBC_DATA_START__";
const VM_BINARY_END_MARKER: &[u8] = b"__ROEBC_DATA_END__";

pub fn embed_bytecode_in_binary(vm_binary_path: &Path, bytecode_path: &Path, output_path: &Path) -> Result<()> {
    // Read the VM binary
    let mut vm_data = fs::read(vm_binary_path)?;
    
    // Read the bytecode file
    let bytecode_data = fs::read(bytecode_path)?;
    
    // Create the embedded data section
    let mut embedded_section = Vec::new();
    embedded_section.extend_from_slice(VM_BINARY_MARKER);
    embedded_section.extend_from_slice(&(bytecode_data.len() as u64).to_le_bytes());
    embedded_section.extend_from_slice(&bytecode_data);
    embedded_section.extend_from_slice(VM_BINARY_END_MARKER);
    
    // Append to VM binary
    vm_data.extend_from_slice(&embedded_section);
    
    // Write the output binary
    fs::write(output_path, vm_data)?;
    
    // Make executable on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(output_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(output_path, perms)?;
    }
    
    Ok(())
}

pub fn extract_embedded_bytecode() -> Result<Option<Vec<u8>>> {
    // Get the current executable path
    let exe_path = std::env::current_exe()?;
    let exe_data = fs::read(&exe_path)?;
    
    // Find the marker (look for the last occurrence to avoid false positives)
    if let Some(start_pos) = find_marker(&exe_data, VM_BINARY_MARKER) {
        let data_start = start_pos + VM_BINARY_MARKER.len();
        
        // Read the length
        if data_start + 8 <= exe_data.len() {
            let length = u64::from_le_bytes([
                exe_data[data_start],
                exe_data[data_start + 1],
                exe_data[data_start + 2],
                exe_data[data_start + 3],
                exe_data[data_start + 4],
                exe_data[data_start + 5],
                exe_data[data_start + 6],
                exe_data[data_start + 7],
            ]) as usize;
            
            let bytecode_start = data_start + 8;
            let bytecode_end = bytecode_start + length;
            
            if bytecode_end <= exe_data.len() {
                return Ok(Some(exe_data[bytecode_start..bytecode_end].to_vec()));
            }
        }
    }
    
    Ok(None)
}

fn find_marker(data: &[u8], marker: &[u8]) -> Option<usize> {
    // Find the LAST occurrence to avoid false positives in the main executable
    data.windows(marker.len()).rposition(|window| window == marker)
}