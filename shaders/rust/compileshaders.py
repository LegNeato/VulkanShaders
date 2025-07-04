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
        "--auto-install-rust-toolchain",  # Auto-install required toolchain in CI
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
                elif entry_point == "main_gs" or "geometry" in entry_point.lower():
                    shader_type = "geom"
                elif entry_point == "main_tcs" or "tesscontrol" in entry_point.lower():
                    shader_type = "tesc"
                elif entry_point == "main_tes" or "tesseval" in entry_point.lower():
                    shader_type = "tese"
                elif entry_point == "main_task" or "task" in entry_point.lower():
                    shader_type = "task"
                elif entry_point == "main_mesh" or "mesh" in entry_point.lower():
                    shader_type = "mesh"
                else:
                    # Skip unknown entry points
                    continue
                
                final_path = shader_dir / f"{shader_name}.{shader_type}.spv"
                
                # Just rename the file - the C++ code will look for the entry point by name
                if final_path.exists():
                    final_path.unlink()  # Remove existing file
                source_path.rename(final_path)
                print(f"  Created {final_path.name} (entry point: {entry_point})")
                
                # Move to parent directory unless there's a top-level Cargo.toml
                # (top-level shaders like triangle, texture, etc. have their own Cargo.toml)
                parent_dir = shader_dir.parent
                if not (parent_dir / "Cargo.toml").exists():
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
                ("main_gs.spv", f"{shader_name}.geom.spv"),
                ("main_tcs.spv", f"{shader_name}.tesc.spv"),
                ("main_tes.spv", f"{shader_name}.tese.spv"),
                ("main_task.spv", f"{shader_name}.task.spv"),
                ("main_mesh.spv", f"{shader_name}.mesh.spv"),
            ]
            
            for old_name, new_name in renames:
                old_path = shader_dir / old_name
                new_path = shader_dir / new_name
                if old_path.exists():
                    if new_path.exists():
                        new_path.unlink()  # Remove existing file
                    old_path.rename(new_path)
                    print(f"  Created {new_name}")
    else:
        print(f"  ERROR: Failed to compile {shader_dir.name} (exit code: {result.returncode})")
    
    return result.returncode == 0

def main():
    # Change to the rust-gpu directory
    rust_gpu_dir = Path(__file__).parent
    os.chdir(rust_gpu_dir)
    
    # Check requirements
    check_requirements()
    
    # Parse command line arguments
    shader_filter = None
    if len(sys.argv) > 1:
        shader_filter = sys.argv[1]
        print(f"Filtering for shader: {shader_filter}")
    
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
                    # Apply filter if specified
                    if shader_filter:
                        # Check if the filter matches the path or package name
                        if shader_filter in str(package_path) or shader_filter == package['name']:
                            shader_dirs.append(package_path)
                    else:
                        shader_dirs.append(package_path)
                    break
    except (subprocess.CalledProcessError, json.JSONDecodeError, KeyError) as e:
        print(f"Error getting workspace metadata: {e}")
        sys.exit(1)
    
    if not shader_dirs:
        if shader_filter:
            print(f"No shader crates found matching '{shader_filter}'")
        else:
            print("No shader crates found")
        sys.exit(1)
    
    print(f"Found {len(shader_dirs)} shader crates to build")
    
    # Compile each shader crate
    total_success = 0
    total_failed = 0
    failed_shaders = []
    
    for shader_dir in sorted(shader_dirs):
        if compile_shader(shader_dir):
            # Count generated .spv files
            spv_files = list(shader_dir.glob("*.spv"))
            total_success += len(spv_files)
        else:
            total_failed += 1
            failed_shaders.append(shader_dir.name)
    
    print(f"\nCompilation complete: {total_success} shaders generated, {total_failed} crates failed")
    
    if total_failed > 0:
        print(f"\nFailed shaders: {', '.join(failed_shaders)}")
        sys.exit(1)

if __name__ == "__main__":
    main()
