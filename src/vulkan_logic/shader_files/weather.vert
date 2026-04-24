// src/vulkan_logic/shader_files/weather.vert
#version 450

layout(location = 0) out vec2 fragUV;
layout(location = 1) out vec4 fragColor;

struct ParticleData {
    vec4 position; // xyz: local position, w: random seed
    vec4 color;    // xyz: color, w: alpha
    vec4 params;   // x: Scale, y: Fall Speed, z: Is Rain (1.0 or 0.0), w: Time
};

// PHASE 3 FIX: Remove hardcoded size limit, use unbounded array
layout(std430, binding = 0) buffer WeatherSSBO {
    mat4 view_proj;
    vec4 camera_pos;     
    vec4 camera_right;   
    vec4 camera_up;      
    uint particle_count;
    uint _pad[3]; 
    ParticleData particles[]; 
} weather;

void main() {
    ParticleData p = weather.particles[gl_InstanceIndex];
    
    // Basic quad rendering logic
    vec2 uv = vec2((gl_VertexIndex << 1) & 2, gl_VertexIndex & 2);
    fragUV = uv;
    fragColor = p.color;

    // Procedural falling logic based on time
    float time = p.params.w;
    float fallSpeed = p.params.y;
    vec3 localPos = p.position.xyz;
    
    // Wrap around logic
    localPos.y = mod(localPos.y - (time * fallSpeed) + p.position.w, 100.0) - 50.0;
    
    // Billboard offset
    vec2 offset = uv * 2.0 - 1.0;
    float scale = p.params.x;
    
    vec3 worldPos = weather.camera_pos.xyz + localPos 
        + weather.camera_right.xyz * offset.x * scale 
        + weather.camera_up.xyz * offset.y * scale;

    gl_Position = weather.view_proj * vec4(worldPos, 1.0);
}