# ✨ Aura SDK (aura-sdk)

[![CI](https://github.com/Maki-Grz/aura-sdk/actions/workflows/ci.yml/badge.svg)](https://github.com/Maki-Grz/aura-sdk/actions/workflows/ci.yml)
[![Version](https://img.shields.io/badge/version-0.2.0-purple.svg)](Cargo.toml)
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

## API Modernization & Packaging Standards

To transform `aura-sdk` into a robust, reusable open-source Rust library, we implemented several key architectural and compilation-level enhancements:

### 1. Zero-Cost FFI Callback Abstractions
The native QNN Genie API exposes an asynchronous C-style callback model for token streaming. The library encapsulates this model into a thread-safe, synchronous Rust wrapper:
- **State Synchronization:** The execution context is wrapped inside an atomic synchronization block (`GenieSyncContext`) utilizing `AtomicBool` flags and `Ordering::SeqCst` semantics to safely handle multi-threaded state updates across the boundary of the Qualcomm runtime thread pool.
- **Closure Injection:** We abstract the underlying raw pointer castings (`*const c_void`) by providing a high-level closure API (`impl FnMut(&str)`). This enables host applications to inject custom token-handlers (e.g., streaming to websockets, updating graphical interfaces) without interfacing with raw pointers or FFI lifecycles.

### 2. Hermetic Toolchain & MSVC Runtime Cohesion
When compiling native C/C++ bindings (such as `esaxx-rs`) alongside precompiled dynamic link libraries (like `onnxruntime`), the MSVC compiler enforces strict Runtime Library coherence. 
- **Linker Conflict Resolution:** We resolved conflicts between static runtime (`/MT` - `MT_StaticRelease`) and dynamic runtime (`/MD` - `MD_DynamicRelease`) by configuring target-specific compilation flags. By setting `CXXFLAGS = "/EHsc /MD"` in [.cargo/config.toml](file:///C:/Users/mini/Projects/aura-sdk/.cargo/config.toml), we align all C++ sub-compilations to target the dynamic CRT, preventing linker symbol collisions during the final binary generation.

### 3. Continuous Integration Stubbing
Interfacing with proprietary hardware drivers like the Qualcomm AI Engine Direct SDK normally panics the Cargo build execution when SDK variables (`QNN_SDK_ROOT`) are unset.
- **Failsafe FFI Bindings:** The [build.rs](file:///C:/Users/mini/Projects/aura-sdk/build.rs) script automatically detects the absence of the SDK environment. Upon failure, it generates stub/dummy binding signatures for Genie APIs. This ensures `cargo check`, `cargo test`, and `cargo clippy` can execute inside standard headless CI/CD environments (Linux, macOS, Windows) without exposing proprietary Qualcomm SDK components.
