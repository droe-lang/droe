// Build script for DroeVM
// No protobuf compilation needed for core VM functionality

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // DroeVM core doesn't need protobuf compilation
    // If LLM client feature is enabled, protobuf compilation would go here
    #[cfg(feature = "llm-client")]
    {
        tonic_build::compile_protos("proto/llm_service.proto")?;
    }
    
    Ok(())
}