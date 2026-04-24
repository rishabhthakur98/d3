// src/vulkan_logic/shader_files/triangle.vert
#version 450
#extension GL_EXT_nonuniform_qualifier : require

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec2 inUV;

layout(location = 0) out vec3 fragNormal;
layout(location = 1) out vec2 fragUV;
layout(location = 2) out vec3 fragPosWorld;
layout(location = 3) out vec4 fragPosLightSpace;

// Use std430 for SSBO arrays
struct GlobalLightData {
    vec4 direction; 
    vec4 color;     
};

struct SpotLightData {
    vec4 position;  
    vec4 direction; 
    vec4 color;     
    vec4 params;    
};

struct PointLightData {
    vec4 position;  
    vec4 color;     
};

struct FogVolumeData {
    vec4 min_bounds;    
    vec4 max_bounds;    
    vec4 color_density; 
};

// PHASE 3 FIX: Changed from 'uniform' to 'buffer' with std430
layout(std430, binding = 0) buffer LightSSBO {
    vec4 ambient_color; 
    vec4 camera_pos;    
    
    uint global_count;
    uint spot_count;
    uint point_count;          
    uint fog_count;      
    
    GlobalLightData global_lights[4];   // Can keep small fixed arrays if they aren't the last element
    SpotLightData spot_lights[16];     
    PointLightData point_lights[32];   
    FogVolumeData fog_volumes[];        // Unbounded array allowed as the last element
} light_data;

layout(push_constant) uniform PushConstants {
    mat4 view_proj;
    mat4 model;
    mat4 light_space_matrix;
} pc;

void main() {
    vec4 worldPos = pc.model * vec4(inPosition, 1.0);
    gl_Position = pc.view_proj * worldPos;
    
    fragNormal = mat3(transpose(inverse(pc.model))) * inNormal;
    fragUV = inUV;
    fragPosWorld = worldPos.xyz;
    fragPosLightSpace = pc.light_space_matrix * worldPos;
}