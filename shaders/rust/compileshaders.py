#!/usr/bin/env python3

import subprocess
import os
import sys
import shutil
import json
from pathlib import Path

def check_requirements():
    """Check if cargo-gpu is installed"""
    if not shutil.which('cargo'):
        print("Error: cargo not found!")
        print("\nTo install Rust, run:")
        print("  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh")
        sys.exit(1)
    
    if not shutil.which('cargo-gpu'):
        print("Error: cargo-gpu not found!")
        print("\nTo install cargo-gpu, run:")
        print("  cargo install --git https://github.com/rust-gpu/cargo-gpu cargo-gpu")
        sys.exit(1)

def compile_shader(shader_dir):
    """Compile a shader crate using cargo-gpu"""
    print(f"\nBuilding {shader_dir.name}...")
    
    # Run cargo gpu build for this shader
    result = subprocess.run([
        "cargo", "gpu", "build",
        "--shader-crate", str(shader_dir),
        "--output-dir", str(shader_dir),
        "--multimodule",  # Split into separate files per entry point
    ], capture_output=False)
    
    if result.returncode == 0:
        # Read the manifest to understand which files were generated
        manifest_path = shader_dir / "manifest.json"
        if manifest_path.exists():
            with open(manifest_path, 'r') as f:
                manifest = json.load(f)
            
            shader_name = shader_dir.name
            
            # Process each entry in the manifest
            for entry in manifest:
                source_path = shader_dir / entry["source_path"]
                entry_point = entry["entry_point"]
                
                if not source_path.exists():
                    continue
                
                # Determine the shader type from the entry point name
                if entry_point == "main_vs" or "vertex" in entry_point.lower():
                    shader_type = "vert"
                elif entry_point == "main_fs" or "fragment" in entry_point.lower():
                    shader_type = "frag"
                elif entry_point == "main_cs" or "compute" in entry_point.lower():
                    shader_type = "comp"
                else:
                    # Skip unknown entry points
                    continue
                
                final_path = shader_dir / f"{shader_name}.{shader_type}.spv"
                
                # Just rename the file - the C++ code will look for the entry point by name
                source_path.rename(final_path)
                print(f"  Created {final_path.name} (entry point: {entry_point})")
                
                # Special case: move to parent directory for nested shader structures
                if shader_dir.parent.name in ["base", "descriptorsets", "dynamicuniformbuffer", "multisampling", "pipelines", "specializationconstants", "computeshader", "texturearray", "screenshot", "negativeviewportheight", "stencilbuffer", "parallaxmapping", "computecloth", "ssao", "shadowmapping", "deferred", "computenbody", "bloom", "hdr", "radialblur", "pbribl", "indirectdraw", "instancing", "texturecubemap", "renderheadless", "sphericalenvmapping", "occlusionquery"]:
                    parent_dir = shader_dir.parent
                    parent_final_path = parent_dir / f"{shader_name}.{shader_type}.spv"
                    shutil.move(str(final_path), str(parent_final_path))
                    print(f"  Moved to {parent_dir.name}/{shader_name}.{shader_type}.spv")
        else:
            # Fallback for when no manifest exists
            shader_name = shader_dir.name
            
            # Check for common naming patterns
            renames = [
                ("main_vs.spv", f"{shader_name}.vert.spv"),
                ("main_fs.spv", f"{shader_name}.frag.spv"),
                ("main_cs.spv", f"{shader_name}.comp.spv"),
            ]
            
            for old_name, new_name in renames:
                old_path = shader_dir / old_name
                new_path = shader_dir / new_name
                if old_path.exists():
                    old_path.rename(new_path)
                    print(f"  Created {new_name}")
    
    return result.returncode == 0

def main():
    # Change to the rust-gpu directory
    rust_gpu_dir = Path(__file__).parent
    os.chdir(rust_gpu_dir)
    
    # Check requirements
    check_requirements()
    
    # Get workspace members from cargo metadata
    try:
        result = subprocess.run(['cargo', 'metadata', '--format-version', '1'], 
                              capture_output=True, text=True, check=True)
        metadata = json.loads(result.stdout)
        
        shader_dirs = []
        for member in metadata['workspace_members']:
            # Parse package ID to get the path
            for package in metadata['packages']:
                if package['id'] == member:
                    package_path = Path(package['manifest_path']).parent
                    shader_dirs.append(package_path)
                    break
    except (subprocess.CalledProcessError, json.JSONDecodeError, KeyError) as e:
        print(f"Error getting workspace metadata: {e}")
        sys.exit(1)
    
    if not shader_dirs:
        print("No shader crates found")
        sys.exit(1)
    
    print(f"Found {len(shader_dirs)} shader crates to build")
    
    # Compile each shader crate
    total_success = 0
    total_failed = 0
    
    for shader_dir in sorted(shader_dirs):
        if compile_shader(shader_dir):
            # Count generated .spv files
            spv_files = list(shader_dir.glob("*.spv"))
            total_success += len(spv_files)
        else:
            print(f"  Failed to build {shader_dir.name}")
            total_failed += 1
    
    print(f"\nCompilation complete: {total_success} shaders generated, {total_failed} crates failed")
    
    if total_failed > 0:
        sys.exit(1)

if __name__ == "__main__":
    main()
