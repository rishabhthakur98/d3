// src/vulkan_logic/shader_files/clouds.vert
#version 450

struct CloudVolumeData {
    vec4 min_bounds;      // xyz: min_bounds, w: coverage
    vec4 max_bounds;      // xyz: max_bounds, w: density
    vec4 base_color;      // xyz: base_color, w: wind_speed
    vec4 highlight_color; // xyz: highlight_color
    vec4 wind_dir;        // xyz: wind_direction
};

layout(set = 0, binding = 0) uniform CloudUBO {
    mat4 inv_view_proj;
    vec4 camera_pos;
    vec4 sun_dir;
    vec4 sun_color;
    float time;
    uint cloud_count;
    uint pad1, pad2;
    CloudVolumeData clouds[10];
} ubo;

layout(location = 0) out vec3 fragRayDir;

void main() {
    // Generate a fullscreen triangle spanning the entire screen
    vec2 uvs[3] = vec2[](vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0));
    vec2 uv = uvs[gl_VertexIndex];
    
    gl_Position = vec4(uv, 0.99999, 1.0); // Push to the absolute far depth plane
    
    vec4 target = ubo.inv_view_proj * gl_Position;
    fragRayDir = target.xyz / target.w - ubo.camera_pos.xyz;
}