use spirv_builder::{MetadataPrintout, ShaderPanicStrategy, SpirvBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SpirvBuilder::new("shader", "spirv-unknown-vulkan1.1")
        .print_metadata(MetadataPrintout::Full)
        .shader_panic_strategy(ShaderPanicStrategy::DebugPrintfThenExit {
            print_inputs: true,
            print_backtrace: true,
        })
        .build()?;
    Ok(())
}
