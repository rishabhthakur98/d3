#!/bin/bash

# Define our input and output directories relative to the project root
SHADER_DIR="src/vulkan_logic/shader_files"
OUT_DIR="src/vulkan_logic/compiled_shaders"

# 1. Verify that the Vulkan SDK compiler is installed and available
if ! command -v glslc &> /dev/null
then
    echo "ERROR: 'glslc' could not be found."
    echo "Please ensure the Vulkan SDK is installed and added to your system PATH."
    exit 1
fi

# 2. Create the output directory if it doesn't already exist
mkdir -p "$OUT_DIR"

echo "=== Starting Shader Compilation ==="

# 3. Compile all Vertex Shaders (.vert)
# We loop through the directory and compile any file ending in .vert
for f in "$SHADER_DIR"/*.vert; do
    # Check if the file actually exists (prevents errors if the folder is empty)
    if [ -f "$f" ]; then
        filename=$(basename -- "$f")
        echo "Compiling Vertex Shader: $filename -> ${filename}.spv"
        
        # Compile to SPIR-V format
        glslc "$f" -o "$OUT_DIR/${filename}.spv"
    fi
done

# 4. Compile all Fragment Shaders (.frag)
# We loop through the directory and compile any file ending in .frag
for f in "$SHADER_DIR"/*.frag; do
    if [ -f "$f" ]; then
        filename=$(basename -- "$f")
        echo "Compiling Fragment Shader: $filename -> ${filename}.spv"
        
        # Compile to SPIR-V format
        glslc "$f" -o "$OUT_DIR/${filename}.spv"
    fi
done

echo "=== Shader Compilation Complete ==="