# ✨ Aura SDK (aura-sdk)

[![CI](https://github.com/Maki-Grz/aura-sdk/actions/workflows/ci.yml/badge.svg)](https://github.com/Maki-Grz/aura-sdk/actions/workflows/ci.yml)
[![Version](https://img.shields.io/badge/version-0.2.0-purple.svg)](Cargo.toml)
[![Platform](https://img.shields.io/badge/platform-Windows--ARM64-red.svg)](https://learn.microsoft.com/en-us/windows/arm/)
[![NPU](https://img.shields.io/badge/accelerator-Qualcomm--Hexagon-orange.svg)](https://www.qualcomm.com/products/technology/processors/snapdragon-x-plus)

Aura SDK is a high-performance AI inference engine library and CLI for **Snapdragon X Elite / Plus** platforms.
It supports two execution modes:

- **Native Genie Engine** – Uses the Qualcomm NPU (Hexagon) for best performance, requires models in Genie binary format.
- **Aura Engine (ORT)** – ONNX Runtime backend with QNN execution provider support.

For in-depth hardware architecture analysis and algorithmic details, see the [RESEARCH.md](RESEARCH.md) file.

---

## ⚙️ Prerequisites & Drivers

- **NPU Driver**: Version **30.0.140.x** or higher (check in Device Manager).
- **⚠️ CRITICAL**: Disable **Memory Integrity** in *Windows Security > Device Security > Core Isolation* and restart.
- **QAIRT SDK**: Install [Qualcomm AI Stack](https://softwarecenter.qualcomm.com/catalog/catalog-suite/Qualcomm%C2%AE%20AI%20Stack) (v2.45.40 recommended).
- **Environment variables** (optional, the SDK will try to set them automatically):
  - `QNN_SDK_ROOT` – path to QAIRT installation.
  - `ADSP_LIBRARY_PATH` – path to Hexagon unsigned libraries (e.g. `...\lib\hexagon-v73\unsigned`).

---

## 📦 Using as a Library

Add `aura-sdk` to your `Cargo.toml`:

```toml
[dependencies]
aura-sdk = { version = "0.2.0" }
```

To enable the ONNX Runtime support (Aura Engine), add the `aura-engine` feature:

```toml
[dependencies]
aura-sdk = { version = "0.2.0", features = ["aura-engine"] }
```

### 1. Genie Engine (Native NPU)

```rust
use std::path::Path;
use aura_sdk::engines::GenieEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = Path::new("phi_3_5_mini_instruct-genie-w4a16-qualcomm/genie_config.json");
    let engine = GenieEngine::new(config_path)?;
    
    engine.query_sync("Explain NPUs in one sentence.", 512, |token| {
        print!("{}", token);
    })?;
    
    Ok(())
}
```

### 2. Aura Engine (ONNX Runtime Mode)

```rust
use std::path::Path;
use aura_sdk::engines::AuraEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model_path = Path::new("Llama-1B-ONNX/onnx/model_q4.onnx");
    let mut engine = AuraEngine::new(model_path, "43")?;
    
    engine.query("Explain NPUs in one sentence.", 512, |token| {
        print!("{}", token);
    })?;
    
    Ok(())
}
```

### 3. Running the Examples

You can run the examples directly from the command line:

```powershell
# Run Genie NPU example
cargo run --example genie_npu

# Run Aura ORT example (requires aura-engine feature)
cargo run --example aura_ort --features aura-engine
```

---

## 🛠️ Building the CLI

```powershell
# Build native engine CLI only (Genie NPU mode)
cargo build --release

# Build CLI with ONNX Runtime support
cargo build --release --features aura-engine
```

---

## 🚀 Running the CLI

### 1. Native Genie Engine (NPU – recommended)

```powershell
.\target\release\aura-sdk.exe --prompt "Explain the benefits of NPU."
```

### 2. Aura Engine (ORT)

```powershell
.\target\release\aura-sdk.exe --ort --model Llama-1B-ONNX/onnx/model_q4.onnx --prompt "Explain NPU in 3 sentences."
```

---

## 📜 License

MIT License
