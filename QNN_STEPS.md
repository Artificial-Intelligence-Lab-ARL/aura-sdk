# End-to-End Guide: LLM Inference on Snapdragon X Elite NPU with Genie SDK

This guide describes the conversion pipeline and runtime execution of an LLM from Rust on Windows ARM64 using QAIRT 2.31.

## 1. Architecture and Compatibility

- **Hardware**: Qualcomm Snapdragon X Elite (v73) / X Plus
- **SDK**: QAIRT 2.31.0.250130
- **Model artifact**: QNN Context Binary (`.bin`)
- **Backend**: HTP (Hexagon Tensor Processor)

---

## 2. Environment Variables

Configure your PowerShell environment (`x86_64` host for conversion, ARM64 runtime for execution):

```powershell
$env:QNN_SDK_ROOT = "C:\Qualcomm\AIStack\QAIRT\2.31.0.250130"
# Conversion toolchain (x86_64)
$env:PATH = "$env:QNN_SDK_ROOT\bin\x86_64-windows-msvc;" + $env:PATH
# Runtime dependencies (ARM64)
$env:PATH = "C:\Users\mini\Projects\qnn-bindings-rs\model_bin\genie_bundle;" + `
            "$env:QNN_SDK_ROOT\lib\aarch64-windows-msvc;" + $env:PATH
```

---

## 3. Conversion Pipeline (ONNX -> QNN `.bin`)

### Step A: ONNX -> QNN IR (x86_64)
```powershell
python $env:QNN_SDK_ROOT\bin\x86_64-windows-msvc\qnn-onnx-converter `
    --input_network model.onnx `
    --output_path model_qnn/model `
    --preserve_io
```

### Step B: Compile generated source into an ARM64 DLL
Build `model.dll` from the converter output (`.cpp`).

### Step C: Generate QNN Context Binary (`.bin`)
```powershell
$env:QNN_SDK_ROOT\bin\aarch64-windows-msvc\qnn-context-binary-generator.exe `
    --model model_qnn/model.dll `
    --backend $env:QNN_SDK_ROOT\lib\aarch64-windows-msvc\QnnHtp.dll `
    --output_dir model_bin/ `
    --binary_file llama_v3_model
```

---

## 4. Genie Configuration (`genie_config.json`)

`htp_backend_ext_config.json` should contain:
```json
{ "devices": [{ "soc_model": 43, "dsp_arch": "v73" }] }
```

`genie_config.json` should reference your generated `.bin` files:
```json
{
  "dialog": {
    "engine": {
      "backend": {
        "type": "QnnHtp",
        "extensions": "htp_backend_ext_config.json"
      },
      "model": {
        "type": "binary",
        "binary": { "ctx-bins": ["llama_v3_model.bin"] }
      }
    }
  }
}
```

---

## 5. Build and Runtime Metrics

### Build the Rust project
```powershell
cargo build --release
```

### Run with a custom prompt
```powershell
.\target\release\qnn-bindings-rs.exe "What is the meaning of life?"
```

### Expected output
The program reports:
- **Inference output** in streaming mode
- **Generated token count**
- **TTFT** (Time To First Token)
- **TPS** (Tokens Per Second)
