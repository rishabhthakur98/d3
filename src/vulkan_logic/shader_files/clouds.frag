// src/vulkan_logic/shader_files/clouds.frag
#version 450

layout(location = 0) in vec3 fragRayDir;
layout(location = 0) out vec4 outColor;

struct CloudVolumeData {
    vec4 min_bounds;     
    vec4 max_bounds;     
    vec4 base_color;     
    vec4 highlight_color;
    vec4 wind_dir;       
};

layout(std430, binding = 0) buffer CloudSSBO {
    mat4 inv_view_proj;  
    vec4 camera_pos;
    vec4 sun_dir;        
    vec4 sun_color;      
    float time;
    uint cloud_count;
    uint _pad[2];
    CloudVolumeData clouds[]; 
} cloud_data;

void main() {
    // Placeholder basic rendering logic
    vec3 dir = normalize(fragRayDir);
    if (dir.y < 0.0) discard; 
    
    vec4 finalColor = vec4(0.0);
    
    for (uint i = 0; i < cloud_data.cloud_count; i++) {
        // Simplified bounds check
        if (dir.y > 0.1) {
            float density = cloud_data.clouds[i].max_bounds.w;
            finalColor = vec4(cloud_data.clouds[i].base_color.rgb, density);
            break;
        }
    }
    
    if (finalColor.a <= 0.0) discard;
    outColor = finalColor;
}