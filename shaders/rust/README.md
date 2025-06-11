# Rust-GPU Shaders

This folder contains shaders written in Rust using [rust-gpu](https://github.com/Rust-GPU/rust-gpu). Rust-GPU allows writing GPU shaders in Rust, which are then compiled to SPIR-V.

## Requirements

- Rust toolchain (install from https://rustup.rs/)
- cargo-gpu (automatically installed by the compilation script)
- Vulkan 1.2 or higher

## Compilation

To compile the shaders, run:

```bash
python compileshaders.py
```

This script will:
1. Install cargo-gpu if not already installed
2. Use cargo-gpu to build each shader crate
3. Generate separate .spv files for each entry point (vertex, fragment, etc.)

Alternatively, you can compile individual shaders manually:

```bash
cargo gpu build --shader-crate triangle --output-dir triangle
```

## Structure

Each shader example is organized as a Rust crate with individual binaries for each shader stage:
- `src/vertex.rs` - Vertex shader
- `src/fragment.rs` - Fragment shader
- `src/compute.rs` - Compute shader (if applicable)

The compiled SPIR-V files follow the same naming convention as other shader languages:
- `<example>.vert.spv` - Vertex shader
- `<example>.frag.spv` - Fragment shader
- `<example>.comp.spv` - Compute shader

## Notes

- rust-gpu is still experimental and may not support all Vulkan features
- Some examples may not be portable to rust-gpu due to language limitations
- The rust-gpu compiler requires a specific nightly toolchain version