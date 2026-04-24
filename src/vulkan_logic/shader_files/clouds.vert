// src/vulkan_logic/shader_files/clouds.vert
#version 450

layout(location = 0) out vec3 fragRayDir;

layout(std430, binding = 0) buffer CloudSSBO {
    mat4 inv_view_proj;  
    vec4 camera_pos;
    vec4 sun_dir;        
    vec4 sun_color;      
    float time;
    uint cloud_count;
    uint _pad[2];
    // CloudData structure omitted for brevity, logic remains same
} clouds;

void main() {
    vec2 uv = vec2((gl_VertexIndex << 1) & 2, gl_VertexIndex & 2);
    vec4 clipPos = vec4(uv * 2.0 - 1.0, 1.0, 1.0); 
    gl_Position = clipPos;
    vec4 worldPos = clouds.inv_view_proj * clipPos;
    fragRayDir = normalize(worldPos.xyz / worldPos.w);
}