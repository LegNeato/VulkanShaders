# Shaders

This folder contains the shaders used by the samples. Source files are available in GLSL, HLSL, [slang](https://shader-slang.org/) and [rust-gpu](https://github.com/Rust-GPU/rust-gpu) and also come with precompiled SPIR-V files that are consumed by the samples. To recompile shaders you can use the `compileshaders.py` scripts in the respective folders or any other means that can generate Vulkan SPIR-V from GLSL, HLSL, slang or rust-gpu. One such option is [this extension for Visual Studio](https://github.com/SaschaWillems/SPIRV-VSExtension).

Note that not all samples may come with all shading language variants. So some samples that have GLSL source files might not come with HLSL, slang and/or rust-gpu source files.

A note for using **slang** shaders: These require a different SPIR-V environment than glsl/hlsl. When selecting slang shaders, the base requirement for all samples is raised to at least Vulkan 1.1 with the SPIRV 1.4 extension.

If you want to compile **slang** shaders to SPIR-V, please use the latest release from [here](https://github.com/shader-slang/slang/releases) to get the latest bug fixes and features required for some of the samples.

A note for using **rust-gpu** shaders: These require Vulkan 1.2 or higher and are compiled using the rust-gpu compiler from the main branch.