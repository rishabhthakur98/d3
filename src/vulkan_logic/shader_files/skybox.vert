// src/vulkan_logic/shader_files/skybox.vert
#version 450

layout(location = 0) out vec3 fragRayDir;

layout(push_constant) uniform PushConstants {
    mat4 inv_view_proj;
} pc;

void main() {
    // Full screen triangle mapping
    vec2 uv = vec2((gl_VertexIndex << 1) & 2, gl_VertexIndex & 2);
    vec4 clipPos = vec4(uv * 2.0 - 1.0, 1.0, 1.0); // Z=1 pushes it to the back
    
    gl_Position = clipPos;
    
    vec4 worldPos = pc.inv_view_proj * clipPos;
    fragRayDir = normalize(worldPos.xyz / worldPos.w);
}