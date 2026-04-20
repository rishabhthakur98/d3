#version 450

// --- INPUTS FROM RUST VERTEX BUFFER ---
layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec4 inColor;
layout(location = 2) in vec3 inNormal;
layout(location = 3) in vec2 inUV;

// --- MATRICES FROM RUST PUSH CONSTANTS ---
// Reduced to 128 bytes to prevent Hardware Segfaults
layout(push_constant) uniform PushConstants {
    mat4 view_proj; // Camera view pre-multiplied with projection (64 bytes)
    mat4 model;     // Object transform (64 bytes)
} pc;

// --- OUTPUTS TO FRAGMENT SHADER ---
layout(location = 0) out vec4 fragColor;
layout(location = 1) out vec2 fragUV;

void main() {
    // Project the 3D point onto the 2D screen using the compressed matrices
    gl_Position = pc.view_proj * pc.model * vec4(inPosition, 1.0);
    
    fragColor = inColor;
    fragUV = inUV;
}