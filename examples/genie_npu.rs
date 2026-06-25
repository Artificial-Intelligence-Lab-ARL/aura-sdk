use aura_sdk::engines::GenieEngine;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = Path::new("phi_3_5_mini_instruct-genie-w4a16-qualcomm/genie_config.json");
    let engine = GenieEngine::new(config_path)?;
    engine.query_sync("Explain NPUs in one sentence.", 512, |token| {
        print!("{}", token);
        use std::io::Write;
        let _ = std::io::stdout().flush();
    })?;
    println!();
    Ok(())
}
