// src/vulkan_logic/shader_files/river.vert
#version 450

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec2 inUV;

layout(location = 0) out vec3 fragWorldPos;
layout(location = 1) out vec3 fragNormal;
layout(location = 2) out vec2 fragUV;
layout(location = 3) out flat uint fragRiverId;

struct RiverData {
    vec4 deep_color;     
    vec4 shallow_color;  
    vec4 foam_color;     
    vec4 sky_reflection_color; 
    vec4 params;               
    vec4 advanced_params1;     
};

layout(std430, binding = 0) buffer RiverSSBO {
    mat4 view_proj;
    vec4 camera_pos;     
    vec4 light_dir;      
    vec4 light_color;    
    uint river_count;
    uint _pad[3];
    RiverData rivers[]; 
} river_sys;

layout(push_constant) uniform PushConstants {
    mat4 model;
    uint river_index;
} pc;

void main() {
    RiverData config = river_sys.rivers[pc.river_index];
    float time = config.params.x;
    float wave_strength = config.params.z;
    
    vec3 pos = inPosition;
    // Simple wave perturbation
    pos.y += sin(pos.x * 5.0 + time) * wave_strength * 0.1;

    vec4 worldPos = pc.model * vec4(pos, 1.0);
    gl_Position = river_sys.view_proj * worldPos;
    
    fragWorldPos = worldPos.xyz;
    fragNormal = mat3(transpose(inverse(pc.model))) * inNormal;
    fragUV = inUV;
    fragRiverId = pc.river_index;
}