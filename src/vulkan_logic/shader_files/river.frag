// src/vulkan_logic/shader_files/river.frag
#version 450

layout(location = 0) in vec3 fragWorldPos;
layout(location = 1) in vec3 fragNormal;
layout(location = 2) in vec2 fragUV;
layout(location = 3) in flat uint fragRiverId;

layout(location = 0) out vec4 outColor;

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

void main() {
    RiverData config = river_sys.rivers[fragRiverId];
    
    vec3 viewDir = normalize(river_sys.camera_pos.xyz - fragWorldPos);
    vec3 normal = normalize(fragNormal);
    
    // Fresnel
    float fresnel = pow(1.0 - max(dot(normal, viewDir), 0.0), config.advanced_params1.y);
    
    // Basic color mix
    vec3 color = mix(config.deep_color.rgb, config.shallow_color.rgb, 0.5); // Simplification without depth texture
    color = mix(color, config.sky_reflection_color.rgb, fresnel * config.sky_reflection_color.a);
    
    outColor = vec4(color, 1.0);
}