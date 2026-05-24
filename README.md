# ✨ Aura SDK (aura-sdk)

[![Version](https://img.shields.io/badge/version-0.1.0-purple.svg)](Cargo.toml)
[![Platform](https://img.shields.io/badge/platform-Windows--ARM64-red.svg)](https://learn.microsoft.com/en-us/windows/arm/)
[![NPU](https://img.shields.io/badge/accelerator-Qualcomm--Hexagon-orange.svg)](https://www.qualcomm.com/products/technology/processors/snapdragon-x-plus)
## Project Overview & Abstract

The `aura-sdk` repository implements a low-latency, hardware-accelerated Artificial Intelligence (AI) inference engine engineered specifically for heterogeneous Windows-ARM64 architectures utilizing Qualcomm Snapdragon X Elite and Plus processors. The core objective of this system is to mitigate the high computational and memory bandwidth bottlenecks inherent in localized Large Language Model (LLM) autoregressive inference at the edge.

During the autoregressive generation phase, LLM inference transitions into a heavily memory-bound regime. The token generation latency $L_{\text{token}}$ for a model with parameter count $P$ and precision $b$ (bytes per parameter) is bounded by the sustained memory bandwidth $BW$ of the host architecture:

$$L_{\text{token}} \ge \frac{P \cdot b}{BW}$$

To optimize this execution path, this SDK abstracts dual execution paradigms that interface with distinct execution blocks of the Snapdragon SoC: a native, NPU-accelerated path leveraging the Qualcomm Hexagon Processor, and a decoupled CPU-bound fallback path utilizing the ONNX Runtime (ORT). The architecture addresses deep-level hardware constraints, driver abstraction layers, and memory-mapping strategies necessary to deliver predictable execution profiles for edge-deployed quantized language models.

---

## Core Architecture & Design Decisions

The internal architecture of `aura-sdk` is written in Rust to ensure memory safety and zero-cost abstractions when interfacing with native C-based hardware driver interfaces. The SDK establishes two mutually exclusive or complementary execution engines, selected based on operator compatibility and performance criteria.

```
                  +---------------------------------------+
                  |               Aura SDK                |
                  +---------------------------------------+
                                      |
                 Target Architecture Evaluation & Routing
                                      |
                 +--------------------+--------------------+
                 |                                         |
                 v                                         v
    [Native Genie Engine Engine]              [Aura Engine Backend (ORT)]
                 |                                         |
                 v                                         v
       Qualcomm AI Stack (QAIRT)                      ONNX Runtime
                 |                                         |
                 v                                         v
       Hexagon NPU Accelerator                    Oryon ARM64 CPU Cores
   (Compute Bound via HVX/HMX Tensor)         (Memory Bound via Cache Hierarchy)

```

### 1. Native Genie Engine (Qualcomm Hexagon NPU Acceleration)

The primary execution vector leverages the Qualcomm AI Engine Direct (QNN) framework via the Genie binary format. The design decision to isolate inference inside the Hexagon NPU is guided by energy-efficiency optimization and deterministic compute density. The Hexagon architecture utilizes Hexagon Vector eXtensions (HVX) and Hexagon Matrix eXtensions (HMX), enabling massive parallelization of fused multiply-add (FMA) operations required for tensor transformations.

* **Trade-off & Constraints:** Execution via the NPU introduces strict compile-time topology constraints. Models must be compiled into native Genie binaries, freezing the layer dimensions and execution graph. This eliminates dynamic shape adjustment but maximizes hardware register utilization.
* **Kernel Security Bypass:** To interface directly with the Hexagon processor without intermediate driver abstraction overhead, the system requires the deactivation of Kernel Memory Integrity (Virtualization-Based Security/Hypervisor-Protected Code Integrity). This design trade-off bypasses deep kernel validation layers to allow the unsigned user-space dynamic link libraries (`.dll`) to perform direct memory mappings into the Advanced Digital Signal Processor (ADSP) memory spaces.

### 2. Aura Engine (ONNX Runtime CPU Fallback)

As an alternative execution vector, the SDK integrates an ONNX Runtime abstraction layer compiled for the `aarch64-pc-windows-msvc` target.

* **Architectural Rationale:** The Hexagon NPU exhibits limited operator coverage; complex or non-standard activation functions and tensor operations fail to compile into the Genie format. The Aura Engine routes these workloads to the Qualcomm Oryon CPU cores. This architecture serves as an evaluation baseline and prototyping pipeline, guaranteeing functional execution for arbitrary standard ONNX computational graphs at the cost of higher thermal dissipation and increased CPU cycle consumption.

---

## Algorithmic Design & Data Flow

The operational execution flow profiles data transition from raw token sequence processing to matrix multiplication acceleration within the execution hardware.

```
[Prompt Input] -> [Tokenization & Graph Assembly] -> [Engine Routing Evaluation]
                                                            |
                  +-----------------------------------------+
                  |
                  +---> (Genie Mode) --> [Load Genie Weights] -> [Zero-Copy Share to NPU] -> [Hexagon Execution]
                  |
                  +---> (ORT Mode)   --> [Instantiate Session] -> [Allocate CPU Heap]     -> [Oryon Core Execution]

```

### Algorithmic Execution Sequences

#### Step 1: Memory Allocation and Context Initialization

Upon initialization, the SDK evaluates environment vectors (`QNN_SDK_ROOT` and `ADSP_LIBRARY_PATH`). For NPU execution, it instantiates an isolated hardware context. The model weight tensors are memory-mapped directly into memory regions accessible by the NPU's internal Direct Memory Access (DMA) controllers, minimizing CPU-to-NPU serialization penalties.

#### Step 2: Autoregressive Token Generation Loop

For each generated token $t_n$ given an antecedent context sequence $t_1, \dots, t_{n-1}$, the execution graph computes the attention matrices. The system optimizes the computational complexity of the scaled dot-product attention:

$$\text{Attention}(Q, K, V) = \text{softmax}\left(\frac{QK^T}{\sqrt{d_k}}\right)V$$

* **In Genie NPU Mode:** The matrix multiplication operations are translated into fixed-point vector arithmetic within the Hexagon tensor processor.
* **In ORT CPU Mode:** The operations are dispatched as multi-threaded instructions utilizing ARM Neon SIMD assembly instructions mapped across the Qualcomm Oryon CPU core matrix.

---

## Technical Specifications & Performance Metrics

The benchmarking metrics demonstrate the separation between compute-bound execution (NPU) and memory-bound execution (CPU fallback) across different quantization profiles on Snapdragon X Plus/Elite silicon.

### Quantization and Precision Profiles

To maximize the utilization of the Hexagon hardware and Oryon L1/L2 cache structures, models are restricted to tight quantization levels. The framework primarily runs:

1. **`w4a16` (Genie Native):** 4-bit integer weights coupled with 16-bit floating-point activations. This minimizes memory size while maintaining structural convergence properties.
2. **`int8` / `q4` (ORT Native):** 8-bit and 4-bit uniform quantization layers evaluated on the CPU core matrix.

### Performance Throughput Metrics

The system yields distinct Token-per-Second (TPS) profiles based on the selected execution block and parameter dimensions:

| Execution Engine | Target Subsystem | Model Scale / Configuration | Throughput Profile (TPS) | Performance Regime |
| --- | --- | --- | --- | --- |
| **Genie Engine** | Hexagon NPU | Specialized Graph (`w4a16`) | $10 - 35$ | Compute & Driver Bound |
| **Aura Engine** | Oryon CPU | 1B Parameter (`int8`/`q4`) | $\sim 50$ | Memory Bandwidth Bound |

The seemingly high throughput of the CPU engine for 1B parameter configurations is mathematically justified by the high L2/L3 cache hit rate of the Oryon cores for small-footprint models. When the total model size $M_{\text{model}}$ sits entirely within the shared processor cache hierarchy:

$$M_{\text{model}} < \text{Cache}_{\text{Size}}$$

The memory bandwidth constraint $BW$ shifts from external LPDDR5X channels to internal silicon caches, drastically lowering retrieval latency. For larger model architectures, the NPU path scales efficiently due to dedicated matrix multipliers, whereas the CPU engine exhibits steep degradation curves due to cache eviction penalties.

---

## Deployment & Computational Requirements

### System Dependencies & Hardware Prerequisites

* **Host OS Architecture:** Windows 11 ARM64 (`aarch64-pc-windows-msvc`).
* **Kernel Requirements:** Virtualization-Based Security (VBS) Core Isolation Memory Integrity must be explicitly disabled to allow user-space driver access to Hexagon registers.
* **Driver Layer:** Qualcomm NPU Driver Suite $\ge \text{v30.0.140.x}$.
* **Linkage SDK:** Qualcomm AI Stack (QAIRT SDK v2.45.40) providing runtime dependencies for `QNN`.

### Compilation Parameters

The compilation environment isolates dependencies via conditional compilation flags managed by Cargo.

```powershell
# Profile 1: Compilation optimized strictly for Native NPU Genie Target
cargo build --release

# Profile 2: Compilation including ONNX Runtime dependencies for CPU execution
cargo build --release --features aura-engine

```

### Runtime Environment Configuration and Execution

Prior to binary execution, the system runtime requires explicit path mapping to load the unsigned hardware-specific dynamic libraries:

```powershell
# Expose the Hexagon runtime library directory to the system environment
$env:ADSP_LIBRARY_PATH = "C:\Qualcomm\AIStack\QAIRT\2.45.40.260406\lib\hexagon-v73\unsigned"

# Append QNN binary dependency paths to the standard environment path
$env:PATH = "C:\Qualcomm\AIStack\QAIRT\2.45.40.260406\lib\aarch64-windows-msvc;" + $env:PATH

# Execute NPU-Accelerated Native Engine Inference
.\target\release\aura-sdk.exe --prompt "Analyze hardware acceleration vectors."

# Execute CPU-Bound Decoupled ONNX Runtime Engine Inference
.\target\release\aura-sdk.exe --ort --model ./models/Llama-1B-ONNX/model_q4.onnx --prompt "Evaluate execution."

```
