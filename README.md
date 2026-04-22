# qnn-bindings-rs

Rust bindings and a minimal CLI runner for Qualcomm Genie/QNN inference on Snapdragon NPUs (Windows ARM64 target).

## Description

This repository exposes Genie C APIs to Rust through `bindgen` and provides a command-line executable that:

- Loads a Genie JSON configuration (`genie_config.json`)
- Sends a prompt to the runtime
- Streams token output
- Reports basic runtime KPIs (generated tokens, TTFT, TPS)

The project is intended for technical experimentation and integration workflows around on-device LLM inference on Qualcomm platforms.

## Technical Scope

- Target runtime: **QAIRT / QNN + Genie**
- Target platform: **Windows ARM64 (Snapdragon X Elite / X Plus)**
- Binding generation: **build-time via `bindgen`**
- Runtime backend: **HTP**

Detailed setup and conversion steps are documented in [QNN_STEPS.md](QNN_STEPS.md).

## Build Requirements

1. Rust toolchain (stable)
2. QAIRT/QNN SDK installed locally
3. `QNN_SDK_ROOT` environment variable set to SDK root

Example:

```powershell
$env:QNN_SDK_ROOT = "C:\Qualcomm\AIStack\QAIRT\2.31.0.250130"
cargo build --release
```

## Run

```powershell
.\target\release\qnn-bindings-rs.exe "Explain quantum physics in one sentence."
```

`genie_config.json` must be present in the working directory.

## Metadata for Public GitHub

- **Author:** mini
- **License:** MIT (see [LICENSE](LICENSE))
- **Description:** Rust bindings and CLI runner for Qualcomm Genie/QNN on Snapdragon NPUs
- **Suggested GitHub topics (tags):** `rust`, `qnn`, `genie-sdk`, `snapdragon`, `llm`, `npu`, `windows-arm64`

## Licensing Notes

This repository is MIT-licensed for its own source code. Qualcomm SDK binaries, model files, and other third-party assets remain governed by their respective licenses and terms.
