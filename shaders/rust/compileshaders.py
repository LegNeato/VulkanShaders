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
    
    # Find all shader crates
    shader_dirs = []
    for item in rust_gpu_dir.iterdir():
        if item.is_dir() and (item / "Cargo.toml").exists() and (item / "src" / "lib.rs").exists():
            shader_dirs.append(item)
    
    if not shader_dirs:
        print("No shader crates found")
        sys.exit(1)
    
    print(f"Found {len(shader_dirs)} shader crates to build")
    
    # Clean old .spv files
    print("\nCleaning old files...")
    for spv_file in rust_gpu_dir.rglob("*.spv"):
        if "target" not in str(spv_file):
            spv_file.unlink()
    
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