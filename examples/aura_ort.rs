#[cfg(feature = "aura-engine")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use aura_sdk::engines::AuraEngine;
    use std::path::Path;

    let model_path = Path::new("Llama-1B-ONNX/onnx/model_q4.onnx");
    let mut engine = AuraEngine::new(model_path, "43")?;
    engine.query("Explain NPUs in one sentence.", 512, |token| {
        print!("{}", token);
        use std::io::Write;
        let _ = std::io::stdout().flush();
    })?;
    println!();
    Ok(())
}

#[cfg(not(feature = "aura-engine"))]
fn main() {
    println!("Please run this example with the `aura-engine` feature enabled:");
    println!("cargo run --example aura_ort --features aura-engine");
}
